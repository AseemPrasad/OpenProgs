use crate::evals::MetricType;
use std::time::{SystemTime, Duration};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MetricSample {
    pub value: f32,
    pub timestamp: u64,
    pub metric_type: MetricType,
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub count: usize,
    pub mean: f32,
    pub stddev: f32,
    pub min: f32,
    pub max: f32,
    pub p50: f32,
    pub p95: f32,
    pub p99: f32,
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            stddev: 0.0,
            min: f32::MAX,
            max: f32::MIN,
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }
}

pub struct MetricWindow {
    duration_secs: u64,
    samples: Arc<RwLock<Vec<MetricSample>>>,
    created_at: u64,
}

impl MetricWindow {
    pub fn new(duration_secs: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            duration_secs,
            samples: Arc::new(RwLock::new(Vec::new())),
            created_at: now,
        }
    }

    pub fn add_sample(&self, sample: MetricSample) {
        self.samples.write().push(sample);
        self.prune_old_samples();
    }

    pub fn get_statistics(&self) -> Statistics {
        let samples = self.samples.read();

        if samples.is_empty() {
            return Statistics::default();
        }

        let mut values: Vec<f32> = samples.iter().map(|s| s.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let count = values.len();
        let mean = values.iter().sum::<f32>() / count as f32;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / count as f32;
        let stddev = variance.sqrt();

        let p50_idx = (count as f32 * 0.50) as usize;
        let p95_idx = (count as f32 * 0.95) as usize;
        let p99_idx = (count as f32 * 0.99) as usize;

        Statistics {
            count,
            mean,
            stddev,
            min: *values.first().unwrap_or(&0.0),
            max: *values.last().unwrap_or(&0.0),
            p50: values.get(p50_idx).copied().unwrap_or(mean),
            p95: values.get(p95_idx).copied().unwrap_or(mean),
            p99: values.get(p99_idx).copied().unwrap_or(mean),
        }
    }

    pub fn compute_z_score(&self, value: f32) -> f32 {
        let stats = self.get_statistics();

        if stats.stddev == 0.0 {
            return 0.0;
        }

        (value - stats.mean) / stats.stddev
    }

    pub fn is_anomaly(&self, value: f32, threshold: f32) -> bool {
        self.compute_z_score(value).abs() > threshold
    }

    pub fn get_baseline(&self) -> Option<Statistics> {
        let samples = self.samples.read();

        if samples.len() < 100 {
            return None; // Need at least 100 samples for baseline
        }

        // Calculate stats on first 100 samples
        let baseline_samples: Vec<f32> = samples[..100]
            .iter()
            .map(|s| s.value)
            .collect();

        let mean = baseline_samples.iter().sum::<f32>() / 100.0;
        let variance = baseline_samples.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / 100.0;
        let stddev = variance.sqrt();

        Some(Statistics {
            count: 100,
            mean,
            stddev,
            min: baseline_samples.iter().cloned().fold(f32::MAX, f32::min),
            max: baseline_samples.iter().cloned().fold(f32::MIN, f32::max),
            p50: baseline_samples[50],
            p95: baseline_samples[95],
            p99: baseline_samples[99],
        })
    }

    pub fn prune_old_samples(&self) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let cutoff_time = now.saturating_sub(self.duration_secs);

        let mut samples = self.samples.write();
        samples.retain(|s| s.timestamp / 1_000_000_000 > cutoff_time);
    }

    pub fn get_sample_count(&self) -> usize {
        self.samples.read().len()
    }

    pub fn get_duration_secs(&self) -> u64 {
        self.duration_secs
    }
}
