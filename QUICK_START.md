# Echeo - Quick Start Guide

## 🚀 Run Your First Scan (30 seconds)

### Option 1: Quick Scan (No AI Required)
```bash
cd payload-cli
cargo run -- --path . --skip-embeddings --skip-summaries
```

**This will:**
- Show Echeo banner
- Scan your code
- List all capabilities found
- **No Ollama needed!**

---

### Option 2: Full Scan (With AI)
```bash
# Terminal 1: Start Ollama
ollama serve

# Terminal 2: Run Echeo
cd payload-cli
cargo run -- --path .
```

**This will:**
- Scan your code
- Generate embeddings (needs Ollama)
- Generate summaries (needs Ollama)
- Show complete capability list

---

## 📋 What You'll See

### Quick Scan Output
```
    _______  _______  _______  _______  _______ 
   (  ____ \(  ____ \(           (  ____ \(  ___  )
   ...
INITIATING ACTIVE SONAR SWEEP...
TARGET SECTOR: .
---------------------------------
[RUST] ./src/main.rs
[TYPESCRIPT] ./src/vectorizer.rs
[GO] ./src/matchmaker.rs
...
SWEEP COMPLETE.
SECTOR DENSITY: 57 Files Scanned
CONTACTS FOUND: 11 VALID SIGNALS with 42 CAPABILITIES
```

---

## 🎯 Next Steps

### 1. Match to Bounties
```bash
# Use sample bounties
cargo run -- --path . --match-needs sample_needs.json
```

### 2. Scrape Your Own Bounties
```bash
# Scrape from GitHub
cargo run -- \
  --github-token YOUR_TOKEN \
  --scrape-github owner/repo \
  --scrape-all bounties.json
```

### 3. Deploy a Match
```bash
# Match first
cargo run -- --path . --match-needs bounties.json

# Then deploy match #1
cargo run -- --path . --match-needs bounties.json --deploy 1
```

---

## 🔧 Common Commands

```bash
# Scan current directory
cargo run -- --path .

# Scan specific directory
cargo run -- --path ~/projects

# Match to bounties
cargo run -- --path . --match-needs bounties.json

# Deploy best match
cargo run -- --path . --match-needs bounties.json --deploy 1

# Generate loadout file
cargo run -- --path . --generate-loadout

# Get help
cargo run -- --help
```

---

## ⚡ Pro Tips

1. **Start without Ollama** - Use `--skip-embeddings` to test quickly
2. **Use release build** - Faster: `cargo build --release && ./target/release/echeo`
3. **Scan your projects folder** - `--path ~/projects` to see all your code
4. **Save bounties** - Use `--scrape-all FILE` to save for later

---

**That's it! You're ready to run Echeo.** 🎯

