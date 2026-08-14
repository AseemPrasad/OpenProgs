pub mod exporter;
pub mod spans;

pub use exporter::{OTLPExporter, OTLPConfig};
pub use spans::{SpanInstrumentation, ScopedSpan};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_trace_all")]
    pub trace_all_paths: bool,

    #[serde(default)]
    pub otlp_grpc_endpoint: Option<String>,

    #[serde(default)]
    pub otlp_http_endpoint: Option<String>,

    #[serde(default = "default_service_name")]
    pub service_name: String,

    #[serde(default = "default_batch_size")]
    pub batch_size: u32,
}

fn default_trace_all() -> bool {
    false
}

fn default_service_name() -> String {
    "secureai-mvp".to_string()
}

fn default_batch_size() -> u32 {
    512
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            trace_all_paths: false,
            otlp_grpc_endpoint: Some("http://localhost:4317".to_string()),
            otlp_http_endpoint: None,
            service_name: default_service_name(),
            batch_size: default_batch_size(),
        }
    }
}

impl TelemetryConfig {
    pub fn to_otlp_config(&self) -> OTLPConfig {
        OTLPConfig {
            enabled: self.enabled,
            grpc_endpoint: self.otlp_grpc_endpoint.clone().unwrap_or_default(),
            http_endpoint: self.otlp_http_endpoint.clone(),
            service_name: self.service_name.clone(),
            batch_size: self.batch_size,
            export_interval_ms: 5000,
        }
    }
}
