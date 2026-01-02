# Bounty Scraper Features ✅

## 🎯 What's Working

### GitHub Issues Scraper
- ✅ Scrapes open issues from GitHub repos
- ✅ Filters by "bounty" or "reward" labels
- ✅ Extracts bounty amounts from text
- ✅ Supports multiple repos
- ✅ Requires GitHub token

### Gitcoin Scraper
- ✅ Scrapes active bounties from Gitcoin API
- ✅ Extracts title, description, amounts
- ✅ Configurable limit
- ✅ Handles API errors gracefully

### Unified Aggregator
- ✅ Combines bounties from all sources
- ✅ Converts to `Need` format
- ✅ Auto-embedding support
- ✅ Save to JSON file

---

## 🚀 Quick Start

### Scrape & Match in One Command
```bash
# Scrape, embed, and match
cargo run -- \
  --github-token YOUR_TOKEN \
  --scrape-github owner/repo \
  --scrape-gitcoin \
  --scrape-all bounties.json \
  --auto-embed-scraped \
  --path . \
  --match-needs bounties.json
```

### Just Scrape
```bash
cargo run -- \
  --github-token YOUR_TOKEN \
  --scrape-github owner/repo \
  --scrape-gitcoin \
  --scrape-all bounties.json
```

---

## 📊 Output Format

Scraped bounties are saved as `Need` objects:
```json
{
  "id": "github-owner-repo-123",
  "title": "Build Solana Dashboard",
  "description": "Need a React dashboard...",
  "bounty": "$2,500 (USDC)",
  "embedding": [0.1, 0.2, ...]
}
```

---

**Status:** ✅ **READY TO USE**

