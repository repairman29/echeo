mod shredder;
mod vectorizer;
mod summarizer;
mod matchmaker;
mod deployer;
mod github;
mod scraper;
mod authorship;

use clap::Parser;
use colored::*;
use git2::{Config, Repository};
use ignore::WalkBuilder;
use shredder::{CapabilityKind, Shredder};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use rayon::prelude::*;
use vectorizer::{Vectorizer, EmbeddedCapability};
use summarizer::Summarizer;
use matchmaker::{Matchmaker, Need};
use deployer::Deployer;
use github::GitHubIntegrator;
use scraper::BountyScraper;

/// ECHEO: The Resonant Engine
/// Scans local code capabilities to match market signals.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The directory to scan (Defaults to current dir)
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Ollama URL (defaults to http://localhost:11434)
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_url: String,

    /// Ollama embedding model (defaults to nomic-embed-text)
    #[arg(long, default_value = "nomic-embed-text")]
    ollama_model: String,

    /// Ollama generation model for summaries (defaults to llama3)
    #[arg(long, default_value = "llama3")]
    ollama_gen_model: String,

    /// Skip embedding generation (faster, no Ollama required)
    #[arg(long)]
    skip_embeddings: bool,

    /// Skip summary generation (faster, no LLM required)
    #[arg(long)]
    skip_summaries: bool,

    /// Match against needs/bounties (JSON file path)
    #[arg(long)]
    match_needs: Option<String>,

    /// Generate embeddings for needs file (outputs JSON with embeddings)
    #[arg(long)]
    embed_needs: Option<String>,

    /// Deploy a match (match index from feed, 1-based)
    #[arg(long)]
    deploy: Option<usize>,

    /// Output directory for deployments (defaults to ./deployments)
    #[arg(long, default_value = "./deployments")]
    deploy_dir: String,

    /// Generate loadout.json file with all capabilities
    #[arg(long)]
    generate_loadout: bool,

    /// Upload capabilities to Echeo API
    #[arg(long)]
    upload: bool,

    /// Echeo API URL (defaults to https://echeo.io/api)
    #[arg(long, default_value = "https://echeo.io/api")]
    api_url: String,

    /// User ID for API upload (required with --upload)
    #[arg(long)]
    user_id: Option<String>,

    /// GitHub personal access token (for scanning GitHub repos)
    #[arg(long)]
    github_token: Option<String>,

    /// Scan GitHub repository (format: owner/repo)
    #[arg(long)]
    github_repo: Option<String>,

    /// List GitHub repositories for authenticated user
    #[arg(long)]
    github_list: bool,

    /// GitHub OAuth client ID (for OAuth flow)
    #[arg(long)]
    github_client_id: Option<String>,

    /// Scrape bounties from GitHub Issues (format: owner/repo, can specify multiple)
    #[arg(long)]
    scrape_github: Vec<String>,

    /// Scrape bounties from Gitcoin
    #[arg(long)]
    scrape_gitcoin: bool,

    /// Limit for Gitcoin scraping (default: 50)
    #[arg(long, default_value = "50")]
    gitcoin_limit: usize,

    /// Scrape all sources and save to file
    #[arg(long)]
    scrape_all: Option<String>,

    /// Auto-embed scraped bounties
    #[arg(long)]
    auto_embed_scraped: bool,
}

// The "High Value" Target List
const HIGH_VALUE_EXTENSIONS: &[&str] = &[
    "ts", "tsx", // TypeScript (The Industry Standard)
    "rs",        // Rust (The Weapon)
    "go",        // Go (The Systems)
    "py",        // Python (The Brains)
    "sol",       // Solidity (The Crypto)
    "rb",        // Ruby (The Legacy)
];

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let root_path = Path::new(&args.path);

    // ECHEO Boot Sequence
    print_echeo_banner();
    println!("{}", "INITIATING ACTIVE SONAR SWEEP...".bright_cyan().bold());
    println!("TARGET SECTOR: {}", root_path.display().to_string().yellow());
    
    // Initialize vectorizer if embeddings are enabled
    let vectorizer = if !args.skip_embeddings {
        let v = Vectorizer::new(Some(args.ollama_url.clone()), Some(args.ollama_model.clone()));
        match v.check_ollama().await {
            Ok(true) => {
                println!("{} Ollama detected. Embeddings enabled.", "[VECTORIZER]".bright_cyan());
                Some(v)
            }
            _ => {
                println!("{} Ollama not available. Run with --skip-embeddings to disable.", "[WARNING]".yellow());
                None
            }
        }
    } else {
        println!("{} Embeddings disabled.", "[VECTORIZER]".dimmed());
        None
    };

    // Initialize summarizer if summaries are enabled
    let summarizer = if !args.skip_summaries {
        let s = Summarizer::new(Some(args.ollama_url.clone()), Some(args.ollama_gen_model.clone()));
        println!("{} Summaries enabled (model: {}).", "[SUMMARIZER]".bright_magenta(), args.ollama_gen_model.cyan());
        Some(s)
    } else {
        println!("{} Summaries disabled.", "[SUMMARIZER]".dimmed());
        None
    };
    
    println!("{}", "---------------------------------".dimmed());

    // Counter for stats
    let file_count = AtomicUsize::new(0);
    let high_value_count = AtomicUsize::new(0);
    let capability_count = AtomicUsize::new(0);

    // 1. THE CRAWLER
    // WalkBuilder respects .gitignore automatically.
    // It's the same engine used by 'ripgrep'.
    let walker = WalkBuilder::new(root_path)
        .hidden(false) // Don't scan hidden files like .git
        .git_ignore(true) // Respect .gitignore
        .build();

    // 2. THE PIPELINE
    // Convert walker to vector for parallel processing (Rayon)
    // In a real 'shredder', we stream this, but for scanning, collecting is fine.
    let entries: Vec<PathBuf> = walker
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_owned())
        .collect();

    // Collect all capabilities for batch embedding (thread-safe)
    let all_capabilities = Mutex::new(Vec::new());

    // 3. THE SCAN (Parallelized)
    entries.par_iter().for_each(|path| {
        if path.is_file() {
            file_count.fetch_add(1, Ordering::Relaxed);
            
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if HIGH_VALUE_EXTENSIONS.contains(&ext_str) {
                        high_value_count.fetch_add(1, Ordering::Relaxed);
                        
                        // THE SHREDDER: Extract capabilities from AST
                        // Try to enable authorship tracking if we're in a git repo
                        let mut shredder = if let Ok(repo_path) = find_git_repo(path) {
                            // Try to get user email/name from git config
                            let user_email = get_git_config("user.email").ok();
                            let user_name = get_git_config("user.name").ok();
                            Shredder::with_authorship(&repo_path, user_email, user_name)
                                .unwrap_or_else(|_| Shredder::new().unwrap())
                        } else {
                            Shredder::new().unwrap()
                        };

                        match shredder.shred_file(path) {
                            Ok(capabilities) => {
                                if !capabilities.is_empty() {
                                    capability_count.fetch_add(capabilities.len(), Ordering::Relaxed);
                                    
                                    // Store for batch embedding
                                    if let Ok(mut caps) = all_capabilities.lock() {
                                        for cap in capabilities {
                                            caps.push((path.clone(), ext_str.to_string(), cap));
                                        }
                                    }
                                    
                                    // Show file detection (embeddings will be shown later)
                                    print_detection(path, ext_str);
                                } else {
                                    print_detection(path, ext_str);
                                }
                            }
                            Err(_) => {
                                // If parsing fails, just show the file
                                print_detection(path, ext_str);
                            }
                        }
                    }
                }
            }
        }
    });

    println!("{}", "---------------------------------".cyan());
    println!("{}", "SWEEP COMPLETE.".green().bold());
    println!(
        "SECTOR DENSITY: {} Files Scanned",
        file_count.load(Ordering::Relaxed).to_string().white()
    );
    println!(
        "CONTACTS FOUND: {} {} with {} {}",
        high_value_count.load(Ordering::Relaxed).to_string().yellow().bold(),
        "VALID SIGNALS".yellow().bold(),
        capability_count.load(Ordering::Relaxed).to_string().bright_cyan().bold(),
        "CAPABILITIES".bright_cyan().bold()
    );

    // 4. THE VECTORIZER: Generate embeddings if enabled
    let all_caps = all_capabilities.into_inner().unwrap();
    let mut embedded_caps: Option<Vec<EmbeddedCapability>> = None;
    
    if let Some(v) = &vectorizer {
        if !all_caps.is_empty() {
            println!("{}", "---------------------------------".dimmed());
            println!("{} Generating embeddings...", "[VECTORIZER]".bright_cyan());
            
            // Prepare capabilities for embedding
            let embedding_tasks: Vec<_> = all_caps
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
                        path.display().to_string(),
                        cap.line,
                        cap.authorship.clone(), // Include authorship info
                    )
                })
                .collect();

            match v.embed_capabilities(embedding_tasks).await {
                Ok(embedded) => {
                    println!(
                        "{} Generated {} embeddings ({} dimensions each)",
                        "[VECTORIZER]".bright_green(),
                        embedded.len().to_string().bright_cyan(),
                        embedded.first().map(|e| e.embedding.len()).unwrap_or(0).to_string().bright_cyan()
                    );
                    
                    // Show sample embeddings
                    for emb in embedded.iter().take(3) {
                        println!(
                            "  {} {} → {}D vector",
                            "[EMBED]".dimmed(),
                            emb.name.white(),
                            emb.embedding.len().to_string().dimmed()
                        );
                    }

                    embedded_caps = Some(embedded.clone());

                    // 5. THE SUMMARIZER: Generate descriptions if enabled
                    if let Some(s) = &summarizer {
                        println!("{}", "---------------------------------".dimmed());
                        println!("{} Generating capability descriptions...", "[SUMMARIZER]".bright_magenta());
                        
                        // Prepare capabilities for summarization
                        let summary_tasks: Vec<_> = embedded
                            .iter()
                            .map(|emb| {
                                (
                                    emb.name.clone(),
                                    emb.code_snippet.clone(),
                                    emb.language.clone(),
                                    emb.kind.clone(),
                                )
                            })
                            .collect();

                        match s.summarize_capabilities(summary_tasks).await {
                            Ok(summaries) => {
                                println!(
                                    "{} Generated {} descriptions",
                                    "[SUMMARIZER]".bright_green(),
                                    summaries.len().to_string().bright_magenta()
                                );
                                
                                // Show sample summaries
                                println!("{}", "---------------------------------".dimmed());
                                println!("{} Sample Capabilities:", "[LOADOUT]".bright_yellow().bold());
                                for (name, summary) in summaries.iter().take(10) {
                                    println!(
                                        "  {} {}",
                                        name.white().bold(),
                                        format!("→ {}", summary).dimmed()
                                    );
                                }
                                
                                if summaries.len() > 10 {
                                    println!(
                                        "  {} ... and {} more capabilities",
                                        "[+]".dimmed(),
                                        (summaries.len() - 10).to_string().dimmed()
                                    );
                                }
                            }
                            Err(e) => {
                                println!("{} Failed to generate summaries: {}", "[ERROR]".red(), e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("{} Failed to generate embeddings: {}", "[ERROR]".red(), e);
                }
            }
        }
    } else if let Some(s) = summarizer {
        // If embeddings are disabled but summaries are enabled, generate summaries from raw capabilities
        if !all_caps.is_empty() {
            println!("{}", "---------------------------------".dimmed());
            println!("{} Generating capability descriptions...", "[SUMMARIZER]".bright_magenta());
            
            let summary_tasks: Vec<_> = all_caps
                .iter()
                .map(|(_, ext, cap)| {
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
                    )
                })
                .collect();

            match s.summarize_capabilities(summary_tasks).await {
                Ok(summaries) => {
                    println!(
                        "{} Generated {} descriptions",
                        "[SUMMARIZER]".bright_green(),
                        summaries.len().to_string().bright_magenta()
                    );
                    
                    println!("{}", "---------------------------------".dimmed());
                    println!("{} Sample Capabilities:", "[LOADOUT]".bright_yellow().bold());
                    for (name, summary) in summaries.iter().take(10) {
                        println!(
                            "  {} {}",
                            name.white().bold(),
                            format!("→ {}", summary).dimmed()
                        );
                    }
                    
                    if summaries.len() > 10 {
                        println!(
                            "  {} ... and {} more capabilities",
                            "[+]".dimmed(),
                            (summaries.len() - 10).to_string().dimmed()
                        );
                    }
                }
                Err(e) => {
                    println!("{} Failed to generate summaries: {}", "[ERROR]".red(), e);
                }
            }
        }
    }

    // 6. THE MATCHMAKER: Match capabilities to needs/bounties
    if let Some(needs_path) = &args.match_needs {
        println!("{}", "---------------------------------".dimmed());
        println!("{} Loading needs from {}...", "[MATCHMAKER]".bright_cyan(), needs_path.cyan());
        
        let needs = match Matchmaker::load_needs_from_file(Path::new(needs_path)) {
            Ok(n) => n,
            Err(e) => {
                println!("{} Failed to load needs: {}", "[ERROR]".red(), e);
                return Ok(());
            }
        };

        // Use embedded capabilities we already generated
        let embedded_caps_for_match = match embedded_caps {
            Some(ref caps) => caps.clone(),
            None => {
                println!("{} Embeddings required for matching. Run without --skip-embeddings", "[ERROR]".red());
                return Ok(());
            }
        };

        let matchmaker = Matchmaker::new(embedded_caps_for_match);
        let matches = matchmaker.match_needs(&needs, 10);

        println!("{}", "---------------------------------".dimmed());
        println!("{} Found {} matches", "[MATCHMAKER]".bright_green(), matches.len().to_string().bright_cyan());
        println!("{}", "---------------------------------".dimmed());
        println!("{} THE FEED:", "[FEED]".bright_yellow().bold());

        for (idx, m) in matches.iter().enumerate() {
            let score_percent = (m.score * 100.0) as u32;
            let score_color = if score_percent >= 80 {
                "bright_green"
            } else if score_percent >= 60 {
                "yellow"
            } else {
                "dimmed"
            };

            println!("\n{} CARD #{}", "[CARD]".bright_cyan(), (idx + 1).to_string().cyan());
            println!("  {} {}", "Title:".bold(), m.need.title.white());
            if let Some(bounty) = &m.need.bounty {
                println!("  {} {}", "Bounty:".bold(), bounty.bright_yellow());
            }
            println!("  {} {}% Match", "Ship Velocity:".bold(), 
                format!("{}", score_percent).color(score_color));
            println!("  {} {}", "Your Capability:".bold(), m.capability.name.white());
            println!("  {} {}", "Why:".bold(), m.reasons.join(", ").dimmed());
            println!("  {} {}", "Description:".dimmed(), m.need.description.dimmed());
            println!("  {} Run: {} --deploy {}", "[DEPLOY]".dimmed(), 
                std::env::args().next().unwrap_or_default().white(),
                (idx + 1).to_string().cyan());
        }

        // Store matches for deploy command
        if args.deploy.is_some() {
            // Save matches to temp file for deploy to read
            let matches_json = serde_json::to_string(&matches)?;
            let temp_file = std::env::temp_dir().join("echeo_matches.json");
            fs::write(&temp_file, matches_json)?;
        }
    }

    // 8. THE DEPLOYER: Deploy a match
    if let Some(match_idx) = args.deploy {
        println!("{}", "---------------------------------".dimmed());
        println!("{} Deploying match #{}...", "[DEPLOYER]".bright_magenta(), match_idx.to_string().cyan());
        
        // Load matches from temp file
        let temp_file = std::env::temp_dir().join("payload_matches.json");
        let matches: Vec<matchmaker::Match> = if temp_file.exists() {
            let content = fs::read_to_string(&temp_file)?;
            serde_json::from_str(&content)?
        } else {
            println!("{} No matches found. Run --match-needs first.", "[ERROR]".red());
            return Ok(());
        };

        if match_idx > matches.len() || match_idx == 0 {
            println!("{} Invalid match index. Choose 1-{}", "[ERROR]".red(), matches.len());
            return Ok(());
        }

        let selected_match = &matches[match_idx - 1];
        let deployer = Deployer::new(args.ollama_url.clone(), args.ollama_gen_model.clone());
        
        let repo_name = format!("{}-{}", 
            selected_match.need.id,
            selected_match.capability.name.to_lowercase().replace(" ", "-")
        );

        let config = deployer::DeployConfig {
            the_match: selected_match.clone(),
            output_dir: PathBuf::from(&args.deploy_dir),
            repo_name,
        };

        match deployer.deploy(config).await {
            Ok(repo_path) => {
                println!("{}", "---------------------------------".dimmed());
                println!("{} Deployment complete!", "[DEPLOYER]".bright_green());
                println!("  {} Next steps:", "[+]".green());
                println!("    cd {}", repo_path.display());
                println!("    git add .");
                println!("    git commit -m 'Initial deployment from Echeo'");
                println!("    # Polish the code, then ship!");
            }
            Err(e) => {
                println!("{} Deployment failed: {}", "[ERROR]".red(), e);
            }
        }
    }

    // 7. EMBED NEEDS: Generate embeddings for a needs file
    if let Some(needs_path) = &args.embed_needs {
        println!("{}", "---------------------------------".dimmed());
        println!("{} Generating embeddings for needs...", "[MATCHMAKER]".bright_cyan());
        
        let mut needs: Vec<Need> = match Matchmaker::load_needs_from_file(Path::new(needs_path)) {
            Ok(n) => n,
            Err(e) => {
                println!("{} Failed to load needs: {}", "[ERROR]".red(), e);
                return Ok(());
            }
        };

        if vectorizer.is_some() {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client");
            let url = args.ollama_url.clone();
            let model = args.ollama_model.clone();
            
            for need in &mut needs {
                if need.embedding.is_empty() {
                    match Vectorizer::embed_single(
                        client.clone(),
                        url.clone(),
                        model.clone(),
                        need.title.clone(),
                        need.description.clone(),
                        "need".to_string(),
                        "bounty".to_string(),
                        need.id.clone(),
                        0,
                    ).await {
                        Ok(embedded) => {
                            need.embedding = embedded.embedding;
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to embed need {}: {}", need.id, e);
                        }
                    }
                }
            }

            // Save back to file
            let output = serde_json::to_string_pretty(&needs)?;
            std::fs::write(needs_path, output)?;
            println!("{} Saved embedded needs to {}", "[MATCHMAKER]".bright_green(), needs_path.cyan());
        } else {
            println!("{} Embeddings required. Run without --skip-embeddings", "[ERROR]".red());
        }
    }

    // 11. THE SCRAPER: Scrape bounties from multiple sources
    if !args.scrape_github.is_empty() || args.scrape_gitcoin || args.scrape_all.is_some() {
        println!("{}", "---------------------------------".dimmed());
        println!("{} Bounty scraping enabled", "[SCRAPER]".bright_magenta());

        let scraper = BountyScraper::new(args.github_token.clone());

        // Parse GitHub repos
        let github_repos: Vec<(String, String)> = args
            .scrape_github
            .iter()
            .filter_map(|spec| {
                let parts: Vec<&str> = spec.split('/').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    println!("  {} Invalid repo format: {} (use owner/repo)", "[WARNING]".yellow(), spec);
                    None
                }
            })
            .collect();

        // Scrape all sources
        match scraper
            .scrape_all(
                if github_repos.is_empty() {
                    None
                } else {
                    Some(github_repos)
                },
                if args.scrape_gitcoin {
                    Some(args.gitcoin_limit)
                } else {
                    None
                },
            )
            .await
        {
            Ok(scraped_bounties) => {
                println!(
                    "{} Scraped {} bounties from all sources",
                    "[SCRAPER]".bright_green(),
                    scraped_bounties.len().to_string().cyan()
                );

                // Convert to Needs format
                let mut needs = scraper.convert_to_needs(scraped_bounties);

                // Auto-embed if requested
                if args.auto_embed_scraped && !args.skip_embeddings {
                    if vectorizer.is_some() {
                        println!("{} Auto-embedding scraped bounties...", "[SCRAPER]".bright_cyan());
                        
                        let client = reqwest::Client::builder()
                            .timeout(std::time::Duration::from_secs(30))
                            .build()
                            .expect("Failed to create HTTP client");
                        let url = args.ollama_url.clone();
                        let model = args.ollama_model.clone();

                        for need in &mut needs {
                            match Vectorizer::embed_single(
                                client.clone(),
                                url.clone(),
                                model.clone(),
                                need.title.clone(),
                                need.description.clone(),
                                "need".to_string(),
                                "bounty".to_string(),
                                need.id.clone(),
                                0,
                            )
                            .await
                            {
                                Ok(embedded) => {
                                    need.embedding = embedded.embedding;
                                }
                                Err(e) => {
                                    eprintln!("Warning: Failed to embed need {}: {}", need.id, e);
                                }
                            }
                        }
                    }
                }

                // Save to file
                if let Some(output_path) = &args.scrape_all {
                    let output = serde_json::to_string_pretty(&needs)?;
                    fs::write(output_path, output)?;
                    println!(
                        "{} Saved {} bounties to {}",
                        "[SCRAPER]".bright_green(),
                        needs.len().to_string().cyan(),
                        output_path.cyan()
                    );
                } else {
                    // Just print summary
                    println!("{}", "---------------------------------".dimmed());
                    println!("{} Sample bounties:", "[SCRAPER]".bright_yellow());
                    for (idx, need) in needs.iter().take(5).enumerate() {
                        println!(
                            "  {} {} - {}",
                            (idx + 1).to_string().cyan(),
                            need.title.white(),
                            need.bounty.as_ref().unwrap_or(&"No bounty".to_string()).bright_yellow()
                        );
                    }
                    if needs.len() > 5 {
                        println!(
                            "  {} ... and {} more bounties",
                            "[+]".dimmed(),
                            (needs.len() - 5).to_string().dimmed()
                        );
                    }
                    println!(
                        "  {} Use --scrape-all <file> to save all bounties",
                        "[TIP]".dimmed()
                    );
                }
            }
            Err(e) => {
                println!("{} Failed to scrape bounties: {}", "[ERROR]".red(), e);
            }
        }
    }

    // 9. GENERATE LOADOUT: Create loadout.json file
    if args.generate_loadout {
        if let Some(ref caps) = embedded_caps {
            let loadout = serde_json::json!({
                "user_handle": "local_ghost",
                "ship_velocity_score": 94,
                "stack_dominance": {
                    "typescript": 0.85,
                    "rust": 0.12,
                    "python": 0.03
                },
                "armory": caps.iter().map(|cap| {
                    serde_json::json!({
                        "name": cap.name,
                        "path": cap.path,
                        "confidence": 0.98,
                        "tags": [cap.language.clone(), cap.kind.clone()]
                    })
                }).collect::<Vec<_>>()
            });

            let loadout_path = Path::new(".echeo").join("loadout.json");
            fs::create_dir_all(".echeo")?;
            fs::write(&loadout_path, serde_json::to_string_pretty(&loadout)?)?;
            
            println!("{}", "---------------------------------".dimmed());
            println!("{} Generated loadout.json at {}", "[LOADOUT]".bright_yellow(), loadout_path.display().to_string().cyan());
        } else {
            println!("{} Embeddings required for loadout. Run without --skip-embeddings", "[ERROR]".red());
        }
    }

    // 10. UPLOAD TO API: Upload capabilities to Echeo platform
    if args.upload {
        if let Some(ref caps) = embedded_caps {
            let user_id = args.user_id.as_ref().ok_or_else(|| {
                anyhow::anyhow!("--user-id is required when using --upload. Get your user ID from https://echeo.io/dashboard")
            })?;

            println!("{}", "---------------------------------".dimmed());
            println!("{} Uploading {} capabilities to Echeo API...", "[UPLOAD]".bright_cyan(), caps.len().to_string().cyan());

            // Calculate stack dominance
            let mut stack_counts: HashMap<String, usize> = HashMap::new();
            for cap in caps.iter() {
                *stack_counts.entry(cap.language.clone()).or_insert(0) += 1;
            }
            let total = caps.len() as f64;
            let stack_dominance: HashMap<String, f64> = stack_counts
                .into_iter()
                .map(|(lang, count)| (lang, count as f64 / total))
                .collect();

            // Calculate ship velocity score (average of all match scores, or default)
            let ship_velocity_score = 85.0; // Could calculate from matches if available

            // Prepare capabilities for upload
            let capabilities: Vec<serde_json::Value> = caps.iter().map(|cap| {
                let mut json = serde_json::json!({
                    "name": cap.name,
                    "language": cap.language,
                    "kind": cap.kind,
                    "code_snippet": "", // CLI doesn't store full snippets, just embeddings
                    "embedding": cap.embedding,
                    "summary": "", // Would need summarizer output
                    "path": cap.path
                });
                
                // Add authorship fields if available
                if let Some(ref email) = cap.author_email {
                    json["author_email"] = serde_json::Value::String(email.clone());
                }
                if let Some(ref name) = cap.author_name {
                    json["author_name"] = serde_json::Value::String(name.clone());
                }
                if let Some(ref sha) = cap.commit_sha {
                    json["commit_sha"] = serde_json::Value::String(sha.clone());
                }
                if let Some(confidence) = cap.authorship_confidence {
                    json["authorship_confidence"] = serde_json::Value::Number(serde_json::Number::from_f64(confidence).unwrap_or(serde_json::Number::from(1)));
                }
                if let Some(is_self) = cap.is_self_authored {
                    json["is_self_authored"] = serde_json::Value::Bool(is_self);
                }
                if let Some(percentage) = cap.contribution_percentage {
                    json["contribution_percentage"] = serde_json::Value::Number(serde_json::Number::from_f64(percentage).unwrap_or(serde_json::Number::from(100)));
                }
                
                json
            }).collect();

            let payload = serde_json::json!({
                "user_id": user_id,
                "capabilities": capabilities,
                "loadout": {
                    "ship_velocity_score": ship_velocity_score,
                    "stack_dominance": stack_dominance
                }
            });

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()?;

            let upload_url = format!("{}/capabilities", args.api_url);
            let response = client
                .post(&upload_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await?;

            if response.status().is_success() {
                let result: serde_json::Value = response.json().await?;
                println!("{}", "---------------------------------".dimmed());
                println!("{} Successfully uploaded {} capabilities!", "[UPLOAD]".bright_green(), result.get("capabilities_count").and_then(|v| v.as_u64()).unwrap_or(0).to_string().bright_cyan());
                println!("{} View your matches at: https://echeo.io/dashboard", "[UPLOAD]".bright_green());
            } else {
                let error_text = response.text().await?;
                println!("{} Upload failed: {}", "[ERROR]".red(), error_text);
                return Err(anyhow::anyhow!("Upload failed: {}", error_text));
            }
        } else {
            println!("{} Embeddings required for upload. Run without --skip-embeddings", "[ERROR]".red());
            return Err(anyhow::anyhow!("Embeddings required for upload"));
        }
    }

    Ok(())
}

fn print_echeo_banner() {
    println!("{}", r#"
    _______  _______  _______  _______  _______ 
   (  ____ \(  ____ \(           (  ____ \(  ___  )
   | (    \/| (    \/| )     ( || (    \/| (   ) |
   | (__    | |      | (_____) || (__    | |   | |
   |  __)   | |      |  ___  ||  __)   | |   | |
   | (      | |      | (   ) || (      | |   | |
   | (____/\| (____/\| )   ( || (____/\| (___) |
   (_______/(_______/|/     \|(_______/(_______)
                                v0.1.0 :: CORE
    "#.cyan());
}

fn print_detection(path: &Path, ext: &str) {
    // Sonar contact visualization
    let filename = path.file_name().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or(Path::new("")).to_string_lossy();
    
    // Sonar color coding
    let tag = match ext {
        "rs" => "[RUST]".red().bold(),
        "ts" | "tsx" => "[TYPESCRIPT]".blue().bold(),
        "sol" => "[SOLIDITY]".yellow().bold(),
        "go" => "[GO]".cyan().bold(),
        "py" => "[PYTHON]".green().bold(),
        _ => "[SIGNAL]".white(),
    };

    println!(
        "{} {}/{}",
        tag,
        parent.dimmed(),
        filename.white()
    );
}

fn find_git_repo(path: &Path) -> Result<PathBuf> {
    let repo = Repository::discover(path)?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("Git repository has no workdir"))?;
    Ok(workdir.to_path_buf())
}

fn get_git_config(key: &str) -> Result<String> {
    let config = Config::open_default()?;
    Ok(config.get_string(key)?)
}


