# 003 - Development Commands

This document provides a comprehensive reference for all available development commands in LazyTables.

## Quick Reference

```bash
# Development
make dev              # Run with auto-reload using cargo watch
make run              # Run debug build
make build            # Build release binary

# Testing & Quality
make test             # Run all tests
make lint             # Run clippy linter
make format           # Auto-format code with rustfmt
make check            # Run format check and clippy

# Database Testing
make db-up            # Start test PostgreSQL in Docker
make db-down          # Stop test PostgreSQL

# Docker
make docker-build     # Build Docker image
make docker-dev       # Run in Docker container

# Utilities
make clean            # Clean build artifacts
make install-deps     # Install development dependencies
make help             # Show all available commands
```

## Development Commands

### `make dev`
**Auto-reload development mode**

```bash
make dev
```

Uses `cargo watch` to automatically recompile and restart the application when source files change. This is the recommended way to develop LazyTables.

**Requirements**: `cargo-watch` (installed via `make install-deps`)

### `make run`
**Run debug build**

```bash
make run
```

Equivalent to `cargo run`. Builds and runs the application in debug mode without auto-reload.

### `make build`
**Build release binary**

```bash
make build
```

Creates an optimized release build in `target/release/lazytables`. Use this for performance testing or creating distribution binaries.

## Testing & Quality Commands

### `make test`
**Run all tests**

```bash
make test
```

Runs all unit, integration, and documentation tests with all features enabled.

**Equivalent to**: `cargo test --all-features`

### `make lint`
**Run Clippy linter**

```bash
make lint
```

Runs Rust's Clippy linter with strict warnings enabled. All warnings are treated as errors.

**Equivalent to**: `cargo clippy -- -D warnings`

### `make format`
**Auto-format code**

```bash
make format
```

Automatically formats all Rust code according to project standards using `rustfmt`.

**Equivalent to**: `cargo fmt`

### `make format-check`
**Check code formatting**

```bash
make format-check
```

Checks if code is properly formatted without making changes. Useful in CI/CD pipelines.

**Equivalent to**: `cargo fmt -- --check`

### `make check`
**Run format check and linting**

```bash
make check
```

Combines `make format-check` and `make lint`. This is what CI runs to validate code quality.

## Database Commands

### `make db-up`
**Start test PostgreSQL database**

```bash
make db-up
```

Starts a PostgreSQL container for testing with the following configuration:
- **Container name**: `lazytables-postgres`
- **Host**: `localhost`
- **Port**: `5432`
- **User**: `lazytables`
- **Password**: `lazytables`
- **Database**: `test_db`
- **Image**: `postgres:16-alpine`

### `make db-down`
**Stop test database**

```bash
make db-down
```

Stops and removes the test PostgreSQL container.

**Note**: This will destroy all data in the test database.

## Docker Commands

### `make docker-build`
**Build Docker image**

```bash
make docker-build
```

Builds a Docker image tagged as `lazytables:latest` using the project Dockerfile.

### `make docker-dev`
**Run in Docker container**

```bash
make docker-dev
```

Starts LazyTables in a Docker container using docker-compose. Useful for testing the containerized version.

## Utility Commands

### `make clean`
**Clean build artifacts**

```bash
make clean
```

Removes all build artifacts and cleans the Cargo cache:
- Runs `cargo clean`
- Removes `target/` directory
- Clears any temporary files

### `make install-deps`
**Install development dependencies**

```bash
make install-deps
```

Installs required development tools:
- `cargo-watch` - For auto-reload during development
- `cargo-audit` - For security auditing dependencies

### `make help`
**Show available commands**

```bash
make help
```

Displays all available Makefile commands with brief descriptions.

## Direct Cargo Commands

While the Makefile provides convenient shortcuts, you can also use Cargo directly:

### Basic Commands
```bash
cargo run                    # Run debug build
cargo build                  # Build debug binary
cargo build --release       # Build release binary
cargo test                   # Run tests
cargo check                  # Check compilation without building
```

### Advanced Commands
```bash
cargo run -- --help         # Run with arguments
cargo test --test integration # Run specific test suite
cargo bench                  # Run benchmarks
cargo doc --open            # Generate and open documentation
cargo tree                  # Show dependency tree
```

### Development Tools
```bash
cargo watch -x run          # Auto-reload (requires cargo-watch)
cargo clippy                # Run linter
cargo fmt                   # Format code
cargo audit                 # Security audit (requires cargo-audit)
```

## Environment Variables

You can customize behavior with environment variables:

```bash
# Enable debug logging
RUST_LOG=debug make run

# Set custom database URL
DATABASE_URL=postgres://user:pass@localhost/db make run

# Use different config file
LAZYTABLES_CONFIG=/path/to/config.toml make run
```

## CI/CD Commands

For continuous integration, use these command sequences:

### Full CI Check
```bash
make check test
```

### Release Build
```bash
make clean build
```

### Security Audit
```bash
cargo audit
```

## Troubleshooting Commands

### Check Dependencies
```bash
cargo tree
```

### Clean Everything
```bash
make clean
cargo clean
rm -rf target/
```

### Reset Development Environment
```bash
make clean
make install-deps
make db-down
make db-up
make dev
```

## Performance Testing

### Release Build Performance
```bash
make build
time ./target/release/lazytables --help
```

### Memory Usage
```bash
# On macOS
make run &
ps -o pid,vsz,rss,comm -p $!

# On Linux  
make run &
ps -o pid,vsz,rss,comm -p $!
```

### Startup Time
```bash
time cargo run -- --help
```

This command reference should cover all your development needs. For specific use cases not covered here, check the other documentation files or ask in GitHub Discussions.