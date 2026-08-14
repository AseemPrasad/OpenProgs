use crate::evals::{
    EvalRequest, EvalMetrics, MetricType, DriftDetector, DynamicSampler,
    SamplingStrategy, EvalsConfig,
};
use tokio::sync::mpsc;
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct EvaluationEngine {
    tx: mpsc::UnboundedSender<EvalRequest>,
    sampler: Arc<DynamicSampler>,
    drift_detectors: Arc<RwLock<HashMap<MetricType, Arc<DriftDetector>>>>,
    config: EvalsConfig,
}

impl EvaluationEngine {
    pub async fn new(config: EvalsConfig) -> anyhow::Result<Self> {
        if !config.enabled {
            tracing::info!("Online evaluation disabled");
            return Ok(Self::disabled());
        }

        let sampling_strategy = if config.sampling_rate > 0.0 {
            SamplingStrategy::AdaptiveRate {
                baseline: config.sampling_rate,
                boosted: config.boost_flagged_requests,
            }
        } else {
            SamplingStrategy::Disabled
        };

        let sampler = Arc::new(DynamicSampler::new(sampling_strategy));

        let (tx, rx) = mpsc::unbounded_channel();

        let drift_detectors = Arc::new(RwLock::new(HashMap::new()));
        let detectors_clone = drift_detectors.clone();

        // Spawn async worker task
        tokio::spawn(async move {
            Self::worker_loop(rx, detectors_clone, config.clone()).await;
        });

        Ok(Self {
            tx,
            sampler,
            drift_detectors,
            config,
        })
    }

    fn disabled() -> Self {
        let (tx, _rx) = mpsc::unbounded_channel();

        Self {
            tx,
            sampler: Arc::new(DynamicSampler::new(SamplingStrategy::Disabled)),
            drift_detectors: Arc::new(RwLock::new(HashMap::new())),
            config: EvalsConfig::default(),
        }
    }

    pub fn evaluate_request_async(&self, request: EvalRequest) -> anyhow::Result<()> {
        // Fire-and-forget: Send to channel without blocking
        self.tx.send(request)?;
        Ok(())
    }

    pub fn should_evaluate(&self, is_flagged: bool) -> bool {
        self.sampler.decide(is_flagged).should_evaluate
    }

    pub fn get_current_metrics(&self, metric_type: MetricType) -> Option<crate::evals::Statistics> {
        self.drift_detectors
            .read()
            .get(&metric_type)
            .map(|detector| detector.get_short_window_stats())
    }

    pub fn get_drift_alerts(&self) -> Vec<crate::evals::EvalAlert> {
        let mut all_alerts = Vec::new();

        for detector in self.drift_detectors.read().values() {
            all_alerts.extend(detector.get_unread_alerts());
        }

        all_alerts
    }

    pub fn get_sampling_stats(&self) -> crate::evals::sampling::SamplingStats {
        self.sampler.get_stats()
    }

    async fn worker_loop(
        mut rx: mpsc::UnboundedReceiver<EvalRequest>,
        detectors: Arc<RwLock<HashMap<MetricType, Arc<DriftDetector>>>>,
        config: EvalsConfig,
    ) {
        while let Some(request) = rx.recv().await {
            // Evaluate request
            let metrics = Self::evaluate_request(&request).await;

            // Initialize detectors on first metric
            {
                let mut dets = detectors.write();
                if dets.is_empty() {
                    for metric_type in &[
                        MetricType::Toxicity,
                        MetricType::HallucinationRisk,
                        MetricType::GuardrailTriggers,
                        MetricType::OutputQuality,
                    ] {
                        dets.insert(
                            *metric_type,
                            Arc::new(DriftDetector::new(
                                *metric_type,
                                config.anomaly_threshold,
                            )),
                        );
                    }
                }
            }

            // Add samples to detectors
            {
                let dets = detectors.read();
                if let Some(detector) = dets.get(&MetricType::Toxicity) {
                    detector.add_sample(metrics.toxicity_score);
                }
                if let Some(detector) = dets.get(&MetricType::HallucinationRisk) {
                    detector.add_sample(metrics.hallucination_risk);
                }
                if let Some(detector) = dets.get(&MetricType::GuardrailTriggers) {
                    detector.add_sample(if metrics.guardrail_triggered { 1.0 } else { 0.0 });
                }
                if let Some(detector) = dets.get(&MetricType::OutputQuality) {
                    detector.add_sample(metrics.output_quality_score);
                }
            }

            // Run drift detection
            {
                let dets = detectors.read();
                for detector in dets.values() {
                    let alerts = detector.detect_anomalies();
                    for alert in alerts {
                        if config.alert_enabled {
                            tracing::warn!("Drift alert: {}", alert.description());
                        }
                    }
                }
            }
        }
    }

    async fn evaluate_request(request: &EvalRequest) -> EvalMetrics {
        use std::time::{SystemTime, UNIX_EPOCH};

        let prompt_len = request.prompt.len();
        let response_len = request.response.len();

        // Mock evaluation (in production, would call actual evaluators)
        let toxicity = Self::mock_toxicity_score(&request.response);
        let hallucination = Self::mock_hallucination_score(&request.response);

        EvalMetrics {
            prompt_length: prompt_len,
            response_length: response_len,
            latency_ms: 10, // Mock
            toxicity_score: toxicity,
            hallucination_risk: hallucination,
            guardrail_triggered: toxicity > 0.7 || hallucination > 0.6,
            output_quality_score: 1.0 - (toxicity.max(hallucination) * 0.5),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        }
    }

    fn mock_toxicity_score(text: &str) -> f32 {
        // Simple mock: check for "toxic" keywords
        let toxic_keywords = ["bad", "evil", "toxic"];
        if toxic_keywords.iter().any(|kw| text.to_lowercase().contains(kw)) {
            0.8
        } else {
            0.1
        }
    }

    fn mock_hallucination_score(text: &str) -> f32 {
        // Simple mock: check for suspicious phrases
        let hallucination_keywords = ["invented", "fictional", "not real"];
        if hallucination_keywords.iter().any(|kw| text.to_lowercase().contains(kw)) {
            0.7
        } else {
            0.2
        }
    }
}

pub fn initialize_evals(engine: Arc<EvaluationEngine>) -> anyhow::Result<()> {
    let mut evals = EVALS_ENGINE.lock();
    *evals = Some(engine);
    Ok(())
}

pub async fn shutdown_evals() -> anyhow::Result<()> {
    let evals = EVALS_ENGINE.lock().take();
    if let Some(_engine) = evals {
        tracing::info!("Evaluation engine shut down");
    }
    Ok(())
}

pub fn get_evals() -> Option<Arc<EvaluationEngine>> {
    EVALS_ENGINE.lock().clone()
}

pub fn is_evals_enabled() -> bool {
    EVALS_ENGINE.lock().is_some()
}

static EVALS_ENGINE: parking_lot::Mutex<Option<Arc<EvaluationEngine>>> =
    parking_lot::Mutex::new(None);
