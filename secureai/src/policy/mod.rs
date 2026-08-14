use serde::Deserialize;
use std::path::PathBuf;
use anyhow::{Result, Context};

pub mod store;

pub use store::{PolicyStore, PolicyStoreUpdate};
use crate::guardrails::ThreatThresholds;

#[derive(Debug, Deserialize, Clone)]
pub struct PolicyConfig {
    pub allowed_paths: Vec<PathBuf>,
    pub network_access: bool,
    pub max_memory_mb: u32,
    pub allowed_models: Vec<String>,

    #[serde(default)]
    pub isolation: Option<IsolationPolicy>,

    #[serde(default)]
    pub guardrails: Option<GuardrailConfig>,

    #[serde(default)]
    pub proxy: Option<crate::proxy::ProxyConfig>,

    #[serde(default)]
    pub router: Option<crate::router::RouterConfig>,

    #[serde(default)]
    pub audit: Option<crate::audit::AuditConfig>,

    #[serde(default)]
    pub telemetry: Option<crate::telemetry::TelemetryConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IsolationPolicy {
    #[serde(default = "default_enable_landlock")]
    pub enable_landlock: bool,

    #[serde(default = "default_enable_seccomp")]
    pub enable_seccomp: bool,

    #[serde(default = "default_enable_cgroups")]
    pub enable_cgroups: bool,

    #[serde(default)]
    pub landlock_paths: Vec<PathBuf>,

    #[serde(default)]
    pub workspace_path: Option<String>,

    #[serde(default = "default_memory_limit")]
    pub memory_limit_mb: u32,

    #[serde(default = "default_cpu_quota")]
    pub cpu_quota: f64,

    #[serde(default = "default_process_limit")]
    pub max_processes: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GuardrailConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_prompt_injection_threshold")]
    pub prompt_injection_threshold: f32,

    #[serde(default = "default_data_exfiltration_threshold")]
    pub data_exfiltration_threshold: f32,

    #[serde(default = "default_privilege_escalation_threshold")]
    pub privilege_escalation_threshold: f32,

    #[serde(default = "default_reverse_shell_threshold")]
    pub reverse_shell_threshold: f32,

    #[serde(default = "default_sql_injection_threshold")]
    pub sql_injection_threshold: f32,

    #[serde(default)]
    pub onnx_model_path: Option<String>,
}

impl GuardrailConfig {
    pub fn to_threat_thresholds(&self) -> ThreatThresholds {
        ThreatThresholds {
            prompt_injection_threshold: self.prompt_injection_threshold,
            data_exfiltration_threshold: self.data_exfiltration_threshold,
            privilege_escalation_threshold: self.privilege_escalation_threshold,
            reverse_shell_threshold: self.reverse_shell_threshold,
            sql_injection_threshold: self.sql_injection_threshold,
        }
    }
}

fn default_enable_landlock() -> bool { true }
fn default_enable_seccomp() -> bool { true }
fn default_enable_cgroups() -> bool { true }
fn default_memory_limit() -> u32 { 512 }
fn default_cpu_quota() -> f64 { 1.0 }
fn default_process_limit() -> u32 { 100 }
fn default_prompt_injection_threshold() -> f32 { 0.82 }
fn default_data_exfiltration_threshold() -> f32 { 0.85 }
fn default_privilege_escalation_threshold() -> f32 { 0.80 }
fn default_reverse_shell_threshold() -> f32 { 0.83 }
fn default_sql_injection_threshold() -> f32 { 0.81 }

impl IsolationPolicy {
    pub fn enabled(&self) -> bool {
        self.enable_landlock || self.enable_seccomp || self.enable_cgroups
    }
}

pub struct PolicyEngine {
    config: PolicyConfig,
    store: Option<PolicyStore>,
    guardrail: Option<crate::guardrails::SemanticGuardrail>,
}

impl PolicyEngine {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read policy file at {}", path))?;
        let config: PolicyConfig = toml::from_str(&content)
            .context("Failed to parse policy TOML")?;

        // Initialize guardrail if configured
        let guardrail = if let Some(gr_config) = &config.guardrails {
            if gr_config.enabled {
                let thresholds = gr_config.to_threat_thresholds();
                match crate::guardrails::SemanticGuardrail::new(thresholds) {
                    Ok(gr) => Some(gr),
                    Err(e) => {
                        tracing::warn!("Failed to initialize semantic guardrail: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            config,
            store: None,
            guardrail,
        })
    }

    pub fn with_store(store: PolicyStore) -> Self {
        let config = store.get_policy().as_ref().clone();

        // Initialize guardrail if configured
        let guardrail = if let Some(gr_config) = &config.guardrails {
            if gr_config.enabled {
                let thresholds = gr_config.to_threat_thresholds();
                match crate::guardrails::SemanticGuardrail::new(thresholds) {
                    Ok(gr) => Some(gr),
                    Err(e) => {
                        tracing::warn!("Failed to initialize semantic guardrail: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        Self {
            config,
            store: Some(store),
            guardrail,
        }
    }

    pub fn validate_task(&self, model: &str, input_path: Option<&PathBuf>) -> bool {
        // Use remote store config if available
        let policy = if let Some(store) = &self.store {
            store.get_policy()
        } else {
            std::sync::Arc::new(self.config.clone())
        };

        // Check if model is allowed
        if !policy.allowed_models.contains(&model.to_string()) {
            return false;
        }

        // Check if input path is within allowed paths
        if let Some(path) = input_path {
            let is_allowed = policy.allowed_paths.iter().any(|allowed| {
                path.starts_with(allowed)
            });
            if !is_allowed {
                return false;
            }
        }

        true
    }

    pub fn get_vm_spec(&self) -> &PolicyConfig {
        &self.config
    }

    pub fn get_current_config(&self) -> std::sync::Arc<PolicyConfig> {
        if let Some(store) = &self.store {
            store.get_policy()
        } else {
            std::sync::Arc::new(self.config.clone())
        }
    }

    pub fn get_isolation_policy(&self) -> Option<&IsolationPolicy> {
        self.config.isolation.as_ref()
    }

    pub fn get_store(&self) -> Option<&PolicyStore> {
        self.store.as_ref()
    }

    pub fn subscribe_to_updates(
        &self,
    ) -> Option<tokio::sync::broadcast::Receiver<PolicyStoreUpdate>> {
        self.store.as_ref().map(|store| store.subscribe_updates())
    }

    pub fn get_guardrail(&self) -> Option<&crate::guardrails::SemanticGuardrail> {
        self.guardrail.as_ref()
    }

    pub async fn check_semantic_guardrails(&self, prompt: &str) -> Result<()> {
        if let Some(guardrail) = &self.guardrail {
            match guardrail.check_prompt(prompt).await? {
                crate::guardrails::GuardrailDecision::Deny { reason, .. } => {
                    return Err(anyhow::anyhow!("Semantic guardrail violation: {}", reason));
                }
                crate::guardrails::GuardrailDecision::Permit => {}
            }
        }
        Ok(())
    }

    pub fn get_audit_config(&self) -> Option<&crate::audit::AuditConfig> {
        self.config.audit.as_ref()
    }

    pub fn get_telemetry_config(&self) -> Option<&crate::telemetry::TelemetryConfig> {
        self.config.telemetry.as_ref()
    }
}
