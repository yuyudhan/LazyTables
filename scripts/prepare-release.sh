#!/bin/bash

# Prepare release for LazyTables
# This script prepares everything needed for a Homebrew release

set -e

VERSION="v0.1.3"
BINARY_NAME="lazytables"

echo "🚀 Preparing LazyTables release ${VERSION}"
echo "=================================="

# 1. Build release binary
echo "📦 Building release binary..."
cargo build --release

# 2. Create distribution archive
echo "📁 Creating distribution archive..."
cd target/release
tar czf "${BINARY_NAME}-${VERSION}-$(uname -m)-$(uname -s | tr '[:upper:]' '[:lower:]').tar.gz" "${BINARY_NAME}"
cd ../..

# 3. Calculate SHA256
echo "🔐 Calculating SHA256..."
SHA256=$(shasum -a 256 "target/release/${BINARY_NAME}-${VERSION}-"*.tar.gz | awk '{print $1}')
echo "SHA256: ${SHA256}"

# 4. Update Formula with SHA256
echo "📝 Updating Formula..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/PLACEHOLDER_SHA256/${SHA256}/" Formula/lazytables.rb
else
    sed -i "s/PLACEHOLDER_SHA256/${SHA256}/" Formula/lazytables.rb
fi

# 5. Create release notes
cat > RELEASE_NOTES.md << EOF
# LazyTables ${VERSION}

## What's New
- Initial stable release for Homebrew distribution
- Full PostgreSQL support with connection management
- Vim-style navigation throughout the interface
- SQL query editor with syntax highlighting
- Table viewer with search and filtering
- Secure credential storage options

## Installation

### Homebrew (macOS)
\`\`\`bash
brew tap yuyudhan/lazytables
brew install lazytables
\`\`\`

### Build from Source
\`\`\`bash
cargo build --release
\`\`\`

## Checksums
- SHA256: ${SHA256}

## Contributors
- @yuyudhan, Ankur Pandey

---
*LazyTables - Terminal-based SQL database management with vim-style navigation*
EOF

echo "✅ Release preparation complete!"
echo ""
echo "Next steps:"
echo "1. Commit all changes: git add . && git commit -m '🚀 Release v0.1.3'"
echo "2. Create and push tag: git tag -a ${VERSION} -m 'Release ${VERSION}' && git push origin ${VERSION}"
echo "3. Push to main branch: git push origin main"
echo "4. GitHub Actions will automatically create the release"
echo ""
echo "For manual Homebrew tap setup:"
echo "1. Create repository: https://github.com/yuyudhan/homebrew-lazytables"
echo "2. Copy Formula/lazytables.rb to the tap repository"
echo "3. Users can then: brew tap yuyudhan/lazytables && brew install lazytables"