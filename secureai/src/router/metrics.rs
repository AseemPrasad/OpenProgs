use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicF64, Ordering};
use std::time::Duration;

pub struct LatencyMetrics {
    measurements: RwLock<VecDeque<Duration>>,
    max_samples: usize,
}

impl LatencyMetrics {
    pub fn new(max_samples: usize) -> Self {
        Self {
            measurements: RwLock::new(VecDeque::with_capacity(max_samples)),
            max_samples,
        }
    }

    pub fn record_latency(&self, latency: Duration) {
        let mut measurements = self.measurements.write();

        measurements.push_back(latency);
        if measurements.len() > self.max_samples {
            measurements.pop_front();
        }
    }

    pub fn get_average_latency(&self) -> Option<Duration> {
        let measurements = self.measurements.read();
        if measurements.is_empty() {
            return None;
        }

        let sum: Duration = measurements.iter().sum();
        Some(sum / measurements.len() as u32)
    }

    pub fn get_p99_latency(&self) -> Option<Duration> {
        let measurements = self.measurements.read();
        if measurements.is_empty() {
            return None;
        }

        let mut sorted: Vec<_> = measurements.iter().copied().collect();
        sorted.sort();

        let p99_index = ((sorted.len() as f64 * 0.99).ceil() as usize).saturating_sub(1);
        sorted.get(p99_index).copied()
    }

    pub fn get_max_latency(&self) -> Option<Duration> {
        self.measurements.read().iter().max().copied()
    }

    pub fn get_sample_count(&self) -> usize {
        self.measurements.read().len()
    }

    pub fn clear(&self) {
        self.measurements.write().clear();
    }
}

pub struct ProviderMetrics {
    ttfb_metrics: LatencyMetrics,      // Time-To-First-Byte
    processing_metrics: LatencyMetrics, // End-to-end processing time
    error_rate: AtomicF64,
    request_count: parking_lot::Mutex<u64>,
    error_count: parking_lot::Mutex<u64>,
}

impl ProviderMetrics {
    pub fn new(window_size: usize) -> Self {
        Self {
            ttfb_metrics: LatencyMetrics::new(window_size),
            processing_metrics: LatencyMetrics::new(window_size),
            error_rate: AtomicF64::new(0.0),
            request_count: parking_lot::Mutex::new(0),
            error_count: parking_lot::Mutex::new(0),
        }
    }

    pub fn record_success(&self, ttfb: Duration, total: Duration) {
        self.ttfb_metrics.record_latency(ttfb);
        self.processing_metrics.record_latency(total);

        let mut req_count = self.request_count.lock();
        *req_count += 1;

        // Update error rate downward
        self.update_error_rate();
    }

    pub fn record_failure(&self) {
        let mut req_count = self.request_count.lock();
        let mut err_count = self.error_count.lock();

        *req_count += 1;
        *err_count += 1;

        // Update error rate upward
        self.update_error_rate();
    }

    fn update_error_rate(&self) {
        let req_count = *self.request_count.lock();
        let err_count = *self.error_count.lock();

        if req_count > 0 {
            let rate = (err_count as f64) / (req_count as f64);
            self.error_rate.store(rate, Ordering::Release);
        }
    }

    pub fn get_error_rate(&self) -> f64 {
        self.error_rate.load(Ordering::Acquire)
    }

    pub fn get_average_ttfb(&self) -> Option<Duration> {
        self.ttfb_metrics.get_average_latency()
    }

    pub fn get_average_processing(&self) -> Option<Duration> {
        self.processing_metrics.get_average_latency()
    }

    pub fn get_p99_ttfb(&self) -> Option<Duration> {
        self.ttfb_metrics.get_p99_latency()
    }

    pub fn get_p99_processing(&self) -> Option<Duration> {
        self.processing_metrics.get_p99_latency()
    }

    pub fn meets_sla(&self, sla_ttfb_ms: u64, sla_processing_ms: u64) -> bool {
        if let Some(avg_ttfb) = self.get_average_ttfb() {
            if avg_ttfb.as_millis() as u64 > sla_ttfb_ms {
                return false;
            }
        }

        if let Some(avg_processing) = self.get_average_processing() {
            if avg_processing.as_millis() as u64 > sla_processing_ms {
                return false;
            }
        }

        true
    }

    pub fn reset(&self) {
        self.ttfb_metrics.clear();
        self.processing_metrics.clear();
        self.error_rate.store(0.0, Ordering::Release);
        *self.request_count.lock() = 0;
        *self.error_count.lock() = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_metrics_creation() {
        let metrics = LatencyMetrics::new(100);
        assert_eq!(metrics.get_sample_count(), 0);
    }

    #[test]
    fn test_latency_metrics_average() {
        let metrics = LatencyMetrics::new(10);

        metrics.record_latency(Duration::from_millis(100));
        metrics.record_latency(Duration::from_millis(200));
        metrics.record_latency(Duration::from_millis(300));

        let avg = metrics.get_average_latency().unwrap();
        assert_eq!(avg.as_millis(), 200);
    }

    #[test]
    fn test_latency_metrics_p99() {
        let metrics = LatencyMetrics::new(100);

        for i in 1..=100 {
            metrics.record_latency(Duration::from_millis(i));
        }

        let p99 = metrics.get_p99_latency().unwrap();
        assert!(p99.as_millis() >= 99);
    }

    #[test]
    fn test_latency_metrics_sliding_window() {
        let metrics = LatencyMetrics::new(3);

        metrics.record_latency(Duration::from_millis(100));
        metrics.record_latency(Duration::from_millis(200));
        metrics.record_latency(Duration::from_millis(300));

        assert_eq!(metrics.get_sample_count(), 3);

        metrics.record_latency(Duration::from_millis(400));

        // Should still be 3 (sliding window)
        assert_eq!(metrics.get_sample_count(), 3);
    }

    #[test]
    fn test_provider_metrics_success() {
        let metrics = ProviderMetrics::new(10);

        metrics.record_success(
            Duration::from_millis(100),
            Duration::from_millis(500),
        );

        assert!(metrics.get_error_rate() < 0.01); // Should be near 0
    }

    #[test]
    fn test_provider_metrics_error_rate() {
        let metrics = ProviderMetrics::new(10);

        for _ in 0..3 {
            metrics.record_success(Duration::from_millis(100), Duration::from_millis(500));
        }
        for _ in 0..1 {
            metrics.record_failure();
        }

        let error_rate = metrics.get_error_rate();
        assert!(error_rate > 0.2 && error_rate < 0.3); // Should be ~0.25
    }

    #[test]
    fn test_provider_metrics_sla() {
        let metrics = ProviderMetrics::new(10);

        metrics.record_success(Duration::from_millis(100), Duration::from_millis(500));
        metrics.record_success(Duration::from_millis(150), Duration::from_millis(550));

        // Meets SLA: avg TTFB ~125ms (< 500ms), avg processing ~525ms (< 1000ms)
        assert!(metrics.meets_sla(500, 1000));

        // Fails SLA: processing threshold too tight
        assert!(!metrics.meets_sla(500, 500));
    }

    #[test]
    fn test_provider_metrics_reset() {
        let metrics = ProviderMetrics::new(10);

        metrics.record_success(Duration::from_millis(100), Duration::from_millis(500));
        assert!(metrics.get_average_ttfb().is_some());

        metrics.reset();
        assert!(metrics.get_average_ttfb().is_none());
        assert_eq!(metrics.get_error_rate(), 0.0);
    }
}
