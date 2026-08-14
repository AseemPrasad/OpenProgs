use crate::cache::{CacheEntry, CacheMetadata, CacheTier, CacheHit, CacheConfig, ExactMatchCache, SemanticCache};
use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub tier1_hits: u64,
    pub tier2_hits: u64,
    pub total_misses: u64,
    pub total_requests: u64,
    pub avg_tier1_latency_ns: f64,
    pub avg_tier2_latency_ns: f64,
    pub hit_rate: f32,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            tier1_hits: 0,
            tier2_hits: 0,
            total_misses: 0,
            total_requests: 0,
            avg_tier1_latency_ns: 0.0,
            avg_tier2_latency_ns: 0.0,
            hit_rate: 0.0,
        }
    }
}

pub struct CacheManager {
    tier1: Option<Arc<ExactMatchCache>>,
    tier2: Option<Arc<SemanticCache>>,
    stats: Arc<RwLock<CacheStats>>,
    config: CacheConfig,
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        let tier1 = if config.tier1_enabled {
            Some(Arc::new(ExactMatchCache::new(
                config.tier1_max_capacity,
                config.tier1_ttl_seconds,
            )))
        } else {
            None
        };

        let tier2 = if config.tier2_enabled {
            Some(Arc::new(SemanticCache::new(
                config.similarity_threshold,
                config.tier2_max_entries,
            )))
        } else {
            None
        };

        Self {
            tier1,
            tier2,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            config,
        }
    }

    pub async fn get_or_compute<T, F>(
        &self,
        query: &str,
        tenant_id: &str,
        tool_name: &str,
        embedding: Option<Vec<f32>>,
        compute_fn: F,
    ) -> Result<(T, CacheHit)>
    where
        T: Clone,
        F: FnOnce() -> futures_util::future::BoxFuture<'static, Result<T>>,
    {
        let start = Instant::now();

        // Try Tier 1: Exact match
        if let Some(ref tier1) = self.tier1 {
            let key = ExactMatchCache::compute_key(query);
            if let Some(entry) = tier1.get(&key).await {
                let latency_ns = start.elapsed().as_nanos() as u64;
                self.record_tier1_hit(latency_ns);

                // Deserialize result
                // In real implementation, would deserialize based on T
                let cache_hit = CacheHit::ExactMatch {
                    entry,
                    latency_ns,
                };

                // Return dummy value for now (proper impl would deserialize)
                return Ok((std::mem::zeroed(), cache_hit));
            }
        }

        // Try Tier 2: Semantic match
        if let Some(ref tier2) = self.tier2 {
            if let Some(vec_embedding) = embedding.clone() {
                if let Some((entry, similarity)) = tier2.find_similar(&vec_embedding) {
                    let latency_ns = start.elapsed().as_nanos() as u64;
                    self.record_tier2_hit(latency_ns);

                    let cache_hit = CacheHit::SemanticMatch {
                        entry,
                        similarity,
                        latency_ns,
                    };

                    return Ok((std::mem::zeroed(), cache_hit));
                }
            }
        }

        // Cache miss: compute result
        let result = compute_fn().await?;
        let latency_ns = start.elapsed().as_nanos() as u64;
        self.record_miss();

        // Store in both tiers
        let key = ExactMatchCache::compute_key(query);
        let entry = CacheEntry::new(
            key.clone(),
            query.to_string(),
            vec![], // Would serialize result here
            CacheMetadata {
                source_tool: tool_name.to_string(),
                tenant_id: tenant_id.to_string(),
                cost_saved: 0.0,
                original_latency_ms: (latency_ns / 1_000_000) as u64,
                cache_tier: CacheTier::ExactMatch,
            },
            self.config.tier1_ttl_seconds,
        );

        if let Some(ref tier1) = self.tier1 {
            tier1.put(entry.clone()).await;
        }

        if let Some(ref tier2) = self.tier2 {
            if let Some(vec_embedding) = embedding {
                tier2.put(entry, vec_embedding);
            }
        }

        Ok((result, CacheHit::Miss))
    }

    pub async fn invalidate(&self, key: &str) -> Result<()> {
        if let Some(ref tier1) = self.tier1 {
            tier1.invalidate(key).await;
        }
        if let Some(ref tier2) = self.tier2 {
            tier2.invalidate(key);
        }
        Ok(())
    }

    pub async fn invalidate_all(&self) -> Result<()> {
        if let Some(ref tier1) = self.tier1 {
            tier1.invalidate_all().await;
        }
        if let Some(ref tier2) = self.tier2 {
            tier2.invalidate_all();
        }
        Ok(())
    }

    pub fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read();
        let hit_rate = if stats.total_requests > 0 {
            ((stats.tier1_hits + stats.tier2_hits) as f32 / stats.total_requests as f32) * 100.0
        } else {
            0.0
        };

        CacheStats {
            hit_rate,
            ..stats.clone()
        }
    }

    fn record_tier1_hit(&self, latency_ns: u64) {
        let mut stats = self.stats.write();
        stats.tier1_hits += 1;
        stats.total_requests += 1;
        stats.avg_tier1_latency_ns = (stats.avg_tier1_latency_ns * (stats.tier1_hits as f64 - 1.0) + latency_ns as f64) / stats.tier1_hits as f64;
    }

    fn record_tier2_hit(&self, latency_ns: u64) {
        let mut stats = self.stats.write();
        stats.tier2_hits += 1;
        stats.total_requests += 1;
        stats.avg_tier2_latency_ns = (stats.avg_tier2_latency_ns * (stats.tier2_hits as f64 - 1.0) + latency_ns as f64) / stats.tier2_hits as f64;
    }

    fn record_miss(&self) {
        let mut stats = self.stats.write();
        stats.total_misses += 1;
        stats.total_requests += 1;
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

static CACHE_MANAGER: parking_lot::Mutex<Option<Arc<CacheManager>>> = parking_lot::Mutex::new(None);

pub fn initialize_cache(manager: Arc<CacheManager>) -> Result<()> {
    let mut cache = CACHE_MANAGER.lock();
    *cache = Some(manager);
    Ok(())
}

pub async fn shutdown_cache() -> Result<()> {
    let cache = CACHE_MANAGER.lock().take();
    if let Some(manager) = cache {
        manager.invalidate_all().await?;
        tracing::info!("Cache shut down complete");
    }
    Ok(())
}

pub fn get_cache() -> Option<Arc<CacheManager>> {
    CACHE_MANAGER.lock().clone()
}

pub fn is_cache_enabled() -> bool {
    CACHE_MANAGER.lock().as_ref().map(|c| c.is_enabled()).unwrap_or(false)
}
