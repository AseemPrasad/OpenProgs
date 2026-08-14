use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheTier {
    ExactMatch,
    SemanticMatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub source_tool: String,
    pub tenant_id: String,
    pub cost_saved: f32,
    pub original_latency_ms: u64,
    pub cache_tier: CacheTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub id: String,
    pub key: String,
    pub query: String,
    pub embedding: Option<Vec<f32>>,
    pub result: Vec<u8>,
    pub metadata: CacheMetadata,
    pub created_at: u64,
    pub last_accessed_at: u64,
    pub access_count: u64,
    pub ttl_seconds: u64,
}

impl CacheEntry {
    pub fn new(
        key: String,
        query: String,
        result: Vec<u8>,
        metadata: CacheMetadata,
        ttl_seconds: u64,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key,
            query,
            embedding: None,
            result,
            metadata,
            created_at: now,
            last_accessed_at: now,
            access_count: 1,
            ttl_seconds,
        }
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    pub fn record_access(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.last_accessed_at = now;
        self.access_count += 1;
    }

    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let age_seconds = (now - self.created_at) / 1_000_000_000;
        age_seconds > self.ttl_seconds
    }
}

#[derive(Debug, Clone)]
pub enum CacheHit {
    ExactMatch {
        entry: CacheEntry,
        latency_ns: u64,
    },
    SemanticMatch {
        entry: CacheEntry,
        similarity: f32,
        latency_ns: u64,
    },
    Miss,
}

impl CacheHit {
    pub fn is_hit(&self) -> bool {
        !matches!(self, CacheHit::Miss)
    }

    pub fn get_latency_ns(&self) -> Option<u64> {
        match self {
            CacheHit::ExactMatch { latency_ns, .. } => Some(*latency_ns),
            CacheHit::SemanticMatch { latency_ns, .. } => Some(*latency_ns),
            CacheHit::Miss => None,
        }
    }
}
