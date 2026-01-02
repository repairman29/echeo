# Changelog

All notable changes to THE ENGINE will be documented in this file.

## [0.1.0] - 2025-01-01

### Added
- **Phase 1: The Armory** - Code scanning, AST parsing, embeddings, summarization
- **Phase 2: The Matchmaking Core** - Vector similarity matching, Ship Velocity Score
- **Phase 3: The Battlefield** - Deployment flow, LLM wiring, loadout generation
- **Phase 4: The Scraper** - GitHub Issues and Gitcoin bounty scraping
- **GitHub Integration** - Repository scanning and OAuth support
- **15 Language Support** - TypeScript, Rust, Python, Go, JavaScript, Java, C++, C, Swift, Kotlin, PHP, Scala, Dart, Solidity, Ruby
- **AST Parsing** - Full support for TypeScript, Rust, Python, Go
- **CLI Interface** - Complete command-line interface with 20+ options
- **Loadout Generation** - JSON export of all capabilities
- **Auto-Embedding** - Automatic embedding generation for scraped bounties

### Features
- Fast parallel scanning with `.gitignore` support
- 768-dimension vector embeddings via Ollama
- 5-word capability descriptions via Llama3
- Cosine similarity matching
- Ship Velocity Score calculation
- Tinder-style match feed
- Automatic deployment with LLM code wiring
- Bounty amount extraction from text
- Score filtering and composite matching

### Performance
- Parallel file scanning with Rayon
- Async operations with Tokio
- Fast vector similarity calculations
- Optimized release builds

---

## [Unreleased]

### Planned
- More AST parsers (Java, C++, Swift)
- More bounty sources (Upwork, Twitter, Discord)
- Embedding cache implementation
- Web UI (when requested)
- Batch operations
- Scheduled scraping

---

**THE ENGINE v0.1.0 - Ready to ship!** 🚀

