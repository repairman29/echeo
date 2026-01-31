use anyhow::Result;
use serde::{Deserialize, Serialize};

/// THE SUMMARIZER: Generates human-readable descriptions using Ollama
pub struct Summarizer {
    client: reqwest::Client,
    ollama_url: String,
    model: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct GenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct GenerateOptions {
    temperature: f32,
    num_predict: usize,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GenerateResponse {
    response: String,
}

impl Summarizer {
    pub fn new(ollama_url: Option<String>, model: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            ollama_url: ollama_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            model: model.unwrap_or_else(|| "llama3".to_string()),
        }
    }

    /// Generate a 5-word description of a capability
    #[allow(dead_code)]
    pub async fn summarize_capability(
        &self,
        name: &str,
        code_snippet: &str,
        language: &str,
        kind: &str,
    ) -> Result<String> {
        let prompt = format!(
            "Describe this code capability in exactly 5 words. Be specific and technical.\n\n\
            Name: {}\n\
            Language: {}\n\
            Type: {}\n\
            Code:\n{}\n\n\
            Description (5 words):",
            name, language, kind, code_snippet
        );

        let request = GenerateRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
            options: GenerateOptions {
                temperature: 0.3, // Lower temperature for more consistent, concise output
                num_predict: 20,  // Limit to ~5 words
            },
        };

        let url = format!("{}/api/generate", self.ollama_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            // Fallback to a simple description
            return Ok(format!("{} {} in {}", kind, name, language));
        }

        let generate_resp: GenerateResponse = response.json().await?;
        
        // Clean up the response - remove newlines, trim, limit to reasonable length
        let description = generate_resp
            .response
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();

        // If description is too long or empty, use fallback
        if description.is_empty() || description.len() > 100 {
            Ok(format!("{} {} in {}", kind, name, language))
        } else {
            Ok(description)
        }
    }

    /// Batch summarize capabilities
    pub async fn summarize_capabilities(
        &self,
        capabilities: Vec<(String, String, String, String)>,
    ) -> Result<Vec<(String, String)>> {
        use futures::future;
        
        let url = self.ollama_url.clone();
        let model = self.model.clone();
        let client = self.client.clone();
        
        let futures: Vec<_> = capabilities
            .into_iter()
            .map(move |(name, code, lang, kind)| {
                let url = url.clone();
                let model = model.clone();
                let client = client.clone();
                async move {
                    let prompt = format!(
                        "Describe this code capability in exactly 5 words. Be specific and technical.\n\n\
                        Name: {}\n\
                        Language: {}\n\
                        Type: {}\n\
                        Code:\n{}\n\n\
                        Description (5 words):",
                        name, lang, kind, code
                    );

                    let request = serde_json::json!({
                        "model": model,
                        "prompt": prompt,
                        "stream": false,
                        "options": {
                            "temperature": 0.3,
                            "num_predict": 20
                        }
                    });

                    let api_url = format!("{}/api/generate", url);
                    let response = client
                        .post(&api_url)
                        .json(&request)
                        .send()
                        .await;

                    let summary = match response {
                        Ok(resp) if resp.status().is_success() => {
                            match resp.json::<serde_json::Value>().await {
                                Ok(json) => {
                                    json.get("response")
                                        .and_then(|r| r.as_str())
                                        .unwrap_or("")
                                        .lines()
                                        .next()
                                        .unwrap_or("")
                                        .trim()
                                        .to_string()
                                }
                                Err(_) => format!("{} {} in {}", kind, name, lang),
                            }
                        }
                        _ => format!("{} {} in {}", kind, name, lang),
                    };

                    let description = if summary.is_empty() || summary.len() > 100 {
                        format!("{} {} in {}", kind, name, lang)
                    } else {
                        summary
                    };

                    (name, description)
                }
            })
            .collect();

        let results = future::join_all(futures).await;
        Ok(results)
    }
}

