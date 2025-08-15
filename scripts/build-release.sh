#!/bin/bash
# FilePath: scripts/build-release.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VERSION=${1:-"0.1.3"}
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DIST_DIR="$PROJECT_DIR/dist"

echo -e "${BLUE}Building LazyTables Release v${VERSION}${NC}"
echo "================================"

cd "$PROJECT_DIR"

# Clean previous builds
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Detect architecture
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

if [ "$OS" = "darwin" ]; then
    if [ "$ARCH" = "arm64" ]; then
        TARGET="aarch64-apple-darwin"
    else
        TARGET="x86_64-apple-darwin"
    fi
elif [ "$OS" = "linux" ]; then
    TARGET="x86_64-unknown-linux-gnu"
else
    echo -e "${RED}Unsupported OS: $OS${NC}"
    exit 1
fi

echo -e "${YELLOW}Building for target: ${TARGET}${NC}"

# Build release binary
cargo build --release --target-dir target

# Create release package
RELEASE_NAME="lazytables-v${VERSION}-${TARGET}"
RELEASE_DIR="$DIST_DIR/$RELEASE_NAME"

mkdir -p "$RELEASE_DIR"
cp "target/release/lazytables" "$RELEASE_DIR/"
cp "README.md" "$RELEASE_DIR/"
cp "LICENSE" "$RELEASE_DIR/"

# Create tar.gz archive
cd "$DIST_DIR"
tar -czf "${RELEASE_NAME}.tar.gz" "$RELEASE_NAME"

# Calculate SHA256
SHA256=$(shasum -a 256 "${RELEASE_NAME}.tar.gz" | awk '{print $1}')

echo ""
echo -e "${GREEN}✓ Release built successfully!${NC}"
echo ""
echo "Release package: $DIST_DIR/${RELEASE_NAME}.tar.gz"
echo "SHA256: $SHA256"
echo ""
echo "To update the Homebrew formula, replace the placeholder SHA256 with:"
echo "  sha256 \"$SHA256\""