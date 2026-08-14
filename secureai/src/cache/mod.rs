pub mod entry;
pub mod tier1;
pub mod tier2;
pub mod manager;
pub mod integration;

#[allow(dead_code)]
pub mod examples;

pub use integration::CacheIntegration;

pub use entry::{CacheEntry, CacheMetadata, CacheTier, CacheHit};
pub use tier1::ExactMatchCache;
pub use tier2::SemanticCache;
pub use manager::{CacheManager, CacheStats, initialize_cache, shutdown_cache, get_cache, is_cache_enabled};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_tier1_enabled")]
    pub tier1_enabled: bool,

    #[serde(default = "default_tier2_enabled")]
    pub tier2_enabled: bool,

    #[serde(default = "default_tier1_max")]
    pub tier1_max_capacity: u64,

    #[serde(default = "default_tier1_ttl")]
    pub tier1_ttl_seconds: u64,

    #[serde(default = "default_tier2_max")]
    pub tier2_max_entries: usize,

    #[serde(default = "default_tier2_ttl")]
    pub tier2_ttl_seconds: u64,

    #[serde(default = "default_threshold")]
    pub similarity_threshold: f32,

    #[serde(default = "default_model")]
    pub embedding_model: String,
}

fn default_tier1_enabled() -> bool {
    true
}

fn default_tier2_enabled() -> bool {
    true
}

fn default_tier1_max() -> u64 {
    10_000
}

fn default_tier1_ttl() -> u64 {
    3600
}

fn default_tier2_max() -> usize {
    1000
}

fn default_tier2_ttl() -> u64 {
    3600
}

fn default_threshold() -> f32 {
    0.05
}

fn default_model() -> String {
    "onnx".to_string()
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            tier1_enabled: default_tier1_enabled(),
            tier2_enabled: default_tier2_enabled(),
            tier1_max_capacity: default_tier1_max(),
            tier1_ttl_seconds: default_tier1_ttl(),
            tier2_max_entries: default_tier2_max(),
            tier2_ttl_seconds: default_tier2_ttl(),
            similarity_threshold: default_threshold(),
            embedding_model: default_model(),
        }
    }
}
