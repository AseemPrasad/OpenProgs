use crate::queue::{Worker, NatsConsumer, Job};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

pub struct WorkerPool {
    workers: Vec<Arc<Worker>>,
    max_workers: usize,
    shutdown_tx: broadcast::Sender<()>,
    shutdown_rx: broadcast::Receiver<()>,
    stats: parking_lot::RwLock<PoolStats>,
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub active_workers: usize,
    pub total_jobs_processed: u64,
    pub total_jobs_failed: u64,
    pub total_jobs_completed: u64,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            active_workers: 0,
            total_jobs_processed: 0,
            total_jobs_failed: 0,
            total_jobs_completed: 0,
        }
    }
}

impl WorkerPool {
    pub async fn new(
        consumer: Arc<NatsConsumer>,
        max_workers: usize,
    ) -> Result<Self> {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

        let mut workers = Vec::new();
        for _ in 0..max_workers {
            let worker = Arc::new(Worker::new(
                consumer.clone(),
                Duration::from_secs(60),
            ));
            workers.push(worker);
        }

        Ok(Self {
            workers,
            max_workers,
            shutdown_tx,
            shutdown_rx,
            stats: parking_lot::RwLock::new(PoolStats {
                active_workers: max_workers,
                ..Default::default()
            }),
        })
    }

    pub fn get_workers(&self) -> Vec<Arc<Worker>> {
        self.workers.clone()
    }

    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down worker pool...");

        // Signal all workers to stop
        let _ = self.shutdown_tx.send(());

        // Wait a bit for in-flight jobs to complete
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Clear active leases
        for worker in &self.workers {
            worker.clear_active_leases().await;
        }

        tracing::info!("Worker pool shut down complete");
        Ok(())
    }

    pub fn get_stats(&self) -> PoolStats {
        self.stats.read().clone()
    }

    pub fn record_success(&self) {
        let mut stats = self.stats.write();
        stats.total_jobs_processed += 1;
        stats.total_jobs_completed += 1;
    }

    pub fn record_failure(&self) {
        let mut stats = self.stats.write();
        stats.total_jobs_processed += 1;
        stats.total_jobs_failed += 1;
    }

    pub async fn check_for_crashed_workers(&self) -> Result<Vec<String>> {
        let mut crashed_jobs = Vec::new();

        for worker in &self.workers {
            let active_leases = worker.get_active_leases().await;
            for job_id in active_leases {
                // In a real implementation, check if lease expired
                tracing::debug!("Worker {} has active lease on job {}", worker.id, job_id);
            }
        }

        Ok(crashed_jobs)
    }

    pub fn get_shutdown_signal(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }
}
