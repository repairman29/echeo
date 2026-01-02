#!/bin/bash

# THE ENGINE - Full Pipeline Test
# This script tests all components of PAYLOAD CLI

set -e

echo "=========================================="
echo "  THE ENGINE - Full Pipeline Test"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test 1: Local Code Scanning
echo -e "${CYAN}[TEST 1]${NC} Local Code Scanning..."
echo "----------------------------------------"
cargo run -- --path . --skip-embeddings --skip-summaries 2>&1 | tail -10
echo ""

# Test 2: Generate Embeddings (if Ollama is running)
echo -e "${CYAN}[TEST 2]${NC} Generating Embeddings..."
echo "----------------------------------------"
if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    echo "Ollama is running, generating embeddings..."
    cargo run -- --path . --skip-summaries 2>&1 | tail -15
else
    echo -e "${YELLOW}Ollama not running, skipping embedding test${NC}"
    echo "Start Ollama with: ollama serve"
fi
echo ""

# Test 3: Generate Summaries (if Ollama is running)
echo -e "${CYAN}[TEST 3]${NC} Generating Summaries..."
echo "----------------------------------------"
if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    echo "Generating capability summaries..."
    cargo run -- --path . 2>&1 | grep -A 5 "SUMMARIZER" | head -10
else
    echo -e "${YELLOW}Ollama not running, skipping summary test${NC}"
fi
echo ""

# Test 4: Embed Needs
echo -e "${CYAN}[TEST 4]${NC} Embedding Needs..."
echo "----------------------------------------"
if [ -f "sample_needs.json" ]; then
    if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
        cargo run -- --embed-needs sample_needs.json 2>&1 | tail -5
    else
        echo -e "${YELLOW}Ollama not running, skipping${NC}"
    fi
else
    echo -e "${YELLOW}sample_needs.json not found, skipping${NC}"
fi
echo ""

# Test 5: Match Capabilities to Needs
echo -e "${CYAN}[TEST 5]${NC} Matching Capabilities to Needs..."
echo "----------------------------------------"
if [ -f "sample_needs.json" ]; then
    if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
        cargo run -- --path . --match-needs sample_needs.json 2>&1 | tail -20
    else
        echo -e "${YELLOW}Ollama not running, skipping${NC}"
    fi
else
    echo -e "${YELLOW}sample_needs.json not found, skipping${NC}"
fi
echo ""

# Test 6: Generate Loadout
echo -e "${CYAN}[TEST 6]${NC} Generating Loadout..."
echo "----------------------------------------"
if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    cargo run -- --path . --generate-loadout 2>&1 | tail -5
    if [ -f ".payload/loadout.json" ]; then
        echo -e "${GREEN}✓ Loadout generated${NC}"
        echo "  File: .payload/loadout.json"
        echo "  Size: $(wc -l < .payload/loadout.json) lines"
    fi
else
    echo -e "${YELLOW}Ollama not running, skipping${NC}"
fi
echo ""

# Test 7: GitHub Integration (if token provided)
echo -e "${CYAN}[TEST 7]${NC} GitHub Integration..."
echo "----------------------------------------"
if [ -n "$GITHUB_TOKEN" ]; then
    echo "Testing GitHub integration..."
    cargo run -- --github-token "$GITHUB_TOKEN" --github-list 2>&1 | head -15
else
    echo -e "${YELLOW}GITHUB_TOKEN not set, skipping${NC}"
    echo "  Set with: export GITHUB_TOKEN=your_token"
fi
echo ""

echo "=========================================="
echo -e "${GREEN}  Tests Complete!${NC}"
echo "=========================================="

