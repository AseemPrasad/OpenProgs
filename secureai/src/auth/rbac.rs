use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    ToolsExecute,
    ToolsWrite,
    PolicyRead,
    PolicyWrite,
    PolicyDelete,
    AuditRead,
    AuditWrite,
    QueueManage,
    CacheManage,
    EvalsRead,
    EvalsWrite,
    AdminAll,
}

impl Permission {
    pub fn as_str(&self) -> &str {
        match self {
            Permission::ToolsExecute => "tools:execute",
            Permission::ToolsWrite => "tools:write",
            Permission::PolicyRead => "policy:read",
            Permission::PolicyWrite => "policy:write",
            Permission::PolicyDelete => "policy:delete",
            Permission::AuditRead => "audit:read",
            Permission::AuditWrite => "audit:write",
            Permission::QueueManage => "queue:manage",
            Permission::CacheManage => "cache:manage",
            Permission::EvalsRead => "evals:read",
            Permission::EvalsWrite => "evals:write",
            Permission::AdminAll => "admin:all",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "tools:execute" => Some(Permission::ToolsExecute),
            "tools:write" => Some(Permission::ToolsWrite),
            "policy:read" => Some(Permission::PolicyRead),
            "policy:write" => Some(Permission::PolicyWrite),
            "policy:delete" => Some(Permission::PolicyDelete),
            "audit:read" => Some(Permission::AuditRead),
            "audit:write" => Some(Permission::AuditWrite),
            "queue:manage" => Some(Permission::QueueManage),
            "cache:manage" => Some(Permission::CacheManage),
            "evals:read" => Some(Permission::EvalsRead),
            "evals:write" => Some(Permission::EvalsWrite),
            "admin:all" => Some(Permission::AdminAll),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    PolicyEditor,
    AuditReader,
    ToolOperator,
    EvalsManager,
    Guest,
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Role::Admin => "admin",
            Role::PolicyEditor => "policy-editor",
            Role::AuditReader => "audit-reader",
            Role::ToolOperator => "tool-operator",
            Role::EvalsManager => "evals-manager",
            Role::Guest => "guest",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(Role::Admin),
            "policy-editor" => Some(Role::PolicyEditor),
            "audit-reader" => Some(Role::AuditReader),
            "tool-operator" => Some(Role::ToolOperator),
            "evals-manager" => Some(Role::EvalsManager),
            "guest" => Some(Role::Guest),
            _ => None,
        }
    }
}

pub struct RbacEngine;

impl RbacEngine {
    pub fn can_perform(role: Role, permission: Permission) -> bool {
        let permissions = Self::permissions_for_role(role);
        permissions.contains(&permission)
    }

    pub fn permissions_for_role(role: Role) -> HashSet<Permission> {
        let mut perms = HashSet::new();

        match role {
            Role::Admin => {
                perms.insert(Permission::AdminAll);
                perms.insert(Permission::ToolsExecute);
                perms.insert(Permission::ToolsWrite);
                perms.insert(Permission::PolicyRead);
                perms.insert(Permission::PolicyWrite);
                perms.insert(Permission::PolicyDelete);
                perms.insert(Permission::AuditRead);
                perms.insert(Permission::AuditWrite);
                perms.insert(Permission::QueueManage);
                perms.insert(Permission::CacheManage);
                perms.insert(Permission::EvalsRead);
                perms.insert(Permission::EvalsWrite);
            }
            Role::PolicyEditor => {
                perms.insert(Permission::PolicyRead);
                perms.insert(Permission::PolicyWrite);
                perms.insert(Permission::AuditRead);
            }
            Role::AuditReader => {
                perms.insert(Permission::AuditRead);
            }
            Role::ToolOperator => {
                perms.insert(Permission::ToolsExecute);
                perms.insert(Permission::AuditRead);
            }
            Role::EvalsManager => {
                perms.insert(Permission::EvalsRead);
                perms.insert(Permission::EvalsWrite);
                perms.insert(Permission::AuditRead);
            }
            Role::Guest => {
                // Guest has no permissions
            }
        }

        perms
    }

    pub fn roles_from_claims(role_claims: &[String]) -> Vec<Role> {
        role_claims
            .iter()
            .filter_map(|r| Role::from_str(r))
            .collect()
    }

    pub fn permissions_from_roles(roles: &[Role]) -> HashSet<Permission> {
        let mut combined = HashSet::new();

        for role in roles {
            let role_perms = Self::permissions_for_role(*role);
            combined.extend(role_perms);
        }

        combined
    }

    pub fn check_any_permission(
        roles: &[Role],
        required_permissions: &[Permission],
    ) -> bool {
        let available = Self::permissions_from_roles(roles);

        required_permissions
            .iter()
            .any(|p| available.contains(p))
    }

    pub fn check_all_permissions(
        roles: &[Role],
        required_permissions: &[Permission],
    ) -> bool {
        let available = Self::permissions_from_roles(roles);

        required_permissions
            .iter()
            .all(|p| available.contains(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_has_all_permissions() {
        assert!(RbacEngine::can_perform(Role::Admin, Permission::AdminAll));
        assert!(RbacEngine::can_perform(Role::Admin, Permission::PolicyWrite));
        assert!(RbacEngine::can_perform(Role::Admin, Permission::AuditRead));
    }

    #[test]
    fn test_guest_has_no_permissions() {
        assert!(!RbacEngine::can_perform(Role::Guest, Permission::ToolsExecute));
        assert!(!RbacEngine::can_perform(Role::Guest, Permission::PolicyWrite));
    }

    #[test]
    fn test_policy_editor_permissions() {
        assert!(RbacEngine::can_perform(Role::PolicyEditor, Permission::PolicyRead));
        assert!(RbacEngine::can_perform(Role::PolicyEditor, Permission::PolicyWrite));
        assert!(!RbacEngine::can_perform(Role::PolicyEditor, Permission::AuditWrite));
    }

    #[test]
    fn test_audit_reader_permissions() {
        assert!(RbacEngine::can_perform(Role::AuditReader, Permission::AuditRead));
        assert!(!RbacEngine::can_perform(Role::AuditReader, Permission::AuditWrite));
        assert!(!RbacEngine::can_perform(Role::AuditReader, Permission::PolicyWrite));
    }

    #[test]
    fn test_tool_operator_permissions() {
        assert!(RbacEngine::can_perform(Role::ToolOperator, Permission::ToolsExecute));
        assert!(RbacEngine::can_perform(Role::ToolOperator, Permission::AuditRead));
        assert!(!RbacEngine::can_perform(Role::ToolOperator, Permission::ToolsWrite));
    }

    #[test]
    fn test_evals_manager_permissions() {
        assert!(RbacEngine::can_perform(Role::EvalsManager, Permission::EvalsRead));
        assert!(RbacEngine::can_perform(Role::EvalsManager, Permission::EvalsWrite));
        assert!(!RbacEngine::can_perform(Role::EvalsManager, Permission::PolicyWrite));
    }

    #[test]
    fn test_roles_from_claims() {
        let claims = vec!["admin".to_string(), "audit-reader".to_string()];
        let roles = RbacEngine::roles_from_claims(&claims);
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&Role::Admin));
        assert!(roles.contains(&Role::AuditReader));
    }

    #[test]
    fn test_permissions_from_roles() {
        let roles = vec![Role::PolicyEditor, Role::AuditReader];
        let perms = RbacEngine::permissions_from_roles(&roles);

        // Should have union of both roles
        assert!(perms.contains(&Permission::PolicyRead));
        assert!(perms.contains(&Permission::PolicyWrite));
        assert!(perms.contains(&Permission::AuditRead));
    }

    #[test]
    fn test_check_any_permission() {
        let roles = vec![Role::AuditReader];
        let required = vec![Permission::PolicyWrite, Permission::AuditRead];

        // Should pass (has AuditRead)
        assert!(RbacEngine::check_any_permission(&roles, &required));
    }

    #[test]
    fn test_check_all_permissions() {
        let roles = vec![Role::Admin];
        let required = vec![Permission::PolicyWrite, Permission::AuditRead];

        // Should pass (admin has all)
        assert!(RbacEngine::check_all_permissions(&roles, &required));
    }

    #[test]
    fn test_check_all_permissions_insufficient() {
        let roles = vec![Role::AuditReader];
        let required = vec![Permission::PolicyWrite, Permission::AuditRead];

        // Should fail (lacks PolicyWrite)
        assert!(!RbacEngine::check_all_permissions(&roles, &required));
    }

    #[test]
    fn test_permission_string_conversion() {
        assert_eq!(Permission::PolicyWrite.as_str(), "policy:write");
        assert_eq!(Permission::from_str("policy:write"), Some(Permission::PolicyWrite));
        assert_eq!(Permission::from_str("invalid"), None);
    }

    #[test]
    fn test_role_string_conversion() {
        assert_eq!(Role::Admin.as_str(), "admin");
        assert_eq!(Role::from_str("admin"), Some(Role::Admin));
        assert_eq!(Role::from_str("invalid"), None);
    }
}
