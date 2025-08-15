#!/bin/bash
# FilePath: scripts/create-github-release.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VERSION=${1:-"0.1.3"}

echo -e "${BLUE}Creating GitHub Release for LazyTables v${VERSION}${NC}"
echo "=============================================="
echo ""

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}GitHub CLI (gh) is not installed${NC}"
    echo "Install it with: brew install gh"
    exit 1
fi

# Check if we're in a git repo
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}Not in a git repository${NC}"
    exit 1
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Build the release package if it doesn't exist
if [ ! -f "dist/lazytables-v${VERSION}-aarch64-apple-darwin.tar.gz" ]; then
    echo -e "${YELLOW}Building release package...${NC}"
    ./scripts/build-release.sh "$VERSION"
fi

# Create git tag if it doesn't exist
if ! git tag | grep -q "^v${VERSION}$"; then
    echo -e "${YELLOW}Creating git tag v${VERSION}...${NC}"
    git tag -a "v${VERSION}" -m "Release v${VERSION}"
    git push origin "v${VERSION}"
else
    echo -e "${GREEN}✓ Tag v${VERSION} already exists${NC}"
fi

# Create release notes
RELEASE_NOTES="## LazyTables v${VERSION}

Terminal-based SQL database viewer and editor with vim-style navigation.

### Installation

#### Homebrew (macOS/Linux)
\`\`\`bash
brew tap yuyudhan/lazytables
brew install lazytables
\`\`\`

#### Direct Download
Download the appropriate binary for your system from the assets below.

### Features
- Vim-style navigation
- SQL query editor with syntax highlighting
- Multiple database support (PostgreSQL, MySQL, SQLite)
- Split-pane interface
- Secure credential storage

### Binary Downloads
- **macOS ARM64 (M1/M2)**: \`lazytables-v${VERSION}-aarch64-apple-darwin.tar.gz\`
- **macOS Intel**: \`lazytables-v${VERSION}-x86_64-apple-darwin.tar.gz\`
- **Linux x64**: \`lazytables-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz\`
"

# Create GitHub release
echo -e "${YELLOW}Creating GitHub release...${NC}"
gh release create "v${VERSION}" \
    --title "LazyTables v${VERSION}" \
    --notes "$RELEASE_NOTES" \
    dist/lazytables-v${VERSION}-*.tar.gz

echo ""
echo -e "${GREEN}✓ GitHub release created successfully!${NC}"
echo ""
echo "View the release at:"
echo "  https://github.com/yuyudhan/LazyTables/releases/tag/v${VERSION}"
echo ""
echo "Next steps:"
echo "1. Update the Homebrew formula with binary URLs"
echo "2. Test installation: brew install yuyudhan/lazytables/lazytables"