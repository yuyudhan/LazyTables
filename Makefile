# FilePath: Makefile

# LazyTables Development Makefile

.PHONY: help dev build test lint format clean

# Default target
help:
	@echo "LazyTables Development Commands:"
	@echo ""
	@echo "Installation:"
	@echo "  make install          - Install LazyTables via cargo"
	@echo "  make uninstall        - Remove LazyTables from system"
	@echo ""
	@echo "Development:"
	@echo "  make dev              - Run application in development mode"
	@echo "  make build            - Build release binary"
	@echo "  make run              - Run debug build"
	@echo ""
	@echo "Testing & Quality:"
	@echo "  make test             - Run all tests"
	@echo "  make lint             - Run clippy linter"
	@echo "  make format           - Format code with rustfmt"
	@echo "  make check            - Run format check and clippy"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean            - Clean build artifacts"

# Development
dev:
	cargo watch -x run

run:
	cargo run

build:
	cargo build --release

# Installation
install:
	@echo "Installing LazyTables via cargo..."
	cargo install --path . --force

uninstall:
	@echo "Uninstalling LazyTables..."
	@cargo uninstall lazytables 2>/dev/null || true
	@echo "LazyTables has been uninstalled"

# Testing & Quality
test:
	cargo test --all-features

lint:
	cargo clippy -- -D warnings

format:
	cargo fmt

format-check:
	cargo fmt -- --check

check: format-check lint


# Cleanup
clean:
	cargo clean
	rm -rf target/
	@echo "Build artifacts cleaned"

# Install development dependencies
install-deps:
	cargo install cargo-watch
	cargo install cargo-audit
	@echo "Development dependencies installed"
