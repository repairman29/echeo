# THE ENGINE - Installation Guide

## 🚀 Quick Install

### Option 1: From Source (Recommended)

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/yourusername/payload-cli.git
cd payload-cli
cargo build --release

# Install globally
cargo install --path .

# Verify installation
payload --version
```

### Option 2: Download Pre-built Binary

**macOS (Apple Silicon):**
```bash
curl -L https://github.com/yourusername/payload-cli/releases/latest/download/payload-cli-aarch64-apple-darwin.tar.gz | tar -xz
sudo mv payload-cli /usr/local/bin/
```

**macOS (Intel):**
```bash
curl -L https://github.com/yourusername/payload-cli/releases/latest/download/payload-cli-x86_64-apple-darwin.tar.gz | tar -xz
sudo mv payload-cli /usr/local/bin/
```

**Linux:**
```bash
curl -L https://github.com/yourusername/payload-cli/releases/latest/download/payload-cli-x86_64-unknown-linux-gnu.tar.gz | tar -xz
sudo mv payload-cli /usr/local/bin/
```

**Windows:**
```powershell
# Download from releases page
# Extract and add to PATH
```

### Option 3: Via Cargo (when published)

```bash
cargo install payload-cli
```

---

## 📋 Prerequisites

### Required
- **Rust** 1.70+ (for building from source)
- **Ollama** (for embeddings and summaries)

### Install Ollama

**macOS:**
```bash
brew install ollama
ollama serve
```

**Linux:**
```bash
curl -fsSL https://ollama.ai/install.sh | sh
ollama serve
```

**Pull Required Models:**
```bash
ollama pull nomic-embed-text  # For embeddings
ollama pull llama3            # For summaries
```

---

## ✅ Verify Installation

```bash
# Check version
payload --version

# Test scan
payload --path . --skip-embeddings --skip-summaries

# Check Ollama connection
curl http://localhost:11434/api/tags
```

---

## 🎯 First Run

```bash
# 1. Scan your code
payload --path ~/projects

# 2. Generate loadout
payload --path ~/projects --generate-loadout

# 3. View your capabilities
cat .payload/loadout.json
```

---

## 🔧 Configuration

### Environment Variables (Optional)

```bash
export PAYLOAD_OLLAMA_URL=http://localhost:11434
export PAYLOAD_OLLAMA_MODEL=nomic-embed-text
export PAYLOAD_GITHUB_TOKEN=your_token_here
```

---

## 🐛 Troubleshooting

### "command not found: payload"
- Make sure `~/.cargo/bin` is in your PATH
- Run: `export PATH="$HOME/.cargo/bin:$PATH"`

### "Ollama connection failed"
- Make sure Ollama is running: `ollama serve`
- Check: `curl http://localhost:11434/api/tags`

### "Failed to generate embeddings"
- Pull required models: `ollama pull nomic-embed-text`
- Check Ollama is running

---

## 📦 Uninstall

```bash
# If installed via cargo install
cargo uninstall payload-cli

# If installed manually
rm /usr/local/bin/payload-cli
```

---

**Ready to ship!** 🚀

