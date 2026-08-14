use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub name: String,
    pub endpoint_url: String,
    pub model: String,
    pub cost_per_1k_tokens: f32,
    pub max_tokens_per_min: u32,
    pub timeout_seconds: u32,
    pub sla_ttfb_ms: u64,
    pub sla_processing_ms: u64,
    pub fallback_to: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RouterConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_default_provider")]
    pub default_provider: String,

    #[serde(default)]
    pub providers: Vec<ProviderConfig>,

    #[serde(default = "default_failure_threshold")]
    pub circuit_breaker_failure_threshold: u32,

    #[serde(default = "default_success_threshold")]
    pub circuit_breaker_success_threshold: u32,

    #[serde(default = "default_timeout_seconds")]
    pub circuit_breaker_timeout_seconds: u32,

    #[serde(default = "default_metrics_window")]
    pub metrics_window_size: usize,

    #[serde(default)]
    pub enable_cost_optimization: bool,

    #[serde(default)]
    pub enable_latency_optimization: bool,
}

fn default_default_provider() -> String {
    "local".to_string()
}

fn default_failure_threshold() -> u32 {
    5
}

fn default_success_threshold() -> u32 {
    3
}

fn default_timeout_seconds() -> u32 {
    60
}

fn default_metrics_window() -> usize {
    100
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_provider: default_default_provider(),
            providers: vec![],
            circuit_breaker_failure_threshold: default_failure_threshold(),
            circuit_breaker_success_threshold: default_success_threshold(),
            circuit_breaker_timeout_seconds: default_timeout_seconds(),
            metrics_window_size: default_metrics_window(),
            enable_cost_optimization: false,
            enable_latency_optimization: true,
        }
    }
}

impl RouterConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        if self.providers.is_empty() {
            return Err(anyhow::anyhow!("Router enabled but no providers configured"));
        }

        if !self.providers.iter().any(|p| p.name == self.default_provider) {
            return Err(anyhow::anyhow!(
                "Default provider '{}' not found in providers list",
                self.default_provider
            ));
        }

        if self.circuit_breaker_failure_threshold == 0 {
            return Err(anyhow::anyhow!("circuit_breaker_failure_threshold must be > 0"));
        }

        Ok(())
    }

    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_config_default() {
        let config = RouterConfig::default();

        assert!(!config.enabled);
        assert_eq!(config.default_provider, "local");
        assert_eq!(config.circuit_breaker_failure_threshold, 5);
    }

    #[test]
    fn test_provider_config() {
        let provider = ProviderConfig {
            name: "anthropic".to_string(),
            endpoint_url: "https://api.anthropic.com".to_string(),
            model: "claude-3".to_string(),
            cost_per_1k_tokens: 0.015,
            max_tokens_per_min: 40000,
            timeout_seconds: 30,
            sla_ttfb_ms: 500,
            sla_processing_ms: 10000,
            fallback_to: None,
        };

        assert_eq!(provider.name, "anthropic");
        assert_eq!(provider.cost_per_1k_tokens, 0.015);
    }

    #[test]
    fn test_router_config_validation_disabled() {
        let config = RouterConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_router_config_validation_no_providers() {
        let mut config = RouterConfig::default();
        config.enabled = true;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_router_config_get_provider() {
        let mut config = RouterConfig::default();
        config.providers.push(ProviderConfig {
            name: "test".to_string(),
            endpoint_url: "http://test".to_string(),
            model: "test-model".to_string(),
            cost_per_1k_tokens: 0.01,
            max_tokens_per_min: 10000,
            timeout_seconds: 30,
            sla_ttfb_ms: 500,
            sla_processing_ms: 10000,
            fallback_to: None,
        });

        let provider = config.get_provider("test");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().name, "test");
    }
}
