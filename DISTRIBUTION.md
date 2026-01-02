# THE ENGINE - Distribution Guide

## 🎯 Distribution Options

### 1. **GitHub Releases** (Recommended)
- Pre-built binaries for macOS, Linux, Windows
- Automatic via GitHub Actions
- Easy download and install

### 2. **Crates.io** (Rust Package Registry)
- Install via `cargo install payload-cli`
- Automatic updates
- Best for Rust developers

### 3. **Homebrew** (macOS)
- `brew install payload-cli`
- Easy updates
- Best for macOS users

### 4. **Direct Download**
- Standalone binaries
- No dependencies
- Best for quick testing

---

## 📦 Building for Distribution

### Quick Build (Current Platform)
```bash
./scripts/build-release.sh
```

### Build All Platforms (Requires Cross-Compilation)
```bash
# Install cross-compilation targets
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-apple-darwin

# Build for each
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target aarch64-apple-darwin
```

### Create Release Archive
```bash
# macOS/Linux
tar czf payload-cli-$PLATFORM.tar.gz payload-cli

# Windows
zip payload-cli-$PLATFORM.zip payload-cli.exe
```

---

## 🚀 Publishing to Crates.io

### 1. Prepare Cargo.toml
```toml
[package]
name = "payload-cli"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your@email.com>"]
description = "THE ENGINE - Connect your code to bounties"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/payload-cli"
homepage = "https://github.com/yourusername/payload-cli"
keywords = ["bounty", "code", "matching", "freelance"]
categories = ["command-line-utilities", "development-tools"]

[[bin]]
name = "payload"
path = "src/main.rs"
```

### 2. Publish
```bash
# Login
cargo login

# Publish
cargo publish
```

### 3. Install
```bash
cargo install payload-cli
```

---

## 🍺 Homebrew Formula

Create `payload-cli.rb`:

```ruby
class PayloadCli < Formula
  desc "THE ENGINE - Connect your code to bounties"
  homepage "https://github.com/yourusername/payload-cli"
  url "https://github.com/yourusername/payload-cli/releases/download/v0.1.0/payload-cli-x86_64-apple-darwin.tar.gz"
  sha256 "..."

  def install
    bin.install "payload-cli" => "payload"
  end

  test do
    system "#{bin}/payload", "--version"
  end
end
```

---

## 📋 Release Checklist

- [ ] Update version in Cargo.toml
- [ ] Update CHANGELOG.md
- [ ] Run tests: `cargo test`
- [ ] Build release: `cargo build --release`
- [ ] Test binary: `./target/release/payload-cli --help`
- [ ] Create GitHub release tag: `git tag v0.1.0`
- [ ] Push tag: `git push origin v0.1.0`
- [ ] GitHub Actions will build binaries automatically
- [ ] Upload to crates.io (if publishing)
- [ ] Update documentation

---

## 🎯 Quick Start for Users

### From Source
```bash
git clone https://github.com/yourusername/payload-cli.git
cd payload-cli
cargo install --path .
```

### From Binary
```bash
# Download from releases
curl -L https://github.com/yourusername/payload-cli/releases/latest/download/payload-cli-x86_64-apple-darwin.tar.gz | tar -xz
sudo mv payload-cli /usr/local/bin/
```

### From Cargo
```bash
cargo install payload-cli
```

---

## 📊 Distribution Stats

**Target Platforms:**
- ✅ macOS (Intel + Apple Silicon)
- ✅ Linux (x86_64)
- ⚠️ Windows (needs testing)

**Package Formats:**
- ✅ tar.gz (macOS/Linux)
- ✅ zip (Windows)
- ✅ Cargo crate

---

**Status:** Ready for distribution! 🚀

