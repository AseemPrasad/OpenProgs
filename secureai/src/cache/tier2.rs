use crate::cache::CacheEntry;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct SemanticCache {
    entries: Arc<RwLock<Vec<(String, CacheEntry, Vec<f32>)>>>,
    similarity_threshold: f32,
    max_entries: usize,
}

impl SemanticCache {
    pub fn new(similarity_threshold: f32, max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            similarity_threshold,
            max_entries,
        }
    }

    pub fn put(&self, entry: CacheEntry, embedding: Vec<f32>) {
        let key = entry.key.clone();
        let mut entries = self.entries.write();

        // Remove if already exists
        entries.retain(|(k, _, _)| k != &key);

        // Add new entry
        entries.push((key, entry, embedding));

        // Prune if exceeds max_entries (remove oldest)
        if entries.len() > self.max_entries {
            entries.remove(0);
        }
    }

    pub fn find_similar(&self, query_embedding: &[f32]) -> Option<(CacheEntry, f32)> {
        let entries = self.entries.read();

        let mut best_match: Option<(CacheEntry, f32)> = None;
        let mut best_similarity = self.similarity_threshold;

        for (_, entry, cached_embedding) in entries.iter() {
            let distance = Self::cosine_distance(query_embedding, cached_embedding);

            // Higher similarity = lower distance
            let similarity = 1.0 - distance;

            if similarity > best_similarity {
                best_similarity = similarity;
                best_match = Some((entry.clone(), similarity));
            }
        }

        best_match
    }

    pub fn cosine_distance(vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() || vec1.is_empty() {
            return 1.0; // Max distance
        }

        let mut dot_product = 0.0;
        let mut mag1 = 0.0;
        let mut mag2 = 0.0;

        for i in 0..vec1.len() {
            dot_product += vec1[i] * vec2[i];
            mag1 += vec1[i] * vec1[i];
            mag2 += vec2[i] * vec2[i];
        }

        mag1 = mag1.sqrt();
        mag2 = mag2.sqrt();

        if mag1 == 0.0 || mag2 == 0.0 {
            return 1.0;
        }

        1.0 - (dot_product / (mag1 * mag2))
    }

    pub fn invalidate(&self, key: &str) {
        let mut entries = self.entries.write();
        entries.retain(|(k, _, _)| k != key);
    }

    pub fn invalidate_all(&self) {
        self.entries.write().clear();
    }

    pub fn get_entry_count(&self) -> usize {
        self.entries.read().len()
    }

    pub fn get_max_entries(&self) -> usize {
        self.max_entries
    }

    pub fn get_threshold(&self) -> f32 {
        self.similarity_threshold
    }
}
