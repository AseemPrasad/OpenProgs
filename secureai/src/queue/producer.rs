use crate::queue::Job;
use async_nats::jetstream::Context as JetStreamContext;
use async_nats::jetstream::stream::RetentionPolicy;
use anyhow::{Result, Context};
use std::time::Duration;

pub struct NatsProducer {
    client: async_nats::Client,
    jetstream: JetStreamContext,
    stream_name: String,
    subject_prefix: String,
}

impl NatsProducer {
    pub async fn new(
        nats_url: &str,
        stream_name: String,
    ) -> Result<Self> {
        let client = async_nats::connect(nats_url)
            .await
            .context(format!("Failed to connect to NATS at {}", nats_url))?;

        let jetstream = async_nats::jetstream::new(client.clone());

        // Create or update stream
        let _ = jetstream
            .create_stream(async_nats::jetstream::stream::Config {
                name: stream_name.clone(),
                description: Some("SecureAI job queue stream".to_string()),
                subjects: vec![format!("secureai.jobs.>")],
                max_age: Duration::from_secs(7 * 24 * 3600), // 7 days
                max_bytes: 100 * 1024 * 1024 * 1024, // 100GB
                storage: async_nats::jetstream::stream::StorageType::File,
                retention: RetentionPolicy::Limits,
                discard: async_nats::jetstream::stream::DiscardPolicy::Old,
                num_replicas: 1,
                duplicate_window: Duration::from_secs(5 * 60), // 5 minutes
                ..Default::default()
            })
            .await;

        let subject_prefix = "secureai.jobs".to_string();

        Ok(Self {
            client,
            jetstream,
            stream_name,
            subject_prefix,
        })
    }

    pub async fn enqueue_job(&self, job: &Job) -> Result<String> {
        let subject = format!("{}.{}", self.subject_prefix, job.tenant_id);
        let payload = serde_json::to_vec(job)?;

        let publish_ack = self
            .jetstream
            .publish(subject, payload.into())
            .await
            .context("Failed to publish job to JetStream")?;

        Ok(publish_ack.sequence)
    }

    pub async fn enqueue_batch(&self, jobs: Vec<Job>) -> Result<Vec<String>> {
        let mut sequences = Vec::new();

        for job in jobs {
            let seq = self.enqueue_job(&job).await?;
            sequences.push(seq);
        }

        Ok(sequences)
    }

    pub async fn get_stream_stats(&self) -> Result<async_nats::jetstream::stream::Info> {
        let info = self
            .jetstream
            .get_stream(&self.stream_name)
            .await
            .context("Failed to get stream info")?;

        Ok(info)
    }

    pub fn get_subject_for_tenant(&self, tenant_id: &str) -> String {
        format!("{}.{}", self.subject_prefix, tenant_id)
    }
}
