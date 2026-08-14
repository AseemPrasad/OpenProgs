use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProxyConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    #[serde(default = "default_upstream_url")]
    pub upstream_url: String,

    #[serde(default = "default_max_tokens_per_tenant")]
    pub max_tokens_per_tenant: u32,

    #[serde(default = "default_token_refill_rate")]
    pub token_refill_rate_per_minute: u32,

    #[serde(default = "default_inspection_window")]
    pub inspection_window_size: usize,

    #[serde(default)]
    pub enable_mid_stream_inspection: bool,

    #[serde(default)]
    pub inspection_mode: InspectionMode,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InspectionMode {
    Disabled,
    Permissive, // Log violations but don't terminate
    Strict,     // Terminate on violations
}

impl Default for InspectionMode {
    fn default() -> Self {
        InspectionMode::Disabled
    }
}

fn default_listen_addr() -> String {
    "127.0.0.1:8080".to_string()
}

fn default_upstream_url() -> String {
    "http://localhost:5000".to_string()
}

fn default_max_tokens_per_tenant() -> u32 {
    4000
}

fn default_token_refill_rate() -> u32 {
    100
}

fn default_inspection_window() -> usize {
    50
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            listen_addr: default_listen_addr(),
            upstream_url: default_upstream_url(),
            max_tokens_per_tenant: default_max_tokens_per_tenant(),
            token_refill_rate_per_minute: default_token_refill_rate(),
            inspection_window_size: default_inspection_window(),
            enable_mid_stream_inspection: false,
            inspection_mode: InspectionMode::Disabled,
        }
    }
}

impl ProxyConfig {
    pub fn is_inspection_enabled(&self) -> bool {
        self.enable_mid_stream_inspection
            && self.inspection_mode != InspectionMode::Disabled
    }

    pub fn is_strict_inspection(&self) -> bool {
        self.inspection_mode == InspectionMode::Strict
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.enabled {
            if self.max_tokens_per_tenant == 0 {
                return Err(anyhow::anyhow!(
                    "max_tokens_per_tenant must be > 0"
                ));
            }

            if self.token_refill_rate_per_minute == 0 {
                return Err(anyhow::anyhow!(
                    "token_refill_rate_per_minute must be > 0"
                ));
            }

            if self.inspection_window_size == 0 {
                return Err(anyhow::anyhow!(
                    "inspection_window_size must be > 0"
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_config_defaults() {
        let config = ProxyConfig::default();

        assert!(!config.enabled);
        assert_eq!(config.listen_addr, "127.0.0.1:8080");
        assert_eq!(config.upstream_url, "http://localhost:5000");
        assert_eq!(config.max_tokens_per_tenant, 4000);
        assert_eq!(config.token_refill_rate_per_minute, 100);
        assert_eq!(config.inspection_window_size, 50);
    }

    #[test]
    fn test_proxy_config_inspection_disabled() {
        let config = ProxyConfig {
            enabled: true,
            enable_mid_stream_inspection: false,
            inspection_mode: InspectionMode::Disabled,
            ..Default::default()
        };

        assert!(!config.is_inspection_enabled());
        assert!(!config.is_strict_inspection());
    }

    #[test]
    fn test_proxy_config_inspection_permissive() {
        let config = ProxyConfig {
            enabled: true,
            enable_mid_stream_inspection: true,
            inspection_mode: InspectionMode::Permissive,
            ..Default::default()
        };

        assert!(config.is_inspection_enabled());
        assert!(!config.is_strict_inspection());
    }

    #[test]
    fn test_proxy_config_inspection_strict() {
        let config = ProxyConfig {
            enabled: true,
            enable_mid_stream_inspection: true,
            inspection_mode: InspectionMode::Strict,
            ..Default::default()
        };

        assert!(config.is_inspection_enabled());
        assert!(config.is_strict_inspection());
    }

    #[test]
    fn test_proxy_config_validation_enabled() {
        let config = ProxyConfig {
            enabled: true,
            max_tokens_per_tenant: 1000,
            token_refill_rate_per_minute: 50,
            ..Default::default()
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_proxy_config_validation_zero_tokens() {
        let config = ProxyConfig {
            enabled: true,
            max_tokens_per_tenant: 0,
            ..Default::default()
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_proxy_config_validation_disabled() {
        let config = ProxyConfig {
            enabled: false,
            max_tokens_per_tenant: 0, // Would fail if enabled
            ..Default::default()
        };

        // Validation passes for disabled proxy
        assert!(config.validate().is_ok());
    }
}
