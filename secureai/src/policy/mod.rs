use serde::Deserialize;
use std::path::PathBuf;
use anyhow::{Result, Context};

pub mod store;

pub use store::{PolicyStore, PolicyStoreUpdate};

#[derive(Debug, Deserialize, Clone)]
pub struct PolicyConfig {
    pub allowed_paths: Vec<PathBuf>,
    pub network_access: bool,
    pub max_memory_mb: u32,
    pub allowed_models: Vec<String>,

    #[serde(default)]
    pub isolation: Option<IsolationPolicy>,
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

fn default_enable_landlock() -> bool { true }
fn default_enable_seccomp() -> bool { true }
fn default_enable_cgroups() -> bool { true }
fn default_memory_limit() -> u32 { 512 }
fn default_cpu_quota() -> f64 { 1.0 }
fn default_process_limit() -> u32 { 100 }

impl IsolationPolicy {
    pub fn enabled(&self) -> bool {
        self.enable_landlock || self.enable_seccomp || self.enable_cgroups
    }
}

pub struct PolicyEngine {
    config: PolicyConfig,
    store: Option<PolicyStore>,
}

impl PolicyEngine {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read policy file at {}", path))?;
        let config: PolicyConfig = toml::from_str(&content)
            .context("Failed to parse policy TOML")?;
        Ok(Self { config, store: None })
    }

    pub fn with_store(store: PolicyStore) -> Self {
        let config = store.get_policy().as_ref().clone();
        Self {
            config,
            store: Some(store),
        }
    }

    pub fn validate_task(&self, model: &str, input_path: Option<&PathBuf>) -> bool {
        // Check if model is allowed
        if !self.config.allowed_models.contains(&model.to_string()) {
            return false;
        }

        // Check if input path is within allowed paths
        if let Some(path) = input_path {
            let is_allowed = self.config.allowed_paths.iter().any(|allowed| {
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

    pub fn get_isolation_policy(&self) -> Option<&IsolationPolicy> {
        self.config.isolation.as_ref()
    }

    pub fn get_store(&self) -> Option<&PolicyStore> {
        self.store.as_ref()
    }
}
