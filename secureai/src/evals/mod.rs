pub mod eval_data;
pub mod metrics;
pub mod drift_detector;
pub mod sampling;
pub mod online;

pub use eval_data::{EvalRequest, EvalMetrics, MetricType, EvalAlert};
pub use metrics::{MetricWindow, Statistics, MetricSample};
pub use drift_detector::DriftDetector;
pub use sampling::{SamplingStrategy, DynamicSampler, SamplingDecision};
pub use online::EvaluationEngine;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct EvalsConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_sampling_rate")]
    pub sampling_rate: f32,

    #[serde(default = "default_boost_flagged")]
    pub boost_flagged_requests: f32,

    #[serde(default = "default_anomaly_threshold")]
    pub anomaly_threshold: f32,

    #[serde(default = "default_short_window")]
    pub short_window_hours: u64,

    #[serde(default = "default_long_window")]
    pub long_window_hours: u64,

    #[serde(default)]
    pub alert_enabled: bool,

    #[serde(default)]
    pub alert_webhook_url: Option<String>,
}

fn default_sampling_rate() -> f32 { 0.1 }
fn default_boost_flagged() -> f32 { 1.0 }
fn default_anomaly_threshold() -> f32 { 3.0 }
fn default_short_window() -> u64 { 1 }
fn default_long_window() -> u64 { 24 }

impl Default for EvalsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sampling_rate: default_sampling_rate(),
            boost_flagged_requests: default_boost_flagged(),
            anomaly_threshold: default_anomaly_threshold(),
            short_window_hours: default_short_window(),
            long_window_hours: default_long_window(),
            alert_enabled: false,
            alert_webhook_url: None,
        }
    }
}
