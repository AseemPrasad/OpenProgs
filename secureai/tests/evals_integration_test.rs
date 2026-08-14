#[cfg(test)]
mod evals_integration_tests {
    use secureai::evals::{
        EvalRequest, EvalMetrics, MetricType, EvalsConfig, EvaluationEngine,
        DriftDetector, DynamicSampler, SamplingStrategy, MetricWindow, Statistics,
    };
    use std::time::Duration;
    use std::collections::HashMap;

    // Test 1: Basic evaluation engine initialization
    #[tokio::test]
    async fn test_evals_engine_initialization() {
        let config = EvalsConfig {
            enabled: true,
            sampling_rate: 0.5,
            boost_flagged_requests: 1.0,
            anomaly_threshold: 3.0,
            short_window_hours: 1,
            long_window_hours: 24,
            alert_enabled: false,
            alert_webhook_url: None,
        };

        let engine = EvaluationEngine::new(config).await;
        assert!(engine.is_ok());
    }

    // Test 2: Disabled evals engine
    #[tokio::test]
    async fn test_disabled_evals_engine() {
        let config = EvalsConfig {
            enabled: false,
            sampling_rate: 0.5,
            boost_flagged_requests: 1.0,
            anomaly_threshold: 3.0,
            short_window_hours: 1,
            long_window_hours: 24,
            alert_enabled: false,
            alert_webhook_url: None,
        };

        let engine = EvaluationEngine::new(config).await;
        assert!(engine.is_ok());

        let eng = engine.unwrap();
        assert!(!eng.should_evaluate(false));
    }

    // Test 3: Async evaluation request queueing
    #[tokio::test]
    async fn test_async_eval_request() {
        let config = EvalsConfig {
            enabled: true,
            sampling_rate: 1.0, // Always evaluate
            boost_flagged_requests: 1.0,
            anomaly_threshold: 3.0,
            short_window_hours: 1,
            long_window_hours: 24,
            alert_enabled: false,
            alert_webhook_url: None,
        };

        let engine = EvaluationEngine::new(config).await.unwrap();

        let request = EvalRequest {
            tenant_id: "tenant-1".to_string(),
            tool_name: "llama3".to_string(),
            prompt: "What is 2+2?".to_string(),
            response: "The answer is 4".to_string(),
            context: HashMap::new(),
        };

        let result = engine.evaluate_request_async(request);
        assert!(result.is_ok());
    }

    // Test 4: Sampling strategy - fixed rate
    #[test]
    fn test_sampling_strategy_fixed_rate() {
        let sampler = DynamicSampler::new(SamplingStrategy::FixedRate(0.5));

        let mut evaluated = 0;
        for _ in 0..100 {
            let decision = sampler.decide(false);
            if decision.should_evaluate {
                evaluated += 1;
            }
        }

        // Should evaluate approximately 50 out of 100
        assert!(evaluated > 30 && evaluated < 70);
    }

    // Test 5: Sampling strategy - adaptive with boost for flagged
    #[test]
    fn test_sampling_strategy_adaptive_boost() {
        let sampler = DynamicSampler::new(SamplingStrategy::AdaptiveRate {
            baseline: 0.1,
            boosted: 1.0,
        });

        // Flagged requests should always evaluate
        let decision = sampler.decide(true);
        assert!(decision.should_evaluate);

        // Non-flagged should sample at baseline rate
        let mut low_samples = 0;
        for _ in 0..100 {
            if sampler.decide(false).should_evaluate {
                low_samples += 1;
            }
        }

        // Should evaluate approximately 10% of non-flagged
        assert!(low_samples < 30);
    }

    // Test 6: Sampling strategy - always sample
    #[test]
    fn test_sampling_strategy_always_sample() {
        let sampler = DynamicSampler::new(SamplingStrategy::AlwaysSample);

        for _ in 0..50 {
            let decision = sampler.decide(false);
            assert!(decision.should_evaluate);
        }
    }

    // Test 7: Metric window statistics
    #[test]
    fn test_metric_window_statistics() {
        use secureai::evals::MetricSample;

        let window = MetricWindow::new(3600); // 1 hour

        // Add samples
        let base_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        for i in 0..10 {
            let sample = MetricSample {
                value: (i as f32) * 0.1,
                timestamp: base_time + (i as u64 * 1000),
                metric_type: MetricType::Toxicity,
            };
            window.add_sample(sample);
        }

        let stats = window.get_statistics();
        assert!(stats.count > 0);
        assert!(stats.mean >= 0.0);
    }

    // Test 8: Metric window with percentiles
    #[test]
    fn test_metric_window_percentiles() {
        use secureai::evals::MetricSample;

        let window = MetricWindow::new(3600);

        // Add 100 samples with values 0.0 to 0.99
        let base_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        for i in 0..100 {
            let sample = MetricSample {
                value: (i as f32) / 100.0,
                timestamp: base_time + (i as u64 * 1000),
                metric_type: MetricType::HallucinationRisk,
            };
            window.add_sample(sample);
        }

        let stats = window.get_statistics();
        assert!(stats.p50 >= 0.4 && stats.p50 <= 0.6);
        assert!(stats.p95 >= 0.9 && stats.p95 <= 1.0);
    }

    // Test 9: Drift detection - 3-sigma rule
    #[test]
    fn test_drift_detection_3sigma() {
        let detector = DriftDetector::new(MetricType::Toxicity, 3.0);

        // Add baseline samples (mean ~0.1, low variance)
        for _ in 0..20 {
            detector.add_sample(0.1);
        }

        // Add normal samples within 1 sigma
        for _ in 0..10 {
            detector.add_sample(0.15);
        }

        // Should not trigger anomaly (within 1 sigma)
        let alerts = detector.detect_anomalies();
        assert!(alerts.is_empty());
    }

    // Test 10: Drift detection - anomaly threshold exceeded
    #[tokio::test]
    async fn test_drift_detection_anomaly() {
        let detector = DriftDetector::new_with_windows(
            MetricType::Toxicity,
            60,  // 1 minute short window
            300, // 5 minute long window
            2.0, // 2-sigma threshold
        );

        // Add baseline samples
        for _ in 0..10 {
            detector.add_sample(0.1);
        }

        // Wait a bit for baseline to settle
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Add drastically different samples (should trigger)
        for _ in 0..5 {
            detector.add_sample(0.9);
        }

        let alerts = detector.detect_anomalies();
        // Anomaly should be detected due to large deviation
        assert!(!alerts.is_empty());
    }

    // Test 11: Drift detector with different metric types
    #[test]
    fn test_drift_detector_metric_types() {
        let metrics = [
            MetricType::Toxicity,
            MetricType::HallucinationRisk,
            MetricType::GuardrailTriggers,
            MetricType::OutputQuality,
        ];

        for metric in &metrics {
            let detector = DriftDetector::new(*metric, 3.0);

            detector.add_sample(0.5);
            detector.add_sample(0.5);
            detector.add_sample(0.5);

            let stats = detector.get_short_window_stats();
            assert_eq!(stats.count, 3);
        }
    }

    // Test 12: Multiple detectors in parallel
    #[test]
    fn test_multiple_drift_detectors() {
        let metrics = [
            MetricType::Toxicity,
            MetricType::HallucinationRisk,
            MetricType::GuardrailTriggers,
        ];

        let detectors: Vec<_> = metrics
            .iter()
            .map(|m| DriftDetector::new(*m, 3.0))
            .collect();

        // Add samples to each
        for (idx, detector) in detectors.iter().enumerate() {
            for _ in 0..10 {
                detector.add_sample((idx as f32) * 0.1);
            }
        }

        // Verify each detector has samples
        for detector in &detectors {
            let stats = detector.get_short_window_stats();
            assert_eq!(stats.count, 10);
        }
    }

    // Test 13: Sampling stats accuracy
    #[test]
    fn test_sampling_stats_accuracy() {
        let sampler = DynamicSampler::new(SamplingStrategy::FixedRate(0.5));

        for _ in 0..100 {
            sampler.decide(false);
        }

        let stats = sampler.get_stats();
        assert_eq!(stats.total_requests, 100);

        // Actual rate should be close to configured 0.5
        assert!(stats.actual_rate > 0.3 && stats.actual_rate < 0.7);
    }

    // Test 14: Evals config deserialization
    #[test]
    fn test_evals_config_defaults() {
        let config = EvalsConfig::default();

        assert_eq!(config.enabled, true);
        assert_eq!(config.sampling_rate, 0.1);
        assert_eq!(config.boost_flagged_requests, 1.0);
        assert_eq!(config.anomaly_threshold, 3.0);
    }

    // Test 15: Multiple concurrent requests
    #[tokio::test]
    async fn test_concurrent_eval_requests() {
        let config = EvalsConfig {
            enabled: true,
            sampling_rate: 1.0,
            boost_flagged_requests: 1.0,
            anomaly_threshold: 3.0,
            short_window_hours: 1,
            long_window_hours: 24,
            alert_enabled: false,
            alert_webhook_url: None,
        };

        let engine = std::sync::Arc::new(EvaluationEngine::new(config).await.unwrap());

        let mut handles = vec![];

        for i in 0..10 {
            let engine_clone = engine.clone();
            let handle = tokio::spawn(async move {
                let request = EvalRequest {
                    tenant_id: format!("tenant-{}", i),
                    tool_name: "llama3".to_string(),
                    prompt: format!("Question {}", i),
                    response: format!("Answer {}", i),
                    context: HashMap::new(),
                };

                engine_clone.evaluate_request_async(request)
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }
    }

    // Test 16: Alert generation
    #[test]
    fn test_alert_generation() {
        let detector = DriftDetector::new(MetricType::Toxicity, 1.0);

        // Add distinct patterns
        for _ in 0..5 {
            detector.add_sample(0.1);
        }

        // Wait and add high values
        for _ in 0..5 {
            detector.add_sample(0.9);
        }

        let alerts = detector.detect_anomalies();

        // Should generate alerts
        assert!(!alerts.is_empty());

        // Alerts should be accessible
        let unread = detector.get_unread_alerts();
        assert_eq!(unread.len(), alerts.len());
    }

    // Test 17: Alert acknowledgment
    #[test]
    fn test_alert_acknowledgment() {
        let detector = DriftDetector::new(MetricType::Toxicity, 1.0);

        detector.add_sample(0.1);
        detector.add_sample(0.1);
        detector.add_sample(0.9);
        detector.add_sample(0.9);

        let alerts = detector.detect_anomalies();
        assert!(!alerts.is_empty());

        detector.acknowledge_alerts();
        let unread = detector.get_unread_alerts();
        assert!(unread.is_empty());
    }

    // Test 18: Window switching (short to long)
    #[test]
    fn test_window_stats_both() {
        let detector = DriftDetector::new_with_windows(
            MetricType::OutputQuality,
            10,
            60,
            3.0,
        );

        for i in 0..20 {
            detector.add_sample((i as f32) * 0.05);
        }

        let short_stats = detector.get_short_window_stats();
        let long_stats = detector.get_long_window_stats();

        // Both should have samples
        assert!(short_stats.count > 0);
        assert!(long_stats.count > 0);

        // Long window should have more samples
        assert!(long_stats.count >= short_stats.count);
    }

    // Test 19: Empty metric window statistics
    #[test]
    fn test_empty_metric_window() {
        let window = MetricWindow::new(3600);
        let stats = window.get_statistics();

        assert_eq!(stats.count, 0);
        assert_eq!(stats.mean, 0.0);
    }

    // Test 20: Single sample metric statistics
    #[test]
    fn test_single_sample_statistics() {
        use secureai::evals::MetricSample;

        let window = MetricWindow::new(3600);
        let base_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let sample = MetricSample {
            value: 0.75,
            timestamp: base_time,
            metric_type: MetricType::HallucinationRisk,
        };

        window.add_sample(sample);

        let stats = window.get_statistics();
        assert_eq!(stats.count, 1);
        assert_eq!(stats.mean, 0.75);
    }
}
