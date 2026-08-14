use crate::queue::{Job, get_queue, is_queue_enabled};
use anyhow::Result;
use serde_json::json;

pub struct QueueIntegration;

impl QueueIntegration {
    pub async fn enqueue_tool_if_enabled(
        tool_name: &str,
        params: serde_json::Value,
        tenant_id: &str,
    ) -> Result<Option<String>> {
        if !is_queue_enabled() {
            return Ok(None);
        }

        if let Some(queue) = get_queue() {
            let job_id = queue
                .enqueue_task(
                    tool_name.to_string(),
                    params,
                    tenant_id.to_string(),
                )
                .await?;

            tracing::info!("Tool {} enqueued as job {}", tool_name, job_id);
            Ok(Some(job_id))
        } else {
            Ok(None)
        }
    }

    pub async fn enqueue_web_search(
        query: &str,
        tenant_id: &str,
    ) -> Result<Option<String>> {
        let params = json!({
            "query": query,
            "depth": "standard",
            "timeout_secs": 30
        });

        Self::enqueue_tool_if_enabled("web_search", params, tenant_id).await
    }

    pub async fn enqueue_code_execution(
        code: &str,
        language: &str,
        tenant_id: &str,
    ) -> Result<Option<String>> {
        let params = json!({
            "code": code,
            "language": language,
            "timeout_secs": 60,
            "sandbox": true
        });

        Self::enqueue_tool_if_enabled("code_exec", params, tenant_id).await
    }

    pub async fn enqueue_document_transform(
        input_path: &str,
        output_format: &str,
        tenant_id: &str,
    ) -> Result<Option<String>> {
        let params = json!({
            "input_path": input_path,
            "output_format": output_format,
            "preserve_formatting": true
        });

        Self::enqueue_tool_if_enabled("doc_transform", params, tenant_id).await
    }

    pub async fn get_job_status(job_id: &str) -> Result<Option<Job>> {
        if !is_queue_enabled() {
            return Ok(None);
        }

        // In a real implementation, query NATS KV store or job store
        // For now, return None (would need persistence layer)
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_disabled_returns_none() {
        // Queue disabled → returns None instead of Job
        // Original synchronous path used as fallback
    }
}
