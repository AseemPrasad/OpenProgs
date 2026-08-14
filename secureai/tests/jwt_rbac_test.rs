#[cfg(test)]
mod jwt_rbac_tests {
    use secureai::auth::{
        AuthConfig, JwtClaims, AuthContext, RbacEngine, Role, Permission,
    };
    use std::collections::HashMap;

    // Test 1: JWT Claims construction and parsing
    #[test]
    fn test_jwt_claims_creation() {
        let claims = JwtClaims {
            sub: "user-123".to_string(),
            aud: "api.example.com".to_string(),
            iss: "https://auth.example.com".to_string(),
            exp: 1700000000,
            roles: vec!["admin".to_string(), "audit-reader".to_string()],
            scopes: vec!["read:all".to_string(), "write:policies".to_string()],
            tenant_id: Some("tenant-1".to_string()),
            extra_claims: HashMap::new(),
        };

        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.roles.len(), 2);
        assert_eq!(claims.scopes.len(), 2);
        assert_eq!(claims.tenant_id, Some("tenant-1".to_string()));
    }

    // Test 2: Auth Config defaults
    #[test]
    fn test_auth_config_defaults() {
        let config = AuthConfig::default();

        assert!(!config.enabled);
        assert_eq!(config.jwks_cache_ttl_secs, 3600);
        assert!(config.oidc_discovery_url.is_none());
        assert!(config.audience.is_none());
    }

    // Test 3: Auth Config with values
    #[test]
    fn test_auth_config_with_values() {
        let config = AuthConfig {
            enabled: true,
            oidc_discovery_url: Some("https://auth.example.com".to_string()),
            jwks_cache_ttl_secs: 7200,
            required_roles: vec!["admin".to_string()],
            audience: Some("api.example.com".to_string()),
            issuer: Some("https://auth.example.com".to_string()),
            require_tenant_claim: true,
        };

        assert!(config.enabled);
        assert_eq!(config.jwks_cache_ttl_secs, 7200);
        assert!(config.oidc_discovery_url.is_some());
        assert!(config.require_tenant_claim);
    }

    // Test 4: Auth Context construction
    #[test]
    fn test_auth_context_creation() {
        let roles = vec![Role::PolicyEditor, Role::AuditReader];
        let permissions = Vec::from_iter(
            RbacEngine::permissions_from_roles(&roles),
        );

        let auth_context = AuthContext {
            user_id: "user-456".to_string(),
            tenant_id: "tenant-2".to_string(),
            roles: roles.clone(),
            permissions,
        };

        assert_eq!(auth_context.user_id, "user-456");
        assert_eq!(auth_context.tenant_id, "tenant-2");
        assert_eq!(auth_context.roles, roles);
        assert!(!auth_context.permissions.is_empty());
    }

    // Test 5: RBAC - Admin has all permissions
    #[test]
    fn test_admin_role_has_all_permissions() {
        let permissions = RbacEngine::permissions_for_role(Role::Admin);

        assert!(permissions.contains(&Permission::AdminAll));
        assert!(permissions.contains(&Permission::PolicyWrite));
        assert!(permissions.contains(&Permission::AuditRead));
        assert!(permissions.contains(&Permission::EvalsWrite));
        assert!(permissions.contains(&Permission::QueueManage));
    }

    // Test 6: RBAC - Guest has no permissions
    #[test]
    fn test_guest_role_has_no_permissions() {
        let permissions = RbacEngine::permissions_for_role(Role::Guest);
        assert!(permissions.is_empty());
    }

    // Test 7: RBAC - PolicyEditor permissions
    #[test]
    fn test_policy_editor_permissions() {
        let permissions = RbacEngine::permissions_for_role(Role::PolicyEditor);

        assert!(permissions.contains(&Permission::PolicyRead));
        assert!(permissions.contains(&Permission::PolicyWrite));
        assert!(permissions.contains(&Permission::AuditRead));
        assert!(!permissions.contains(&Permission::EvalsWrite));
        assert!(!permissions.contains(&Permission::AuditWrite));
    }

    // Test 8: RBAC - ToolOperator permissions
    #[test]
    fn test_tool_operator_permissions() {
        let permissions = RbacEngine::permissions_for_role(Role::ToolOperator);

        assert!(permissions.contains(&Permission::ToolsExecute));
        assert!(permissions.contains(&Permission::AuditRead));
        assert!(!permissions.contains(&Permission::ToolsWrite));
        assert!(!permissions.contains(&Permission::PolicyWrite));
    }

    // Test 9: RBAC - EvalsManager permissions
    #[test]
    fn test_evals_manager_permissions() {
        let permissions = RbacEngine::permissions_for_role(Role::EvalsManager);

        assert!(permissions.contains(&Permission::EvalsRead));
        assert!(permissions.contains(&Permission::EvalsWrite));
        assert!(permissions.contains(&Permission::AuditRead));
        assert!(!permissions.contains(&Permission::PolicyWrite));
    }

    // Test 10: RBAC - can_perform single permission
    #[test]
    fn test_rbac_can_perform() {
        assert!(RbacEngine::can_perform(Role::Admin, Permission::PolicyWrite));
        assert!(RbacEngine::can_perform(Role::PolicyEditor, Permission::PolicyRead));
        assert!(!RbacEngine::can_perform(Role::Guest, Permission::PolicyRead));
        assert!(!RbacEngine::can_perform(Role::AuditReader, Permission::PolicyWrite));
    }

    // Test 11: RBAC - roles_from_claims parsing
    #[test]
    fn test_roles_from_claims() {
        let claims = vec![
            "admin".to_string(),
            "audit-reader".to_string(),
            "invalid-role".to_string(),
        ];

        let roles = RbacEngine::roles_from_claims(&claims);

        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&Role::Admin));
        assert!(roles.contains(&Role::AuditReader));
    }

    // Test 12: RBAC - permissions_from_roles (union)
    #[test]
    fn test_permissions_union_from_multiple_roles() {
        let roles = vec![Role::PolicyEditor, Role::AuditReader];
        let permissions = RbacEngine::permissions_from_roles(&roles);

        // Should have union: PolicyRead, PolicyWrite, AuditRead
        assert!(permissions.contains(&Permission::PolicyRead));
        assert!(permissions.contains(&Permission::PolicyWrite));
        assert!(permissions.contains(&Permission::AuditRead));
        assert!(!permissions.contains(&Permission::EvalsWrite));
    }

    // Test 13: RBAC - check_any_permission (at least one)
    #[test]
    fn test_check_any_permission() {
        let roles = vec![Role::AuditReader];
        let required = vec![Permission::PolicyWrite, Permission::AuditRead];

        // Should pass (has AuditRead)
        assert!(RbacEngine::check_any_permission(&roles, &required));
    }

    // Test 14: RBAC - check_any_permission fails
    #[test]
    fn test_check_any_permission_fails() {
        let roles = vec![Role::Guest];
        let required = vec![Permission::PolicyWrite, Permission::AuditRead];

        // Should fail (has no permissions)
        assert!(!RbacEngine::check_any_permission(&roles, &required));
    }

    // Test 15: RBAC - check_all_permissions
    #[test]
    fn test_check_all_permissions() {
        let roles = vec![Role::Admin];
        let required = vec![Permission::PolicyWrite, Permission::AuditRead];

        // Should pass (admin has all)
        assert!(RbacEngine::check_all_permissions(&roles, &required));
    }

    // Test 16: RBAC - check_all_permissions fails
    #[test]
    fn test_check_all_permissions_fails() {
        let roles = vec![Role::AuditReader];
        let required = vec![Permission::PolicyWrite, Permission::AuditRead];

        // Should fail (lacks PolicyWrite)
        assert!(!RbacEngine::check_all_permissions(&roles, &required));
    }

    // Test 17: Permission string conversion
    #[test]
    fn test_permission_as_str() {
        assert_eq!(Permission::PolicyWrite.as_str(), "policy:write");
        assert_eq!(Permission::ToolsExecute.as_str(), "tools:execute");
        assert_eq!(Permission::AuditRead.as_str(), "audit:read");
        assert_eq!(Permission::AdminAll.as_str(), "admin:all");
    }

    // Test 18: Permission from_str parsing
    #[test]
    fn test_permission_from_str() {
        assert_eq!(Permission::from_str("policy:write"), Some(Permission::PolicyWrite));
        assert_eq!(Permission::from_str("tools:execute"), Some(Permission::ToolsExecute));
        assert_eq!(Permission::from_str("admin:all"), Some(Permission::AdminAll));
        assert_eq!(Permission::from_str("invalid"), None);
    }

    // Test 19: Role string conversion
    #[test]
    fn test_role_as_str() {
        assert_eq!(Role::Admin.as_str(), "admin");
        assert_eq!(Role::PolicyEditor.as_str(), "policy-editor");
        assert_eq!(Role::AuditReader.as_str(), "audit-reader");
        assert_eq!(Role::Guest.as_str(), "guest");
    }

    // Test 20: Role from_str parsing
    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from_str("admin"), Some(Role::Admin));
        assert_eq!(Role::from_str("policy-editor"), Some(Role::PolicyEditor));
        assert_eq!(Role::from_str("tool-operator"), Some(Role::ToolOperator));
        assert_eq!(Role::from_str("invalid"), None);
    }

    // Test 21: Multi-tenant isolation with roles
    #[test]
    fn test_multi_tenant_context() {
        let roles = vec![Role::ToolOperator];
        let permissions = Vec::from_iter(
            RbacEngine::permissions_from_roles(&roles),
        );

        let tenant1_context = AuthContext {
            user_id: "user-1".to_string(),
            tenant_id: "tenant-1".to_string(),
            roles: roles.clone(),
            permissions: permissions.clone(),
        };

        let tenant2_context = AuthContext {
            user_id: "user-1".to_string(),
            tenant_id: "tenant-2".to_string(),
            roles: roles.clone(),
            permissions,
        };

        // Same user but different tenants
        assert_ne!(tenant1_context.tenant_id, tenant2_context.tenant_id);
        assert_eq!(tenant1_context.user_id, tenant2_context.user_id);
    }

    // Test 22: Empty permission check with empty required
    #[test]
    fn test_check_empty_permissions() {
        let roles = vec![Role::Guest];
        let required: Vec<Permission> = vec![];

        // Empty requirement should pass
        assert!(RbacEngine::check_any_permission(&roles, &required));
        assert!(RbacEngine::check_all_permissions(&roles, &required));
    }

    // Test 23: Permission combination for complex scenarios
    #[test]
    fn test_complex_role_combination() {
        let roles = vec![
            Role::PolicyEditor,
            Role::EvalsManager,
            Role::AuditReader,
        ];
        let permissions = RbacEngine::permissions_from_roles(&roles);

        // Should have union of all three roles
        assert!(permissions.contains(&Permission::PolicyRead));
        assert!(permissions.contains(&Permission::PolicyWrite));
        assert!(permissions.contains(&Permission::EvalsRead));
        assert!(permissions.contains(&Permission::EvalsWrite));
        assert!(permissions.contains(&Permission::AuditRead));

        // But not write permissions for audit (only AuditReader provides read)
        assert!(!permissions.contains(&Permission::AuditWrite));
    }

    // Test 24: JWT claims with no roles
    #[test]
    fn test_jwt_claims_no_roles() {
        let claims = JwtClaims {
            sub: "user-789".to_string(),
            aud: "api.example.com".to_string(),
            iss: "https://auth.example.com".to_string(),
            exp: 1700000000,
            roles: vec![],
            scopes: vec![],
            tenant_id: None,
            extra_claims: HashMap::new(),
        };

        let roles = RbacEngine::roles_from_claims(&claims.roles);
        assert!(roles.is_empty());

        let permissions = RbacEngine::permissions_from_roles(&roles);
        assert!(permissions.is_empty());
    }

    // Test 25: Tenant ID defaulting in claims
    #[test]
    fn test_tenant_id_defaults_to_default() {
        let claims = JwtClaims {
            sub: "user-999".to_string(),
            aud: "api.example.com".to_string(),
            iss: "https://auth.example.com".to_string(),
            exp: 1700000000,
            roles: vec!["admin".to_string()],
            scopes: vec![],
            tenant_id: None,
            extra_claims: HashMap::new(),
        };

        let roles = RbacEngine::roles_from_claims(&claims.roles);
        let permissions = Vec::from_iter(
            RbacEngine::permissions_from_roles(&roles),
        );

        let context = AuthContext {
            user_id: claims.sub,
            tenant_id: claims.tenant_id.unwrap_or_else(|| "default".to_string()),
            roles,
            permissions,
        };

        assert_eq!(context.tenant_id, "default");
    }
}
