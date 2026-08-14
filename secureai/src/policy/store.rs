use anyhow::Result;
use arc_swap::ArcSwap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::policy::PolicyConfig;

pub struct PolicyStore {
    policies: Arc<ArcSwap<PolicyConfig>>,
    version: Arc<AtomicU64>,
    update_tx: tokio::sync::broadcast::Sender<PolicyStoreUpdate>,
}

#[derive(Clone, Debug)]
pub struct PolicyStoreUpdate {
    pub version: u64,
    pub timestamp: i64,
    pub tenant_id: String,
}

impl PolicyStore {
    pub fn new(initial_config: PolicyConfig) -> (Self, tokio::sync::broadcast::Receiver<PolicyStoreUpdate>) {
        let (tx, rx) = tokio::sync::broadcast::channel(100);

        let store = Self {
            policies: Arc::new(ArcSwap::new(Arc::new(initial_config))),
            version: Arc::new(AtomicU64::new(1)),
            update_tx: tx,
        };

        (store, rx)
    }

    pub fn get_policy(&self) -> Arc<PolicyConfig> {
        self.policies.load_full()
    }

    pub fn get_version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }

    pub fn update_policy(&self, new_config: PolicyConfig) -> Result<u64> {
        let new_version = self.version.fetch_add(1, Ordering::Release) + 1;

        self.policies.swap(Arc::new(new_config));

        let update = PolicyStoreUpdate {
            version: new_version,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            tenant_id: "default".to_string(),
        };

        let _ = self.update_tx.send(update);

        Ok(new_version)
    }

    pub fn subscribe_updates(&self) -> tokio::sync::broadcast::Receiver<PolicyStoreUpdate> {
        self.update_tx.subscribe()
    }

    pub fn num_subscribers(&self) -> usize {
        self.update_tx.receiver_count()
    }
}

impl Clone for PolicyStore {
    fn clone(&self) -> Self {
        Self {
            policies: Arc::clone(&self.policies),
            version: Arc::clone(&self.version),
            update_tx: self.update_tx.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_config() -> PolicyConfig {
        PolicyConfig {
            allowed_paths: vec![PathBuf::from("/tmp")],
            network_access: false,
            max_memory_mb: 512,
            allowed_models: vec!["llama3".to_string()],
            isolation: None,
        }
    }

    #[test]
    fn test_store_creation() {
        let config = create_test_config();
        let (store, _rx) = PolicyStore::new(config.clone());

        let loaded = store.get_policy();
        assert_eq!(loaded.max_memory_mb, 512);
        assert_eq!(loaded.allowed_models.len(), 1);
    }

    #[test]
    fn test_lock_free_read() {
        let config = create_test_config();
        let (store, _rx) = PolicyStore::new(config);

        // Multiple reads should not block
        let policy1 = store.get_policy();
        let policy2 = store.get_policy();

        assert_eq!(policy1.max_memory_mb, policy2.max_memory_mb);
    }

    #[tokio::test]
    async fn test_atomic_policy_update() {
        let config = create_test_config();
        let (store, mut rx) = PolicyStore::new(config);

        let mut new_config = create_test_config();
        new_config.max_memory_mb = 1024;

        let version = store.update_policy(new_config).unwrap();
        assert_eq!(version, 2);

        let updated = store.get_policy();
        assert_eq!(updated.max_memory_mb, 1024);

        // Verify update notification
        let update = rx.recv().await.unwrap();
        assert_eq!(update.version, 2);
    }

    #[tokio::test]
    async fn test_version_increments() {
        let config = create_test_config();
        let (store, _rx) = PolicyStore::new(config);

        assert_eq!(store.get_version(), 1);

        let mut new_config = create_test_config();
        new_config.max_memory_mb = 768;

        store.update_policy(new_config).unwrap();
        assert_eq!(store.get_version(), 2);

        let mut new_config = create_test_config();
        new_config.max_memory_mb = 1024;

        store.update_policy(new_config).unwrap();
        assert_eq!(store.get_version(), 3);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let config = create_test_config();
        let (store, _rx) = PolicyStore::new(config);

        let sub1 = store.subscribe_updates();
        let sub2 = store.subscribe_updates();

        let mut new_config = create_test_config();
        new_config.max_memory_mb = 256;

        store.update_policy(new_config).unwrap();

        assert!(sub1.try_recv().is_ok());
        assert!(sub2.try_recv().is_ok());
    }

    #[test]
    fn test_store_clone() {
        let config = create_test_config();
        let (store1, _rx) = PolicyStore::new(config);

        let store2 = store1.clone();

        assert_eq!(store1.get_version(), store2.get_version());
    }
}
