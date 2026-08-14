use anyhow::{Result, Context};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::time::{interval, Duration};

use crate::policy::{PolicyStore, PolicyConfig};

#[derive(Debug, Clone)]
pub enum WatchSource {
    File(String),
    // Redis(String),  // Future: for distributed deployments
}

pub struct PolicyWatcher {
    store: Arc<PolicyStore>,
    source: WatchSource,
    poll_interval: Duration,
}

impl PolicyWatcher {
    pub fn new(
        store: Arc<PolicyStore>,
        source: WatchSource,
    ) -> Self {
        Self {
            store,
            source,
            poll_interval: Duration::from_secs(5),
        }
    }

    pub fn with_interval(mut self, duration: Duration) -> Self {
        self.poll_interval = duration;
        self
    }

    pub async fn watch(&self) -> Result<()> {
        match &self.source {
            WatchSource::File(path) => {
                self.watch_file(path).await
            }
        }
    }

    async fn watch_file(&self, path: &str) -> Result<()> {
        let mut last_modified = self.get_file_mtime(path).await.unwrap_or(0);
        let mut interval = interval(self.poll_interval);

        loop {
            interval.tick().await;

            match self.get_file_mtime(path).await {
                Ok(current_modified) => {
                    if current_modified > last_modified {
                        tracing::info!(
                            "Policy file changed (mtime: {} -> {}), reloading...",
                            last_modified,
                            current_modified
                        );

                        match self.load_and_update_policy(path).await {
                            Ok(version) => {
                                tracing::info!("Policy updated successfully to version {}", version);
                                last_modified = current_modified;
                            }
                            Err(e) => {
                                tracing::error!("Failed to reload policy: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Error reading policy file: {}", e);
                }
            }
        }
    }

    async fn load_and_update_policy(&self, path: &str) -> Result<u64> {
        let content = fs::read_to_string(path)
            .await
            .context("Failed to read policy file")?;

        let config: PolicyConfig = toml::from_str(&content)
            .context("Failed to parse policy TOML")?;

        let version = self.store.update_policy(config)?;

        Ok(version)
    }

    async fn get_file_mtime(&self, path: &str) -> Result<u64> {
        let metadata = fs::metadata(path)
            .await
            .context("Failed to read file metadata")?;

        let modified = metadata
            .modified()
            .context("Failed to get modification time")?
            .duration_since(std::time::UNIX_EPOCH)
            .context("Invalid modification time")?
            .as_secs();

        Ok(modified)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio::io::AsyncWriteExt;
    use tempfile::NamedTempFile;

    fn create_test_config() -> PolicyConfig {
        PolicyConfig {
            allowed_paths: vec![PathBuf::from("/tmp")],
            network_access: false,
            max_memory_mb: 512,
            allowed_models: vec!["llama3".to_string()],
            isolation: None,
        }
    }

    #[tokio::test]
    async fn test_watcher_creation() {
        let initial_config = create_test_config();
        let (store, _rx) = PolicyStore::new(initial_config);

        let watcher = PolicyWatcher::new(
            Arc::new(store),
            WatchSource::File("/tmp/policy.toml".to_string()),
        );

        assert_eq!(watcher.poll_interval, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_watcher_custom_interval() {
        let initial_config = create_test_config();
        let (store, _rx) = PolicyStore::new(initial_config);

        let watcher = PolicyWatcher::new(
            Arc::new(store),
            WatchSource::File("/tmp/policy.toml".to_string()),
        )
        .with_interval(Duration::from_secs(10));

        assert_eq!(watcher.poll_interval, Duration::from_secs(10));
    }

    #[tokio::test]
    #[ignore] // Requires real file system access
    async fn test_watch_file_detects_changes() {
        let initial_config = create_test_config();
        let (store, _rx) = PolicyStore::new(initial_config);
        let store = Arc::new(store);

        // Create a temporary policy file
        let mut temp_file = NamedTempFile::new().unwrap();
        let initial_policy = toml::to_string(&create_test_config()).unwrap();
        let _ = temp_file.write_all(initial_policy.as_bytes()).await;
        temp_file.flush().await.unwrap();

        let path = temp_file.path().to_string_lossy().to_string();

        let watcher = PolicyWatcher::new(store.clone(), WatchSource::File(path.clone()))
            .with_interval(Duration::from_millis(100));

        let initial_version = store.get_version();

        // Spawn watcher in background
        let watcher_handle = tokio::spawn(async move {
            let _ = watcher.watch().await;
        });

        // Wait a bit for watcher to detect initial file
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Modify the file
        let mut new_config = create_test_config();
        new_config.max_memory_mb = 1024;

        let new_policy = toml::to_string(&new_config).unwrap();
        fs::write(&path, new_policy).await.unwrap();

        // Wait for watcher to detect change
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Version should have incremented
        let new_version = store.get_version();
        assert!(new_version > initial_version);

        watcher_handle.abort();
    }
}
