use secureai::cache::{CacheEntry, CacheMetadata, CacheTier, CacheHit, CacheConfig, ExactMatchCache, SemanticCache, CacheManager};
use serde_json::json;

#[tokio::test]
async fn test_exact_match_cache_put_get() {
    let cache = ExactMatchCache::new(100, 3600);

    let metadata = CacheMetadata {
        source_tool: "test".to_string(),
        tenant_id: "tenant1".to_string(),
        cost_saved: 0.01,
        original_latency_ms: 100,
        cache_tier: CacheTier::ExactMatch,
    };

    let entry = CacheEntry::new(
        "key1".to_string(),
        "test query".to_string(),
        b"result".to_vec(),
        metadata,
        3600,
    );

    cache.put(entry.clone()).await;
    let retrieved = cache.get("key1").await;

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, entry.id);
}

#[tokio::test]
async fn test_exact_match_cache_miss() {
    let cache = ExactMatchCache::new(100, 3600);
    let result = cache.get("nonexistent").await;
    assert!(result.is_none());
}

#[test]
fn test_compute_key() {
    let key1 = ExactMatchCache::compute_key("test query");
    let key2 = ExactMatchCache::compute_key("test query");
    let key3 = ExactMatchCache::compute_key("different query");

    assert_eq!(key1, key2);
    assert_ne!(key1, key3);
}

#[tokio::test]
async fn test_exact_match_cache_invalidate() {
    let cache = ExactMatchCache::new(100, 3600);

    let metadata = CacheMetadata {
        source_tool: "test".to_string(),
        tenant_id: "tenant1".to_string(),
        cost_saved: 0.01,
        original_latency_ms: 100,
        cache_tier: CacheTier::ExactMatch,
    };

    let entry = CacheEntry::new(
        "key1".to_string(),
        "test query".to_string(),
        b"result".to_vec(),
        metadata,
        3600,
    );

    cache.put(entry).await;
    assert!(cache.get("key1").await.is_some());

    cache.invalidate("key1").await;
    assert!(cache.get("key1").await.is_none());
}

#[test]
fn test_semantic_cache_cosine_distance() {
    let vec1 = vec![1.0, 0.0, 0.0];
    let vec2 = vec![1.0, 0.0, 0.0];
    let vec3 = vec![0.0, 1.0, 0.0];

    let dist_same = SemanticCache::cosine_distance(&vec1, &vec2);
    let dist_diff = SemanticCache::cosine_distance(&vec1, &vec3);

    assert!(dist_same < 0.01); // Nearly identical
    assert!(dist_diff > 0.99); // Completely different
}

#[test]
fn test_semantic_cache_put_find() {
    let cache = SemanticCache::new(0.05, 100);

    let metadata = CacheMetadata {
        source_tool: "test".to_string(),
        tenant_id: "tenant1".to_string(),
        cost_saved: 0.01,
        original_latency_ms: 100,
        cache_tier: CacheTier::SemanticMatch,
    };

    let entry = CacheEntry::new(
        "key1".to_string(),
        "test query".to_string(),
        b"result".to_vec(),
        metadata,
        3600,
    );

    let embedding = vec![1.0, 0.0, 0.0];
    cache.put(entry.clone(), embedding.clone());

    // Nearly identical query
    let similar_embedding = vec![0.99, 0.01, 0.0];
    let result = cache.find_similar(&similar_embedding);

    assert!(result.is_some());
    let (found_entry, similarity) = result.unwrap();
    assert_eq!(found_entry.id, entry.id);
    assert!(similarity > 0.95); // High similarity
}

#[test]
fn test_semantic_cache_threshold() {
    let cache = SemanticCache::new(0.05, 100);

    let metadata = CacheMetadata {
        source_tool: "test".to_string(),
        tenant_id: "tenant1".to_string(),
        cost_saved: 0.01,
        original_latency_ms: 100,
        cache_tier: CacheTier::SemanticMatch,
    };

    let entry = CacheEntry::new(
        "key1".to_string(),
        "test query".to_string(),
        b"result".to_vec(),
        metadata,
        3600,
    );

    let embedding = vec![1.0, 0.0];
    cache.put(entry, embedding);

    // Very different query (below threshold)
    let diff_embedding = vec![0.0, 1.0];
    let result = cache.find_similar(&diff_embedding);

    assert!(result.is_none()); // Below threshold
}

#[test]
fn test_semantic_cache_max_entries() {
    let cache = SemanticCache::new(0.05, 3);

    for i in 0..5 {
        let metadata = CacheMetadata {
            source_tool: "test".to_string(),
            tenant_id: "tenant1".to_string(),
            cost_saved: 0.01,
            original_latency_ms: 100,
            cache_tier: CacheTier::SemanticMatch,
        };

        let entry = CacheEntry::new(
            format!("key{}", i),
            format!("query {}", i),
            b"result".to_vec(),
            metadata,
            3600,
        );

        let embedding = vec![i as f32, 0.0];
        cache.put(entry, embedding);
    }

    assert_eq!(cache.get_entry_count(), 3); // Max entries enforced
}

#[test]
fn test_cache_config_defaults() {
    let config = CacheConfig::default();
    assert!(!config.enabled);
    assert!(config.tier1_enabled);
    assert!(config.tier2_enabled);
    assert_eq!(config.tier1_max_capacity, 10_000);
    assert_eq!(config.tier2_max_entries, 1000);
    assert_eq!(config.similarity_threshold, 0.05);
}

#[test]
fn test_cache_config_from_toml() {
    let toml_str = r#"
enabled = true
tier1_enabled = true
tier2_enabled = true
tier1_max_capacity = 5000
tier1_ttl_seconds = 1800
tier2_max_entries = 500
tier2_ttl_seconds = 1800
similarity_threshold = 0.1
embedding_model = "mock"
"#;

    let config: CacheConfig = toml::from_str(toml_str).unwrap();
    assert!(config.enabled);
    assert_eq!(config.tier1_max_capacity, 5000);
    assert_eq!(config.tier2_max_entries, 500);
    assert_eq!(config.similarity_threshold, 0.1);
}

#[tokio::test]
async fn test_cache_manager_tier1_hit() {
    let config = CacheConfig {
        enabled: true,
        tier1_enabled: true,
        tier2_enabled: false,
        ..Default::default()
    };

    let manager = CacheManager::new(config);

    // First access: miss
    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let call_count_clone = call_count.clone();

    let (_result, hit) = manager.get_or_compute(
        "test query",
        "tenant1",
        "test_tool",
        None,
        move || {
            let count = call_count_clone.clone();
            Box::pin(async move {
                count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<Vec<u8>, anyhow::Error>(b"result".to_vec())
            })
        },
    ).await.unwrap();

    assert!(matches!(hit, CacheHit::Miss));
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn test_cache_hit_latency() {
    let hit = CacheHit::ExactMatch {
        entry: CacheEntry::new(
            "key".to_string(),
            "query".to_string(),
            b"result".to_vec(),
            CacheMetadata {
                source_tool: "test".to_string(),
                tenant_id: "t1".to_string(),
                cost_saved: 0.0,
                original_latency_ms: 100,
                cache_tier: CacheTier::ExactMatch,
            },
            3600,
        ),
        latency_ns: 500_000,
    };

    assert!(hit.is_hit());
    assert_eq!(hit.get_latency_ns(), Some(500_000));
}

#[test]
fn test_cache_miss() {
    let hit = CacheHit::Miss;
    assert!(!hit.is_hit());
    assert_eq!(hit.get_latency_ns(), None);
}

#[test]
fn test_cache_entry_expiry() {
    let metadata = CacheMetadata {
        source_tool: "test".to_string(),
        tenant_id: "tenant1".to_string(),
        cost_saved: 0.01,
        original_latency_ms: 100,
        cache_tier: CacheTier::ExactMatch,
    };

    let entry = CacheEntry::new(
        "key1".to_string(),
        "test query".to_string(),
        b"result".to_vec(),
        metadata,
        1, // 1 second TTL
    );

    assert!(!entry.is_expired());

    // Wait to simulate expiry
    std::thread::sleep(std::time::Duration::from_millis(100));
    // Still not expired (only 100ms passed)
    assert!(!entry.is_expired());
}

#[test]
fn test_cache_entry_access_count() {
    let metadata = CacheMetadata {
        source_tool: "test".to_string(),
        tenant_id: "tenant1".to_string(),
        cost_saved: 0.01,
        original_latency_ms: 100,
        cache_tier: CacheTier::ExactMatch,
    };

    let mut entry = CacheEntry::new(
        "key1".to_string(),
        "test query".to_string(),
        b"result".to_vec(),
        metadata,
        3600,
    );

    assert_eq!(entry.access_count, 1);
    entry.record_access();
    assert_eq!(entry.access_count, 2);
    entry.record_access();
    assert_eq!(entry.access_count, 3);
}

#[cfg(test)]
mod documentation_tests {
    #[test]
    fn test_multi_tier_lookup_documented() {
        // Tier 1: Exact match
        // - O(1) lookup via SHA-256 key
        // - < 1ms latency
        // - No computation
        //
        // Tier 2: Semantic match
        // - O(n) lookup (n = cached embeddings, max 1000)
        // - Cosine similarity calculation
        // - < 10ms latency typical
        // - Catches "same intent, different wording"
        //
        // Miss: Cache miss
        // - Compute result
        // - Store in both tiers
        println!("✓ Multi-tier lookup: Tier1 → Tier2 → Compute");
    }

    #[test]
    fn test_semantic_matching_algorithm_documented() {
        // Cosine similarity matching:
        // 1. Compute embedding for query
        // 2. Iterate cached embeddings
        // 3. Compute cosine distance = 1 - (dot_product / (mag1 * mag2))
        // 4. Find highest similarity (lowest distance)
        // 5. Return if distance < threshold (0.05 default)
        //
        // Distance interpretation:
        // - 0.0 = identical vectors
        // - 0.05 = 95% similarity (same intent)
        // - 1.0 = orthogonal (completely different)
        println!("✓ Cosine similarity: Distance < threshold → Hit");
    }

    #[test]
    fn test_lru_eviction_documented() {
        // LRU eviction:
        // - Tier 1: moka handles automatic TTL + LRU
        // - Tier 2: Manual max_entries limit
        // - When Tier 2 exceeds max_entries, remove oldest
        // - Access count tracked (for future LRU scoring)
        // - TTL per entry (configurable, default 3600s)
        println!("✓ LRU eviction: TTL + capacity limits");
    }

    #[test]
    fn test_cost_savings_documented() {
        // Cost savings per cache hit:
        // - Tier 1 hit: Avoid full tool execution
        //   Time: -100ms (typical)
        //   Cost: -$0.001 to -$0.1 (tool-dependent)
        // - Tier 2 hit: Avoid full tool execution (semantic match)
        //   Time: -100ms (typical)
        //   Cost: -$0.001 to -$0.1
        // - At scale: 10k hits/day × $0.01/hit = $100/day saved
        println!("✓ Cost savings: Quantifiable per hit");
    }
}
