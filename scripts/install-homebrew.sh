#!/bin/bash
# FilePath: scripts/install-homebrew.sh

set -e

echo "Installing LazyTables via Homebrew..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo -e "${YELLOW}Building LazyTables...${NC}"

# Change to project directory
cd "$PROJECT_DIR"

# Build the release binary first
if cargo build --release; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

# Check if binary was created
if [ ! -f "target/release/lazytables" ]; then
    echo -e "${RED}✗ Binary not found at target/release/lazytables${NC}"
    exit 1
fi

echo -e "${YELLOW}Installing with Homebrew...${NC}"

# Remove any existing tap
brew untap yuyudhan/lazytables 2>/dev/null || true

# Create a local tap
BREW_TAP_DIR="$(brew --prefix)/Homebrew/Library/Taps/yuyudhan/homebrew-lazytables"
mkdir -p "$BREW_TAP_DIR/Formula"

# Copy the formula
cp "$PROJECT_DIR/Formula/lazytables.rb" "$BREW_TAP_DIR/Formula/"

# Install from the tap
if brew install yuyudhan/lazytables/lazytables --verbose; then
    echo -e "${GREEN}✓ Installation successful!${NC}"
    echo ""
    echo "LazyTables has been installed to: $(which lazytables)"
    echo ""
    echo "To get started, run:"
    echo "  lazytables"
else
    echo -e "${RED}✗ Installation failed${NC}"
    echo ""
    echo "Alternative installation method:"
    echo "  1. Build manually: cargo build --release"
    echo "  2. Copy binary: sudo cp target/release/lazytables /usr/local/bin/"
    echo "  3. Make executable: sudo chmod +x /usr/local/bin/lazytables"
    exit 1
fi