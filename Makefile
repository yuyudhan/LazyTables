# FilePath: Makefile

# LazyTables Development Makefile

.PHONY: help dev build test lint format clean docker-build docker-dev

# Default target
help:
	@echo "LazyTables Development Commands:"
	@echo ""
	@echo "Installation:"
	@echo "  make install          - Install LazyTables to /usr/local/bin"
	@echo "  make install-homebrew - Install via Homebrew formula"
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
	@echo "Docker:"
	@echo "  make docker-build     - Build Docker image"
	@echo "  make docker-dev       - Run in Docker container"
	@echo ""
	@echo "Database:"
	@echo "  make db-up            - Start test PostgreSQL database"
	@echo "  make db-down          - Stop test database"
	@echo "  make db-logs          - View database logs"
	@echo "  make db-reset         - Reset databases with fresh data"
	@echo "  make db-status        - Show database container status"
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
install: build
	@echo "Installing LazyTables..."
	@./scripts/install.sh

install-homebrew:
	@echo "Installing LazyTables via Homebrew..."
	@./scripts/install-homebrew.sh

install-cargo:
	@echo "Installing LazyTables via cargo..."
	cargo install --path .

uninstall:
	@echo "Uninstalling LazyTables..."
	@sudo rm -f /usr/local/bin/lazytables
	@cargo uninstall lazytables 2>/dev/null || true
	@brew uninstall lazytables 2>/dev/null || true
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

# Docker
docker-build:
	docker build -t lazytables:latest .

docker-dev:
	docker-compose up -d
	@echo "LazyTables running in Docker"

# Database (for testing)
db-up:
	docker-compose -f docker-compose.test.yml up -d
	@echo "Test PostgreSQL database started:"
	@echo "  Host:            localhost"
	@echo "  Port:            5432"
	@echo "  Database:        test_db"
	@echo "  Container:       lazytables_test_postgres"
	@echo "  User/Pass:       lazytables / lazytables_dev"

db-down:
	docker-compose -f docker-compose.test.yml down
	@echo "Test database stopped"

db-logs:
	docker-compose -f docker-compose.test.yml logs -f

db-reset:
	docker-compose -f docker-compose.test.yml down -v
	docker-compose -f docker-compose.test.yml up -d
	@echo "Test database reset with fresh data"

db-status:
	docker-compose -f docker-compose.test.yml ps

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
