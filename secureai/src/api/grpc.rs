use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

include!(concat!(env!("OUT_DIR"), "/secureai.policy.rs"));

use policy_service_server::PolicyService;
use policy_service_server::PolicyServiceServer;

use crate::policy::PolicyConfig;
use crate::api::evals_integration::EvalsIntegration;
use crate::api::auth_middleware::AuthMiddleware;
use crate::auth::Permission;

pub struct PolicyServiceImpl {
    policy_store: Arc<RwLock<PolicyConfig>>,
    update_sender: tokio::sync::broadcast::Sender<PolicyUpdate>,
    guardrail: Option<Arc<crate::guardrails::SemanticGuardrail>>,
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

        // Run semantic guardrail check if available (via guardrail member)
        if let Some(guardrail) = &self.guardrail {
            if let Err(e) = guardrail.check_prompt(&req.tool_name).await {
                return Ok(Response::new(EvaluatePolicyResponse {
                    decision: EvaluatePolicyResponse::Decision::Denied as i32,
                    rule_id: "semantic-guardrail".to_string(),
                    reason: format!("Semantic guardrail triggered: {}", e),
                    metadata: std::collections::HashMap::new(),
                }));
            }
        }

        // Optional: Check OAuth2/OIDC authentication if enabled
        if crate::auth::is_auth_enabled() {
            match AuthMiddleware::authenticate_request(&request).await {
                Ok(auth_context) => {
                    // Check if user has permission to execute tools
                    if let Err(status) = AuthMiddleware::check_permission(&auth_context, Permission::ToolsExecute) {
                        let mut err_metadata = std::collections::HashMap::new();
                        err_metadata.insert("user_id".to_string(), auth_context.user_id);
                        return Ok(Response::new(EvaluatePolicyResponse {
                            decision: EvaluatePolicyResponse::Decision::Denied as i32,
                            rule_id: "auth-permission-denied".to_string(),
                            reason: status.message().to_string(),
                            metadata: err_metadata,
                        }));
                    }
                    // Store auth context in extensions for downstream handlers
                    // Note: This is for future use, not currently propagated in gRPC
                    tracing::debug!("Request authenticated: user={}, tenant={}",
                                    auth_context.user_id, auth_context.tenant_id);
                }
                Err(status) => {
                    return Ok(Response::new(EvaluatePolicyResponse {
                        decision: EvaluatePolicyResponse::Decision::Denied as i32,
                        rule_id: "auth-failed".to_string(),
                        reason: status.message().to_string(),
                        metadata: std::collections::HashMap::new(),
                    }));
                }
            }
        }

        // Evaluate policy for the request
        let decision = if policy.allowed_models.contains(&req.tool_name) {
            EvaluatePolicyResponse::Decision::Allowed
        } else {
            EvaluatePolicyResponse::Decision::Denied
        };

        let mut metadata = std::collections::HashMap::new();

        // Enqueue for evaluation if policy allows and evals is enabled
        if decision == EvaluatePolicyResponse::Decision::Allowed {
            let prompt = req.context.get("prompt").map(|s| s.as_str()).unwrap_or("");
            let response_text = req.context.get("response").map(|s| s.as_str()).unwrap_or("");

            if let Err(e) = EvalsIntegration::evaluate_request_async(
                &req.tenant_id,
                &req.tool_name,
                prompt,
                response_text,
                &req.context,
            )
            .await
            {
                tracing::warn!("Failed to enqueue evaluation: {}", e);
            }
        }

        // Check for drift alerts and include in metadata
        let alerts = EvalsIntegration::get_drift_alerts();
        if !alerts.is_empty() {
            metadata.insert(
                "drift_alerts_count".to_string(),
                alerts.len().to_string(),
            );
            for (idx, alert) in alerts.iter().enumerate() {
                metadata.insert(
                    format!("drift_alert_{}", idx),
                    alert.description(),
                );
            }
        }

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
            metadata,
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
            guardrail: None,
        }
    }

    pub fn with_guardrail(
        policy_store: Arc<RwLock<PolicyConfig>>,
        update_sender: tokio::sync::broadcast::Sender<PolicyUpdate>,
        guardrail: Arc<crate::guardrails::SemanticGuardrail>,
    ) -> Self {
        Self {
            policy_store,
            update_sender,
            guardrail: Some(guardrail),
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
