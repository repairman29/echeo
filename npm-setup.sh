#!/bin/bash

# Echeo npm Setup Script
# This script helps you set up and publish Echeo to npm

set -e

echo "🚀 Echeo npm Setup"
echo "=================="
echo ""

# Check if logged into npm
if ! npm whoami &> /dev/null; then
    echo "❌ Not logged into npm"
    echo ""
    echo "Please log in:"
    echo "  npm login"
    echo ""
    exit 1
fi

echo "✅ Logged into npm as: $(npm whoami)"
echo ""

# Check if package name is available
PACKAGE_NAME="echeo"
echo "📦 Checking if package name '$PACKAGE_NAME' is available..."

if npm view "$PACKAGE_NAME" &> /dev/null; then
    echo "⚠️  Package name '$PACKAGE_NAME' is already taken!"
    echo ""
    echo "Options:"
    echo "1. Use a scoped package: @yourusername/echeo"
    echo "2. Choose a different name"
    echo ""
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    echo "✅ Package name '$PACKAGE_NAME' is available!"
fi

echo ""
echo "📋 Pre-publish checklist:"
echo ""

# Check version
VERSION=$(node -p "require('./package.json').version")
echo "  Version: $VERSION"

# Check if bin directory exists
if [ ! -d "bin" ]; then
    echo "  Creating bin directory..."
    mkdir -p bin
fi

# Check if binary exists
if [ ! -f "bin/echeo" ] && [ ! -f "bin/echeo.exe" ]; then
    echo "  ⚠️  Binary not found in bin/"
    echo "  Options:"
    echo "    1. Build from source: cargo build --release && cp target/release/echeo bin/echeo"
    echo "    2. Download from GitHub Releases"
    echo "    3. Let install script handle it (recommended)"
    echo ""
else
    echo "  ✅ Binary found"
fi

# Check install scripts
if [ ! -f "install.js" ]; then
    echo "  ❌ install.js not found"
    exit 1
else
    echo "  ✅ install.js found"
fi

if [ ! -f "postinstall.js" ]; then
    echo "  ❌ postinstall.js not found"
    exit 1
else
    echo "  ✅ postinstall.js found"
fi

echo ""
echo "🧪 Testing package..."
npm pack --dry-run

echo ""
echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Test locally: npm pack"
echo "  2. Install locally: npm install -g ./echeo-$VERSION.tgz"
echo "  3. Test: echeo --version"
echo "  4. Publish: npm publish"
echo ""

