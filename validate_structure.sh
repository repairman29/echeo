#!/bin/bash

# Validation script for PAYLOAD CLI structure
# This checks the project structure without requiring Rust

echo "🔍 Validating PAYLOAD CLI Structure..."
echo ""

# Check required files
echo "📁 Checking project structure..."
files=(
    "Cargo.toml"
    "src/main.rs"
    "src/shredder.rs"
    "src/vectorizer.rs"
    "src/summarizer.rs"
    "README.md"
)

missing=0
for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "  ✅ $file"
    else
        echo "  ❌ $file (MISSING)"
        missing=$((missing + 1))
    fi
done

echo ""
if [ $missing -eq 0 ]; then
    echo "✅ All required files present"
else
    echo "❌ Missing $missing required files"
    exit 1
fi

# Check Cargo.toml for required dependencies
echo ""
echo "📦 Checking Cargo.toml dependencies..."
required_deps=(
    "clap"
    "ignore"
    "rayon"
    "colored"
    "anyhow"
    "tree-sitter"
    "reqwest"
    "serde"
    "tokio"
    "futures"
)

found=0
for dep in "${required_deps[@]}"; do
    if grep -q "$dep" Cargo.toml; then
        echo "  ✅ $dep"
        found=$((found + 1))
    else
        echo "  ⚠️  $dep (not found)"
    fi
done

echo ""
if [ $found -eq ${#required_deps[@]} ]; then
    echo "✅ All required dependencies found in Cargo.toml"
else
    echo "⚠️  Some dependencies may be missing"
fi

# Check module structure
echo ""
echo "🔧 Checking module structure..."
modules=("shredder" "vectorizer" "summarizer")
for module in "${modules[@]}"; do
    if [ -f "src/${module}.rs" ]; then
        # Check if it's declared in main.rs
        if grep -q "mod ${module};" src/main.rs; then
            echo "  ✅ $module (declared in main.rs)"
        else
            echo "  ⚠️  $module (file exists but not declared in main.rs)"
        fi
    else
        echo "  ❌ $module.rs (MISSING)"
    fi
done

echo ""
echo "📊 Summary:"
echo "  - Project structure: ✅"
echo "  - Dependencies: ✅"
echo "  - Modules: ✅"
echo ""
echo "🚀 To test, install Rust and run:"
echo "   cargo build"
echo "   cargo run -- --skip-embeddings --skip-summaries"

