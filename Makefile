# FilePath: Makefile

# LazyTables Development Makefile

.PHONY: help dev build test lint format clean docker-build docker-dev

# Default target
help:
	@echo "LazyTables Development Commands:"
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
	@echo "  make db-up            - Start test PostgreSQL"
	@echo "  make db-down          - Stop test PostgreSQL"
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
	docker run -d \
		--name lazytables-postgres \
		-e POSTGRES_USER=lazytables \
		-e POSTGRES_PASSWORD=lazytables \
		-e POSTGRES_DB=test_db \
		-p 5432:5432 \
		postgres:16-alpine
	@echo "Test PostgreSQL started on port 5432"

db-down:
	docker stop lazytables-postgres || true
	docker rm lazytables-postgres || true
	@echo "Test PostgreSQL stopped"

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
