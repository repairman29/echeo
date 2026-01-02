# ECHEO - The Resonant Engine

**Mission:** Find where your code resonates with market needs.

## 🎯 What It Does

Echeo connects your existing code to live bounties. It:

1. **Scans** your codebase for capabilities (functions, classes, components)
2. **Extracts** code snippets and generates vector embeddings
3. **Summarizes** capabilities with AI-generated descriptions
4. **Matches** your capabilities to bounties/needs using vector similarity
5. **Deploys** matches by creating new repos and wiring code together

## 🚀 Quick Start

### Installation

### Via npm (Recommended)
```bash
npm install -g echeo
echeo --version
```

### Via Cargo (Rust)
```bash
cargo install --path .
# Or from crates.io (when published)
cargo install echeo
```

### From Source
```bash
git clone https://github.com/repairman29/echeo-core.git
cd echeo-core
cargo build --release
./target/release/echeo --version
```

## Quick Start

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Ollama (for embeddings and summaries)
brew install ollama
ollama pull nomic-embed-text
ollama pull llama3
ollama serve
```

### Basic Usage

```bash
# Scan local code
cargo run -- --path ~/projects

# Scan GitHub repository
cargo run -- --github-token YOUR_TOKEN --github-repo owner/repo

# List GitHub repositories
cargo run -- --github-token YOUR_TOKEN --github-list

# Match against bounties
cargo run -- --path . --match-needs sample_needs.json

# Deploy a match
cargo run -- --path . --match-needs sample_needs.json --deploy 1

# Generate loadout.json
cargo run -- --path . --generate-loadout
```

## 📋 Full Pipeline

### Phase 1: The Armory ✅
- **The Crawler**: Scans directories, respects `.gitignore`
- **The Shredder**: Extracts capabilities from AST
- **The Vectorizer**: Generates 768-dim embeddings
- **The Summarizer**: Creates 5-word descriptions

### Phase 2: The Matchmaking Core ✅
- **Vector Similarity**: Cosine similarity matching
- **Ship Velocity Score**: Calculates match quality
- **The Feed**: Tinder-style match cards

### Phase 3: The Battlefield ✅
- **The Deployer**: Creates repos and wires code
- **LLM Wiring**: Generates connecting code
- **Loadout.json**: Exports all capabilities

## 🎮 Example Workflow

```bash
# 1. Scan your code
cargo run -- --path ~/projects

# 2. Generate embeddings for needs
cargo run -- --embed-needs bounties.json

# 3. Match capabilities to bounties
cargo run -- --path ~/projects --match-needs bounties.json

# 4. Deploy the best match
cargo run -- --path ~/projects --match-needs bounties.json --deploy 1

# 5. Polish and ship!
cd deployments/need-1-processpayment
# ... make it perfect ...
git push origin main
```

## 📊 Output Example

```
[FEED] THE FEED:

[CARD] CARD #1
  Title: Solana Meme Coin Dashboard
  Bounty: $2,500 (USDC)
  Ship Velocity: 87% Match
  Your Capability: processPayment
  Why: High semantic similarity (85%), Language match: typescript
  [DEPLOY] Run: echeo --deploy 1
```

## 🔧 CLI Options

```bash
--path <DIR>              # Directory to scan (default: .)
--skip-embeddings         # Skip embedding generation
--skip-summaries          # Skip summary generation
--match-needs <FILE>      # Match against needs JSON
--embed-needs <FILE>      # Generate embeddings for needs
--deploy <INDEX>          # Deploy match by index
--deploy-dir <DIR>        # Deployment directory (default: ./deployments)
--generate-loadout        # Generate loadout.json
--github-token <TOKEN>    # GitHub personal access token
--github-repo <OWNER/REPO> # Scan GitHub repository
--github-list             # List GitHub repositories
--github-client-id <ID>   # GitHub OAuth client ID
--scrape-github <REPO>    # Scrape bounties from GitHub Issues (can specify multiple)
--scrape-gitcoin          # Scrape bounties from Gitcoin
--gitcoin-limit <N>       # Limit for Gitcoin (default: 50)
--scrape-all <FILE>       # Scrape all sources and save to file
--auto-embed-scraped      # Auto-embed scraped bounties
--min-score <FLOAT>       # Minimum match score threshold (default: 0.3)
--ollama-url <URL>        # Ollama URL (default: http://localhost:11434)
--ollama-model <MODEL>    # Embedding model (default: nomic-embed-text)
--ollama-gen-model <MODEL> # Generation model (default: llama3)
```

## 📁 Project Structure

```
echeo/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── shredder.rs      # AST parsing
│   ├── vectorizer.rs    # Embedding generation
│   ├── summarizer.rs    # Description generation
│   ├── matchmaker.rs    # Vector matching
│   └── deployer.rs      # Deployment flow
├── sample_needs.json    # Example needs file
└── Cargo.toml
```

## 🎯 Status

**Phase 1: ✅ COMPLETE** - The Armory  
**Phase 2: ✅ COMPLETE** - The Matchmaking Core  
**Phase 3: ✅ COMPLETE** - The Battlefield  
**Phase 4: ✅ COMPLETE** - The Scraper (Bounty Collection)  
**GitHub Integration: ✅ COMPLETE** - Cloud repository scanning

**ECHEO is fully operational with all core features.**

---

**Find where your code resonates.**
