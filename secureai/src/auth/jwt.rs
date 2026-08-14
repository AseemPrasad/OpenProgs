use crate::auth::{AuthConfig, JwtClaims};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use lru::LruCache;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_TIMESTAMP};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonWebKey {
    pub kty: String,
    pub kid: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
    pub x5t: Option<String>,
    pub x5c: Option<Vec<String>>,
    pub alg: Option<String>,
    pub use_: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwksResponse {
    pub keys: Vec<JsonWebKey>,
}

struct CachedJwks {
    jwks: JwksResponse,
    fetched_at: u64,
}

pub struct JwksCache {
    cache: Arc<RwLock<LruCache<String, CachedJwks>>>,
    ttl_secs: u64,
}

impl JwksCache {
    pub fn new(capacity: usize, ttl_secs: u64) -> Self {
        let cache = LruCache::new(NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(10).unwrap()));
        Self {
            cache: Arc::new(RwLock::new(cache)),
            ttl_secs,
        }
    }

    pub async fn get_or_fetch(&self, issuer_url: &str) -> anyhow::Result<JwksResponse> {
        // Check cache
        {
            let cache = self.cache.read();
            if let Some(cached) = cache.peek(issuer_url) {
                let now = SystemTime::now()
                    .duration_since(UNIX_TIMESTAMP)
                    .unwrap_or_default()
                    .as_secs();

                if now - cached.fetched_at < self.ttl_secs {
                    return Ok(cached.jwks.clone());
                }
            }
        }

        // Fetch from issuer
        let discovery_url = format!("{}/.well-known/openid-configuration", issuer_url.trim_end_matches('/'));
        let client = reqwest::Client::new();

        let discovery: DiscoveryDocument = client
            .get(&discovery_url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch OIDC discovery: {}", e))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse discovery document: {}", e))?;

        let jwks: JwksResponse = client
            .get(&discovery.jwks_uri)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch JWKS: {}", e))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse JWKS: {}", e))?;

        // Cache result
        let now = SystemTime::now()
            .duration_since(UNIX_TIMESTAMP)
            .unwrap_or_default()
            .as_secs();

        {
            let mut cache = self.cache.write();
            cache.put(
                issuer_url.to_string(),
                CachedJwks {
                    jwks: jwks.clone(),
                    fetched_at: now,
                },
            );
        }

        Ok(jwks)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DiscoveryDocument {
    jwks_uri: String,
    issuer: String,
}

pub struct JwtValidator {
    cache: JwksCache,
    config: AuthConfig,
}

impl JwtValidator {
    pub async fn new(config: AuthConfig) -> anyhow::Result<Self> {
        if !config.enabled {
            return Ok(Self {
                cache: JwksCache::new(10, config.jwks_cache_ttl_secs),
                config,
            });
        }

        let _ = config.oidc_discovery_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("oidc_discovery_url required when auth is enabled"))?;

        Ok(Self {
            cache: JwksCache::new(10, config.jwks_cache_ttl_secs),
            config,
        })
    }

    pub async fn validate_token(&self, token: &str) -> anyhow::Result<JwtClaims> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Auth not enabled"));
        }

        let issuer = self.config.issuer.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Issuer not configured"))?;

        // Decode header to find key ID
        let header = decode_header(token)
            .map_err(|e| anyhow::anyhow!("Invalid token header: {}", e))?;

        let kid = header.kid.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Token missing kid (key ID) in header"))?;

        // Fetch JWKS
        let discovery_url = self.config.oidc_discovery_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OIDC discovery URL not configured"))?;

        let jwks = self.cache.get_or_fetch(discovery_url).await?;

        // Find key
        let key = jwks.keys.iter()
            .find(|k| k.kid.as_ref().map(|id| id == kid).unwrap_or(false))
            .ok_or_else(|| anyhow::anyhow!("Key not found in JWKS: {}", kid))?;

        // Build decoding key
        let n = key.n.as_ref()
            .ok_or_else(|| anyhow::anyhow!("RSA public key missing 'n' component"))?;
        let e = key.e.as_ref()
            .ok_or_else(|| anyhow::anyhow!("RSA public key missing 'e' component"))?;

        let decoding_key = DecodingKey::from_rsa_components(n, e)
            .map_err(|e| anyhow::anyhow!("Failed to build decoding key: {}", e))?;

        // Build validation rules
        let mut validation = Validation::new(Algorithm::RS256);

        if let Some(aud) = &self.config.audience {
            validation.set_audience(&[aud]);
        }

        if let Some(iss) = &self.config.issuer {
            validation.set_issuer(&[iss]);
        }

        // Decode and validate
        let token_data = decode::<JwtClaims>(token, &decoding_key, &validation)
            .map_err(|e| anyhow::anyhow!("JWT validation failed: {}", e))?;

        Ok(token_data.claims)
    }

    pub fn get_config(&self) -> &AuthConfig {
        &self.config
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jwt_validator_disabled() {
        let config = AuthConfig::default();
        let validator = JwtValidator::new(config).await;
        assert!(validator.is_ok());
        assert!(!validator.unwrap().is_enabled());
    }

    #[test]
    fn test_jwks_cache_creation() {
        let cache = JwksCache::new(5, 3600);
        assert_eq!(cache.ttl_secs, 3600);
    }

    #[test]
    fn test_discovery_document_parsing() {
        let json = r#"{"jwks_uri": "https://auth.example.com/.well-known/jwks.json", "issuer": "https://auth.example.com"}"#;
        let doc: DiscoveryDocument = serde_json::from_str(json).unwrap();
        assert_eq!(doc.jwks_uri, "https://auth.example.com/.well-known/jwks.json");
    }

    #[test]
    fn test_json_web_key_parsing() {
        let json = r#"{"kty": "RSA", "kid": "key-1", "n": "n_value", "e": "AQAB"}"#;
        let key: JsonWebKey = serde_json::from_str(json).unwrap();
        assert_eq!(key.kty, "RSA");
        assert_eq!(key.kid, Some("key-1".to_string()));
    }
}
