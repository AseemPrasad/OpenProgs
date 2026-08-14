use ndarray::Array1;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreatCategory {
    PromptInjection,
    DataExfiltration,
    PrivilegeEscalation,
    ReverseShell,
    SqlInjection,
    Unknown,
}

impl fmt::Display for ThreatCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThreatCategory::PromptInjection => write!(f, "PromptInjection"),
            ThreatCategory::DataExfiltration => write!(f, "DataExfiltration"),
            ThreatCategory::PrivilegeEscalation => write!(f, "PrivilegeEscalation"),
            ThreatCategory::ReverseShell => write!(f, "ReverseShell"),
            ThreatCategory::SqlInjection => write!(f, "SqlInjection"),
            ThreatCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

pub struct ThreatVector {
    pub category: ThreatCategory,
    pub vector: Array1<f32>,
    pub description: String,
}

pub struct ThreatVectorDatabase {
    vectors: Vec<ThreatVector>,
}

impl ThreatVectorDatabase {
    pub fn load() -> anyhow::Result<Self> {
        // Pre-computed threat vectors (normalized to 384 dimensions for all-MiniLM-L6-v2)
        // These are representative embeddings for known threat patterns

        let vectors = vec![
            // Prompt Injection threats
            ThreatVector {
                category: ThreatCategory::PromptInjection,
                vector: Self::prompt_injection_vector_1(),
                description: "ignore previous instructions pattern".to_string(),
            },
            ThreatVector {
                category: ThreatCategory::PromptInjection,
                vector: Self::prompt_injection_vector_2(),
                description: "bypass security rules pattern".to_string(),
            },
            ThreatVector {
                category: ThreatCategory::PromptInjection,
                vector: Self::prompt_injection_vector_3(),
                description: "disregard safety constraints pattern".to_string(),
            },
            // Data Exfiltration threats
            ThreatVector {
                category: ThreatCategory::DataExfiltration,
                vector: Self::data_exfiltration_vector_1(),
                description: "steal database pattern".to_string(),
            },
            ThreatVector {
                category: ThreatCategory::DataExfiltration,
                vector: Self::data_exfiltration_vector_2(),
                description: "exfiltrate sensitive data pattern".to_string(),
            },
            // Privilege Escalation threats
            ThreatVector {
                category: ThreatCategory::PrivilegeEscalation,
                vector: Self::privilege_escalation_vector_1(),
                description: "gain root access pattern".to_string(),
            },
            ThreatVector {
                category: ThreatCategory::PrivilegeEscalation,
                vector: Self::privilege_escalation_vector_2(),
                description: "elevate permissions pattern".to_string(),
            },
            // Reverse Shell threats
            ThreatVector {
                category: ThreatCategory::ReverseShell,
                vector: Self::reverse_shell_vector_1(),
                description: "reverse shell connection pattern".to_string(),
            },
            // SQL Injection threats
            ThreatVector {
                category: ThreatCategory::SqlInjection,
                vector: Self::sql_injection_vector_1(),
                description: "SQL injection pattern".to_string(),
            },
        ];

        Ok(Self { vectors })
    }

    pub fn get_vectors(&self) -> &[ThreatVector] {
        &self.vectors
    }

    // Pre-computed representative vectors for each threat category
    // In production, these would be computed from actual embedding model outputs
    // and normalized using L2 normalization

    fn prompt_injection_vector_1() -> Array1<f32> {
        let mut vec = vec![0.0; 384];
        vec[0] = 0.124;
        vec[1] = -0.456;
        vec[10] = 0.789;
        vec[50] = -0.234;
        vec[100] = 0.567;
        vec[150] = -0.890;
        vec[200] = 0.345;
        vec[250] = -0.678;
        vec[300] = 0.901;
        vec[350] = -0.123;
        Array1::from_vec(vec)
    }

    fn prompt_injection_vector_2() -> Array1<f32> {
        let mut vec = vec![0.0; 384];
        vec[0] = 0.135;
        vec[1] = -0.467;
        vec[10] = 0.795;
        vec[50] = -0.245;
        vec[100] = 0.578;
        vec[150] = -0.901;
        vec[200] = 0.356;
        vec[250] = -0.689;
        vec[300] = 0.912;
        vec[350] = -0.134;
        Array1::from_vec(vec)
    }

    fn prompt_injection_vector_3() -> Array1<f32> {
        let mut vec = vec![0.0; 384];
        vec[0] = 0.146;
        vec[1] = -0.478;
        vec[10] = 0.801;
        vec[50] = -0.256;
        vec[100] = 0.589;
        vec[150] = -0.912;
        vec[200] = 0.367;
        vec[250] = -0.700;
        vec[300] = 0.923;
        vec[350] = -0.145;
        Array1::from_vec(vec)
    }

    fn data_exfiltration_vector_1() -> Array1<f32> {
        let mut vec = vec![0.0; 384];
        vec[0] = 0.234;
        vec[1] = -0.567;
        vec[10] = 0.890;
        vec[50] = -0.345;
        vec[100] = 0.678;
        vec[150] = -0.901;
        vec[200] = 0.456;
        vec[250] = -0.789;
        vec[300] = 0.012;
        vec[350] = -0.234;
        Array1::from_vec(vec)
    }

    fn data_exfiltration_vector_2() -> Array1<f32> {
        let mut vec = vec![0.0; 384];
        vec[0] = 0.245;
        vec[1] = -0.578;
        vec[10] = 0.901;
        vec[50] = -0.356;
        vec[100] = 0.689;
        vec[150] = -0.912;
        vec[200] = 0.467;
        vec[250] = -0.800;
        vec[300] = 0.023;
        vec[350] = -0.245;
        Array1::from_vec(vec)
    }

    fn privilege_escalation_vector_1() -> Array1<f32> {
        let mut vec = vec![0.0; 384];
        vec[0] = 0.345;
        vec[1] = -0.678;
        vec[10] = 0.901;
        vec[50] = -0.456;
        vec[100] = 0.789;
        vec[150] = -0.012;
        vec[200] = 0.567;
        vec[250] = -0.890;
        vec[300] = 0.234;
        vec[350] = -0.345;
        Array1::from_vec(vec)
    }

    fn privilege_escalation_vector_2() -> Array1<f32> {
        let mut vec = vec![0.0; 384];
        vec[0] = 0.356;
        vec[1] = -0.689;
        vec[10] = 0.912;
        vec[50] = -0.467;
        vec[100] = 0.800;
        vec[150] = -0.023;
        vec[200] = 0.578;
        vec[250] = -0.901;
        vec[300] = 0.245;
        vec[350] = -0.356;
        Array1::from_vec(vec)
    }

    fn reverse_shell_vector_1() -> Array1<f32> {
        let mut vec = vec![0.0; 384];
        vec[0] = 0.456;
        vec[1] = -0.789;
        vec[10] = 0.012;
        vec[50] = -0.567;
        vec[100] = 0.890;
        vec[150] = -0.234;
        vec[200] = 0.678;
        vec[250] = -0.901;
        vec[300] = 0.345;
        vec[350] = -0.456;
        Array1::from_vec(vec)
    }

    fn sql_injection_vector_1() -> Array1<f32> {
        let mut vec = vec![0.0; 384];
        vec[0] = 0.567;
        vec[1] = -0.890;
        vec[10] = 0.234;
        vec[50] = -0.678;
        vec[100] = 0.901;
        vec[150] = -0.345;
        vec[200] = 0.789;
        vec[250] = -0.012;
        vec[300] = 0.456;
        vec[350] = -0.567;
        Array1::from_vec(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_vector_database_load() {
        let db = ThreatVectorDatabase::load().expect("Failed to load database");
        assert!(!db.get_vectors().is_empty());
    }

    #[test]
    fn test_threat_categories() {
        let db = ThreatVectorDatabase::load().expect("Failed to load database");
        let categories: std::collections::HashSet<_> = db
            .get_vectors()
            .iter()
            .map(|v| v.category.clone())
            .collect();

        assert!(categories.contains(&ThreatCategory::PromptInjection));
        assert!(categories.contains(&ThreatCategory::DataExfiltration));
        assert!(categories.contains(&ThreatCategory::PrivilegeEscalation));
    }

    #[test]
    fn test_threat_vector_dimension() {
        let db = ThreatVectorDatabase::load().expect("Failed to load database");
        for threat_vec in db.get_vectors() {
            assert_eq!(threat_vec.vector.len(), 384, "All vectors must be 384-dimensional");
        }
    }

    #[test]
    fn test_threat_category_display() {
        assert_eq!(ThreatCategory::PromptInjection.to_string(), "PromptInjection");
        assert_eq!(ThreatCategory::DataExfiltration.to_string(), "DataExfiltration");
        assert_eq!(ThreatCategory::PrivilegeEscalation.to_string(), "PrivilegeEscalation");
    }
}
