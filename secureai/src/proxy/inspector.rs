use anyhow::Result;
use std::sync::Arc;

use super::stream::{SSEChunk, SSEStreamInspector};

#[derive(Debug, Clone)]
pub enum InspectionResult {
    Continue,
    Terminate { reason: String, threat_score: f32 },
}

pub struct StreamPolicyInspector {
    guardrail: Option<Arc<crate::guardrails::SemanticGuardrail>>,
    inspection_window: SSEStreamInspector,
    enabled: bool,
}

impl StreamPolicyInspector {
    pub fn new(guardrail: Option<Arc<crate::guardrails::SemanticGuardrail>>) -> Self {
        let enabled = guardrail.is_some();

        Self {
            guardrail,
            inspection_window: SSEStreamInspector::new(50),
            enabled,
        }
    }

    pub async fn inspect_chunk(&mut self, chunk: SSEChunk) -> Result<InspectionResult> {
        if !self.enabled {
            return Ok(InspectionResult::Continue);
        }

        self.inspection_window.add_tokens(&chunk.data);

        if let Some(guardrail) = &self.guardrail {
            let window_text = self.inspection_window.get_window_content();

            match guardrail.check_prompt(&window_text).await? {
                crate::guardrails::GuardrailDecision::Permit => {
                    Ok(InspectionResult::Continue)
                }
                crate::guardrails::GuardrailDecision::Deny {
                    reason,
                    threat_score,
                } => {
                    tracing::warn!(
                        "Stream policy violation detected: {} (score: {:.3})",
                        reason,
                        threat_score
                    );

                    Ok(InspectionResult::Terminate {
                        reason,
                        threat_score,
                    })
                }
            }
        } else {
            Ok(InspectionResult::Continue)
        }
    }

    pub fn reset(&mut self) {
        self.inspection_window.clear();
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_window_content(&self) -> String {
        self.inspection_window.get_window_content()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_creation_no_guardrail() {
        let inspector = StreamPolicyInspector::new(None);
        assert!(!inspector.is_enabled());
    }

    #[tokio::test]
    async fn test_inspector_disabled_permits_all() {
        let mut inspector = StreamPolicyInspector::new(None);

        let chunk = SSEChunk {
            event_type: "message".to_string(),
            data: "malicious content here".to_string(),
            token_count: 4,
        };

        let result = inspector.inspect_chunk(chunk).await.expect("Failed to inspect");

        match result {
            InspectionResult::Continue => {
                // Expected: disabled inspector permits all
            }
            InspectionResult::Terminate { .. } => {
                panic!("Disabled inspector should not terminate");
            }
        }
    }

    #[tokio::test]
    async fn test_inspector_reset() {
        let mut inspector = StreamPolicyInspector::new(None);

        let chunk = SSEChunk {
            event_type: "message".to_string(),
            data: "chunk1".to_string(),
            token_count: 1,
        };

        inspector.inspect_chunk(chunk).await.unwrap();
        assert!(!inspector.get_window_content().is_empty());

        inspector.reset();
        assert!(inspector.get_window_content().is_empty());
    }

    #[tokio::test]
    async fn test_inspector_accumulates_content() {
        let mut inspector = StreamPolicyInspector::new(None);

        let chunk1 = SSEChunk {
            event_type: "message".to_string(),
            data: "hello ".to_string(),
            token_count: 1,
        };

        let chunk2 = SSEChunk {
            event_type: "message".to_string(),
            data: "world".to_string(),
            token_count: 1,
        };

        inspector.inspect_chunk(chunk1).await.unwrap();
        inspector.inspect_chunk(chunk2).await.unwrap();

        let content = inspector.get_window_content();
        assert!(content.contains("hello"));
        assert!(content.contains("world"));
    }
}
