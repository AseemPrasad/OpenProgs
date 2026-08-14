use anyhow::{Result, Context};
use tonic::{Request, Status};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: String,
    pub caller_identity: String,
    pub auth_token: String,
    pub metadata: HashMap<String, String>,
}

impl TenantContext {
    pub fn new(tenant_id: String, caller_identity: String, auth_token: String) -> Self {
        Self {
            tenant_id,
            caller_identity,
            auth_token,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

pub fn extract_tenant_context(metadata: &tonic::metadata::MetadataMap) -> Result<TenantContext> {
    // Extract tenant ID from metadata
    let tenant_id = metadata
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default")
        .to_string();

    // Extract caller identity
    let caller_identity = metadata
        .get("x-caller-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous")
        .to_string();

    // Extract authorization token
    let auth_token = metadata
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let context = TenantContext::new(tenant_id, caller_identity, auth_token);

    Ok(context)
}

pub fn validate_tenant_context(context: &TenantContext) -> Result<()> {
    if context.tenant_id.is_empty() {
        return Err(anyhow::anyhow!("Tenant ID is required"));
    }

    if context.auth_token.is_empty() {
        return Err(anyhow::anyhow!("Authentication token is required"));
    }

    Ok(())
}

pub async fn tenant_interceptor<T>(
    mut req: Request<T>,
) -> Result<Request<T>, Status> {
    let metadata = req.metadata().clone();
    let context = extract_tenant_context(&metadata)
        .map_err(|e| Status::unauthenticated(format!("Failed to extract tenant context: {}", e)))?;

    if validate_tenant_context(&context).is_err() {
        return Err(Status::permission_denied("Invalid tenant context"));
    }

    req.extensions_mut().insert(context);
    Ok(req)
}

pub fn extract_context_from_request<T>(req: &Request<T>) -> Option<TenantContext> {
    req.extensions().get::<TenantContext>().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_context_creation() {
        let context = TenantContext::new(
            "tenant-123".to_string(),
            "agent-1".to_string(),
            "token-abc".to_string(),
        );

        assert_eq!(context.tenant_id, "tenant-123");
        assert_eq!(context.caller_identity, "agent-1");
        assert_eq!(context.auth_token, "token-abc");
    }

    #[test]
    fn test_tenant_context_with_metadata() {
        let context = TenantContext::new(
            "tenant-123".to_string(),
            "agent-1".to_string(),
            "token-abc".to_string(),
        )
        .with_metadata("region".to_string(), "us-east-1".to_string());

        assert_eq!(context.metadata.get("region").unwrap(), "us-east-1");
    }

    #[test]
    fn test_validate_valid_context() {
        let context = TenantContext::new(
            "tenant-123".to_string(),
            "agent-1".to_string(),
            "token-abc".to_string(),
        );

        assert!(validate_tenant_context(&context).is_ok());
    }

    #[test]
    fn test_validate_missing_tenant_id() {
        let context = TenantContext::new(
            "".to_string(),
            "agent-1".to_string(),
            "token-abc".to_string(),
        );

        assert!(validate_tenant_context(&context).is_err());
    }

    #[test]
    fn test_validate_missing_token() {
        let context = TenantContext::new(
            "tenant-123".to_string(),
            "agent-1".to_string(),
            "".to_string(),
        );

        assert!(validate_tenant_context(&context).is_err());
    }
}
