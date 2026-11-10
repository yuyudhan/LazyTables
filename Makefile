# FilePath: Makefile

# LazyTables Development Makefile

.PHONY: help dev build test lint format clean db-up db-down db-reset db-probe db-status

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
	@echo "  make run-debug        - Run with debug logging enabled"
	@echo ""
	@echo "Testing & Quality:"
	@echo "  make test             - Run all tests"
	@echo "  make lint             - Run clippy linter"
	@echo "  make format           - Format code with rustfmt"
	@echo "  make check            - Run format check and clippy"
	@echo ""
	@echo "Test databases (docker, see docs/dev/test-databases.md):"
	@echo "  make db-up PROFILE=pg     - Start a profile (pg, pg-cluster, mysql, mysql-cluster,"
	@echo "                              mariadb, mariadb-cluster, sqlite) and wait until healthy"
	@echo "  make db-probe PROFILE=pg  - Run the SQL probe against a running profile"
	@echo "  make db-reset PROFILE=pg  - Recreate a profile from the seed scripts"
	@echo "  make db-down [PROFILE=pg] - Stop a profile (default: all) and drop its data"
	@echo "  make db-status            - List running test-database containers"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean            - Clean build artifacts"

# Development
dev:
	cargo watch -x run

run:
	cargo run

run-debug:
	cargo run -- --log-level debug

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

# Test databases (docker/compose.yml via scripts/testdb.sh)
db-up:
	scripts/testdb.sh up $(PROFILE)

db-probe:
	scripts/testdb.sh probe $(PROFILE)

db-reset:
	scripts/testdb.sh reset $(PROFILE)

db-down:
	scripts/testdb.sh down $(PROFILE)

db-status:
	scripts/testdb.sh status


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
