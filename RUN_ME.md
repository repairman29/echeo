# 🚀 How to Run Echeo - Simple Guide

## Quick Start (30 seconds)

### 1. Navigate to the project
```bash
cd payload-cli
```

### 2. Run your first scan (no AI needed)
```bash
cargo run -- --path . --skip-embeddings --skip-summaries
```

**That's it!** You'll see:
- Echeo banner
- Files being scanned
- Capabilities found
- Summary report

---

## What Each Command Does

### Basic Scan
```bash
cargo run -- --path ~/projects --skip-embeddings --skip-summaries
```
- Scans your code
- Extracts capabilities
- Shows what you have
- **Fast (no AI needed)**

### Full Scan (With AI)
```bash
# Terminal 1: Start Ollama
ollama serve

# Terminal 2: Run Echeo
cargo run -- --path ~/projects
```
- Scans your code
- Generates embeddings (needs Ollama)
- Generates summaries (needs Ollama)
- Complete capability analysis

### Match to Bounties
```bash
cargo run -- --path . --match-needs sample_needs.json
```
- Uses your scanned capabilities
- Matches them to bounties
- Shows Ship Velocity Scores
- Tells you which bounties you can tackle

### Deploy a Match
```bash
cargo run -- --path . --match-needs bounties.json --deploy 1
```
- Creates new repo
- Copies your capability files
- Generates wiring code
- Ready to ship!

---

## Common Workflows

### Workflow 1: Quick Check
```bash
cargo run -- --path . --skip-embeddings --skip-summaries
```
**Use when:** You just want to see what capabilities you have

### Workflow 2: Full Analysis
```bash
# Start Ollama first
ollama serve

# Then run
cargo run -- --path ~/projects
```
**Use when:** You want complete analysis with embeddings

### Workflow 3: Find Work
```bash
# 1. Scan your code
cargo run -- --path ~/projects

# 2. Match to bounties
cargo run -- --path ~/projects --match-needs sample_needs.json
```
**Use when:** You want to find bounties that match your code

### Workflow 4: Scrape & Match
```bash
# 1. Scrape bounties
cargo run -- \
  --github-token YOUR_TOKEN \
  --scrape-github owner/repo \
  --scrape-all bounties.json

# 2. Match your code
cargo run -- --path ~/projects --match-needs bounties.json
```
**Use when:** You want fresh bounties from GitHub/Gitcoin

---

## Prerequisites

### For Basic Scan
- ✅ Rust installed
- ✅ That's it!

### For Full Scan
- ✅ Rust installed
- ✅ Ollama installed (`brew install ollama`)
- ✅ Ollama running (`ollama serve`)
- ✅ Models pulled (`ollama pull nomic-embed-text`)

---

## Troubleshooting

### "command not found: echeo"
**Fix:** Use `cargo run --` instead

### "Ollama connection failed"
**Fix:** Start Ollama: `ollama serve`

### "No bounties found"
**Fix:** Use sample file: `--match-needs sample_needs.json`

---

## Pro Tips

1. **Start simple** - Use `--skip-embeddings` first
2. **Scan your projects folder** - `--path ~/projects`
3. **Save bounties** - Use `--scrape-all FILE`
4. **Get help** - `cargo run -- --help`

---

**Ready to run!** 🎯

