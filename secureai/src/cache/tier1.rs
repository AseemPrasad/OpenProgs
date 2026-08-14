use crate::cache::CacheEntry;
use moka::future::Cache;
use sha2::{Sha256, Digest};
use std::time::Duration;

pub struct ExactMatchCache {
    cache: Cache<String, CacheEntry>,
    max_capacity: u64,
    ttl: Duration,
}

impl ExactMatchCache {
    pub fn new(max_capacity: u64, ttl_seconds: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();

        Self {
            cache,
            max_capacity,
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn put(&self, entry: CacheEntry) {
        let key = entry.key.clone();
        self.cache.insert(key, entry).await;
    }

    pub async fn get(&self, key: &str) -> Option<CacheEntry> {
        self.cache.get(key).await
    }

    pub fn compute_key(query: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(query.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub async fn invalidate(&self, key: &str) {
        self.cache.remove(key).await;
    }

    pub async fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }

    pub async fn get_entry_count(&self) -> u64 {
        self.cache.entry_count()
    }

    pub fn get_max_capacity(&self) -> u64 {
        self.max_capacity
    }
}
