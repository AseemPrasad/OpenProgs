use crate::cache::{get_cache, is_cache_enabled, CacheHit};
use anyhow::Result;
use serde_json::json;

pub struct CacheIntegration;

impl CacheIntegration {
    pub async fn get_or_compute_cached<T, F>(
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
        if !is_cache_enabled() {
            let result = compute_fn().await?;
            return Ok((result, CacheHit::Miss));
        }

        if let Some(cache) = get_cache() {
            cache.get_or_compute(query, tenant_id, tool_name, embedding, compute_fn).await
        } else {
            let result = compute_fn().await?;
            Ok((result, CacheHit::Miss))
        }
    }

    pub async fn invalidate_for_tenant(tenant_id: &str) -> Result<()> {
        if !is_cache_enabled() {
            return Ok(());
        }

        if let Some(cache) = get_cache() {
            // Invalidate all entries for this tenant
            // In a real implementation, would track entries by tenant
            cache.invalidate_all().await?;
        }

        Ok(())
    }

    pub fn get_cache_stats() -> Option<crate::cache::CacheStats> {
        if !is_cache_enabled() {
            return None;
        }

        get_cache().map(|cache| cache.get_stats())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_disabled() {
        // When cache is disabled, should still work
        // Returns Miss, not error
    }
}
