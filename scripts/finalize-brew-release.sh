#!/bin/bash

# Script to finalize Homebrew release for LazyTables
# This updates the formula with correct SHA256 and pushes everything

set -e

VERSION="v0.1.3"
REPO="yuyudhan/LazyTables"

echo "🍺 Finalizing Homebrew Release for LazyTables ${VERSION}"
echo "======================================================="

# 1. Check if the tag exists locally
if ! git tag | grep -q "^${VERSION}$"; then
    echo "📌 Creating local tag ${VERSION}..."
    git tag -a "${VERSION}" -m "Release ${VERSION} - Homebrew ready"
fi

# 2. Push the tag to GitHub
echo "📤 Pushing tag to GitHub..."
git push origin "${VERSION}" 2>/dev/null || echo "Tag already exists on remote"

# 3. Wait a moment for GitHub to process
echo "⏳ Waiting for GitHub to process the tag..."
sleep 2

# 4. Download the source tarball and calculate SHA256
echo "📥 Downloading source tarball..."
rm -f "${VERSION}.tar.gz"
curl -L -o "${VERSION}.tar.gz" "https://github.com/${REPO}/archive/refs/tags/${VERSION}.tar.gz"

if [ ! -f "${VERSION}.tar.gz" ]; then
    echo "❌ Failed to download tarball. Make sure the tag ${VERSION} is pushed to GitHub."
    exit 1
fi

# 5. Calculate SHA256
SHA256=$(shasum -a 256 "${VERSION}.tar.gz" | awk '{print $1}')
echo "🔐 SHA256: ${SHA256}"

# 6. Update the formula
echo "📝 Updating Formula/lazytables.rb with correct SHA256..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/PLACEHOLDER_SHA256/${SHA256}/" Formula/lazytables.rb
    sed -i '' "s/sha256 \".*\"/sha256 \"${SHA256}\"/" Formula/lazytables.rb
else
    # Linux
    sed -i "s/PLACEHOLDER_SHA256/${SHA256}/" Formula/lazytables.rb
    sed -i "s/sha256 \".*\"/sha256 \"${SHA256}\"/" Formula/lazytables.rb
fi

# 7. Show the updated formula
echo ""
echo "📋 Updated formula:"
echo "===================="
grep -A1 "url\|sha256" Formula/lazytables.rb

# 8. Commit and push the formula update
echo ""
echo "💾 Committing formula update..."
git add Formula/lazytables.rb
git commit -m "🍺 Update formula SHA256 for v0.1.3 release

SHA256: ${SHA256}
Author: @yuyudhan, Ankur Pandey" || echo "Formula already up to date"

echo "📤 Pushing to GitHub..."
git push origin main || git push origin development

# 9. Clean up
rm -f "${VERSION}.tar.gz"

echo ""
echo "✅ Release finalized successfully!"
echo ""
echo "🎉 Installation Instructions:"
echo "============================="
echo ""
echo "For new users:"
echo "  brew tap yuyudhan/lazytables https://github.com/yuyudhan/LazyTables.git"
echo "  brew install lazytables"
echo ""
echo "For existing tap users:"
echo "  brew update"
echo "  brew upgrade lazytables"
echo ""
echo "Or install directly:"
echo "  brew install yuyudhan/lazytables/lazytables"
echo ""
echo "Test the installation:"
echo "  lazytables --version"