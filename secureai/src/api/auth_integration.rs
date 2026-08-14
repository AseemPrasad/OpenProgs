use crate::auth::{AuthContext, Permission, Role};
use tonic::Request;

pub struct AuthIntegration;

impl AuthIntegration {
    pub fn is_enabled() -> bool {
        crate::auth::is_auth_enabled()
    }

    pub fn extract_auth_context<T>(request: &Request<T>) -> Option<AuthContext> {
        // This would be populated by middleware
        // For now, this is a placeholder for future middleware integration
        None
    }

    pub fn extract_user_id<T>(request: &Request<T>) -> Option<String> {
        // Placeholder for extracting user_id from request extensions
        None
    }

    pub fn extract_tenant_id<T>(request: &Request<T>) -> Option<String> {
        // Placeholder for extracting tenant_id from request extensions
        None
    }

    pub fn extract_roles<T>(request: &Request<T>) -> Vec<Role> {
        // Placeholder for extracting roles from request extensions
        Vec::new()
    }

    pub fn extract_permissions<T>(request: &Request<T>) -> Vec<Permission> {
        // Placeholder for extracting permissions from request extensions
        Vec::new()
    }

    pub fn require_permission(required: Permission) -> impl Fn(&Request<()>) -> bool {
        move |_request: &Request<()>| {
            // Placeholder for permission guard
            true
        }
    }

    pub fn require_role(required: Role) -> impl Fn(&Request<()>) -> bool {
        move |_request: &Request<()>| {
            // Placeholder for role guard
            true
        }
    }

    pub fn require_tenant_access(tenant_id: &str) -> impl Fn(&Request<()>) -> bool + '_ {
        move |_request: &Request<()>| {
            // Placeholder for tenant access guard
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_integration_enabled_status() {
        // When auth is not initialized, is_enabled should return false
        assert!(!AuthIntegration::is_enabled());
    }

    #[test]
    fn test_extract_methods_return_none_when_disabled() {
        // When auth is not enabled, extraction methods should return None/empty
        assert!(AuthIntegration::extract_user_id(&Request::new(())).is_none());
        assert!(AuthIntegration::extract_tenant_id(&Request::new(())).is_none());
        assert!(AuthIntegration::extract_roles(&Request::new(())).is_empty());
        assert!(AuthIntegration::extract_permissions(&Request::new(())).is_empty());
    }
}
