# Bounty Sources - Current & Planned

## 🔴 Current State: Manual JSON File

**Right now, bounties come from a JSON file you provide:**

```bash
cargo run -- --match-needs sample_needs.json
```

The file format is:
```json
[
  {
    "id": "need-1",
    "title": "Solana Meme Coin Dashboard",
    "description": "I need a React dashboard...",
    "bounty": "$2,500 (USDC)",
    "embedding": []
  }
]
```

**Location:** `payload-cli/src/matchmaker.rs` → `load_needs_from_file()`

---

## 🟡 Planned Sources (Not Yet Built)

According to the original spec, bounties should come from:

### 1. **Gitcoin** 
- Scrape Gitcoin Grants/Quests
- Extract bounty descriptions and amounts
- Format as `Need` objects

### 2. **Upwork**
- Scrape Upwork job postings
- Filter for development work
- Extract requirements and budgets

### 3. **Twitter/X**
- Monitor hashtags like `#bounty`, `#freelance`, `#web3dev`
- Parse tweets for project descriptions
- Extract bounty amounts from replies/DMs

### 4. **Specialized Discords**
- Join developer Discord servers
- Monitor `#jobs`, `#bounties`, `#freelance` channels
- Parse messages for project needs

### 5. **GitHub Issues**
- Scan GitHub repos for "bounty" labeled issues
- Extract issue descriptions
- Parse bounty amounts from comments

---

## 🚀 What Needs to Be Built

### Phase 4: The Feed Scraper

**Module:** `src/scraper.rs` (or separate service)

**Components:**
1. **Gitcoin Scraper**
   - API client for Gitcoin
   - Parse quest/bounty data
   - Convert to `Need` format

2. **Upwork Scraper**
   - Web scraping (or API if available)
   - Job posting parser
   - Budget extraction

3. **Twitter/X Scraper**
   - Twitter API integration
   - Hashtag monitoring
   - Tweet parsing

4. **Discord Bot**
   - Discord API client
   - Channel monitoring
   - Message parsing

5. **GitHub Issues Scraper**
   - GitHub API client
   - Issue filtering
   - Label-based detection

**Output:** Unified `needs.json` file or direct database storage

---

## 💡 Quick Implementation Options

### Option 1: Manual JSON (Current)
- ✅ Works now
- ✅ Full control
- ❌ Manual updates

### Option 2: Simple Scraper Service
- Build one scraper at a time
- Start with Gitcoin (has API)
- Add others incrementally

### Option 3: Webhook/API Integration
- Let users submit bounties via API
- Webhook from Gitcoin/Upwork
- Real-time updates

---

## 🎯 Recommended Next Steps

1. **Start with Gitcoin API**
   - Easiest to integrate
   - Has structured data
   - Good bounty volume

2. **Add GitHub Issues**
   - Already have GitHub integration
   - Easy to extend
   - Good developer community

3. **Build Feed Aggregator**
   - Combine all sources
   - Deduplicate
   - Rank by relevance

---

## 📝 Current Workaround

For now, you can:
1. Manually collect bounties from sources
2. Format as JSON
3. Run `--embed-needs` to generate embeddings
4. Use `--match-needs` to match

**Example workflow:**
```bash
# 1. Collect bounties manually, save to bounties.json
# 2. Generate embeddings
cargo run -- --embed-needs bounties.json

# 3. Match your code to bounties
cargo run -- --path . --match-needs bounties.json
```

---

**Status:** Bounty scraping is Phase 4 - not yet implemented. Currently using manual JSON files.

