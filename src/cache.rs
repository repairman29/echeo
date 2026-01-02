use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use crate::vectorizer::EmbeddedCapability;

/// THE CACHE: Stores embeddings to avoid re-computation
pub struct EmbeddingCache {
    cache_path: PathBuf,
    cache: HashMap<String, Vec<f32>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    hash: String,
    embedding: Vec<f32>,
}

impl EmbeddingCache {
    pub fn new(cache_dir: &Path) -> Result<Self> {
        let cache_path = cache_dir.join(".payload").join("embeddings_cache.json");
        let cache = if cache_path.exists() {
            let content = fs::read_to_string(&cache_path)?;
            let entries: Vec<CacheEntry> = serde_json::from_str(&content).unwrap_or_default();
            entries
                .into_iter()
                .map(|e| (e.hash, e.embedding))
                .collect()
        } else {
            HashMap::new()
        };

        Ok(Self { cache_path, cache })
    }

    /// Generate hash for a capability
    fn hash_capability(name: &str, code: &str, kind: &str) -> String {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        code.hash(&mut hasher);
        kind.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get cached embedding
    pub fn get(&self, name: &str, code: &str, kind: &str) -> Option<Vec<f32>> {
        let hash = Self::hash_capability(name, code, kind);
        self.cache.get(&hash).cloned()
    }

    /// Store embedding in cache
    pub fn set(&mut self, name: &str, code: &str, kind: &str, embedding: Vec<f32>) -> Result<()> {
        let hash = Self::hash_capability(name, code, kind);
        self.cache.insert(hash, embedding);
        self.save()?;
        Ok(())
    }

    /// Save cache to disk
    fn save(&self) -> Result<()> {
        if let Some(parent) = self.cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let entries: Vec<CacheEntry> = self
            .cache
            .iter()
            .map(|(hash, embedding)| CacheEntry {
                hash: hash.clone(),
                embedding: embedding.clone(),
            })
            .collect();

        let content = serde_json::to_string_pretty(&entries)?;
        fs::write(&self.cache_path, content)?;
        Ok(())
    }

    /// Get cache stats
    pub fn stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.values().map(|v| v.len()).sum::<usize>())
    }
}

