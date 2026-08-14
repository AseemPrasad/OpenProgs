use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRequest {
    pub id: String,
    pub timestamp: u64,
    pub tenant_id: String,
    pub prompt: String,
    pub response: String,
    pub model: String,
    pub tool_name: Option<String>,
    pub metadata: serde_json::Value,
}

impl EvalRequest {
    pub fn new(
        tenant_id: String,
        prompt: String,
        response: String,
        model: String,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: now,
            tenant_id,
            prompt,
            response,
            model,
            tool_name: None,
            metadata: serde_json::json!({}),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalMetrics {
    pub prompt_length: usize,
    pub response_length: usize,
    pub latency_ms: u64,
    pub toxicity_score: f32,
    pub hallucination_risk: f32,
    pub guardrail_triggered: bool,
    pub output_quality_score: f32,
    pub timestamp: u64,
}

impl EvalMetrics {
    pub fn new(
        prompt_length: usize,
        response_length: usize,
        latency_ms: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            prompt_length,
            response_length,
            latency_ms,
            toxicity_score: 0.0,
            hallucination_risk: 0.0,
            guardrail_triggered: false,
            output_quality_score: 1.0,
            timestamp: now,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    Toxicity,
    HallucinationRisk,
    GuardrailTriggers,
    OutputQuality,
    PromptLength,
    ResponseLatency,
}

impl MetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::Toxicity => "toxicity",
            MetricType::HallucinationRisk => "hallucination_risk",
            MetricType::GuardrailTriggers => "guardrail_triggers",
            MetricType::OutputQuality => "output_quality",
            MetricType::PromptLength => "prompt_length",
            MetricType::ResponseLatency => "response_latency",
        }
    }
}

#[derive(Debug, Clone)]
pub enum EvalAlert {
    ToxicitySpikeDetected {
        score: f32,
        baseline: f32,
        z_score: f32,
    },
    HallucinationRiskElevated {
        risk: f32,
        baseline: f32,
        z_score: f32,
    },
    GuardrailTriggerRateAnomaly {
        rate: f32,
        baseline: f32,
        z_score: f32,
    },
    PolicyDriftDetected {
        metric: String,
        z_score: f32,
        threshold: f32,
    },
}

impl EvalAlert {
    pub fn description(&self) -> String {
        match self {
            EvalAlert::ToxicitySpikeDetected { score, baseline, z_score } => {
                format!("Toxicity spike detected: {:.3} (baseline: {:.3}, z-score: {:.2})", score, baseline, z_score)
            }
            EvalAlert::HallucinationRiskElevated { risk, baseline, z_score } => {
                format!("Hallucination risk elevated: {:.3} (baseline: {:.3}, z-score: {:.2})", risk, baseline, z_score)
            }
            EvalAlert::GuardrailTriggerRateAnomaly { rate, baseline, z_score } => {
                format!("Guardrail trigger rate anomaly: {:.1}% (baseline: {:.1}%, z-score: {:.2})", rate * 100.0, baseline * 100.0, z_score)
            }
            EvalAlert::PolicyDriftDetected { metric, z_score, threshold } => {
                format!("Policy drift detected for {}: z-score {:.2} > threshold {:.2}", metric, z_score, threshold)
            }
        }
    }
}
