#!/bin/bash

# Setup script for Homebrew tap repository
# This creates the initial structure for yuyudhan/homebrew-lazytables

set -e

echo "Setting up Homebrew tap for LazyTables..."

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed. Please install it first:"
    echo "  brew install gh"
    exit 1
fi

# Create tap repository if it doesn't exist
REPO_NAME="homebrew-lazytables"
echo "Creating repository: yuyudhan/$REPO_NAME"

# Create the repository using GitHub CLI
gh repo create yuyudhan/$REPO_NAME --public --description "Homebrew tap for LazyTables" || echo "Repository may already exist"

# Clone the repository
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"
git clone "https://github.com/yuyudhan/$REPO_NAME.git" || git clone "git@github.com:yuyudhan/$REPO_NAME.git"
cd "$REPO_NAME"

# Create Formula directory
mkdir -p Formula

# Copy the formula file
cp "$(dirname "$0")/../homebrew/lazytables.rb" Formula/

# Create README
cat > README.md << 'EOF'
# Homebrew LazyTables

Homebrew tap for [LazyTables](https://github.com/yuyudhan/LazyTables)

## Installation

```bash
brew tap yuyudhan/lazytables
brew install lazytables
```

## Development

To install the development version:

```bash
brew install --HEAD yuyudhan/lazytables/lazytables
```

## License

WTFPL - Do What The F*ck You Want To Public License
EOF

# Commit and push
git add .
git commit -m "Initial tap setup for LazyTables v0.1.3"
git push origin main

echo "✅ Homebrew tap setup complete!"
echo ""
echo "Next steps:"
echo "1. Update the SHA256 hash in Formula/lazytables.rb after creating the release"
echo "2. Test the tap with: brew tap yuyudhan/lazytables && brew install lazytables"
echo ""
echo "Repository: https://github.com/yuyudhan/$REPO_NAME"