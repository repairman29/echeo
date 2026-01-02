# Quick Distribution Guide

## 🚀 Fastest Way to Share

### Option 1: GitHub Release (Recommended)

1. **Tag a release:**
```bash
git tag v0.1.0
git push origin v0.1.0
```

2. **GitHub Actions will automatically:**
   - Build binaries for macOS, Linux, Windows
   - Create release with all binaries
   - Generate release notes

3. **Share the release URL:**
   ```
   https://github.com/yourusername/payload-cli/releases/latest
   ```

---

### Option 2: Build Locally & Share

1. **Build for your platform:**
```bash
cargo build --release
```

2. **Create archive:**
```bash
cd target/release
tar czf payload-cli.tar.gz payload-cli
# or on Windows: zip payload-cli.zip payload-cli.exe
```

3. **Share the binary:**
   - Upload to file sharing service
   - Email directly
   - Host on your website

---

### Option 3: Source Distribution

1. **Share the repo:**
```bash
git clone https://github.com/yourusername/payload-cli.git
cd payload-cli
cargo install --path .
```

2. **Or create a tarball:**
```bash
git archive --format=tar.gz --prefix=payload-cli/ HEAD > payload-cli-source.tar.gz
```

---

## 📋 What Users Need

1. **Rust** (if building from source)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Ollama** (for embeddings)
   ```bash
   brew install ollama  # macOS
   # or
   curl -fsSL https://ollama.ai/install.sh | sh  # Linux
   ```

3. **Models:**
   ```bash
   ollama pull nomic-embed-text
   ollama pull llama3
   ```

---

## 🎯 One-Liner Install (After Release)

```bash
# macOS
curl -L https://github.com/yourusername/payload-cli/releases/latest/download/payload-cli-aarch64-apple-darwin.tar.gz | tar -xz && sudo mv payload-cli /usr/local/bin/

# Linux
curl -L https://github.com/yourusername/payload-cli/releases/latest/download/payload-cli-x86_64-unknown-linux-gnu.tar.gz | tar -xz && sudo mv payload-cli /usr/local/bin/
```

---

## ✅ Test Before Sharing

```bash
# Build
cargo build --release

# Test
./target/release/payload-cli --help
./target/release/payload-cli --path . --skip-embeddings --skip-summaries

# Verify
./target/release/payload-cli --version
```

---

**Ready to distribute!** 🚀

