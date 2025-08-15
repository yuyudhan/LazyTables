#!/bin/bash
# FilePath: scripts/install.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}LazyTables Installation Script${NC}"
echo "================================"
echo ""

# Detect OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     OS_TYPE=Linux;;
    Darwin*)    OS_TYPE=Mac;;
    *)          OS_TYPE="UNKNOWN:${OS}"
esac

if [ "$OS_TYPE" = "UNKNOWN:${OS}" ]; then
    echo -e "${RED}✗ Unsupported OS: ${OS}${NC}"
    echo "LazyTables supports macOS and Linux only."
    exit 1
fi

echo -e "${GREEN}✓ Detected OS: ${OS_TYPE}${NC}"

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ Rust is not installed${NC}"
    echo ""
    echo "Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo -e "${GREEN}✓ Rust is installed${NC}"

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Build the project
echo ""
echo -e "${YELLOW}Building LazyTables...${NC}"
if cargo build --release; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

# Install the binary
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="lazytables"
SOURCE_BINARY="target/release/${BINARY_NAME}"

if [ ! -f "$SOURCE_BINARY" ]; then
    echo -e "${RED}✗ Binary not found at ${SOURCE_BINARY}${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Installing to ${INSTALL_DIR}...${NC}"
echo "This may require sudo permissions."

# Copy binary to install directory
if sudo cp "$SOURCE_BINARY" "${INSTALL_DIR}/${BINARY_NAME}"; then
    sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    echo -e "${GREEN}✓ Installation successful!${NC}"
else
    echo -e "${RED}✗ Installation failed${NC}"
    exit 1
fi

# Create config directories
CONFIG_DIR="$HOME/.lazytables"
echo ""
echo -e "${YELLOW}Creating configuration directories...${NC}"
mkdir -p "$CONFIG_DIR/connections"
mkdir -p "$CONFIG_DIR/sql_files"
mkdir -p "$CONFIG_DIR/logs"
mkdir -p "$CONFIG_DIR/backups"

# Create sample SQL file
SAMPLE_SQL="$CONFIG_DIR/sql_files/sample_queries.sql"
if [ ! -f "$SAMPLE_SQL" ]; then
    cat > "$SAMPLE_SQL" <<'EOF'
-- Sample SQL Queries for LazyTables

-- Show all tables
SELECT * FROM information_schema.tables 
WHERE table_schema = 'public';

-- Show table columns
SELECT * FROM information_schema.columns 
WHERE table_name = 'your_table_name';

-- Sample data query
SELECT * FROM your_table LIMIT 10;
EOF
    echo -e "${GREEN}✓ Created sample SQL file${NC}"
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}LazyTables has been installed!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Binary location: ${INSTALL_DIR}/${BINARY_NAME}"
echo "Config location: ${CONFIG_DIR}"
echo ""
echo "To get started:"
echo "  ${BINARY_NAME}"
echo ""
echo "For help:"
echo "  ${BINARY_NAME} --help"