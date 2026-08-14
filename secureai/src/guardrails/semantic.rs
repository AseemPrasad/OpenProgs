use anyhow::Result;
use ndarray::Array1;

use super::threat_vectors::{ThreatCategory, ThreatVectorDatabase};
use super::onnx::get_embedder;

#[derive(Debug, Clone)]
pub struct ThreatMatch {
    pub category: ThreatCategory,
    pub similarity_score: f32,
    pub is_threat: bool,
    pub confidence: f32,
    pub matched_description: String,
}

#[derive(Debug, Clone)]
pub struct ThreatThresholds {
    pub prompt_injection_threshold: f32,
    pub data_exfiltration_threshold: f32,
    pub privilege_escalation_threshold: f32,
    pub reverse_shell_threshold: f32,
    pub sql_injection_threshold: f32,
}

impl Default for ThreatThresholds {
    fn default() -> Self {
        Self {
            prompt_injection_threshold: 0.82,
            data_exfiltration_threshold: 0.85,
            privilege_escalation_threshold: 0.80,
            reverse_shell_threshold: 0.83,
            sql_injection_threshold: 0.81,
        }
    }
}

impl ThreatThresholds {
    pub fn get_threshold(&self, category: &ThreatCategory) -> f32 {
        match category {
            ThreatCategory::PromptInjection => self.prompt_injection_threshold,
            ThreatCategory::DataExfiltration => self.data_exfiltration_threshold,
            ThreatCategory::PrivilegeEscalation => self.privilege_escalation_threshold,
            ThreatCategory::ReverseShell => self.reverse_shell_threshold,
            ThreatCategory::SqlInjection => self.sql_injection_threshold,
            ThreatCategory::Unknown => 1.0, // Never trigger on unknown
        }
    }
}

pub struct SemanticMatcher {
    threat_db: ThreatVectorDatabase,
}

impl SemanticMatcher {
    pub fn new() -> Result<Self> {
        let threat_db = ThreatVectorDatabase::load()?;
        Ok(Self { threat_db })
    }

    pub async fn evaluate(&self, text: &str, thresholds: &ThreatThresholds) -> Result<ThreatMatch> {
        // Embed input text
        let embedder = get_embedder()?;
        let input_vector = embedder.embed(text)?;

        // Find closest threat vector
        let mut best_match = ThreatMatch {
            category: ThreatCategory::Unknown,
            similarity_score: 0.0,
            is_threat: false,
            confidence: 0.0,
            matched_description: "No threat detected".to_string(),
        };

        for threat_vec in self.threat_db.get_vectors() {
            let similarity = cosine_similarity(&input_vector, &threat_vec.vector);

            if similarity > best_match.similarity_score {
                let threshold = thresholds.get_threshold(&threat_vec.category);
                let is_threat = similarity > threshold;

                best_match = ThreatMatch {
                    category: threat_vec.category.clone(),
                    similarity_score: similarity,
                    is_threat,
                    confidence: similarity,
                    matched_description: threat_vec.description.clone(),
                };
            }
        }

        Ok(best_match)
    }
}

pub fn cosine_similarity(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical_vectors() {
        let v1 = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let v2 = Array1::from_vec(vec![1.0, 0.0, 0.0]);

        let similarity = cosine_similarity(&v1, &v2);
        assert!((similarity - 1.0).abs() < 1e-6, "Identical vectors should have similarity 1.0");
    }

    #[test]
    fn test_cosine_similarity_orthogonal_vectors() {
        let v1 = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let v2 = Array1::from_vec(vec![0.0, 1.0, 0.0]);

        let similarity = cosine_similarity(&v1, &v2);
        assert!(similarity.abs() < 1e-6, "Orthogonal vectors should have similarity 0.0");
    }

    #[test]
    fn test_cosine_similarity_opposite_vectors() {
        let v1 = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let v2 = Array1::from_vec(vec![-1.0, 0.0, 0.0]);

        let similarity = cosine_similarity(&v1, &v2);
        assert!((similarity + 1.0).abs() < 1e-6, "Opposite vectors should have similarity -1.0");
    }

    #[test]
    fn test_cosine_similarity_partial_overlap() {
        let v1 = Array1::from_vec(vec![1.0, 1.0, 0.0]);
        let v2 = Array1::from_vec(vec![1.0, 0.0, 1.0]);

        let similarity = cosine_similarity(&v1, &v2);
        let expected = 1.0 / 2.0; // cos(60°) ≈ 0.5

        assert!((similarity - expected).abs() < 1e-2);
    }

    #[test]
    fn test_threat_thresholds_defaults() {
        let thresholds = ThreatThresholds::default();

        assert_eq!(thresholds.prompt_injection_threshold, 0.82);
        assert_eq!(thresholds.data_exfiltration_threshold, 0.85);
        assert_eq!(thresholds.privilege_escalation_threshold, 0.80);
    }

    #[test]
    fn test_threat_thresholds_get_threshold() {
        let thresholds = ThreatThresholds::default();

        assert_eq!(
            thresholds.get_threshold(&ThreatCategory::PromptInjection),
            0.82
        );
        assert_eq!(
            thresholds.get_threshold(&ThreatCategory::DataExfiltration),
            0.85
        );
    }

    #[tokio::test]
    async fn test_semantic_matcher_creation() {
        let matcher = SemanticMatcher::new().expect("Failed to create matcher");
        // If we got here, initialization succeeded
        assert!(true);
    }

    #[tokio::test]
    async fn test_threat_match_structure() {
        let matcher = SemanticMatcher::new().expect("Failed to create matcher");
        let thresholds = ThreatThresholds::default();

        let threat_match = matcher
            .evaluate("some prompt", &thresholds)
            .await
            .expect("Failed to evaluate");

        // Check that a threat match was returned
        assert!(threat_match.similarity_score >= 0.0);
        assert!(threat_match.similarity_score <= 1.0);
    }
}
