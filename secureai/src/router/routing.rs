use crate::router::provider::ProviderConfig;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStrategy {
    CostOptimized,
    LatencyOptimized,
    Balanced,
    Primary,
}

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub provider_name: String,
    pub reason: String,
    pub fallback_available: bool,
}

pub struct AdaptiveRouter {
    providers: HashMap<String, ProviderConfig>,
    default_provider: String,
    cost_optimization_enabled: bool,
    latency_optimization_enabled: bool,
}

impl AdaptiveRouter {
    pub fn new(
        providers: Vec<ProviderConfig>,
        default_provider: String,
        cost_optimization_enabled: bool,
        latency_optimization_enabled: bool,
    ) -> anyhow::Result<Self> {
        let mut provider_map = HashMap::new();
        for provider in providers {
            provider_map.insert(provider.name.clone(), provider);
        }

        Ok(Self {
            providers: provider_map,
            default_provider,
            cost_optimization_enabled,
            latency_optimization_enabled,
        })
    }

    pub fn decide_route(
        &self,
        strategy: RoutingStrategy,
        healthy_providers: &[String],
    ) -> anyhow::Result<RoutingDecision> {
        match strategy {
            RoutingStrategy::CostOptimized if self.cost_optimization_enabled => {
                self.find_cheapest_provider(healthy_providers)
            }
            RoutingStrategy::LatencyOptimized if self.latency_optimization_enabled => {
                self.find_fastest_provider(healthy_providers)
            }
            RoutingStrategy::Balanced
                if self.cost_optimization_enabled && self.latency_optimization_enabled =>
            {
                self.find_balanced_provider(healthy_providers)
            }
            _ => self.route_primary_with_fallback(&self.default_provider),
        }
    }

    fn find_cheapest_provider(
        &self,
        healthy_providers: &[String],
    ) -> anyhow::Result<RoutingDecision> {
        let mut cheapest: Option<&ProviderConfig> = None;

        for name in healthy_providers {
            if let Some(provider) = self.providers.get(name) {
                if cheapest.is_none()
                    || provider.cost_per_1k_tokens < cheapest.unwrap().cost_per_1k_tokens
                {
                    cheapest = Some(provider);
                }
            }
        }

        if let Some(provider) = cheapest {
            Ok(RoutingDecision {
                provider_name: provider.name.clone(),
                reason: format!("Cost optimization (${:.4}/1k tokens)", provider.cost_per_1k_tokens),
                fallback_available: provider.fallback_to.is_some(),
            })
        } else {
            Err(anyhow::anyhow!("No healthy providers available"))
        }
    }

    fn find_fastest_provider(
        &self,
        healthy_providers: &[String],
    ) -> anyhow::Result<RoutingDecision> {
        let mut fastest: Option<&ProviderConfig> = None;

        for name in healthy_providers {
            if let Some(provider) = self.providers.get(name) {
                if fastest.is_none()
                    || provider.sla_ttfb_ms < fastest.unwrap().sla_ttfb_ms
                {
                    fastest = Some(provider);
                }
            }
        }

        if let Some(provider) = fastest {
            Ok(RoutingDecision {
                provider_name: provider.name.clone(),
                reason: format!("Latency optimization (TTFB SLA: {}ms)", provider.sla_ttfb_ms),
                fallback_available: provider.fallback_to.is_some(),
            })
        } else {
            Err(anyhow::anyhow!("No healthy providers available"))
        }
    }

    fn find_balanced_provider(
        &self,
        healthy_providers: &[String],
    ) -> anyhow::Result<RoutingDecision> {
        let mut best_provider: Option<&ProviderConfig> = None;
        let mut best_score = -1.0;

        for name in healthy_providers {
            if let Some(provider) = self.providers.get(name) {
                // Normalize cost (inverse, lower is better)
                let cost_score = 1.0 / (provider.cost_per_1k_tokens + 0.001);

                // Normalize latency (inverse, lower is better)
                let latency_score = 1000.0 / (provider.sla_ttfb_ms as f64 + 1.0);

                // Balanced score: 50/50 weight
                let score = (cost_score * 0.5) + (latency_score * 0.5);

                if score > best_score {
                    best_score = score;
                    best_provider = Some(provider);
                }
            }
        }

        if let Some(provider) = best_provider {
            Ok(RoutingDecision {
                provider_name: provider.name.clone(),
                reason: "Balanced routing (cost + latency)".to_string(),
                fallback_available: provider.fallback_to.is_some(),
            })
        } else {
            Err(anyhow::anyhow!("No healthy providers available"))
        }
    }

    fn route_primary_with_fallback(&self, provider_name: &str) -> anyhow::Result<RoutingDecision> {
        if let Some(provider) = self.providers.get(provider_name) {
            Ok(RoutingDecision {
                provider_name: provider_name.to_string(),
                reason: "Primary provider routing".to_string(),
                fallback_available: provider.fallback_to.is_some(),
            })
        } else {
            Err(anyhow::anyhow!("Provider '{}' not found", provider_name))
        }
    }

    pub fn get_fallback_chain(&self, provider_name: &str) -> Vec<String> {
        let mut chain = vec![provider_name.to_string()];
        let mut current = provider_name;

        while let Some(provider) = self.providers.get(current) {
            if let Some(fallback) = &provider.fallback_to {
                chain.push(fallback.clone());
                current = fallback;
            } else {
                break;
            }
        }

        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::provider::ProviderConfig;

    fn create_test_provider(name: &str, cost: f32) -> ProviderConfig {
        ProviderConfig {
            name: name.to_string(),
            endpoint_url: format!("http://{}", name),
            model: "test".to_string(),
            cost_per_1k_tokens: cost,
            max_tokens_per_min: 10000,
            timeout_seconds: 30,
            sla_ttfb_ms: 500,
            sla_processing_ms: 10000,
            fallback_to: None,
        }
    }

    #[test]
    fn test_routing_decision_primary() {
        let router = AdaptiveRouter::new(
            vec![create_test_provider("provider1", 0.01)],
            "provider1".to_string(),
            false,
            false,
        )
        .unwrap();

        let decision =
            router.decide_route(RoutingStrategy::Primary, &["provider1".to_string()]);

        assert!(decision.is_ok());
        assert_eq!(decision.unwrap().provider_name, "provider1");
    }

    #[test]
    fn test_routing_decision_cost() {
        let router = AdaptiveRouter::new(
            vec![
                create_test_provider("cheap", 0.001),
                create_test_provider("expensive", 0.01),
            ],
            "cheap".to_string(),
            true,
            false,
        )
        .unwrap();

        let decision = router.decide_route(
            RoutingStrategy::CostOptimized,
            &["cheap".to_string(), "expensive".to_string()],
        );

        assert!(decision.is_ok());
        assert_eq!(decision.unwrap().provider_name, "cheap");
    }

    #[test]
    fn test_fallback_chain() {
        let mut p1 = create_test_provider("p1", 0.01);
        p1.fallback_to = Some("p2".to_string());

        let mut p2 = create_test_provider("p2", 0.01);
        p2.fallback_to = Some("p3".to_string());

        let p3 = create_test_provider("p3", 0.01);

        let router =
            AdaptiveRouter::new(vec![p1, p2, p3], "p1".to_string(), false, false).unwrap();

        let chain = router.get_fallback_chain("p1");
        assert_eq!(chain, vec!["p1", "p2", "p3"]);
    }
}
