//! Cache Usage Examples
//!
//! This module demonstrates how to use the multi-tier semantic cache
//! in real tool execution scenarios.

#![allow(dead_code)]

use crate::cache::{CacheIntegration, CacheHit};
use anyhow::Result;
use serde_json::json;

/// Example: Caching a web search result
pub async fn example_cached_web_search(
    query: &str,
    tenant_id: &str,
) -> Result<String> {
    let embedding = vec![]; // Would compute via SemanticGuardrail

    let (result, hit) = CacheIntegration::get_or_compute_cached(
        query,
        tenant_id,
        "web_search",
        if embedding.is_empty() { None } else { Some(embedding) },
        || {
            Box::pin(async {
                // Original web search logic
                Ok("search results".to_string())
            })
        },
    )
    .await?;

    match hit {
        CacheHit::ExactMatch { latency_ns, .. } => {
            tracing::info!("Web search: Exact cache hit in {}ms", latency_ns / 1_000_000);
        }
        CacheHit::SemanticMatch { similarity, latency_ns, .. } => {
            tracing::info!(
                "Web search: Semantic hit ({:.1}% similar) in {}ms",
                similarity * 100.0,
                latency_ns / 1_000_000
            );
        }
        CacheHit::Miss => {
            tracing::debug!("Web search: Cache miss, computed fresh result");
        }
    }

    Ok(result)
}

/// Example: Caching code execution results
pub async fn example_cached_code_exec(
    code: &str,
    tenant_id: &str,
) -> Result<String> {
    let embedding = vec![]; // Would compute via SemanticGuardrail

    let (result, hit) = CacheIntegration::get_or_compute_cached(
        code,
        tenant_id,
        "code_exec",
        if embedding.is_empty() { None } else { Some(embedding) },
        || {
            Box::pin(async {
                // Original code execution logic
                Ok("execution output".to_string())
            })
        },
    )
    .await?;

    // Log cache statistics
    if let Some(stats) = CacheIntegration::get_cache_stats() {
        tracing::debug!(
            "Cache stats: {}/{} hits ({:.1}%)",
            stats.tier1_hits + stats.tier2_hits,
            stats.total_requests,
            stats.hit_rate
        );
    }

    Ok(result)
}

/// Example: Handling semantic cache hits for similar queries
///
/// Queries like "Summarize report" and "Give me a summary of the report"
/// should both hit the cache (different wording, same intent)
pub async fn example_semantic_similarity() -> Result<()> {
    // Query 1: "Summarize report"
    let query1 = "Summarize report";
    let embedding1 = vec![0.9, 0.1, 0.0]; // Mock embedding

    // Query 2: "Give me a summary of the report"  (different wording, same intent)
    let query2 = "Give me a summary of the report";
    let embedding2 = vec![0.89, 0.11, 0.01]; // Similar embedding (cosine distance ~0.02)

    // First query caches with embedding1
    let (_result1, hit1) = CacheIntegration::get_or_compute_cached(
        query1,
        "tenant_1",
        "summarize",
        Some(embedding1),
        || Box::pin(async { Ok("Summary of report...".to_string()) }),
    )
    .await?;

    // Should be a miss (first time)
    assert!(matches!(hit1, CacheHit::Miss));

    // Second query with similar embedding hits semantic cache
    let (_result2, hit2) = CacheIntegration::get_or_compute_cached(
        query2,
        "tenant_1",
        "summarize",
        Some(embedding2),
        || Box::pin(async { Ok("Summary of report...".to_string()) }),
    )
    .await?;

    // Should be a semantic hit if similarity > threshold
    match hit2 {
        CacheHit::SemanticMatch { similarity, latency_ns, .. } => {
            tracing::info!(
                "Semantic similarity: {:.1}% in {}ns",
                similarity * 100.0,
                latency_ns
            );
        }
        _ => {
            tracing::debug!("No semantic hit (similarity too low)");
        }
    }

    Ok(())
}

/// Example: Multi-tenant cache isolation
///
/// Tenant A and Tenant B have separate cache entries even for same query
pub async fn example_multi_tenant_isolation() -> Result<()> {
    let query = "Summarize report";
    let embedding = Some(vec![0.9, 0.1, 0.0]);

    // Tenant A executes query
    let (_result_a, _hit_a) = CacheIntegration::get_or_compute_cached(
        query,
        "tenant_a",
        "summarize",
        embedding.clone(),
        || Box::pin(async { Ok("Summary for Tenant A".to_string()) }),
    )
    .await?;

    // Tenant B executes same query
    let (_result_b, _hit_b) = CacheIntegration::get_or_compute_cached(
        query,
        "tenant_b",
        "summarize",
        embedding.clone(),
        || Box::pin(async { Ok("Summary for Tenant B".to_string()) }),
    )
    .await?;

    // Both tenants have separate cache entries (isolation guaranteed)
    // Invalidating Tenant A's cache doesn't affect Tenant B
    let _ = CacheIntegration::invalidate_for_tenant("tenant_a").await?;

    Ok(())
}

/// Example: Cache statistics and metrics
pub async fn example_cache_metrics() -> Result<()> {
    if let Some(stats) = CacheIntegration::get_cache_stats() {
        tracing::info!(
            "Cache metrics: T1={}, T2={}, Miss={}, Rate={:.1}%",
            stats.tier1_hits,
            stats.tier2_hits,
            stats.total_misses,
            stats.hit_rate
        );

        tracing::debug!(
            "Latency: T1={}ns (avg), T2={}ns (avg)",
            stats.avg_tier1_latency_ns as u64,
            stats.avg_tier2_latency_ns as u64
        );
    }

    Ok(())
}

/// Configuration for different deployment scenarios
pub mod config_examples {
    use crate::cache::CacheConfig;

    /// Development: Caching disabled
    pub fn dev_config() -> CacheConfig {
        CacheConfig {
            enabled: false,
            ..Default::default()
        }
    }

    /// Staging: Both tiers enabled, small capacity
    pub fn staging_config() -> CacheConfig {
        CacheConfig {
            enabled: true,
            tier1_enabled: true,
            tier2_enabled: true,
            tier1_max_capacity: 1000,
            tier2_max_entries: 100,
            tier1_ttl_seconds: 1800, // 30 minutes
            tier2_ttl_seconds: 1800,
            similarity_threshold: 0.05,
            embedding_model: "onnx".to_string(),
        }
    }

    /// Production: Aggressive caching for cost savings
    pub fn production_config() -> CacheConfig {
        CacheConfig {
            enabled: true,
            tier1_enabled: true,
            tier2_enabled: true,
            tier1_max_capacity: 100_000, // 100k entries
            tier2_max_entries: 10_000, // 10k semantic entries
            tier1_ttl_seconds: 3600, // 1 hour
            tier2_ttl_seconds: 3600,
            similarity_threshold: 0.05,
            embedding_model: "onnx".to_string(),
        }
    }
}
