pub mod job;
pub mod producer;
pub mod consumer;
pub mod pool;
pub mod hooks;

pub use job::{Job, JobState, Artifact, Lease, JobError};
pub use producer::NatsProducer;
pub use consumer::{NatsConsumer, Worker};
pub use pool::{WorkerPool, PoolStats};
pub use hooks::{QueueService, QueueStats, QueueHooks, initialize_queue, shutdown_queue, get_queue, is_queue_enabled};

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct QueueConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_nats_url")]
    pub nats_url: String,

    #[serde(default = "default_stream_name")]
    pub stream_name: String,

    #[serde(default = "default_consumer_name")]
    pub consumer_name: String,

    #[serde(default = "default_max_workers")]
    pub max_workers: usize,

    #[serde(default = "default_ack_wait")]
    pub ack_wait_seconds: u64,

    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_nats_url() -> String {
    "nats://localhost:4222".to_string()
}

fn default_stream_name() -> String {
    "secureai_jobs".to_string()
}

fn default_consumer_name() -> String {
    "secureai_workers".to_string()
}

fn default_max_workers() -> usize {
    4
}

fn default_ack_wait() -> u64 {
    60
}

fn default_max_retries() -> u32 {
    3
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            nats_url: default_nats_url(),
            stream_name: default_stream_name(),
            consumer_name: default_consumer_name(),
            max_workers: default_max_workers(),
            ack_wait_seconds: default_ack_wait(),
            max_retries: default_max_retries(),
        }
    }
}
