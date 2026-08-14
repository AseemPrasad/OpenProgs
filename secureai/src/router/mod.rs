pub mod circuit_breaker;
pub mod metrics;
pub mod provider;
pub mod routing;

pub use circuit_breaker::{CircuitBreaker, CircuitState};
pub use metrics::ProviderMetrics;
pub use provider::{ProviderConfig, RouterConfig};
pub use routing::{AdaptiveRouter, RoutingDecision, RoutingStrategy};

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub struct Router {
    circuit_breakers: Arc<HashMap<String, Arc<CircuitBreaker>>>,
    metrics: Arc<HashMap<String, Arc<ProviderMetrics>>>,
    adaptive_router: Arc<AdaptiveRouter>,
    config: RouterConfig,
    enabled: bool,
}

impl Router {
    pub fn new(config: RouterConfig) -> anyhow::Result<Self> {
        if !config.enabled {
            tracing::info!("Router disabled");
            return Ok(Self {
                circuit_breakers: Arc::new(HashMap::new()),
                metrics: Arc::new(HashMap::new()),
                adaptive_router: Arc::new(AdaptiveRouter::new(
                    vec![],
                    config.default_provider.clone(),
                    config.enable_cost_optimization,
                    config.enable_latency_optimization,
                )?),
                config,
                enabled: false,
            });
        }

        config.validate()?;

        // Initialize circuit breakers for each provider
        let mut circuit_breakers = HashMap::new();
        for provider in &config.providers {
            let cb = CircuitBreaker::new(
                config.circuit_breaker_failure_threshold,
                config.circuit_breaker_success_threshold,
                Duration::from_secs(config.circuit_breaker_timeout_seconds as u64),
            );
            circuit_breakers.insert(provider.name.clone(), Arc::new(cb));
        }

        // Initialize metrics for each provider
        let mut metrics = HashMap::new();
        for provider in &config.providers {
            let m = ProviderMetrics::new(config.metrics_window_size);
            metrics.insert(provider.name.clone(), Arc::new(m));
        }

        let adaptive_router = AdaptiveRouter::new(
            config.providers.clone(),
            config.default_provider.clone(),
            config.enable_cost_optimization,
            config.enable_latency_optimization,
        )?;

        tracing::info!(
            "Router initialized with {} providers",
            config.providers.len()
        );

        Ok(Self {
            circuit_breakers: Arc::new(circuit_breakers),
            metrics: Arc::new(metrics),
            adaptive_router: Arc::new(adaptive_router),
            config,
            enabled: true,
        })
    }

    pub fn decide_route(&self, strategy: RoutingStrategy) -> anyhow::Result<RoutingDecision> {
        if !self.enabled {
            return Err(anyhow::anyhow!("Router disabled"));
        }

        // Get healthy providers (circuit breaker allows requests)
        let healthy_providers: Vec<String> = self
            .circuit_breakers
            .iter()
            .filter(|(_, cb)| cb.is_request_allowed())
            .map(|(name, _)| name.clone())
            .collect();

        if healthy_providers.is_empty() {
            return Err(anyhow::anyhow!("No healthy providers available"));
        }

        self.adaptive_router.decide_route(strategy, &healthy_providers)
    }

    pub fn record_success(&self, provider_name: &str, ttfb: Duration, total: Duration) {
        if let Some(cb) = self.circuit_breakers.get(provider_name) {
            cb.record_success();
        }

        if let Some(metrics) = self.metrics.get(provider_name) {
            metrics.record_success(ttfb, total);
        }
    }

    pub fn record_failure(&self, provider_name: &str) {
        if let Some(cb) = self.circuit_breakers.get(provider_name) {
            cb.record_failure();
        }

        if let Some(metrics) = self.metrics.get(provider_name) {
            metrics.record_failure();
        }
    }

    pub fn get_circuit_state(&self, provider_name: &str) -> Option<CircuitState> {
        self.circuit_breakers
            .get(provider_name)
            .map(|cb| cb.get_state())
    }

    pub fn get_provider_health(&self, provider_name: &str) -> Option<ProviderHealth> {
        let metrics = self.metrics.get(provider_name)?;

        Some(ProviderHealth {
            provider_name: provider_name.to_string(),
            circuit_state: self.get_circuit_state(provider_name),
            avg_ttfb_ms: metrics.get_average_ttfb().map(|d| d.as_millis() as u64),
            avg_processing_ms: metrics.get_average_processing().map(|d| d.as_millis() as u64),
            p99_ttfb_ms: metrics.get_p99_ttfb().map(|d| d.as_millis() as u64),
            p99_processing_ms: metrics.get_p99_processing().map(|d| d.as_millis() as u64),
            error_rate: metrics.get_error_rate(),
            meets_sla: metrics.meets_sla(
                self.config
                    .get_provider(provider_name)
                    .map(|p| p.sla_ttfb_ms)
                    .unwrap_or(500),
                self.config
                    .get_provider(provider_name)
                    .map(|p| p.sla_processing_ms)
                    .unwrap_or(10000),
            ),
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub provider_name: String,
    pub circuit_state: Option<CircuitState>,
    pub avg_ttfb_ms: Option<u64>,
    pub avg_processing_ms: Option<u64>,
    pub p99_ttfb_ms: Option<u64>,
    pub p99_processing_ms: Option<u64>,
    pub error_rate: f64,
    pub meets_sla: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> RouterConfig {
        RouterConfig {
            enabled: false,
            ..Default::default()
        }
    }

    #[test]
    fn test_router_disabled() {
        let config = create_test_config();
        let router = Router::new(config).unwrap();

        assert!(!router.is_enabled());
    }

    #[test]
    fn test_router_creation() {
        let config = RouterConfig::default();
        let router = Router::new(config);

        assert!(router.is_ok());
    }
}
