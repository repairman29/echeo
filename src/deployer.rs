use anyhow::Result;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::matchmaker::Match;
use crate::vectorizer::EmbeddedCapability;

/// THE DEPLOYER: Creates new repos and wires capabilities together
pub struct Deployer {
    ollama_url: String,
    ollama_model: String,
}

#[derive(Debug, Clone)]
pub struct DeployConfig {
    pub the_match: Match,
    pub output_dir: PathBuf,
    pub repo_name: String,
}

impl Deployer {
    pub fn new(ollama_url: String, ollama_model: String) -> Self {
        Self {
            ollama_url,
            ollama_model,
        }
    }

    /// Deploy a match - create repo and wire capabilities
    pub async fn deploy(&self, config: DeployConfig) -> Result<PathBuf> {
        let repo_path = config.output_dir.join(&config.repo_name);

        // 1. Create directory
        fs::create_dir_all(&repo_path)?;

        // 2. Copy capability files to new repo
        self.copy_capability_files(&config.the_match.capability, &repo_path)?;

        // 3. Create basic project structure
        self.create_project_structure(&repo_path, &config.the_match)?;

        // 4. Wire capabilities together using LLM
        self.wire_capabilities(&repo_path, &config.the_match).await?;

        // 5. Initialize git repo
        self.init_git_repo(&repo_path)?;

        println!(
            "{} Deployed to: {}",
            "[DEPLOY]".bright_green(),
            repo_path.display()
        );

        Ok(repo_path)
    }

    /// Copy capability source files to new repo
    fn copy_capability_files(&self, capability: &EmbeddedCapability, repo_path: &Path) -> Result<()> {
        let source_path = Path::new(&capability.path);
        
        if source_path.exists() {
            // Create src directory
            let src_dir = repo_path.join("src");
            fs::create_dir_all(&src_dir)?;

            // Copy the file
            let dest_path = src_dir.join(source_path.file_name().unwrap_or_default());
            fs::copy(source_path, &dest_path)?;

                println!(
                    "  {} Copied {} → {}",
                    "[+]".green(),
                    source_path.display(),
                    dest_path.display()
                );
        }

        Ok(())
    }

    /// Create basic project structure (package.json, README, etc.)
    fn create_project_structure(&self, repo_path: &Path, m: &Match) -> Result<()> {
        // Create README
        let readme = format!(
            "# {}\n\n{}\n\n## Bounty\n{}\n\n## Match Score\n{}% Ship Velocity\n\n## Your Capability\n{}\n\n## Description\n{}\n",
            m.need.title,
            m.need.description,
            m.need.bounty.as_ref().unwrap_or(&"N/A".to_string()),
            (m.score * 100.0) as u32,
            m.capability.name,
            m.capability.code_snippet.chars().take(200).collect::<String>()
        );

        fs::write(repo_path.join("README.md"), readme)?;

        // Create .gitignore
        let gitignore = "node_modules/\ndist/\nbuild/\n.env\n*.log\n";
        fs::write(repo_path.join(".gitignore"), gitignore)?;

        println!("  {} Created project structure", "[+]".green());

        Ok(())
    }

    /// Wire capabilities together using LLM
    async fn wire_capabilities(&self, repo_path: &Path, m: &Match) -> Result<()> {
        let prompt = format!(
            "You are wiring together code for a bounty project.\n\n\
            Bounty: {}\n\
            Description: {}\n\
            Capability: {}\n\
            Code:\n{}\n\n\
            Generate a simple main file or entry point that uses this capability to fulfill the bounty. \
            Keep it concise and working. Output only the code, no explanations.",
            m.need.title,
            m.need.description,
            m.capability.name,
            m.capability.code_snippet.chars().take(500).collect::<String>()
        );

        // Call Ollama to generate wiring code
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        let request = serde_json::json!({
            "model": self.ollama_model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.3,
                "num_predict": 500
            }
        });

        let url = format!("{}/api/generate", self.ollama_url);
        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            if let Some(code) = json.get("response").and_then(|r| r.as_str()) {
                // Clean up the code (remove markdown if present)
                let code = code
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("```"))
                    .collect::<Vec<_>>()
                    .join("\n");

                // Determine file extension based on capability language
                let ext = match m.capability.language.as_str() {
                    "typescript" | "ts" => "ts",
                    "javascript" | "js" => "js",
                    "rust" | "rs" => "rs",
                    "python" | "py" => "py",
                    "go" => "go",
                    _ => "ts",
                };

                let main_file = repo_path.join("src").join(format!("main.{}", ext));
                fs::write(&main_file, code.trim())?;

                println!("  {} Generated wiring code: {}", "[+]".green(), main_file.display());
            }
        } else {
            println!("  {} Failed to generate wiring code (continuing anyway)", "[WARNING]".yellow());
        }

        Ok(())
    }

    /// Initialize git repository
    fn init_git_repo(&self, repo_path: &Path) -> Result<()> {
        Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()?;

        println!("  {} Initialized git repository", "[+]".green());

        Ok(())
    }
}

