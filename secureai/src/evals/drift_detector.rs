use crate::evals::{MetricWindow, EvalAlert, MetricType};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct DriftDetector {
    metric_type: MetricType,
    short_window: MetricWindow,
    long_window: MetricWindow,
    anomaly_threshold: f32,
    alert_history: Arc<RwLock<Vec<EvalAlert>>>,
}

impl DriftDetector {
    pub fn new(metric_type: MetricType, anomaly_threshold: f32) -> Self {
        Self {
            metric_type,
            short_window: MetricWindow::new(3600), // 1 hour
            long_window: MetricWindow::new(86400), // 24 hours
            anomaly_threshold,
            alert_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn new_with_windows(
        metric_type: MetricType,
        short_window_secs: u64,
        long_window_secs: u64,
        anomaly_threshold: f32,
    ) -> Self {
        Self {
            metric_type,
            short_window: MetricWindow::new(short_window_secs),
            long_window: MetricWindow::new(long_window_secs),
            anomaly_threshold,
            alert_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_sample(&self, value: f32) {
        use crate::evals::MetricSample;
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let sample = MetricSample {
            value,
            timestamp: now,
            metric_type: self.metric_type,
        };

        self.short_window.add_sample(sample.clone());
        self.long_window.add_sample(sample);
    }

    pub fn detect_anomalies(&self) -> Vec<EvalAlert> {
        let mut alerts = Vec::new();

        // 3-Sigma Rule: z_score > 3.0 indicates anomaly (~99.7% confidence)
        if let Some(baseline) = self.short_window.get_baseline() {
            let current_stats = self.short_window.get_statistics();

            // Check if current mean deviates from baseline
            let mean_z_score = (current_stats.mean - baseline.mean).abs() / baseline.stddev.max(0.001);

            if mean_z_score > self.anomaly_threshold {
                alerts.push(self.create_alert(&current_stats, &baseline, mean_z_score));
            }
        }

        // Store alerts in history
        if !alerts.is_empty() {
            self.alert_history.write().extend(alerts.clone());
        }

        alerts
    }

    fn create_alert(
        &self,
        current: &crate::evals::Statistics,
        baseline: &crate::evals::Statistics,
        z_score: f32,
    ) -> EvalAlert {
        match self.metric_type {
            MetricType::Toxicity => EvalAlert::ToxicitySpikeDetected {
                score: current.mean,
                baseline: baseline.mean,
                z_score,
            },
            MetricType::HallucinationRisk => EvalAlert::HallucinationRiskElevated {
                risk: current.mean,
                baseline: baseline.mean,
                z_score,
            },
            MetricType::GuardrailTriggers => EvalAlert::GuardrailTriggerRateAnomaly {
                rate: current.mean,
                baseline: baseline.mean,
                z_score,
            },
            _ => EvalAlert::PolicyDriftDetected {
                metric: self.metric_type.as_str().to_string(),
                z_score,
                threshold: self.anomaly_threshold,
            },
        }
    }

    pub fn get_unread_alerts(&self) -> Vec<EvalAlert> {
        self.alert_history.read().clone()
    }

    pub fn acknowledge_alerts(&self) {
        self.alert_history.write().clear();
    }

    pub fn get_short_window_stats(&self) -> crate::evals::Statistics {
        self.short_window.get_statistics()
    }

    pub fn get_long_window_stats(&self) -> crate::evals::Statistics {
        self.long_window.get_statistics()
    }

    pub fn get_metric_type(&self) -> MetricType {
        self.metric_type
    }

    pub fn get_anomaly_threshold(&self) -> f32 {
        self.anomaly_threshold
    }
}
