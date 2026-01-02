use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::vectorizer::EmbeddedCapability;

/// THE MATCHMAKER: Connects capabilities to bounties using vector similarity
pub struct Matchmaker {
    capabilities: Vec<EmbeddedCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Need {
    pub id: String,
    pub title: String,
    pub description: String,
    pub bounty: Option<String>, // e.g., "$2,500 (USDC)"
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub need: Need,
    pub capability: EmbeddedCapability,
    pub score: f32, // Ship Velocity Score (0.0 - 1.0)
    pub reasons: Vec<String>, // Why this is a match
}

impl Matchmaker {
    pub fn new(capabilities: Vec<EmbeddedCapability>) -> Self {
        Self { capabilities }
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Calculate Ship Velocity Score
    /// Higher score = more code already exists, faster to ship
    fn calculate_ship_velocity_score(
        similarity: f32,
        capability: &EmbeddedCapability,
        need: &Need,
    ) -> (f32, Vec<String>) {
        let mut score = similarity;
        let mut reasons = Vec::new();

        // Base score from vector similarity (0.0 - 1.0)
        // High similarity means the capability matches the need
        if similarity > 0.7 {
            reasons.push(format!(
                "High semantic similarity ({:.0}%)",
                similarity * 100.0
            ));
        } else if similarity > 0.5 {
            reasons.push(format!(
                "Moderate semantic similarity ({:.0}%)",
                similarity * 100.0
            ));
        }

        // Boost for language match
        if need.description.to_lowercase().contains(&capability.language.to_lowercase()) {
            score += 0.1;
            reasons.push(format!("Language match: {}", capability.language));
        }

        // Boost for capability type relevance
        let need_lower = need.description.to_lowercase();
        let kind_lower = capability.kind.to_lowercase();
        
        if (kind_lower.contains("function") && need_lower.contains("function"))
            || (kind_lower.contains("component") && need_lower.contains("component"))
            || (kind_lower.contains("class") && need_lower.contains("class"))
        {
            score += 0.05;
            reasons.push(format!("Type match: {}", capability.kind));
        }

        // Cap at 1.0
        score = score.min(1.0);

        // Add capability description if available
        if !capability.code_snippet.is_empty() {
            reasons.push(format!("Has existing: {}", capability.name));
        }

        (score, reasons)
    }

    /// Match a single need against all capabilities
    pub fn match_need(&self, need: &Need) -> Vec<Match> {
        let mut matches = Vec::new();

        for capability in &self.capabilities {
            // Skip if capability has no embedding
            if capability.embedding.is_empty() {
                continue;
            }

            // Calculate similarity
            let similarity = Self::cosine_similarity(&need.embedding, &capability.embedding);

            // Only consider matches above threshold
            if similarity > 0.3 {
                let (score, reasons) =
                    Self::calculate_ship_velocity_score(similarity, capability, need);

                matches.push(Match {
                    need: need.clone(),
                    capability: capability.clone(),
                    score,
                    reasons,
                });
            }
        }

        // Sort by score (highest first)
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        matches
    }

    /// Match multiple needs and return top matches
    pub fn match_needs(&self, needs: &[Need], top_k: usize) -> Vec<Match> {
        let mut all_matches = Vec::new();

        for need in needs {
            let matches = self.match_need(need);
            all_matches.extend(matches);
        }

        // Sort all matches by score
        all_matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Return top K
        all_matches.into_iter().take(top_k).collect()
    }

    /// Match a need against multiple capabilities (composite match)
    /// Returns the best combination of capabilities that fulfill the need
    pub fn match_need_composite(&self, need: &Need, max_capabilities: usize) -> Vec<Match> {
        let mut matches = self.match_need(need);
        
        // Group by need and find best combinations
        // For now, just return top matches (can be enhanced later)
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        matches.into_iter().take(max_capabilities).collect()
    }

    /// Filter matches by minimum score threshold
    pub fn filter_by_score(matches: Vec<Match>, min_score: f32) -> Vec<Match> {
        matches.into_iter().filter(|m| m.score >= min_score).collect()
    }

    /// Group matches by need (for better display)
    pub fn group_by_need(matches: Vec<Match>) -> std::collections::HashMap<String, Vec<Match>> {
        let mut grouped = std::collections::HashMap::new();
        for m in matches {
            grouped.entry(m.need.id.clone()).or_insert_with(Vec::new).push(m);
        }
        grouped
    }

    /// Load needs from a JSON file
    pub fn load_needs_from_file(path: &Path) -> Result<Vec<Need>> {
        let content = std::fs::read_to_string(path)?;
        let needs: Vec<Need> = serde_json::from_str(&content)?;
        Ok(needs)
    }

    /// Create a sample need (for testing)
    pub fn create_sample_need(
        id: String,
        title: String,
        description: String,
        bounty: Option<String>,
        embedding: Vec<f32>,
    ) -> Need {
        Need {
            id,
            title,
            description,
            bounty,
            embedding,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((Matchmaker::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((Matchmaker::cosine_similarity(&a, &b) - 0.0).abs() < 0.001);
    }
}

