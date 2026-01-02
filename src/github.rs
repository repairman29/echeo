use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

use crate::shredder::{Shredder, CapabilityKind};
use crate::vectorizer::{Vectorizer, EmbeddedCapability};

/// THE GITHUB INTEGRATOR: Scans GitHub repositories for capabilities
pub struct GitHubIntegrator {
    token: String,
    client: reqwest::Client,
    vectorizer: Option<Vectorizer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub url: String,
    pub language: Option<String>,
    pub private: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    name: String,
    full_name: String,
    html_url: String,
    language: Option<String>,
    private: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubTreeItem {
    path: Option<String>,
    sha: Option<String>,
    #[serde(rename = "type")]
    item_type: String,
}

#[derive(Debug, Deserialize)]
struct GitHubTree {
    tree: Vec<GitHubTreeItem>,
}

#[derive(Debug, Deserialize)]
struct GitHubBlob {
    content: Option<String>,
    encoding: Option<String>,
}

impl GitHubIntegrator {
    /// Create a new GitHub integrator with OAuth token
    pub fn new(token: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            token,
            client,
            vectorizer: None,
        })
    }

    /// Set vectorizer for embedding generation
    pub fn with_vectorizer(mut self, vectorizer: Vectorizer) -> Self {
        self.vectorizer = Some(vectorizer);
        self
    }

    /// List user's repositories
    pub async fn list_repos(&self, username: Option<&str>) -> Result<Vec<Repository>> {
        let url = if let Some(user) = username {
            format!("https://api.github.com/users/{}/repos?per_page=100", user)
        } else {
            "https://api.github.com/user/repos?per_page=100".to_string()
        };

        let repos: Vec<GitHubRepo> = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "payload-cli")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?
            .json()
            .await?;

        Ok(repos
            .into_iter()
            .map(|r| Repository {
                name: r.name,
                full_name: r.full_name,
                url: r.html_url,
                language: r.language,
                private: r.private,
            })
            .collect())
    }

    /// Scan a GitHub repository for capabilities
    pub async fn scan_repo(
        &self,
        owner: &str,
        repo: &str,
        output_dir: Option<PathBuf>,
    ) -> Result<Vec<EmbeddedCapability>> {
        println!(
            "{} Scanning {}/{}...",
            "[GITHUB]".bright_cyan(),
            owner.cyan(),
            repo.cyan()
        );

        // 1. Get default branch
        let repo_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        let repo_info: serde_json::Value = self
            .client
            .get(&repo_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "payload-cli")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?
            .json()
            .await?;

        let default_branch = repo_info
            .get("default_branch")
            .and_then(|b| b.as_str())
            .unwrap_or("main");

        // 2. Get tree for default branch
        let tree_url = format!(
            "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
            owner, repo, default_branch
        );
        
        let tree: GitHubTree = self
            .client
            .get(&tree_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "payload-cli")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?
            .json()
            .await?;

        // 3. Filter for code files
        let code_files: Vec<_> = tree
            .tree
            .into_iter()
            .filter(|item| {
                if item.item_type == "blob" {
                    if let Some(path) = &item.path {
                        path.ends_with(".ts")
                            || path.ends_with(".tsx")
                            || path.ends_with(".rs")
                            || path.ends_with(".go")
                            || path.ends_with(".py")
                            || path.ends_with(".js")
                            || path.ends_with(".jsx")
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .take(50) // Limit to avoid rate limits
            .collect();

        println!(
            "  {} Found {} code files",
            "[+]".green(),
            code_files.len().to_string().cyan()
        );

        // 4. Download and process files
        let mut all_capabilities = Vec::new();
        let mut shredder = Shredder::new()?;

        for file in code_files {
            if let (Some(path), Some(sha)) = (file.path, file.sha) {
                // Get file content
                match self.get_file_content(owner, repo, &sha).await {
                    Ok(content) => {
                        // Save to temp file for shredder
                        let temp_path = if let Some(dir) = &output_dir {
                            dir.join(&path)
                        } else {
                            std::env::temp_dir().join(format!("{}_{}", repo, path.replace("/", "_")))
                        };

                        if let Some(parent) = temp_path.parent() {
                            fs::create_dir_all(parent)?;
                        }

                        fs::write(&temp_path, &content)?;

                        // Shred the file
                        match shredder.shred_file(&temp_path) {
                            Ok(capabilities) => {
                                for cap in capabilities {
                                    all_capabilities.push((
                                        temp_path.clone(),
                                        path.split('.').last().unwrap_or("").to_string(),
                                        cap,
                                    ));
                                }
                            }
                            Err(_) => {
                                // Skip files that can't be parsed
                            }
                        }
                    }
                    Err(_) => {
                        // Skip files we can't read
                    }
                }
            }
        }

        println!(
            "  {} Extracted {} capabilities",
            "[+]".green(),
            all_capabilities.len().to_string().cyan()
        );

        // 5. Generate embeddings if vectorizer is available
        if let Some(v) = &self.vectorizer {
            println!("  {} Generating embeddings...", "[+]".green());
            
            let embedding_tasks: Vec<_> = all_capabilities
                .iter()
                .map(|(path, ext, cap)| {
                    let kind_str = match cap.kind {
                        CapabilityKind::Function => "function",
                        CapabilityKind::Class => "class",
                        CapabilityKind::Component => "component",
                        CapabilityKind::ApiRoute => "api_route",
                    };
                    (
                        cap.name.clone(),
                        cap.code_snippet.clone(),
                        ext.clone(),
                        kind_str.to_string(),
                        format!("github.com/{}/{}", owner, repo),
                        cap.line,
                    )
                })
                .collect();

            match v.embed_capabilities(embedding_tasks).await {
                Ok(embedded) => {
                    println!(
                        "  {} Generated {} embeddings",
                        "[+]".green(),
                        embedded.len().to_string().cyan()
                    );
                    Ok(embedded)
                }
                Err(e) => {
                    println!("  {} Failed to generate embeddings: {}", "[WARNING]".yellow(), e);
                    // Return capabilities without embeddings
                    Ok(all_capabilities
                        .into_iter()
                        .map(|(_, _, cap)| {
                            EmbeddedCapability {
                                name: cap.name,
                                code_snippet: cap.code_snippet,
                                embedding: vec![],
                                language: "unknown".to_string(),
                                kind: match cap.kind {
                                    CapabilityKind::Function => "function".to_string(),
                                    CapabilityKind::Class => "class".to_string(),
                                    CapabilityKind::Component => "component".to_string(),
                                    CapabilityKind::ApiRoute => "api_route".to_string(),
                                },
                                path: format!("github.com/{}/{}", owner, repo),
                                line: cap.line,
                            }
                        })
                        .collect())
                }
            }
        } else {
            // Return capabilities without embeddings
            Ok(all_capabilities
                .into_iter()
                .map(|(_, _, cap)| {
                    EmbeddedCapability {
                        name: cap.name,
                        code_snippet: cap.code_snippet,
                        embedding: vec![],
                        language: "unknown".to_string(),
                        kind: match cap.kind {
                            CapabilityKind::Function => "function".to_string(),
                            CapabilityKind::Class => "class".to_string(),
                            CapabilityKind::Component => "component".to_string(),
                            CapabilityKind::ApiRoute => "api_route".to_string(),
                        },
                        path: format!("github.com/{}/{}", owner, repo),
                        line: cap.line,
                    }
                })
                .collect())
        }
    }

    /// Get file content from GitHub
    async fn get_file_content(&self, owner: &str, repo: &str, sha: &str) -> Result<String> {
        let blob_url = format!(
            "https://api.github.com/repos/{}/{}/git/blobs/{}",
            owner, repo, sha
        );

        let blob: GitHubBlob = self
            .client
            .get(&blob_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "payload-cli")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?
            .json()
            .await?;

        if let Some(content) = blob.content {
            // GitHub API returns base64 encoded content
            use base64::Engine;
            let content_clean: String = content.replace('\n', "");
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(&content_clean)?;
            Ok(String::from_utf8(decoded)?)
        } else {
            Err(anyhow::anyhow!("No content in blob"))
        }
    }

    /// Get OAuth URL for GitHub authentication
    pub fn get_oauth_url(client_id: &str, redirect_uri: &str) -> String {
        format!(
            "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=repo,read:user",
            client_id, redirect_uri
        )
    }
}
