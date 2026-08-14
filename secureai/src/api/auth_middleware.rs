use crate::auth::{AuthContext, RbacEngine, Permission, Role, JwtValidator};
use std::collections::HashMap;
use tonic::{Request, Status};

pub struct AuthMiddleware;

impl AuthMiddleware {
    pub async fn authenticate_request<T>(
        request: &Request<T>,
    ) -> Result<AuthContext, Status> {
        // Extract Authorization header
        let metadata = request.metadata();
        let auth_header = metadata
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing Authorization header"))?;

        // Extract Bearer token
        let token = if auth_header.starts_with("Bearer ") {
            &auth_header[7..]
        } else {
            return Err(Status::unauthenticated("Invalid Authorization header format"));
        };

        // Get validator
        let validator = crate::auth::get_auth_validator()
            .ok_or_else(|| Status::internal("Auth validator not initialized"))?;

        // Validate token
        let claims = validator
            .validate_token(token)
            .await
            .map_err(|e| {
                tracing::warn!("JWT validation failed: {}", e);
                Status::unauthenticated("Invalid JWT token")
            })?;

        // Parse roles from claims
        let roles = RbacEngine::roles_from_claims(&claims.roles);

        // Build auth context
        let auth_context = AuthContext {
            user_id: claims.sub,
            tenant_id: claims.tenant_id.unwrap_or_else(|| "default".to_string()),
            roles,
            permissions: Vec::from_iter(
                RbacEngine::permissions_from_roles(&roles),
            ),
        };

        Ok(auth_context)
    }

    pub fn check_permission(
        auth_context: &AuthContext,
        required_permission: Permission,
    ) -> Result<(), Status> {
        if auth_context.permissions.contains(&required_permission) {
            Ok(())
        } else {
            Err(Status::permission_denied(format!(
                "User lacks required permission: {}",
                required_permission.as_str()
            )))
        }
    }

    pub fn check_any_permission(
        auth_context: &AuthContext,
        required_permissions: &[Permission],
    ) -> Result<(), Status> {
        if required_permissions.is_empty() {
            return Ok(());
        }

        if required_permissions
            .iter()
            .any(|p| auth_context.permissions.contains(p))
        {
            Ok(())
        } else {
            Err(Status::permission_denied("User lacks any of the required permissions"))
        }
    }

    pub fn check_all_permissions(
        auth_context: &AuthContext,
        required_permissions: &[Permission],
    ) -> Result<(), Status> {
        if required_permissions.is_empty() {
            return Ok(());
        }

        if required_permissions
            .iter()
            .all(|p| auth_context.permissions.contains(p))
        {
            Ok(())
        } else {
            Err(Status::permission_denied("User lacks all required permissions"))
        }
    }

    pub fn check_role(
        auth_context: &AuthContext,
        required_role: Role,
    ) -> Result<(), Status> {
        if auth_context.roles.contains(&required_role) {
            Ok(())
        } else {
            Err(Status::permission_denied(format!(
                "User does not have required role: {}",
                required_role.as_str()
            )))
        }
    }

    pub fn check_tenant_access(
        auth_context: &AuthContext,
        target_tenant: &str,
    ) -> Result<(), Status> {
        if auth_context.tenant_id == target_tenant || auth_context.roles.contains(&Role::Admin) {
            Ok(())
        } else {
            Err(Status::permission_denied("User not authorized for this tenant"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataMap;

    fn create_auth_context(
        roles: Vec<Role>,
    ) -> AuthContext {
        AuthContext {
            user_id: "test-user".to_string(),
            tenant_id: "tenant-1".to_string(),
            roles: roles.clone(),
            permissions: Vec::from_iter(
                RbacEngine::permissions_from_roles(&roles),
            ),
        }
    }

    #[test]
    fn test_check_permission_allowed() {
        let ctx = create_auth_context(vec![Role::PolicyEditor]);
        let result = AuthMiddleware::check_permission(&ctx, Permission::PolicyWrite);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_permission_denied() {
        let ctx = create_auth_context(vec![Role::Guest]);
        let result = AuthMiddleware::check_permission(&ctx, Permission::PolicyWrite);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_any_permission_allowed() {
        let ctx = create_auth_context(vec![Role::AuditReader]);
        let result = AuthMiddleware::check_any_permission(
            &ctx,
            &[Permission::PolicyWrite, Permission::AuditRead],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_all_permissions_allowed() {
        let ctx = create_auth_context(vec![Role::Admin]);
        let result = AuthMiddleware::check_all_permissions(
            &ctx,
            &[Permission::PolicyWrite, Permission::AuditRead],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_all_permissions_denied() {
        let ctx = create_auth_context(vec![Role::AuditReader]);
        let result = AuthMiddleware::check_all_permissions(
            &ctx,
            &[Permission::PolicyWrite, Permission::AuditRead],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_check_role_allowed() {
        let ctx = create_auth_context(vec![Role::PolicyEditor]);
        let result = AuthMiddleware::check_role(&ctx, Role::PolicyEditor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_role_denied() {
        let ctx = create_auth_context(vec![Role::Guest]);
        let result = AuthMiddleware::check_role(&ctx, Role::Admin);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_tenant_access_allowed() {
        let ctx = create_auth_context(vec![Role::ToolOperator]);
        let result = AuthMiddleware::check_tenant_access(&ctx, "tenant-1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_tenant_access_denied() {
        let ctx = create_auth_context(vec![Role::ToolOperator]);
        let result = AuthMiddleware::check_tenant_access(&ctx, "tenant-2");
        assert!(result.is_err());
    }

    #[test]
    fn test_check_tenant_access_admin_bypass() {
        let ctx = create_auth_context(vec![Role::Admin]);
        let result = AuthMiddleware::check_tenant_access(&ctx, "tenant-2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_auth_context_creation() {
        let ctx = create_auth_context(vec![Role::PolicyEditor, Role::AuditReader]);
        assert_eq!(ctx.user_id, "test-user");
        assert_eq!(ctx.tenant_id, "tenant-1");
        assert_eq!(ctx.roles.len(), 2);
    }
}
