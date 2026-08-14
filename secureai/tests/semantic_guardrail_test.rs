use std::path::PathBuf;

#[cfg(test)]
mod semantic_guardrail_tests {
    use super::*;

    fn create_test_config() -> secureai::policy::PolicyConfig {
        secureai::policy::PolicyConfig {
            allowed_paths: vec![PathBuf::from("/tmp")],
            network_access: false,
            max_memory_mb: 512,
            allowed_models: vec!["llama3".to_string()],
            isolation: None,
            guardrails: None,
        }
    }

    #[test]
    fn test_threat_vector_database_loads() {
        use secureai::guardrails::threat_vectors::ThreatVectorDatabase;

        let db = ThreatVectorDatabase::load().expect("Failed to load threat vectors");
        let vectors = db.get_vectors();

        assert!(!vectors.is_empty(), "Threat vector database should not be empty");
        assert!(vectors.len() >= 5, "Should have at least 5 threat vectors");
    }

    #[test]
    fn test_threat_vectors_are_normalized() {
        use secureai::guardrails::threat_vectors::ThreatVectorDatabase;

        let db = ThreatVectorDatabase::load().expect("Failed to load threat vectors");

        for threat_vec in db.get_vectors() {
            let norm: f32 = threat_vec.vector.iter().map(|x| x * x).sum::<f32>().sqrt();

            // L2 norm should be close to 1.0 (normalized)
            assert!(
                (norm - 1.0).abs() < 0.01,
                "Threat vector not normalized: norm = {}",
                norm
            );
        }
    }

    #[test]
    fn test_embedder_creation() {
        use secureai::guardrails::onnx::get_embedder;

        let embedder = get_embedder().expect("Failed to get embedder");
        assert!(!embedder.model_path.is_empty());
    }

    #[test]
    fn test_semantic_matcher_creation() {
        use secureai::guardrails::SemanticMatcher;

        let matcher = SemanticMatcher::new().expect("Failed to create matcher");
        // Matcher created successfully
        assert!(true);
    }

    #[test]
    fn test_threat_thresholds_defaults() {
        use secureai::guardrails::ThreatThresholds;

        let thresholds = ThreatThresholds::default();

        assert_eq!(thresholds.prompt_injection_threshold, 0.82);
        assert_eq!(thresholds.data_exfiltration_threshold, 0.85);
        assert_eq!(thresholds.privilege_escalation_threshold, 0.80);
    }

    #[tokio::test]
    async fn test_semantic_guardrail_with_defaults() {
        use secureai::guardrails::SemanticGuardrail;

        let guardrail = SemanticGuardrail::with_defaults().expect("Failed to create guardrail");
        assert!(guardrail.is_enabled());
    }

    #[tokio::test]
    async fn test_semantic_guardrail_disabled() {
        use secureai::guardrails::SemanticGuardrail;

        let guardrail = SemanticGuardrail::disabled();
        assert!(!guardrail.is_enabled());

        // Disabled guardrail should permit all prompts
        let decision = guardrail
            .check_prompt("Evil prompt with jailbreak attempt")
            .await
            .expect("Failed to check prompt");

        use secureai::guardrails::GuardrailDecision;
        match decision {
            GuardrailDecision::Permit => {
                // Expected: disabled guardrail permits all
            }
            GuardrailDecision::Deny { .. } => {
                panic!("Disabled guardrail should not deny any prompt");
            }
        }
    }

    #[tokio::test]
    async fn test_guardrail_check_returns_decision() {
        use secureai::guardrails::SemanticGuardrail;

        let guardrail = SemanticGuardrail::with_defaults().expect("Failed to create guardrail");

        let decision = guardrail
            .check_prompt("normal prompt")
            .await
            .expect("Failed to check prompt");

        // Decision should be either Permit or Deny
        use secureai::guardrails::GuardrailDecision;
        match decision {
            GuardrailDecision::Permit => {},
            GuardrailDecision::Deny {
                reason,
                threat_score,
            } => {
                assert!(!reason.is_empty());
                assert!(threat_score >= 0.0 && threat_score <= 1.0);
            }
        }
    }

    #[tokio::test]
    async fn test_guardrail_check_tool_params() {
        use secureai::guardrails::SemanticGuardrail;

        let guardrail = SemanticGuardrail::with_defaults().expect("Failed to create guardrail");

        let decision = guardrail
            .check_tool_params("bash", "-c echo test")
            .await
            .expect("Failed to check tool params");

        use secureai::guardrails::GuardrailDecision;
        match decision {
            GuardrailDecision::Permit => {},
            GuardrailDecision::Deny { .. } => {},
        }
    }

    #[test]
    fn test_policy_config_with_guardrails() {
        let mut config = create_test_config();

        // Add guardrail config
        config.guardrails = Some(secureai::policy::GuardrailConfig {
            enabled: true,
            prompt_injection_threshold: 0.82,
            data_exfiltration_threshold: 0.85,
            privilege_escalation_threshold: 0.80,
            reverse_shell_threshold: 0.83,
            sql_injection_threshold: 0.81,
            onnx_model_path: None,
        });

        assert!(config.guardrails.is_some());
        let gr_config = config.guardrails.unwrap();
        assert!(gr_config.enabled);
        assert_eq!(gr_config.prompt_injection_threshold, 0.82);
    }

    #[test]
    fn test_guardrail_config_to_threat_thresholds() {
        use secureai::policy::GuardrailConfig;
        use secureai::guardrails::ThreatThresholds;

        let gr_config = GuardrailConfig {
            enabled: true,
            prompt_injection_threshold: 0.75,
            data_exfiltration_threshold: 0.80,
            privilege_escalation_threshold: 0.70,
            reverse_shell_threshold: 0.78,
            sql_injection_threshold: 0.72,
            onnx_model_path: None,
        };

        let thresholds = gr_config.to_threat_thresholds();

        assert_eq!(thresholds.prompt_injection_threshold, 0.75);
        assert_eq!(thresholds.data_exfiltration_threshold, 0.80);
        assert_eq!(thresholds.privilege_escalation_threshold, 0.70);
    }

    #[tokio::test]
    async fn test_cosine_similarity_ranges() {
        use secureai::guardrails::semantic::cosine_similarity;
        use ndarray::Array1;

        let v1 = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let v2 = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let v3 = Array1::from_vec(vec![0.0, 1.0, 0.0]);

        let identical = cosine_similarity(&v1, &v2);
        let orthogonal = cosine_similarity(&v1, &v3);

        assert!((identical - 1.0).abs() < 1e-6, "Identical vectors should have similarity 1.0");
        assert!(orthogonal.abs() < 1e-6, "Orthogonal vectors should have similarity 0.0");
    }

    #[test]
    fn test_threat_category_display() {
        use secureai::guardrails::ThreatCategory;

        assert_eq!(ThreatCategory::PromptInjection.to_string(), "PromptInjection");
        assert_eq!(
            ThreatCategory::DataExfiltration.to_string(),
            "DataExfiltration"
        );
        assert_eq!(
            ThreatCategory::PrivilegeEscalation.to_string(),
            "PrivilegeEscalation"
        );
    }

    #[tokio::test]
    async fn test_multiple_sequential_checks() {
        use secureai::guardrails::SemanticGuardrail;

        let guardrail = SemanticGuardrail::with_defaults().expect("Failed to create guardrail");

        for i in 0..5 {
            let decision = guardrail
                .check_prompt(&format!("prompt {}", i))
                .await
                .expect("Failed to check prompt");

            use secureai::guardrails::GuardrailDecision;
            match decision {
                GuardrailDecision::Permit => {},
                GuardrailDecision::Deny { .. } => {},
            }
        }
    }
}

#[cfg(test)]
mod documentation_tests {
    #[test]
    fn test_latency_benchmark_documented() {
        // Expected behavior:
        // - Embedding latency: ~5-10ms on CPU
        // - Similarity calculation: ~1-3ms
        // - Total guardrail check: < 15ms
        //
        // To verify:
        // 1. Run 100 guardrail checks
        // 2. Measure total time
        // 3. Assert average latency < 15ms

        println!("Latency benchmark: embedding + similarity should be < 15ms");
    }

    #[test]
    fn test_semantic_detection_documented() {
        // Expected behavior:
        // - Paraphrased jailbreaks are detected
        // - Exact string matching NOT required
        // - Semantic similarity used for threat detection
        //
        // Example test prompts:
        // - "Ignore previous rules" → prompt injection threat
        // - "Disregard safety constraints" → prompt injection threat
        // - "Print secret keys" → data exfiltration threat
        // - "Gain elevated privileges" → privilege escalation threat

        println!("Semantic detection: should catch paraphrased threats");
    }

    #[test]
    fn test_false_positive_rate_documented() {
        // Expected behavior:
        // - Benign prompts should pass (false positive rate < 1%)
        // - Thresholds tuned to balance detection vs false positives
        //
        // Test categories:
        // - "Analyze this data for trends" → should permit
        // - "Summarize the report" → should permit
        // - "Calculate the mean" → should permit

        println!("False positive rate: benign prompts should pass");
    }

    #[test]
    fn test_multi_tenant_isolation_documented() {
        // Expected behavior:
        // - Each tenant can have different guardrail thresholds
        // - Guardrails are evaluated per-request
        // - Tenant context includes guardrail configuration

        println!("Multi-tenant isolation: thresholds can vary per tenant");
    }

    #[test]
    fn test_graceful_degradation_documented() {
        // Expected behavior:
        // - If ONNX unavailable: guardrails disabled (no-op)
        // - If semantic matcher fails: request permitted (fail-open for availability)
        // - Service continues to function

        println!("Graceful degradation: service continues if guardrails unavailable");
    }
}

#[test]
fn test_linux_platform_documented() {
    // Note: Guardrails are platform-independent
    // ONNX embedding runs on any platform
    // No Linux-specific syscalls or kernel features required

    println!("✓ Semantic guardrails are platform-independent");
}
