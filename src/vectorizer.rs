use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::Duration;

/// THE VECTORIZER: Generates embeddings using Ollama
#[derive(Clone)]
pub struct Vectorizer {
    client: reqwest::Client,
    ollama_url: String,
    model: String,
}

#[derive(Debug, Clone, Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedCapability {
    pub name: String,
    pub code_snippet: String,
    pub embedding: Vec<f32>,
    pub language: String,
    pub kind: String,
    pub path: String,
    pub line: usize,
}

impl Vectorizer {
    pub fn new(ollama_url: Option<String>, model: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            ollama_url: ollama_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            model: model.unwrap_or_else(|| "nomic-embed-text".to_string()),
        }
    }

    /// Check if Ollama is available
    pub async fn check_ollama(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.ollama_url);
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // Also check if the model is available
                    let body: serde_json::Value = response.json().await?;
                    if let Some(models) = body.get("models").and_then(|m| m.as_array()) {
                        let model_exists = models.iter().any(|m| {
                            m.get("name")
                                .and_then(|n| n.as_str())
                                .map(|n| n.contains(&self.model))
                                .unwrap_or(false)
                        });
                        Ok(model_exists)
                    } else {
                        Ok(true) // Can't verify model, but Ollama is running
                    }
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }

    /// Generate embedding for a capability
    pub async fn embed_capability(
        &self,
        name: &str,
        code_snippet: &str,
        language: &str,
        kind: &str,
        path: &str,
        line: usize,
    ) -> Result<EmbeddedCapability> {
        // Create a descriptive prompt for the embedding
        let prompt = format!(
            "Capability: {}\nLanguage: {}\nType: {}\nCode:\n{}",
            name, language, kind, code_snippet
        );

        let request = EmbeddingRequest {
            model: self.model.clone(),
            prompt: prompt.clone(),
        };

        let url = format!("{}/api/embeddings", self.ollama_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            // If embedding fails, return empty vector (graceful degradation)
            return Ok(EmbeddedCapability {
                name: name.to_string(),
                code_snippet: code_snippet.to_string(),
                embedding: vec![],
                language: language.to_string(),
                kind: kind.to_string(),
                path: path.to_string(),
                line,
            });
        }

        let embedding_resp: EmbeddingResponse = response.json().await?;

        Ok(EmbeddedCapability {
            name: name.to_string(),
            code_snippet: code_snippet.to_string(),
            embedding: embedding_resp.embedding,
            language: language.to_string(),
            kind: kind.to_string(),
            path: path.to_string(),
            line,
        })
    }

    /// Helper function to embed a single capability or need
    pub async fn embed_single(
        client: reqwest::Client,
        url: String,
        model: String,
        name: String,
        content: String,
        lang: String,
        kind: String,
        path: String,
        line: usize,
    ) -> Result<EmbeddedCapability> {
        let prompt = format!(
            "Capability: {}\nLanguage: {}\nType: {}\nContent:\n{}",
            name, lang, kind, content
        );

        let request = serde_json::json!({
            "model": model,
            "prompt": prompt
        });

        let api_url = format!("{}/api/embeddings", url);
        let response = client
            .post(&api_url)
            .json(&request)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let embedding = json
                            .get("embedding")
                            .and_then(|e| e.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                                    .collect()
                            })
                            .unwrap_or_default();

                        Ok(EmbeddedCapability {
                            name,
                            code_snippet: content,
                            embedding,
                            language: lang,
                            kind,
                            path,
                            line,
                        })
                    }
                    Err(_) => {
                        Ok(EmbeddedCapability {
                            name,
                            code_snippet: content,
                            embedding: vec![],
                            language: lang,
                            kind,
                            path,
                            line,
                        })
                    }
                }
            }
            _ => {
                Ok(EmbeddedCapability {
                    name,
                    code_snippet: content,
                    embedding: vec![],
                    language: lang,
                    kind,
                    path,
                    line,
                })
            }
        }
    }

    /// Batch embed capabilities (for efficiency)
    pub async fn embed_capabilities(
        &self,
        capabilities: Vec<(String, String, String, String, String, usize)>,
    ) -> Result<Vec<EmbeddedCapability>> {
        let mut embedded = Vec::new();

        // Process in parallel batches (Ollama can handle concurrent requests)
        use futures::future;
        let url = self.ollama_url.clone();
        let model = self.model.clone();
        let client = self.client.clone();
        
        let futures: Vec<_> = capabilities
            .into_iter()
            .map(|(name, code, lang, kind, path, line)| {
                Self::embed_single(
                    client.clone(),
                    url.clone(),
                    model.clone(),
                    name,
                    code,
                    lang,
                    kind,
                    path,
                    line,
                )
            })
            .collect();

        let results = future::join_all(futures).await;
        for result in results {
            match result {
                Ok(embedded_cap) => embedded.push(embedded_cap),
                Err(_) => {
                    // Errors are already handled in embed_single
                }
            }
        }

        Ok(embedded)
    }
}

