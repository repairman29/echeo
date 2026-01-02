#!/bin/bash

# THE ENGINE - Installation Script

set -e

echo "🚀 Installing THE ENGINE..."

# Detect platform
PLATFORM=""
if [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ $(uname -m) == "arm64" ]]; then
        PLATFORM="aarch64-apple-darwin"
    else
        PLATFORM="x86_64-apple-darwin"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="x86_64-unknown-linux-gnu"
else
    echo "❌ Unsupported platform: $OSTYPE"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
fi

# Build from source
echo "🔨 Building from source..."
cd "$(dirname "$0")/.."
cargo build --release

# Install globally
echo "📦 Installing globally..."
cargo install --path . --force

# Verify installation
if command -v payload &> /dev/null; then
    echo "✅ Installation complete!"
    echo ""
    echo "Run: payload --help"
    echo ""
    echo "Don't forget to install Ollama:"
    echo "  brew install ollama  # macOS"
    echo "  curl -fsSL https://ollama.ai/install.sh | sh  # Linux"
    echo ""
    echo "Then pull models:"
    echo "  ollama pull nomic-embed-text"
    echo "  ollama pull llama3"
else
    echo "⚠️  Installation may have failed. Check PATH includes ~/.cargo/bin"
fi

