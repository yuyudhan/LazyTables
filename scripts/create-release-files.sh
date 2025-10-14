#!/usr/bin/env bash
set -euo pipefail

################################################################################
# LazyTables Release File Creator
# Creates binstall-compatible release archives for multiple platforms
#
# Usage:
#   ./scripts/create-release-files.sh [OPTIONS]
#
# Options:
#   -v, --version <VERSION>    Specify version (e.g., v0.2.3 or 0.2.3)
#                              If not provided, extracts from Cargo.toml
#   -t, --target <TARGET>      Build specific target only (can be repeated)
#                              Available: x86_64-apple-darwin, aarch64-apple-darwin,
#                                        x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu
#   -a, --all                  Build all targets (default)
#   -h, --help                 Show this help message
#
# Examples:
#   ./scripts/create-release-files.sh
#   ./scripts/create-release-files.sh -v 0.2.4
#   ./scripts/create-release-files.sh -t x86_64-apple-darwin
#   ./scripts/create-release-files.sh -v 0.2.4 -t aarch64-apple-darwin -t x86_64-apple-darwin
#
# Requirements:
#   - Rust toolchain installed
#   - cross (for Linux builds on macOS): cargo install cross
#   - Docker Desktop (running) for cross-compilation
################################################################################

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BINARY_NAME="lazytables"
RELEASE_DIR="target/release/binstall-builds"
ALL_TARGETS=(
    "x86_64-apple-darwin"      # macOS Intel
    "aarch64-apple-darwin"     # macOS Apple Silicon
    "x86_64-unknown-linux-gnu" # Linux x86_64
    "aarch64-unknown-linux-gnu" # Linux ARM64
)

# Parse command line arguments
VERSION=""
SELECTED_TARGETS=()
BUILD_ALL=true

show_help() {
    grep "^#" "$0" | grep -v "^#!/" | sed 's/^# \?//' | sed 's/^#//'
    exit 0
}

while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--version)
            VERSION="$2"
            VERSION="${VERSION#v}"  # Strip 'v' prefix if present
            shift 2
            ;;
        -t|--target)
            SELECTED_TARGETS+=("$2")
            BUILD_ALL=false
            shift 2
            ;;
        -a|--all)
            BUILD_ALL=true
            shift
            ;;
        -h|--help)
            show_help
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            echo "Run with -h or --help for usage information"
            exit 1
            ;;
    esac
done

# Determine which targets to build
if [ "$BUILD_ALL" = true ]; then
    TARGETS=("${ALL_TARGETS[@]}")
else
    TARGETS=("${SELECTED_TARGETS[@]}")
fi

# Get current OS
OS=$(uname -s)
ARCH=$(uname -m)

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  LazyTables Release File Creator      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Extract version from Cargo.toml if not provided
if [ -z "$VERSION" ]; then
    echo -e "${YELLOW}→${NC} Extracting version from Cargo.toml..."
    VERSION=$(grep "^version" Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
fi

echo -e "${GREEN}✓${NC} Version: v${VERSION}"
echo ""

# Create release directory
mkdir -p "${RELEASE_DIR}"
rm -rf "${RELEASE_DIR}"/*

echo -e "${YELLOW}→${NC} Release files will be created in: ${RELEASE_DIR}/"
echo ""

# Check for required tools
check_tool() {
    if command -v "$1" &> /dev/null; then
        echo -e "${GREEN}✓${NC} $1 found"
        return 0
    else
        echo -e "${RED}✗${NC} $1 not found"
        return 1
    fi
}

echo -e "${BLUE}Checking required tools...${NC}"
check_tool "cargo" || { echo -e "${RED}Error: cargo is required${NC}"; exit 1; }
check_tool "rustc" || { echo -e "${RED}Error: rustc is required${NC}"; exit 1; }

# Check for optional cross-compilation tool
HAS_CROSS=false
if check_tool "cross"; then
    HAS_CROSS=true
fi

echo ""

# Detect current platform target
NATIVE_TARGET=""
if [ "$OS" = "Darwin" ]; then
    if [ "$ARCH" = "arm64" ]; then
        NATIVE_TARGET="aarch64-apple-darwin"
    else
        NATIVE_TARGET="x86_64-apple-darwin"
    fi
elif [ "$OS" = "Linux" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        NATIVE_TARGET="x86_64-unknown-linux-gnu"
    elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
        NATIVE_TARGET="aarch64-unknown-linux-gnu"
    fi
fi

echo -e "${BLUE}Detected platform:${NC} $OS ($ARCH)"
echo -e "${BLUE}Native target:${NC} ${NATIVE_TARGET}"
echo ""

# Function to check if target is installed
check_target_installed() {
    local target=$1
    if rustup target list --installed | grep -q "^${target}$"; then
        return 0
    else
        return 1
    fi
}

# Function to build for a target
build_target() {
    local target=$1
    local use_cross=$2

    echo -e "${YELLOW}→${NC} Building for ${target}..."

    # Check if target is installed
    if ! check_target_installed "$target" && [ "$use_cross" = false ]; then
        echo -e "${YELLOW}  Installing target ${target}...${NC}"
        rustup target add "$target" || {
            echo -e "${RED}✗${NC} Failed to install target ${target}"
            return 1
        }
    fi

    # Build
    if [ "$use_cross" = true ]; then
        if [ "$HAS_CROSS" = false ]; then
            echo -e "${YELLOW}⚠${NC}  Skipping ${target} (cross not installed)"
            echo -e "    Install with: ${BLUE}cargo install cross${NC}"
            return 1
        fi
        cross build --release --target "$target" 2>&1 | grep -v "^warning:" || true
    else
        cargo build --release --target "$target" 2>&1 | grep -v "^warning:" || true
    fi

    if [ ${PIPESTATUS[0]} -ne 0 ]; then
        echo -e "${RED}✗${NC} Build failed for ${target}"
        return 1
    fi

    # Verify binary exists
    local binary_path="target/${target}/release/${BINARY_NAME}"
    if [ ! -f "$binary_path" ]; then
        echo -e "${RED}✗${NC} Binary not found: ${binary_path}"
        return 1
    fi

    echo -e "${GREEN}✓${NC} Build successful"
    return 0
}

# Function to create archive
create_archive() {
    local target=$1
    local binary_path="target/${target}/release/${BINARY_NAME}"
    local archive_name="${BINARY_NAME}-v${VERSION}-${target}.tar.gz"
    local archive_path="${RELEASE_DIR}/${archive_name}"

    echo -e "${YELLOW}→${NC} Creating archive for ${target}..."

    # Create temporary directory for packaging
    local temp_dir=$(mktemp -d)
    cp "$binary_path" "${temp_dir}/${BINARY_NAME}"

    # Create archive
    (cd "$temp_dir" && tar czf - "${BINARY_NAME}") > "$archive_path"

    # Cleanup temp directory
    rm -rf "$temp_dir"

    # Generate checksum
    if [ "$OS" = "Darwin" ]; then
        shasum -a 256 "$archive_path" > "${archive_path}.sha256"
    else
        sha256sum "$archive_path" > "${archive_path}.sha256"
    fi

    local size=$(du -h "$archive_path" | cut -f1)
    echo -e "${GREEN}✓${NC} Created ${archive_name} (${size})"

    return 0
}

# Build and package for each target
echo -e "${BLUE}════════════════════════════════════════${NC}"
echo -e "${BLUE}Building for all targets...${NC}"
echo -e "${BLUE}════════════════════════════════════════${NC}"
echo ""

SUCCESSFUL_BUILDS=0
FAILED_BUILDS=0
SKIPPED_BUILDS=0

for target in "${TARGETS[@]}"; do
    echo -e "${BLUE}[${target}]${NC}"

    # Determine if we need to use cross
    use_cross=false

    # Determine build strategy based on host OS and target
    if [ "$OS" = "Darwin" ]; then
        # macOS can build macOS targets natively
        if [[ "$target" =~ "apple-darwin" ]]; then
            use_cross=false
        # macOS can build Linux targets with cross + Docker
        elif [[ "$target" =~ "linux" ]]; then
            if [ "$HAS_CROSS" = false ]; then
                echo -e "${YELLOW}⚠${NC}  Skipping ${target} (cross not installed - run: cargo install cross)"
                SKIPPED_BUILDS=$((SKIPPED_BUILDS + 1))
                echo ""
                continue
            fi
            use_cross=true
        else
            echo -e "${YELLOW}⚠${NC}  Skipping ${target} (unsupported target on macOS)"
            SKIPPED_BUILDS=$((SKIPPED_BUILDS + 1))
            echo ""
            continue
        fi
    elif [ "$OS" = "Linux" ]; then
        # Linux can build Linux targets
        if [[ "$target" =~ "linux" ]]; then
            # Use cross for ARM64 on x86_64 Linux
            if [ "$target" = "aarch64-unknown-linux-gnu" ] && [ "$ARCH" = "x86_64" ]; then
                use_cross=true
            else
                use_cross=false
            fi
        # Linux cannot build macOS targets
        elif [[ "$target" =~ "apple-darwin" ]]; then
            echo -e "${YELLOW}⚠${NC}  Skipping ${target} (macOS targets cannot be built on Linux)"
            SKIPPED_BUILDS=$((SKIPPED_BUILDS + 1))
            echo ""
            continue
        fi
    fi

    # Build
    if build_target "$target" "$use_cross"; then
        if create_archive "$target"; then
            SUCCESSFUL_BUILDS=$((SUCCESSFUL_BUILDS + 1))
        else
            FAILED_BUILDS=$((FAILED_BUILDS + 1))
        fi
    else
        FAILED_BUILDS=$((FAILED_BUILDS + 1))
    fi

    echo ""
done

# Summary
echo -e "${BLUE}════════════════════════════════════════${NC}"
echo -e "${BLUE}Release Summary${NC}"
echo -e "${BLUE}════════════════════════════════════════${NC}"
echo -e "${GREEN}Successful builds:${NC} ${SUCCESSFUL_BUILDS}"
if [ $FAILED_BUILDS -gt 0 ]; then
    echo -e "${RED}Failed builds:${NC}     ${FAILED_BUILDS}"
fi
if [ $SKIPPED_BUILDS -gt 0 ]; then
    echo -e "${YELLOW}Skipped builds:${NC}    ${SKIPPED_BUILDS}"
fi
echo ""

if [ $SUCCESSFUL_BUILDS -gt 0 ]; then
    echo -e "${GREEN}✓${NC} Release files created in: ${RELEASE_DIR}/"
    echo ""
    echo -e "${BLUE}Files:${NC}"
    ls -lh "${RELEASE_DIR}" | tail -n +2 | awk '{print "  " $9 " (" $5 ")"}'
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo -e "  1. Test the binaries: ${YELLOW}tar xzf release/${BINARY_NAME}-v${VERSION}-*.tar.gz${NC}"
    echo -e "  2. Create GitHub release: ${YELLOW}gh release create v${VERSION}${NC}"
    echo -e "  3. Upload files: ${YELLOW}gh release upload v${VERSION} release/*${NC}"
    echo -e "  4. Test binstall: ${YELLOW}cargo binstall ${BINARY_NAME}${NC}"
else
    echo -e "${RED}✗${NC} No successful builds"
    exit 1
fi

echo ""
echo -e "${GREEN}✓ Done!${NC}"
