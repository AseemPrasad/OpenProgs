use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod jwt;
pub mod rbac;
pub mod middleware;
pub mod integration;

pub use jwt::JwtValidator;
pub use rbac::{RbacEngine, Permission, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,                          // Subject (user ID)
    pub aud: String,                          // Audience
    pub iss: String,                          // Issuer
    pub exp: i64,                             // Expiration timestamp
    pub roles: Vec<String>,                   // User roles
    pub scopes: Vec<String>,                  // OAuth2 scopes
    pub tenant_id: Option<String>,            // Multi-tenant support
    #[serde(flatten)]
    pub extra_claims: HashMap<String, serde_json::Value>, // Additional claims
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub oidc_discovery_url: Option<String>,

    #[serde(default = "default_jwks_cache_ttl")]
    pub jwks_cache_ttl_secs: u64,

    #[serde(default)]
    pub required_roles: Vec<String>,

    #[serde(default)]
    pub audience: Option<String>,

    #[serde(default)]
    pub issuer: Option<String>,

    #[serde(default)]
    pub require_tenant_claim: bool,
}

fn default_jwks_cache_ttl() -> u64 { 3600 } // 1 hour

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            oidc_discovery_url: None,
            jwks_cache_ttl_secs: default_jwks_cache_ttl(),
            required_roles: Vec::new(),
            audience: None,
            issuer: None,
            require_tenant_claim: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub tenant_id: String,
    pub roles: Vec<Role>,
    pub permissions: Vec<Permission>,
}

pub fn initialize_auth(validator: std::sync::Arc<JwtValidator>) -> anyhow::Result<()> {
    let mut auth = AUTH_VALIDATOR.lock();
    *auth = Some(validator);
    Ok(())
}

pub async fn shutdown_auth() -> anyhow::Result<()> {
    let auth = AUTH_VALIDATOR.lock().take();
    if auth.is_some() {
        tracing::info!("Auth validator shut down");
    }
    Ok(())
}

pub fn get_auth_validator() -> Option<std::sync::Arc<JwtValidator>> {
    AUTH_VALIDATOR.lock().clone()
}

pub fn is_auth_enabled() -> bool {
    AUTH_VALIDATOR.lock().is_some()
}

static AUTH_VALIDATOR: parking_lot::Mutex<Option<std::sync::Arc<JwtValidator>>> =
    parking_lot::Mutex::new(None);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_defaults() {
        let config = AuthConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.jwks_cache_ttl_secs, 3600);
    }

    #[test]
    fn test_auth_disabled() {
        assert!(!is_auth_enabled());
    }

    #[test]
    fn test_jwt_claims_structure() {
        let claims = JwtClaims {
            sub: "user-123".to_string(),
            aud: "api.example.com".to_string(),
            iss: "https://auth.example.com".to_string(),
            exp: 1700000000,
            roles: vec!["admin".to_string()],
            scopes: vec!["read:all".to_string()],
            tenant_id: Some("tenant-1".to_string()),
            extra_claims: HashMap::new(),
        };

        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.roles.len(), 1);
    }
}
