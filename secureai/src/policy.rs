use serde::Deserialize;
use std::path::PathBuf;
use anyhow::{Result, Context};

#[derive(Debug, Deserialize)]
pub struct PolicyConfig {
    pub allowed_paths: Vec<PathBuf>,
    pub network_access: bool,
    pub max_memory_mb: u32,
    pub allowed_models: Vec<String>,
}

pub struct PolicyEngine {
    config: PolicyConfig,
}

impl PolicyEngine {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read policy file at {}", path))?;
        let config: PolicyConfig = toml::from_str(&content)
            .context("Failed to parse policy TOML")?;
        Ok(Self { config })
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
}
