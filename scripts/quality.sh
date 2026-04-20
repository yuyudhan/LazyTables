#!/usr/bin/env bash
# FilePath: scripts/quality.sh
# Unified quality check script for LazyTables

set -uo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

PASS=0
FAIL=0
SKIP=0

MODE="full"

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --fix     Auto-fix formatting and lint issues"
    echo "  --quick   Skip tests and build, just format check + lint"
    echo "  --help    Show this help message"
    echo ""
    echo "No args runs the full quality check (format, lint, test, build)."
}

step() {
    local step_num="$1"
    local label="$2"
    printf "\n${BLUE}${BOLD}[%s]${NC} %s\n" "$step_num" "$label"
}

pass() {
    printf "  ${GREEN}✓ %s${NC}\n" "$1"
    PASS=$((PASS + 1))
}

fail() {
    printf "  ${RED}✗ %s${NC}\n" "$1"
    FAIL=$((FAIL + 1))
}

skip() {
    printf "  ${YELLOW}- %s (skipped)${NC}\n" "$1"
    SKIP=$((SKIP + 1))
}

for arg in "$@"; do
    case "$arg" in
        --fix)   MODE="fix" ;;
        --quick) MODE="quick" ;;
        --help)  usage; exit 0 ;;
        *)
            echo "Unknown option: $arg"
            usage
            exit 1
            ;;
    esac
done

printf "${BOLD}LazyTables Quality Check${NC} (mode: ${YELLOW}%s${NC})\n" "$MODE"
echo "─────────────────────────────────────────"

# Step 1: Formatting
if [ "$MODE" = "fix" ]; then
    step "1/4" "Formatting (auto-fix)"
    if cargo fmt 2>&1; then
        pass "cargo fmt applied"
    else
        fail "cargo fmt failed"
    fi
else
    step "1/4" "Formatting check"
    if cargo fmt -- --check 2>&1; then
        pass "formatting OK"
    else
        fail "formatting issues found (run with --fix to auto-fix)"
    fi
fi

# Step 2: Linting
if [ "$MODE" = "fix" ]; then
    step "2/4" "Linting (auto-fix)"
    if cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged 2>&1; then
        pass "clippy fixes applied"
    else
        fail "clippy --fix failed"
    fi
else
    step "2/4" "Linting"
    if cargo clippy --all-targets --all-features -- -D warnings 2>&1; then
        pass "clippy clean"
    else
        fail "clippy found issues"
    fi
fi

# Step 3: Tests
if [ "$MODE" = "quick" ]; then
    step "3/4" "Tests"
    skip "tests (quick mode)"
else
    step "3/4" "Tests"
    if cargo test --all-features 2>&1; then
        pass "all tests passed"
    else
        fail "tests failed"
    fi
fi

# Step 4: Build
if [ "$MODE" = "quick" ]; then
    step "4/4" "Build"
    skip "build (quick mode)"
else
    step "4/4" "Build"
    if cargo build 2>&1; then
        pass "build succeeded"
    else
        fail "build failed"
    fi
fi

# Summary
echo ""
echo "─────────────────────────────────────────"
printf "${BOLD}Summary:${NC} "
printf "${GREEN}%d passed${NC}" "$PASS"
[ "$FAIL" -gt 0 ] && printf ", ${RED}%d failed${NC}" "$FAIL"
[ "$SKIP" -gt 0 ] && printf ", ${YELLOW}%d skipped${NC}" "$SKIP"
echo ""

if [ "$FAIL" -gt 0 ]; then
    printf "${RED}${BOLD}Quality check failed.${NC}\n"
    exit 1
else
    printf "${GREEN}${BOLD}Quality check passed.${NC}\n"
    exit 0
fi
