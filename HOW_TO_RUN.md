# How to Run Echeo - Complete Guide

## 🚀 Quick Start

### Step 1: Build It
```bash
cd payload-cli
cargo build --release
```

### Step 2: Run It
```bash
# Basic scan (no embeddings, fast)
cargo run -- --path . --skip-embeddings --skip-summaries

# Or use the release binary
./target/release/echeo --path . --skip-embeddings --skip-summaries
```

---

## 📋 Prerequisites

### 1. Install Rust (if not installed)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Install Ollama (for embeddings/summaries)
```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.ai/install.sh | sh
```

### 3. Start Ollama & Pull Models
```bash
# Start Ollama (in background or separate terminal)
ollama serve

# Pull required models (in another terminal)
ollama pull nomic-embed-text  # For embeddings
ollama pull llama3            # For summaries
```

---

## 🎯 Common Workflows

### Workflow 1: Quick Scan (No AI)
**Fastest way to see your capabilities:**
```bash
cargo run -- --path ~/projects --skip-embeddings --skip-summaries
```

**What it does:**
- Scans your code
- Extracts capabilities
- Shows you what you have
- **No Ollama needed**

---

### Workflow 2: Full Pipeline (With AI)
**Complete scan with embeddings and summaries:**
```bash
# Make sure Ollama is running first!
ollama serve  # In one terminal

# Then run Echeo
cargo run -- --path ~/projects
```

**What it does:**
- Scans your code
- Extracts capabilities
- Generates embeddings (needs Ollama)
- Generates summaries (needs Ollama)
- Shows complete capability list

---

### Workflow 3: Match to Bounties
**Find bounties that match your code:**
```bash
# 1. Scan your code (with embeddings)
cargo run -- --path ~/projects

# 2. Match to bounties
cargo run -- --path ~/projects --match-needs sample_needs.json
```

**What it does:**
- Uses your scanned capabilities
- Matches them to bounties in the JSON file
- Shows you matches with Ship Velocity Scores
- Tells you which bounties you can tackle

---

### Workflow 4: Scrape & Match
**Scrape bounties, then match:**
```bash
# 1. Scrape bounties from GitHub
cargo run -- \
  --github-token YOUR_TOKEN \
  --scrape-github owner/repo \
  --scrape-gitcoin \
  --scrape-all bounties.json \
  --auto-embed-scraped

# 2. Match your code to scraped bounties
cargo run -- --path ~/projects --match-needs bounties.json
```

---

### Workflow 5: Deploy a Match
**Deploy your best match:**
```bash
# 1. Match to bounties
cargo run -- --path ~/projects --match-needs bounties.json

# 2. Deploy match #1 (or whichever number you want)
cargo run -- --path ~/projects --match-needs bounties.json --deploy 1
```

**What it does:**
- Creates a new repo in `./deployments/`
- Copies your capability files
- Generates wiring code with LLM
- Initializes git repo
- Ready for you to polish and ship!

---

## 🎮 Step-by-Step Tutorial

### Tutorial: Your First Scan

**Step 1: Navigate to the project**
```bash
cd payload-cli
```

**Step 2: Build it (first time only)**
```bash
cargo build --release
```

**Step 3: Quick scan (no AI needed)**
```bash
cargo run -- --path . --skip-embeddings --skip-summaries
```

**You should see:**
```
    _______  _______  _______  _______  _______ 
   (  ____ \(  ____ \(           (  ____ \(  ___  )
   ...
INITIATING ACTIVE SONAR SWEEP...
TARGET SECTOR: .
---------------------------------
[RUST] ./src/main.rs
[TYPESCRIPT] ./src/vectorizer.rs
...
SWEEP COMPLETE.
SECTOR DENSITY: 57 Files Scanned
CONTACTS FOUND: 11 VALID SIGNALS with 42 CAPABILITIES
```

---

### Tutorial: Full Pipeline with AI

**Step 1: Start Ollama (in separate terminal)**
```bash
ollama serve
```

**Step 2: Pull models (if first time)**
```bash
ollama pull nomic-embed-text
ollama pull llama3
```

**Step 3: Run full scan**
```bash
cargo run -- --path .
```

**You should see:**
- Echeo banner
- File scanning
- Embedding generation
- Summary generation
- Complete capability list

---

### Tutorial: Match to Bounties

**Step 1: Make sure you have bounties file**
```bash
# Use the sample file
ls sample_needs.json

# Or scrape your own
cargo run -- --github-token TOKEN --scrape-github owner/repo --scrape-all bounties.json
```

**Step 2: Match your code**
```bash
cargo run -- --path . --match-needs sample_needs.json
```

**You should see:**
```
[FEED] THE FEED:

[CARD] CARD #1
  Title: Stripe Payment Integration
  Bounty: $1,800 (USDC)
  Ship Velocity: 61% Match
  Your Capability: Need
  Why: High semantic similarity (61%)
  [DEPLOY] Run: echeo --deploy 1
```

---

## 🔧 Command Reference

### Basic Commands
```bash
# Scan code
echeo --path ~/projects

# Scan with options
echeo --path ~/projects --skip-embeddings --skip-summaries

# Match to bounties
echeo --path ~/projects --match-needs bounties.json

# Deploy a match
echeo --path ~/projects --match-needs bounties.json --deploy 1

# Generate loadout
echeo --path ~/projects --generate-loadout
```

### Scraping Commands
```bash
# Scrape GitHub Issues
echeo --github-token TOKEN --scrape-github owner/repo

# Scrape Gitcoin
echeo --scrape-gitcoin

# Scrape all and save
echeo --github-token TOKEN --scrape-github owner/repo --scrape-gitcoin --scrape-all bounties.json
```

### GitHub Commands
```bash
# List repos
echeo --github-token TOKEN --github-list

# Scan GitHub repo
echeo --github-token TOKEN --github-repo owner/repo
```

---

## 🐛 Troubleshooting

### "command not found: echeo"
**Fix:** Use `cargo run --` instead, or install:
```bash
cargo install --path .
```

### "Ollama connection failed"
**Fix:** Make sure Ollama is running:
```bash
ollama serve
# Check it's working
curl http://localhost:11434/api/tags
```

### "Failed to generate embeddings"
**Fix:** Pull the model:
```bash
ollama pull nomic-embed-text
```

### "No bounties found"
**Fix:** Make sure you have a needs file:
```bash
# Use sample
echeo --path . --match-needs sample_needs.json

# Or scrape your own
echeo --scrape-github owner/repo --scrape-all bounties.json
```

---

## 📊 What Each Command Does

| Command | What It Does | Needs Ollama? |
|---------|--------------|---------------|
| `--path .` | Scans directory | No (unless embeddings) |
| `--skip-embeddings` | Skips embedding generation | No |
| `--skip-summaries` | Skips summary generation | No |
| `--match-needs FILE` | Matches to bounties | Yes (if needs not embedded) |
| `--deploy N` | Deploys match #N | Yes (for wiring code) |
| `--scrape-github REPO` | Scrapes GitHub Issues | No |
| `--scrape-gitcoin` | Scrapes Gitcoin | No |
| `--generate-loadout` | Exports capabilities | Yes |

---

## 🎯 Recommended First Run

**Start simple:**
```bash
# 1. Quick scan (no AI)
cargo run -- --path . --skip-embeddings --skip-summaries

# 2. If that works, try with AI (start Ollama first!)
ollama serve  # In another terminal
cargo run -- --path .
```

**Then try matching:**
```bash
# Match to sample bounties
cargo run -- --path . --match-needs sample_needs.json
```

---

## 💡 Pro Tips

1. **Start without Ollama** - Use `--skip-embeddings` to test quickly
2. **Use release build** - Faster: `cargo build --release && ./target/release/echeo`
3. **Save bounties** - Use `--scrape-all FILE` to save scraped bounties
4. **Filter matches** - Use `--min-score 0.5` to only see good matches
5. **Check help** - `cargo run -- --help` for all options

---

**Ready to run!** 🚀

