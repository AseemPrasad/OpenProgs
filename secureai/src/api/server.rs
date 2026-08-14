use anyhow::{Result, Context};
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Server;

use crate::policy::{PolicyConfig, PolicyStore};
use super::grpc::PolicyServiceImpl;
use super::policy_watcher::{PolicyWatcher, WatchSource};

#[derive(Debug, Clone)]
pub struct GrpcConfig {
    pub enabled: bool,
    pub addr: String,
    pub policy_source: Option<WatchSource>,
    pub max_concurrent_streams: usize,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            addr: "127.0.0.1:50051".to_string(),
            policy_source: None,
            max_concurrent_streams: 1000,
        }
    }
}

pub struct GrpcServer {
    config: GrpcConfig,
    policy_store: Arc<PolicyStore>,
}

impl GrpcServer {
    pub fn new(config: GrpcConfig, initial_policy: PolicyConfig) -> Self {
        let (store, _rx) = PolicyStore::new(initial_policy);
        let policy_store = Arc::new(store);

        Self {
            config,
            policy_store,
        }
    }

    pub fn with_store(config: GrpcConfig, store: Arc<PolicyStore>) -> Self {
        Self {
            config,
            policy_store: store,
        }
    }

    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("gRPC server disabled");
            return Ok(());
        }

        let addr = self.config.addr.parse()
            .context("Failed to parse gRPC server address")?;

        // Create service
        let (tx, _) = tokio::sync::broadcast::channel(100);
        let service = PolicyServiceImpl::new(
            Arc::new(RwLock::new(self.policy_store.get_policy().as_ref().clone())),
            tx,
        );

        // Create server with configuration
        let mut server = Server::builder()
            .max_concurrent_streams(Some(self.config.max_concurrent_streams as u32));

        // Add service
        server = server.add_service(service.into_server());

        tracing::info!(
            "Starting gRPC PolicyService on {}",
            self.config.addr
        );

        server.serve(addr)
            .await
            .context("gRPC server error")?;

        Ok(())
    }

    pub async fn start_with_watcher(&self) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("gRPC server disabled");
            return Ok(());
        }

        let server_future = tokio::spawn({
            let config = self.config.clone();
            let store = Arc::clone(&self.policy_store);
            async move {
                let server = GrpcServer::with_store(config, store);
                server.start().await
            }
        });

        if let Some(watch_source) = &self.config.policy_source {
            let watcher_future = tokio::spawn({
                let store = Arc::clone(&self.policy_store);
                let source = watch_source.clone();
                async move {
                    let watcher = PolicyWatcher::new(store, source);
                    watcher.watch().await
                }
            });

            tokio::select! {
                server_result = server_future => {
                    server_result?.context("Server task failed")?
                }
                watcher_result = watcher_future => {
                    watcher_result?.context("Watcher task failed")?
                }
            }
        } else {
            server_future.await?.context("Server task failed")?
        }

        Ok(())
    }

    pub fn get_policy_store(&self) -> Arc<PolicyStore> {
        Arc::clone(&self.policy_store)
    }

    pub fn get_config(&self) -> &GrpcConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_config() -> GrpcConfig {
        GrpcConfig {
            enabled: true,
            addr: "127.0.0.1:50052".to_string(),
            policy_source: None,
            max_concurrent_streams: 500,
        }
    }

    fn create_test_policy() -> PolicyConfig {
        PolicyConfig {
            allowed_paths: vec![PathBuf::from("/tmp")],
            network_access: false,
            max_memory_mb: 512,
            allowed_models: vec!["llama3".to_string()],
            isolation: None,
        }
    }

    #[test]
    fn test_grpc_config_default() {
        let config = GrpcConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.addr, "127.0.0.1:50051");
    }

    #[test]
    fn test_grpc_server_creation() {
        let config = create_test_config();
        let policy = create_test_policy();

        let server = GrpcServer::new(config.clone(), policy);

        assert!(server.get_config().enabled);
        assert_eq!(server.get_config().max_concurrent_streams, 500);
    }

    #[test]
    fn test_grpc_server_with_store() {
        let config = create_test_config();
        let policy = create_test_policy();
        let (store, _rx) = PolicyStore::new(policy);

        let server = GrpcServer::with_store(config.clone(), Arc::new(store));

        assert!(server.get_config().enabled);
    }

    #[test]
    fn test_grpc_server_disabled() {
        let mut config = create_test_config();
        config.enabled = false;

        let policy = create_test_policy();
        let server = GrpcServer::new(config, policy);

        assert!(!server.get_config().enabled);
    }

    #[tokio::test]
    async fn test_grpc_server_start_disabled() {
        let mut config = create_test_config();
        config.enabled = false;

        let policy = create_test_policy();
        let server = GrpcServer::new(config, policy);

        assert!(server.start().await.is_ok());
    }
}
