# Testing PAYLOAD CLI

## Prerequisites

Rust is not currently installed. To test the CLI, you'll need to:

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Verify Installation

```bash
rustc --version
cargo --version
```

### 3. Build the Project

```bash
cd payload-cli
cargo build
```

### 4. Run Basic Test (No Ollama)

```bash
# Test on current directory
cargo run -- --skip-embeddings --skip-summaries

# Test on a specific directory
cargo run -- --path ~/projects --skip-embeddings --skip-summaries
```

### 5. Test with Ollama (Full Pipeline)

First, install and start Ollama:

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Pull required models
ollama pull nomic-embed-text
ollama pull llama3

# Start Ollama (in a separate terminal)
ollama serve
```

Then run the full pipeline:

```bash
cargo run -- --path ~/projects
```

## Expected Output

### Without Ollama (Fast Mode)
```
INITIALIZING PAYLOAD SCANNER...
Target: /Users/jeffadkins/Smugglers/payload-cli
[VECTORIZER] Embeddings disabled.
[SUMMARIZER] Summaries disabled.
---------------------------------
[TYPESCRIPT] src/main.rs
[RUST] src/shredder.rs
---------------------------------
SCAN COMPLETE. Scanned X files. Detected Y HIGH VALUE files with Z extracted capabilities.
```

### With Ollama (Full Pipeline)
```
INITIALIZING PAYLOAD SCANNER...
Target: ~/projects
[VECTORIZER] Ollama detected. Embeddings enabled.
[SUMMARIZER] Summaries enabled (model: llama3).
---------------------------------
[TYPESCRIPT] src/components/auth.tsx
[RUST] lib/crypto.rs
---------------------------------
SCAN COMPLETE. Scanned 1247 files. Detected 89 HIGH VALUE files with 342 extracted capabilities.
---------------------------------
[VECTORIZER] Generating embeddings...
[VECTORIZER] Generated 342 embeddings (768 dimensions each)
  [EMBED] processPayment → 768D vector
---------------------------------
[SUMMARIZER] Generating capability descriptions...
[SUMMARIZER] Generated 342 descriptions
---------------------------------
[LOADOUT] Sample Capabilities:
  processPayment → Stripe subscription payment handler function
  AuthComponent → React authentication form component
```

## Troubleshooting

### Build Errors
- Make sure Rust is properly installed: `rustup update`
- Check dependencies: `cargo check`

### Ollama Connection Errors
- Verify Ollama is running: `curl http://localhost:11434/api/tags`
- Check models are pulled: `ollama list`
- Use `--skip-embeddings` and `--skip-summaries` to test without Ollama

### No Files Detected
- Check the path is correct
- Verify files have supported extensions (.ts, .rs, .go, .py, etc.)
- Check `.gitignore` isn't excluding everything

