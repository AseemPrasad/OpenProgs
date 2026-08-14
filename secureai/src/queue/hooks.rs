use crate::queue::{Job, JobState, NatsProducer, NatsConsumer, WorkerPool, QueueConfig};
use async_nats::jetstream::Context as JetStreamContext;
use anyhow::{Result, Context};
use std::sync::Arc;
use std::time::Duration;

pub struct QueueService {
    producer: Arc<NatsProducer>,
    pool: Arc<WorkerPool>,
    config: QueueConfig,
}

impl QueueService {
    pub async fn new(config: QueueConfig) -> Result<Self> {
        if !config.enabled {
            return Err(anyhow::anyhow!("Queue is not enabled in config"));
        }

        // Connect to NATS
        let client = async_nats::connect(&config.nats_url)
            .await
            .context("Failed to connect to NATS")?;

        let jetstream = async_nats::jetstream::new(client.clone());

        // Create producer
        let producer = Arc::new(NatsProducer::new(&config.nats_url, config.stream_name.clone()).await?);

        // Create consumer
        let consumer = Arc::new(
            NatsConsumer::new(
                jetstream,
                config.stream_name.clone(),
                config.consumer_name.clone(),
                config.ack_wait_seconds,
                10,
            )
            .await?
        );

        // Create worker pool
        let pool = Arc::new(WorkerPool::new(consumer, config.max_workers).await?);

        tracing::info!("Queue service initialized with {} workers", config.max_workers);

        Ok(Self {
            producer,
            pool,
            config,
        })
    }

    pub async fn enqueue_task(
        &self,
        tool_name: String,
        tool_params: serde_json::Value,
        tenant_id: String,
    ) -> Result<String> {
        let job = Job::new(tool_name, tool_params, tenant_id, self.config.max_retries);
        let job_id = job.id.clone();

        self.producer.enqueue_job(&job).await?;

        tracing::info!("Enqueued job {} for tenant {}", job_id, job.tenant_id);
        Ok(job_id)
    }

    pub async fn get_queue_stats(&self) -> Result<QueueStats> {
        let pool_stats = self.pool.get_stats();
        let stream_info = self.producer.get_stream_stats().await?;

        Ok(QueueStats {
            stream_name: self.config.stream_name.clone(),
            consumer_name: self.config.consumer_name.clone(),
            max_workers: self.config.max_workers,
            active_workers: pool_stats.active_workers,
            jobs_pending: stream_info.state.messages,
            jobs_completed: pool_stats.total_jobs_completed,
            jobs_failed: pool_stats.total_jobs_failed,
        })
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.pool.shutdown().await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct QueueStats {
    pub stream_name: String,
    pub consumer_name: String,
    pub max_workers: usize,
    pub active_workers: usize,
    pub jobs_pending: u64,
    pub jobs_completed: u64,
    pub jobs_failed: u64,
}

pub trait QueueHooks {
    async fn on_job_started(&self, job_id: &str);
    async fn on_job_completed(&self, job_id: &str, artifacts: Vec<crate::queue::Artifact>);
    async fn on_job_failed(&self, job_id: &str, error: &str, retry_count: u32);
}

static QUEUE_SERVICE: parking_lot::Mutex<Option<Arc<QueueService>>> = parking_lot::Mutex::new(None);

pub fn initialize_queue(service: Arc<QueueService>) -> Result<()> {
    let mut queue = QUEUE_SERVICE.lock();
    *queue = Some(service);
    Ok(())
}

pub async fn shutdown_queue() -> Result<()> {
    let queue = QUEUE_SERVICE.lock().take();
    if let Some(service) = queue {
        service.shutdown().await?;
    }
    Ok(())
}

pub fn get_queue() -> Option<Arc<QueueService>> {
    QUEUE_SERVICE.lock().clone()
}

pub fn is_queue_enabled() -> bool {
    QUEUE_SERVICE.lock().is_some()
}
