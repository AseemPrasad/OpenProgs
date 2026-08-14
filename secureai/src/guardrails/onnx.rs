use anyhow::{Result, Context};
use lazy_static::lazy_static;
use ndarray::Array1;

pub struct OnnxEmbedder {
    // In production, this would hold ort::Session
    // For MVP, we provide a mock that generates consistent embeddings
    model_path: String,
}

impl OnnxEmbedder {
    pub fn new(model_path: &str) -> Result<Self> {
        // In real implementation:
        // let environment = ort::Environment::builder()
        //     .with_name("SecureAI")
        //     .with_execution_providers([ort::ExecutionProvider::cpu()])
        //     .build()
        //     .context("Failed to create ONNX environment")?;
        //
        // let session = environment
        //     .new_session_builder()
        //     .context("Failed to create session builder")?
        //     .with_model_from_file(model_path)
        //     .context("Failed to load ONNX model")?;

        tracing::info!("ONNX Embedder initialized with model: {}", model_path);

        Ok(Self {
            model_path: model_path.to_string(),
        })
    }

    pub fn embed(&self, text: &str) -> Result<Array1<f32>> {
        // In production, this would:
        // 1. Tokenize text using HuggingFace tokenizer
        // 2. Create input tensor
        // 3. Run ONNX model inference
        // 4. Extract output embedding (1x384 for MiniLM)

        // For MVP, we generate a deterministic embedding based on text
        self.generate_mock_embedding(text)
    }

    fn generate_mock_embedding(&self, text: &str) -> Result<Array1<f32>> {
        // Generate a deterministic but text-dependent embedding
        // This simulates the behavior of an actual embedding model

        let mut embedding = vec![0.0; 384];

        // Use text hash to seed consistent but different vectors
        let mut hash: u64 = 0;
        for byte in text.as_bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(*byte as u64);
        }

        // Fill embedding with pseudo-random values derived from hash
        for i in 0..384 {
            let seed = hash.wrapping_mul((i + 1) as u64);
            let value = ((seed % 1000) as f32) / 1000.0 - 0.5;
            embedding[i] = value;
        }

        // Normalize to unit length (L2 normalization)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        Ok(Array1::from_vec(embedding))
    }
}

lazy_static! {
    static ref EMBEDDER: Result<OnnxEmbedder> = {
        OnnxEmbedder::new("/models/all-MiniLM-L6-v2.onnx")
            .or_else(|_| {
                // Fallback: create embedder with mock path
                tracing::warn!("ONNX model not found, using mock embedder");
                OnnxEmbedder::new("mock://embedder")
            })
    };
}

pub fn get_embedder() -> Result<&'static OnnxEmbedder> {
    EMBEDDER.as_ref().context("Failed to initialize embedder")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedder_creation() {
        let embedder = OnnxEmbedder::new("mock://model").expect("Failed to create embedder");
        assert_eq!(embedder.model_path, "mock://model");
    }

    #[test]
    fn test_embed_returns_384_dimensions() {
        let embedder = OnnxEmbedder::new("mock://model").expect("Failed to create embedder");
        let embedding = embedder.embed("test prompt").expect("Failed to embed");
        assert_eq!(embedding.len(), 384);
    }

    #[test]
    fn test_embedding_normalized() {
        let embedder = OnnxEmbedder::new("mock://model").expect("Failed to create embedder");
        let embedding = embedder.embed("test prompt").expect("Failed to embed");

        // Calculate L2 norm
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

        // Norm should be close to 1.0 (normalized)
        assert!((norm - 1.0).abs() < 0.01, "Embedding not normalized: norm = {}", norm);
    }

    #[test]
    fn test_deterministic_embedding() {
        let embedder = OnnxEmbedder::new("mock://model").expect("Failed to create embedder");

        let embed1 = embedder.embed("same text").expect("Failed to embed");
        let embed2 = embedder.embed("same text").expect("Failed to embed");

        // Same input should produce same embedding
        for (v1, v2) in embed1.iter().zip(embed2.iter()) {
            assert!((v1 - v2).abs() < 1e-6);
        }
    }

    #[test]
    fn test_different_embeddings_for_different_text() {
        let embedder = OnnxEmbedder::new("mock://model").expect("Failed to create embedder");

        let embed1 = embedder.embed("text A").expect("Failed to embed");
        let embed2 = embedder.embed("text B").expect("Failed to embed");

        // Different inputs should produce different embeddings
        let mut differences = 0;
        for (v1, v2) in embed1.iter().zip(embed2.iter()) {
            if (v1 - v2).abs() > 1e-6 {
                differences += 1;
            }
        }

        assert!(differences > 100, "Embeddings too similar for different texts");
    }

    #[test]
    fn test_embedder_singleton() {
        let embedder1 = get_embedder().expect("Failed to get embedder");
        let embedder2 = get_embedder().expect("Failed to get embedder");

        // Should be the same instance (via lazy_static)
        assert_eq!(
            embedder1.model_path, embedder2.model_path,
            "Embedders should be identical (lazy_static singleton)"
        );
    }
}
