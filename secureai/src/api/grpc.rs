use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

include!(concat!(env!("OUT_DIR"), "/secureai.policy.rs"));

use policy_service_server::PolicyService;
use policy_service_server::PolicyServiceServer;

use crate::policy::PolicyConfig;

pub struct PolicyServiceImpl {
    policy_store: Arc<RwLock<PolicyConfig>>,
    update_sender: tokio::sync::broadcast::Sender<PolicyUpdate>,
}

#[tonic::async_trait]
impl PolicyService for PolicyServiceImpl {
    async fn evaluate_policy(
        &self,
        request: Request<EvaluatePolicyRequest>,
    ) -> Result<Response<EvaluatePolicyResponse>, Status> {
        let req = request.into_inner();

        // Read policy without blocking writes
        let policy = self.policy_store.read().await;

        // Evaluate policy for the request
        let decision = if policy.allowed_models.contains(&req.tool_name) {
            EvaluatePolicyResponse::Decision::Allowed
        } else {
            EvaluatePolicyResponse::Decision::Denied
        };

        let response = EvaluatePolicyResponse {
            decision: decision as i32,
            rule_id: format!("rule-{}-{}", req.tenant_id, req.tool_name),
            reason: match decision {
                EvaluatePolicyResponse::Decision::Allowed => {
                    format!("Tool '{}' is allowed", req.tool_name)
                }
                EvaluatePolicyResponse::Decision::Denied => {
                    format!("Tool '{}' is not in allowed_models", req.tool_name)
                }
                EvaluatePolicyResponse::Decision::RequiresApproval => {
                    format!("Tool '{}' requires human approval", req.tool_name)
                }
            },
            metadata: std::collections::HashMap::new(),
        };

        Ok(Response::new(response))
    }

    type WatchPolicyUpdatesStream =
        tokio::sync::broadcast::Receiver<PolicyUpdate>;

    async fn watch_policy_updates(
        &self,
        _request: Request<google::protobuf::Empty>,
    ) -> Result<Response<Self::WatchPolicyUpdatesStream>, Status> {
        let receiver = self.update_sender.subscribe();
        Ok(Response::new(receiver))
    }
}

impl PolicyServiceImpl {
    pub fn new(
        policy_store: Arc<RwLock<PolicyConfig>>,
        update_sender: tokio::sync::broadcast::Sender<PolicyUpdate>,
    ) -> Self {
        Self {
            policy_store,
            update_sender,
        }
    }

    pub fn into_server(self) -> PolicyServiceServer<Self> {
        PolicyServiceServer::new(self)
    }

    pub async fn update_policy(&self, new_config: PolicyConfig) -> Result<()> {
        let mut policy = self.policy_store.write().await;
        *policy = new_config.clone();

        let update = PolicyUpdate {
            tenant_id: "default".to_string(),
            policy_config: vec![],
            version: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        };

        let _ = self.update_sender.send(update);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_grpc_service_creation() {
        let policy = PolicyConfig {
            allowed_paths: vec![],
            network_access: false,
            max_memory_mb: 512,
            allowed_models: vec!["llama3".to_string()],
            isolation: None,
        };

        let (tx, _) = tokio::sync::broadcast::channel(100);
        let service = PolicyServiceImpl::new(Arc::new(RwLock::new(policy)), tx);

        assert!(service.into_server().check_service().is_ok());
    }

    #[tokio::test]
    async fn test_evaluate_allowed_model() {
        let policy = PolicyConfig {
            allowed_paths: vec![],
            network_access: false,
            max_memory_mb: 512,
            allowed_models: vec!["llama3".to_string()],
            isolation: None,
        };

        let (tx, _) = tokio::sync::broadcast::channel(100);
        let service = PolicyServiceImpl::new(Arc::new(RwLock::new(policy)), tx);

        let request = Request::new(EvaluatePolicyRequest {
            tenant_id: "tenant-1".to_string(),
            caller_identity: "agent-1".to_string(),
            tool_name: "llama3".to_string(),
            target_path: "/data".to_string(),
            context: std::collections::HashMap::new(),
        });

        let response = service.evaluate_policy(request).await;
        assert!(response.is_ok());

        let resp = response.unwrap().into_inner();
        assert_eq!(resp.decision, EvaluatePolicyResponse::Decision::Allowed as i32);
    }

    #[tokio::test]
    async fn test_evaluate_denied_model() {
        let policy = PolicyConfig {
            allowed_paths: vec![],
            network_access: false,
            max_memory_mb: 512,
            allowed_models: vec!["llama3".to_string()],
            isolation: None,
        };

        let (tx, _) = tokio::sync::broadcast::channel(100);
        let service = PolicyServiceImpl::new(Arc::new(RwLock::new(policy)), tx);

        let request = Request::new(EvaluatePolicyRequest {
            tenant_id: "tenant-1".to_string(),
            caller_identity: "agent-1".to_string(),
            tool_name: "forbidden-model".to_string(),
            target_path: "/data".to_string(),
            context: std::collections::HashMap::new(),
        });

        let response = service.evaluate_policy(request).await;
        assert!(response.is_ok());

        let resp = response.unwrap().into_inner();
        assert_eq!(resp.decision, EvaluatePolicyResponse::Decision::Denied as i32);
    }
}
