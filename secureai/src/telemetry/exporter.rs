use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry::KeyValue;
use serde::Deserialize;
use std::time::Duration;
use anyhow::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct OTLPConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_grpc_endpoint")]
    pub grpc_endpoint: String,

    #[serde(default)]
    pub http_endpoint: Option<String>,

    #[serde(default = "default_service_name")]
    pub service_name: String,

    #[serde(default = "default_batch_size")]
    pub batch_size: u32,

    #[serde(default = "default_export_interval")]
    pub export_interval_ms: u64,
}

fn default_grpc_endpoint() -> String {
    "http://localhost:4317".to_string()
}

fn default_service_name() -> String {
    "secureai-mvp".to_string()
}

fn default_batch_size() -> u32 {
    512
}

fn default_export_interval() -> u64 {
    5000
}

impl Default for OTLPConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            grpc_endpoint: default_grpc_endpoint(),
            http_endpoint: None,
            service_name: default_service_name(),
            batch_size: default_batch_size(),
            export_interval_ms: default_export_interval(),
        }
    }
}

pub struct OTLPExporter;

impl OTLPExporter {
    pub fn initialize(config: OTLPConfig) -> Result<()> {
        if !config.enabled {
            return Ok(());
        }

        let resource = Resource::new(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new(
                "service.version",
                env!("CARGO_PKG_VERSION").to_string(),
            ),
            KeyValue::new(
                "deployment.environment",
                std::env::var("DEPLOY_ENV").unwrap_or_else(|_| "development".to_string()),
            ),
            KeyValue::new(
                "host.name",
                hostname::get()
                    .ok()
                    .and_then(|h| h.to_str().map(String::from))
                    .unwrap_or_else(|| "unknown".to_string()),
            ),
        ]);

        // Create OTLP exporter (gRPC preferred, fallback to HTTP)
        let otlp_exporter = if let Some(http_endpoint) = &config.http_endpoint {
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(http_endpoint.clone())
                .build_span_exporter()?
        } else {
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(config.grpc_endpoint.clone())
                .build_span_exporter()?
        };

        // Create tracer provider with batch processor
        let tracer_provider = TracerProvider::builder()
            .with_batch_exporter(
                otlp_exporter,
                opentelemetry_sdk::trace::BatchSpanProcessor::builder()
                    .with_max_export_batch_size(config.batch_size as usize)
                    .with_scheduled_delay(Duration::from_millis(config.export_interval_ms))
                    .build(),
            )
            .with_resource(resource)
            .build();

        // Set as global tracer provider
        global::set_tracer_provider(tracer_provider);

        tracing::info!("OpenTelemetry OTLP exporter initialized");
        Ok(())
    }

    pub fn shutdown() -> Result<()> {
        let _ = global::shutdown_tracer_provider();
        Ok(())
    }
}
