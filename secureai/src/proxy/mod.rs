pub mod config;
pub mod handler;
pub mod inspector;
pub mod stream;
pub mod token_budget;

pub use config::ProxyConfig;
pub use handler::ProxyState;
pub use token_budget::TokenBudgetManager;

use anyhow::{Result, Context};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub async fn create_proxy_server(
    config: ProxyConfig,
    guardrail: Option<Arc<crate::guardrails::SemanticGuardrail>>,
) -> Result<()> {
    if !config.enabled {
        tracing::info!("Stream proxy disabled");
        return Ok(());
    }

    // Validate configuration
    config.validate()
        .context("Invalid proxy configuration")?;

    let token_manager = Arc::new(TokenBudgetManager::new());

    let state = ProxyState {
        token_manager,
        max_tokens_per_tenant: config.max_tokens_per_tenant,
        refill_rate: config.token_refill_rate_per_minute,
        enable_inspection: config.is_inspection_enabled(),
        guardrail,
    };

    // Create router with proxy endpoints
    let router = Router::new()
        .route("/proxy/stream", post(handler::proxy_stream))
        .route("/proxy/status", get(handler::get_token_status))
        .with_state(state);

    // Bind and serve
    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .context(format!("Failed to bind proxy listener on {}", config.listen_addr))?;

    tracing::info!(
        "Stream proxy server listening on {} (inspection: {:?})",
        config.listen_addr,
        config.inspection_mode
    );

    axum::serve(listener, router)
        .await
        .context("Stream proxy server error")?;

    Ok(())
}

pub async fn start_proxy_server_background(
    config: ProxyConfig,
    guardrail: Option<Arc<crate::guardrails::SemanticGuardrail>>,
) -> Result<tokio::task::JoinHandle<Result<()>>> {
    if !config.enabled {
        tracing::info!("Stream proxy disabled, skipping background server");
        // Return a task that completes immediately
        return Ok(tokio::spawn(async { Ok(()) }));
    }

    let handle = tokio::spawn(async move {
        create_proxy_server(config, guardrail).await
    });

    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_module_loads() {
        // If the module compiles and loads, this test passes
        assert!(true);
    }

    #[tokio::test]
    async fn test_proxy_server_disabled() {
        let config = ProxyConfig {
            enabled: false,
            ..Default::default()
        };

        let result = create_proxy_server(config, None).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_proxy_config_validation_in_server() {
        let config = ProxyConfig {
            enabled: true,
            max_tokens_per_tenant: 0, // Invalid
            ..Default::default()
        };

        // Would fail at runtime when create_proxy_server is called
        assert!(config.validate().is_err());
    }
}
