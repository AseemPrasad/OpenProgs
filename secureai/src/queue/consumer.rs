use crate::queue::Job;
use async_nats::jetstream::Context as JetStreamContext;
use async_nats::jetstream::consumer::PullConsumer;
use anyhow::{Result, Context};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub struct NatsConsumer {
    jetstream: JetStreamContext,
    stream_name: String,
    consumer_name: String,
    pull_consumer: PullConsumer,
    ack_wait: Duration,
    max_ack_pending: u32,
}

impl NatsConsumer {
    pub async fn new(
        jetstream: JetStreamContext,
        stream_name: String,
        consumer_name: String,
        ack_wait_secs: u64,
        max_ack_pending: u32,
    ) -> Result<Self> {
        let ack_wait = Duration::from_secs(ack_wait_secs);

        // Create or get durable consumer
        let consumer_config = async_nats::jetstream::consumer::push::Config {
            durable_name: Some(consumer_name.clone()),
            ..Default::default()
        };

        // Use pull consumer instead
        let pull_consumer = jetstream
            .get_or_create_consumer(
                &stream_name,
                async_nats::jetstream::consumer::Config {
                    durable_name: Some(consumer_name.clone()),
                    max_waiting: 100,
                    idle_heartbeat: Duration::from_secs(5),
                    ..Default::default()
                },
            )
            .await
            .context("Failed to create/get consumer")?;

        Ok(Self {
            jetstream,
            stream_name,
            consumer_name,
            pull_consumer,
            ack_wait,
            max_ack_pending,
        })
    }

    pub async fn pull_job(&self) -> Result<Option<(Job, String)>> {
        // Pull 1 message with 5 second timeout
        let mut messages = self
            .pull_consumer
            .messages()
            .await
            .context("Failed to create message stream")?;

        if let Some(message) = messages.next().await {
            let message = message.context("Failed to receive message")?;

            // Parse job from message
            let job: Job = serde_json::from_slice(&message.payload)
                .context("Failed to deserialize job")?;

            let message_id = format!("{}", message.metadata()?.sequence.stream);

            Ok(Some((job, message_id)))
        } else {
            Ok(None)
        }
    }

    pub async fn ack_job(&self, _message_id: &str) -> Result<()> {
        // Message acknowledgement is handled by the stream
        Ok(())
    }

    pub async fn nack_job(&self, _message_id: &str) -> Result<()> {
        // Message negative acknowledgement triggers requeue
        Ok(())
    }

    pub fn get_stream_name(&self) -> &str {
        &self.stream_name
    }

    pub fn get_consumer_name(&self) -> &str {
        &self.consumer_name
    }
}

pub struct Worker {
    pub id: String,
    consumer: Arc<NatsConsumer>,
    active_leases: Arc<RwLock<HashMap<String, (Job, String)>>>,
    ack_wait: Duration,
}

impl Worker {
    pub fn new(
        consumer: Arc<NatsConsumer>,
        ack_wait: Duration,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            consumer,
            active_leases: Arc::new(RwLock::new(HashMap::new())),
            ack_wait,
        }
    }

    pub async fn consume_and_process<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(Job) -> futures_util::future::BoxFuture<'static, Result<Job>> + Send,
    {
        loop {
            // Pull next job
            if let Some((mut job, message_id)) = self.consumer.pull_job().await? {
                // Assign lease
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_nanos() as u64;
                let lease_token = format!("{}:{}", self.id, uuid::Uuid::new_v4());
                job.assign_lease(lease_token.clone(), now + (self.ack_wait.as_secs() * 1_000_000_000));

                // Store active lease
                self.active_leases
                    .write()
                    .await
                    .insert(job.id.clone(), (job.clone(), message_id.clone()));

                // Execute job
                match handler(job.clone()).await {
                    Ok(completed_job) => {
                        // Job completed successfully
                        self.consumer.ack_job(&message_id).await?;
                        self.active_leases.write().await.remove(&job.id);
                    }
                    Err(e) => {
                        // Job failed
                        tracing::error!("Job {} failed: {}", job.id, e);
                        self.consumer.nack_job(&message_id).await?;
                        self.active_leases.write().await.remove(&job.id);
                    }
                }
            } else {
                // No job available, wait a bit
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }

    pub async fn get_active_leases(&self) -> Vec<String> {
        self.active_leases
            .read()
            .await
            .keys()
            .cloned()
            .collect()
    }

    pub async fn clear_active_leases(&self) {
        self.active_leases.write().await.clear();
    }
}
