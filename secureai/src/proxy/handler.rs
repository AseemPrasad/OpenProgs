use anyhow::{Result, Context};
use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::token_budget::TokenBudgetManager;
use super::stream::SSEStreamParser;
use super::inspector::StreamPolicyInspector;

#[derive(Clone)]
pub struct ProxyState {
    pub token_manager: Arc<TokenBudgetManager>,
    pub max_tokens_per_tenant: u32,
    pub refill_rate: u32,
    pub enable_inspection: bool,
    pub guardrail: Option<Arc<crate::guardrails::SemanticGuardrail>>,
}

#[derive(Debug, Deserialize)]
pub struct StreamRequest {
    tenant_id: String,
    #[serde(default)]
    user_prompt: String,
}

#[derive(Debug, Serialize)]
pub struct StreamResponse {
    success: bool,
    message: String,
    remaining_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct TokenStatusResponse {
    tenant_id: String,
    used_tokens: u32,
    remaining_tokens: u32,
}

pub async fn proxy_stream(
    State(state): State<ProxyState>,
    Query(req): Query<StreamRequest>,
) -> Response {
    // Get or create token budget for tenant
    let budget = state
        .token_manager
        .get_or_create_budget(&req.tenant_id, state.max_tokens_per_tenant, state.refill_rate);

    // Try to reserve tokens for input prompt
    let prompt_tokens = state.guardrail
        .as_ref()
        .and_then(|_| {
            let tokenizer = match SSEStreamParser::new() {
                Ok(p) => p,
                Err(_) => return None,
            };
            Some(tokenizer.count_tokens(&req.user_prompt))
        })
        .unwrap_or(10); // Estimate if we can't count

    if let Err(e) = budget.consume(prompt_tokens) {
        return (
            StatusCode::PAYMENT_REQUIRED,
            Json(StreamResponse {
                success: false,
                message: format!(
                    "Insufficient token budget. Needed: {}, Available: {}",
                    e.needed, e.available
                ),
                remaining_tokens: Some(budget.get_remaining()),
            }),
        )
            .into_response();
    }

    // Create stream inspector if enabled
    let mut inspector = if state.enable_inspection {
        StreamPolicyInspector::new(state.guardrail.clone())
    } else {
        StreamPolicyInspector::new(None)
    };

    // Create SSE parser for token counting
    let parser = match SSEStreamParser::new() {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(StreamResponse {
                    success: false,
                    message: "Failed to initialize stream parser".to_string(),
                    remaining_tokens: None,
                }),
            )
                .into_response();
        }
    };

    // Simulate streaming response (in production, would proxy to actual LLM)
    let response_text = format!("Processing prompt for tenant: {}", req.tenant_id);
    let chunk_count = parser.count_tokens(&response_text);

    // Check budget before streaming
    if let Err(e) = budget.consume(chunk_count) {
        return (
            StatusCode::PAYMENT_REQUIRED,
            Json(StreamResponse {
                success: false,
                message: format!(
                    "Token budget exhausted during streaming. Needed: {}, Available: {}",
                    e.needed, e.available
                ),
                remaining_tokens: Some(budget.get_remaining()),
            }),
        )
            .into_response();
    }

    // Simulate SSE chunk for testing
    let sse_chunk = super::stream::SSEChunk {
        event_type: "message".to_string(),
        data: response_text.clone(),
        token_count: chunk_count,
    };

    // Inspect chunk
    match inspector.inspect_chunk(sse_chunk).await {
        Ok(super::inspector::InspectionResult::Continue) => {
            (
                StatusCode::OK,
                Json(StreamResponse {
                    success: true,
                    message: response_text,
                    remaining_tokens: Some(budget.get_remaining()),
                }),
            )
                .into_response()
        }
        Ok(super::inspector::InspectionResult::Terminate { reason, .. }) => {
            (
                StatusCode::FORBIDDEN,
                Json(StreamResponse {
                    success: false,
                    message: format!("Stream terminated: {}", reason),
                    remaining_tokens: Some(budget.get_remaining()),
                }),
            )
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(StreamResponse {
                    success: false,
                    message: format!("Inspection error: {}", e),
                    remaining_tokens: None,
                }),
            )
                .into_response()
        }
    }
}

pub async fn get_token_status(
    State(state): State<ProxyState>,
    Query(req): Query<StreamRequest>,
) -> Response {
    let budget = state
        .token_manager
        .get_or_create_budget(&req.tenant_id, state.max_tokens_per_tenant, state.refill_rate);

    (
        StatusCode::OK,
        Json(TokenStatusResponse {
            tenant_id: req.tenant_id,
            used_tokens: budget.get_used(),
            remaining_tokens: budget.get_remaining(),
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_state_creation() {
        let state = ProxyState {
            token_manager: Arc::new(TokenBudgetManager::new()),
            max_tokens_per_tenant: 4000,
            refill_rate: 100,
            enable_inspection: false,
            guardrail: None,
        };

        assert_eq!(state.max_tokens_per_tenant, 4000);
        assert!(!state.enable_inspection);
    }

    #[tokio::test]
    async fn test_proxy_stream_budget_sufficient() {
        let state = ProxyState {
            token_manager: Arc::new(TokenBudgetManager::new()),
            max_tokens_per_tenant: 1000,
            refill_rate: 100,
            enable_inspection: false,
            guardrail: None,
        };

        let req = StreamRequest {
            tenant_id: "test-tenant".to_string(),
            user_prompt: "test prompt".to_string(),
        };

        let response = proxy_stream(State(state), Query(req)).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_proxy_state_with_guardrail() {
        let state = ProxyState {
            token_manager: Arc::new(TokenBudgetManager::new()),
            max_tokens_per_tenant: 4000,
            refill_rate: 100,
            enable_inspection: true,
            guardrail: None, // Could be Some(guardrail) if initialized
        };

        assert!(state.enable_inspection);
    }
}
