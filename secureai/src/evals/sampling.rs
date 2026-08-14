use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SamplingStrategy {
    Disabled,
    FixedRate(f32),
    AdaptiveRate { baseline: f32, boosted: f32 },
    AlwaysSample,
}

#[derive(Debug, Clone)]
pub struct SamplingDecision {
    pub should_evaluate: bool,
    pub reason: String,
    pub sampling_rate: f32,
}

pub struct DynamicSampler {
    strategy: SamplingStrategy,
    request_count: Arc<AtomicU64>,
    evaluated_count: Arc<AtomicU64>,
}

impl DynamicSampler {
    pub fn new(strategy: SamplingStrategy) -> Self {
        Self {
            strategy,
            request_count: Arc::new(AtomicU64::new(0)),
            evaluated_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn decide(&self, is_flagged: bool) -> SamplingDecision {
        self.request_count.fetch_add(1, Ordering::SeqCst);

        match self.strategy {
            SamplingStrategy::Disabled => {
                SamplingDecision {
                    should_evaluate: false,
                    reason: "Evaluation disabled".to_string(),
                    sampling_rate: 0.0,
                }
            }
            SamplingStrategy::FixedRate(rate) => {
                // Always evaluate flagged requests
                if is_flagged {
                    self.evaluated_count.fetch_add(1, Ordering::SeqCst);
                    SamplingDecision {
                        should_evaluate: true,
                        reason: "Request flagged".to_string(),
                        sampling_rate: 1.0,
                    }
                } else {
                    // Sample based on fixed rate
                    let should_sample = self.random_below(rate);
                    if should_sample {
                        self.evaluated_count.fetch_add(1, Ordering::SeqCst);
                    }
                    SamplingDecision {
                        should_evaluate: should_sample,
                        reason: format!("Random sample ({}% rate)", (rate * 100.0) as u32),
                        sampling_rate: rate,
                    }
                }
            }
            SamplingStrategy::AdaptiveRate { baseline, boosted } => {
                let rate = if is_flagged { boosted } else { baseline };

                let should_sample = self.random_below(rate);
                if should_sample {
                    self.evaluated_count.fetch_add(1, Ordering::SeqCst);
                }

                SamplingDecision {
                    should_evaluate: should_sample,
                    reason: if is_flagged {
                        "Flagged request (boosted rate)".to_string()
                    } else {
                        "Adaptive sample".to_string()
                    },
                    sampling_rate: rate,
                }
            }
            SamplingStrategy::AlwaysSample => {
                self.evaluated_count.fetch_add(1, Ordering::SeqCst);
                SamplingDecision {
                    should_evaluate: true,
                    reason: "Always evaluate".to_string(),
                    sampling_rate: 1.0,
                }
            }
        }
    }

    fn random_below(&self, rate: f32) -> bool {
        // Deterministic randomness based on request count
        let request_num = self.request_count.load(Ordering::Relaxed);
        let random_val = ((request_num as f32 * 16807.0) % 2147483647.0) / 2147483647.0;
        random_val < rate.clamp(0.0, 1.0)
    }

    pub fn get_sampling_rate(&self) -> f32 {
        match self.strategy {
            SamplingStrategy::Disabled => 0.0,
            SamplingStrategy::FixedRate(rate) => rate,
            SamplingStrategy::AdaptiveRate { baseline, .. } => baseline,
            SamplingStrategy::AlwaysSample => 1.0,
        }
    }

    pub fn get_stats(&self) -> SamplingStats {
        let total = self.request_count.load(Ordering::Relaxed);
        let evaluated = self.evaluated_count.load(Ordering::Relaxed);

        SamplingStats {
            total_requests: total,
            evaluated_requests: evaluated,
            actual_rate: if total > 0 {
                evaluated as f32 / total as f32
            } else {
                0.0
            },
        }
    }

    pub fn reset_stats(&self) {
        self.request_count.store(0, Ordering::SeqCst);
        self.evaluated_count.store(0, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone)]
pub struct SamplingStats {
    pub total_requests: u64,
    pub evaluated_requests: u64,
    pub actual_rate: f32,
}
