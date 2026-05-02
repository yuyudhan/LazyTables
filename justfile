# FilePath: justfile

# LazyTables task runner
# Run `just` to see all available recipes

default:
    @just --list --unsorted

# ============================================================
# Installation
# ============================================================

# Install LazyTables via cargo
install:
    @echo "Installing LazyTables via cargo..."
    cargo install --path . --force

# Remove LazyTables from system
uninstall:
    @echo "Uninstalling LazyTables..."
    @cargo uninstall lazytables 2>/dev/null || true
    @echo "LazyTables has been uninstalled"

# Install development dependencies (cargo-watch, cargo-audit)
install-deps:
    cargo install cargo-watch
    cargo install cargo-audit
    @echo "Development dependencies installed"

# ============================================================
# Development
# ============================================================

# Run with cargo watch for auto-reload
dev:
    cargo watch -x run

# Run debug build
run:
    cargo run

# Run with debug logging enabled
run-debug:
    cargo run -- --log-level debug

# Build release binary
build:
    cargo build --release

# ============================================================
# Testing & Quality
# ============================================================

# Run all tests
test:
    cargo test --all-features

# Run clippy linter with warnings as errors
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Auto-format code with rustfmt
format:
    cargo fmt

# Check formatting without modifying files
format-check:
    cargo fmt -- --check

# Run format check and clippy (CI-friendly)
check: format-check lint

# Full quality check (format, lint, test, build)
quality:
    @./scripts/quality.sh

# Quick check (format + lint only)
quick-lint:
    @./scripts/quality.sh --quick

# Auto-fix formatting and lint issues
fix:
    @./scripts/quality.sh --fix

# ============================================================
# Diagnostics
# ============================================================

# Check that required tools are installed
doctor:
    #!/usr/bin/env bash
    set -euo pipefail
    ok=true
    check() {
        if command -v "$1" &>/dev/null; then
            printf "  %-16s %s\n" "$1" "$(command -v "$1")"
        else
            printf "  %-16s MISSING\n" "$1"
            ok=false
        fi
    }
    echo "Checking required tools:"
    check cargo
    check rustc
    echo ""
    echo "Checking optional tools:"
    check cargo-watch
    check cargo-audit
    echo ""
    if [ "$ok" = true ]; then
        echo "All tools found."
    else
        echo "Some tools are missing. Run 'just install-deps' to install optional ones."
    fi

# ============================================================
# Cleanup
# ============================================================

# Clean build artifacts and target/ directory
clean:
    cargo clean
    rm -rf target/
    @echo "Build artifacts cleaned"

# ============================================================
# Help
# ============================================================

# Show formatted help text
help:
    @echo "LazyTables Development Commands:"
    @echo ""
    @echo "Installation:"
    @echo "  just install          - Install LazyTables via cargo"
    @echo "  just uninstall        - Remove LazyTables from system"
    @echo "  just install-deps     - Install dev dependencies"
    @echo ""
    @echo "Development:"
    @echo "  just dev              - Run with cargo watch (auto-reload)"
    @echo "  just build            - Build release binary"
    @echo "  just run              - Run debug build"
    @echo "  just run-debug        - Run with debug logging enabled"
    @echo ""
    @echo "Testing & Quality:"
    @echo "  just test             - Run all tests"
    @echo "  just lint             - Run clippy linter"
    @echo "  just format           - Format code with rustfmt"
    @echo "  just check            - Run format check and clippy"
    @echo "  just quality          - Full quality check (format, lint, test, build)"
    @echo "  just quick-lint       - Quick check (format + lint only)"
    @echo "  just fix              - Auto-fix formatting and lint issues"
    @echo ""
    @echo "Diagnostics:"
    @echo "  just doctor           - Check that required tools are installed"
    @echo ""
    @echo "Cleanup:"
    @echo "  just clean            - Clean build artifacts"
