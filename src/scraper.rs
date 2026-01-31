use anyhow::Result;
use colored::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::matchmaker::Need;

/// THE SCRAPER: Aggregates bounties from multiple sources
pub struct BountyScraper {
    github_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedBounty {
    pub id: String,
    pub title: String,
    pub description: String,
    pub bounty: Option<String>,
    pub source: String, // "github", "gitcoin", etc.
    pub url: Option<String>,
    pub created_at: Option<String>,
}

impl BountyScraper {
    pub fn new(github_token: Option<String>) -> Self {
        Self { github_token }
    }

    /// Scrape bounties from GitHub Issues
    pub async fn scrape_github_issues(
        &self,
        owner: &str,
        repo: &str,
        labels: Option<Vec<&str>>,
    ) -> Result<Vec<ScrapedBounty>> {
        if self.github_token.is_none() {
            return Err(anyhow::anyhow!("GitHub token required for issue scraping"));
        }

        let token = self.github_token.as_ref().unwrap();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let mut url = format!(
            "https://api.github.com/repos/{}/{}/issues?state=open&per_page=100",
            owner, repo
        );

        if let Some(labels) = labels {
            let labels_str = labels.join(",");
            url = format!("{}&labels={}", url, labels_str);
        }

        println!(
            "{} Scraping GitHub Issues from {}/{}...",
            "[SCRAPER]".bright_cyan(),
            owner.cyan(),
            repo.cyan()
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "echeo-cli")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch GitHub issues: {}",
                response.status()
            ));
        }

        #[derive(Debug, Deserialize)]
        struct GitHubIssue {
            number: u64,
            title: String,
            body: Option<String>,
            html_url: String,
            labels: Vec<GitHubLabel>,
            created_at: String,
        }

        #[derive(Debug, Deserialize)]
        struct GitHubLabel {
            name: String,
        }

        let issues: Vec<GitHubIssue> = response.json().await?;

        let mut bounties = Vec::new();
        let bounty_regex = Regex::new(r#"\$[\d,]+|(\d+)\s*(USD|USDC|ETH|BTC)"#)?;

        for issue in issues {
            // Check if issue has bounty-related labels or mentions
            let has_bounty_label = issue
                .labels
                .iter()
                .any(|l| l.name.to_lowercase().contains("bounty") || l.name.to_lowercase().contains("reward"));

            let body = issue.body.as_deref().unwrap_or("");
            let has_bounty_text = bounty_regex.is_match(&format!("{} {}", issue.title, body));

            if has_bounty_label || has_bounty_text {
                // Extract bounty amount
                let bounty_amount = self.extract_bounty_amount(&format!("{} {}", issue.title, body));

                let description = if body.len() > 500 {
                    format!("{}...", &body[..500])
                } else {
                    body.to_string()
                };

                bounties.push(ScrapedBounty {
                    id: format!("github-{}-{}", repo, issue.number),
                    title: issue.title,
                    description,
                    bounty: bounty_amount,
                    source: "github".to_string(),
                    url: Some(issue.html_url),
                    created_at: Some(issue.created_at),
                });
            }
        }

        println!(
            "  {} Found {} bounties",
            "[+]".green(),
            bounties.len().to_string().cyan()
        );

        Ok(bounties)
    }

    /// Scrape bounties from Gitcoin
    pub async fn scrape_gitcoin(&self, limit: Option<usize>) -> Result<Vec<ScrapedBounty>> {
        println!("{} Scraping Gitcoin bounties...", "[SCRAPER]".bright_cyan());

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        // Gitcoin API endpoint (this may need adjustment based on actual API)
        let url = "https://gitcoin.co/api/v0/bounties/?limit=100&order_by=-web3_created";

        let response = client
            .get(url)
            .header("User-Agent", "echeo-cli")
            .send()
            .await?;

        if !response.status().is_success() {
            println!(
                "  {} Gitcoin API not accessible (status: {}). Skipping.",
                "[WARNING]".yellow(),
                response.status()
            );
            return Ok(Vec::new());
        }

        #[derive(Debug, Deserialize)]
        struct GitcoinBounty {
            id: u64,
            title: String,
            description: Option<String>,
            value_in_token: Option<f64>,
            token_name: Option<String>,
            web3_created: Option<String>,
            url: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        struct GitcoinResponse {
            results: Vec<GitcoinBounty>,
        }

        let data: GitcoinResponse = response.json().await?;

        let mut bounties = Vec::new();
        let limit = limit.unwrap_or(50);

        for (_idx, bounty) in data.results.iter().take(limit).enumerate() {
            let bounty_amount = if let (Some(value), Some(token)) = (bounty.value_in_token, &bounty.token_name) {
                Some(format!("${:.2} {}", value, token))
            } else {
                None
            };

            let description = bounty
                .description
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(500)
                .collect::<String>();

            bounties.push(ScrapedBounty {
                id: format!("gitcoin-{}", bounty.id),
                title: bounty.title.clone(),
                description: if description.len() == 500 {
                    format!("{}...", description)
                } else {
                    description
                },
                bounty: bounty_amount,
                source: "gitcoin".to_string(),
                url: bounty.url.clone(),
                created_at: bounty.web3_created.clone(),
            });
        }

        println!(
            "  {} Found {} bounties",
            "[+]".green(),
            bounties.len().to_string().cyan()
        );

        Ok(bounties)
    }

    /// Convert scraped bounties to Needs format
    pub fn convert_to_needs(&self, bounties: Vec<ScrapedBounty>) -> Vec<Need> {
        bounties
            .into_iter()
            .map(|b| Need {
                id: b.id,
                title: b.title,
                description: b.description,
                bounty: b.bounty,
                embedding: vec![], // Will be filled by embed_needs
            })
            .collect()
    }

    /// Extract bounty amount from text
    fn extract_bounty_amount(&self, text: &str) -> Option<String> {
        // Try to find $ amounts
        let dollar_regex = Regex::new(r#"\$[\d,]+"#).ok()?;
        if let Some(cap) = dollar_regex.captures(text) {
            return Some(cap.get(0)?.as_str().to_string());
        }

        // Try to find crypto amounts
        let crypto_regex = Regex::new(r#"(\d+(?:\.\d+)?)\s*(USDC|ETH|BTC|USD)"#).ok()?;
        if let Some(cap) = crypto_regex.captures(text) {
            let amount = cap.get(1)?.as_str();
            let currency = cap.get(2)?.as_str();
            return Some(format!("{} {}", amount, currency));
        }

        None
    }

    /// Aggregate bounties from all sources
    pub async fn scrape_all(
        &self,
        github_repos: Option<Vec<(String, String)>>, // (owner, repo)
        gitcoin_limit: Option<usize>,
    ) -> Result<Vec<ScrapedBounty>> {
        let mut all_bounties = Vec::new();

        // Scrape GitHub Issues
        if let Some(repos) = github_repos {
            for (owner, repo) in repos {
                match self.scrape_github_issues(&owner, &repo, Some(vec!["bounty", "reward"])).await {
                    Ok(mut bounties) => {
                        all_bounties.append(&mut bounties);
                    }
                    Err(e) => {
                        println!(
                            "  {} Failed to scrape {}/{}: {}",
                            "[WARNING]".yellow(),
                            owner,
                            repo,
                            e
                        );
                    }
                }
            }
        }

        // Scrape Gitcoin
        match self.scrape_gitcoin(gitcoin_limit).await {
            Ok(mut bounties) => {
                all_bounties.append(&mut bounties);
            }
            Err(e) => {
                println!("  {} Failed to scrape Gitcoin: {}", "[WARNING]".yellow(), e);
            }
        }

        println!(
            "{} Total bounties scraped: {}",
            "[SCRAPER]".bright_green(),
            all_bounties.len().to_string().cyan()
        );

        Ok(all_bounties)
    }
}

