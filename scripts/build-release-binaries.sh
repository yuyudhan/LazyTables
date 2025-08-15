#!/bin/bash

# Build release binaries for all platforms
set -e

VERSION="v0.1.3"

echo "🔨 Building LazyTables ${VERSION} binaries"
echo "========================================="

# Build for current platform
echo "📦 Building for current platform..."
cargo build --release

# Create release directory
mkdir -p releases

# Package current platform binary
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

if [[ "$PLATFORM" == "darwin" ]]; then
    if [[ "$ARCH" == "arm64" ]]; then
        ARCH="aarch64"
    fi
    TAR_NAME="lazytables-${VERSION}-${ARCH}-apple-darwin.tar.gz"
else
    TAR_NAME="lazytables-${VERSION}-${ARCH}-${PLATFORM}.tar.gz"
fi

echo "📁 Creating ${TAR_NAME}..."
cd target/release
tar czf "../../releases/${TAR_NAME}" lazytables
cd ../..

# Calculate SHA256
SHA256=$(shasum -a 256 "releases/${TAR_NAME}" | awk '{print $1}')
echo "SHA256: ${SHA256}"

echo ""
echo "✅ Binary package created: releases/${TAR_NAME}"
echo ""
echo "📤 To create GitHub release with binaries:"
echo "gh release create ${VERSION} releases/*.tar.gz --title 'LazyTables ${VERSION}' --notes-file docs/release/v0.1.3.md"