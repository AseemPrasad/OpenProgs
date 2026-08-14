use crate::evals::{EvalRequest, EvalAlert};
use std::collections::HashMap;

pub struct EvalsIntegration;

impl EvalsIntegration {
    pub fn should_evaluate_request(tenant_id: &str, context: &HashMap<String, String>) -> bool {
        if let Some(evals) = crate::evals::get_evals() {
            let is_flagged = context.get("flagged").map(|v| v == "true").unwrap_or(false);
            evals.should_evaluate(is_flagged)
        } else {
            false
        }
    }

    pub async fn evaluate_request_async(
        tenant_id: &str,
        tool_name: &str,
        prompt: &str,
        response: &str,
        context: &HashMap<String, String>,
    ) -> anyhow::Result<()> {
        if let Some(evals) = crate::evals::get_evals() {
            let is_flagged = context.get("flagged").map(|v| v == "true").unwrap_or(false);

            if evals.should_evaluate(is_flagged) {
                let request = EvalRequest {
                    tenant_id: tenant_id.to_string(),
                    tool_name: tool_name.to_string(),
                    prompt: prompt.to_string(),
                    response: response.to_string(),
                    context: context.clone(),
                };

                evals.evaluate_request_async(request)?;
            }
        }
        Ok(())
    }

    pub fn get_drift_alerts() -> Vec<EvalAlert> {
        if let Some(evals) = crate::evals::get_evals() {
            evals.get_drift_alerts()
        } else {
            Vec::new()
        }
    }

    pub fn get_current_metrics(metric_type: crate::evals::MetricType) -> Option<crate::evals::Statistics> {
        if let Some(evals) = crate::evals::get_evals() {
            evals.get_current_metrics(metric_type)
        } else {
            None
        }
    }

    pub fn is_enabled() -> bool {
        crate::evals::is_evals_enabled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_evaluate_when_disabled() {
        let context = HashMap::new();
        assert!(!EvalsIntegration::should_evaluate_request("tenant-1", &context));
    }

    #[test]
    fn test_is_enabled_when_disabled() {
        assert!(!EvalsIntegration::is_enabled());
    }
}
