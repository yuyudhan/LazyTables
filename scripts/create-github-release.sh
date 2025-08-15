#!/bin/bash

# Script to create GitHub release and setup Homebrew tap
# Run this after pushing the v0.1.3 tag

set -e

VERSION="v0.1.3"
REPO="yuyudhan/LazyTables"

echo "📦 Creating GitHub Release for LazyTables ${VERSION}"
echo "============================================"

# 1. First, ensure we have the gh CLI
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is not installed."
    echo "Install it with: brew install gh"
    exit 1
fi

# 2. Build the release binary
echo "🔨 Building release binary..."
cargo build --release

# 3. Create tarball for the source code
echo "📁 Creating release tarball..."
cd target/release
tar czf "lazytables-${VERSION}-$(uname -m)-$(uname -s).tar.gz" lazytables
cd ../..

# 4. Calculate SHA256 for Homebrew formula
echo "🔐 Calculating SHA256..."
SHA256=$(shasum -a 256 "target/release/lazytables-${VERSION}-"*.tar.gz | awk '{print $1}')
echo "SHA256: ${SHA256}"

# 5. Create the release using gh CLI
echo "🚀 Creating GitHub release..."
gh release create "${VERSION}" \
    --repo "${REPO}" \
    --title "LazyTables ${VERSION}" \
    --notes-file "docs/release/v0.1.3.md" \
    "target/release/lazytables-${VERSION}-"*.tar.gz

# 6. Get the source tarball SHA256
echo "📥 Downloading source tarball to calculate SHA256..."
wget -q "https://github.com/${REPO}/archive/refs/tags/${VERSION}.tar.gz" -O "${VERSION}.tar.gz"
SOURCE_SHA256=$(shasum -a 256 "${VERSION}.tar.gz" | awk '{print $1}')
echo "Source SHA256: ${SOURCE_SHA256}"

# 7. Update the formula with correct SHA256
echo "📝 Updating Homebrew formula..."
cat > Formula/lazytables.rb << EOF
class Lazytables < Formula
  desc "Terminal-based SQL database viewer and editor with vim-style navigation"
  homepage "https://github.com/yuyudhan/LazyTables"
  url "https://github.com/yuyudhan/LazyTables/archive/refs/tags/v0.1.3.tar.gz"
  sha256 "${SOURCE_SHA256}"
  license "WTFPL"
  version "0.1.3"

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release", "--locked"
    bin.install "target/release/lazytables"
  end

  test do
    # Test that the binary runs and shows version
    assert_match "0.1.3", shell_output("#{bin}/lazytables --version 2>&1", 0)
  end
end
EOF

echo "✅ Release created successfully!"
echo ""
echo "📌 Now you need to:"
echo "1. Create the tap repository at: https://github.com/yuyudhan/homebrew-lazytables"
echo "2. Make it PUBLIC (very important!)"
echo "3. Copy Formula/lazytables.rb to that repository"
echo "4. Users can then install with:"
echo "   brew tap yuyudhan/lazytables"
echo "   brew install lazytables"

# Cleanup
rm -f "${VERSION}.tar.gz"