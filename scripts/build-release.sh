#!/bin/bash

# THE ENGINE - Release Build Script
# Builds binaries for all platforms

set -e

echo "🚀 Building THE ENGINE for release..."

# Colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Create release directory
RELEASE_DIR="target/release"
mkdir -p $RELEASE_DIR

# Build for current platform
echo -e "${CYAN}Building for current platform...${NC}"
cargo build --release

# Copy binary
PLATFORM=$(rustc -vV | grep host | cut -d' ' -f2)
BINARY_NAME="payload-cli"

if [[ "$OSTYPE" == "darwin"* ]]; then
    cp target/release/$BINARY_NAME $RELEASE_DIR/payload-cli-$PLATFORM
    echo -e "${GREEN}✅ Built for $PLATFORM${NC}"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    cp target/release/$BINARY_NAME $RELEASE_DIR/payload-cli-$PLATFORM
    echo -e "${GREEN}✅ Built for $PLATFORM${NC}"
fi

# Create archive
cd $RELEASE_DIR
if [[ "$OSTYPE" == "darwin"* ]]; then
    tar czf payload-cli-$PLATFORM.tar.gz payload-cli-$PLATFORM
    echo -e "${GREEN}✅ Created payload-cli-$PLATFORM.tar.gz${NC}"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    tar czf payload-cli-$PLATFORM.tar.gz payload-cli-$PLATFORM
    echo -e "${GREEN}✅ Created payload-cli-$PLATFORM.tar.gz${NC}"
fi

cd ../..

echo ""
echo -e "${GREEN}🎯 Release build complete!${NC}"
echo -e "${CYAN}Binaries are in: $RELEASE_DIR${NC}"
echo ""
echo "To build for other platforms, install cross-compilation targets:"
echo "  rustup target add x86_64-unknown-linux-gnu"
echo "  rustup target add x86_64-pc-windows-msvc"
echo "  rustup target add aarch64-apple-darwin"

