# Tech Stack

This is the **DEFINITIVE technology selection** for the entire LazyTables project. All development must use these exact versions and technologies.

## Technology Stack Table

| Category | Technology | Version | Purpose | Rationale |
|----------|------------|---------|----------|-----------|
| Core Language | Rust | 1.75+ | System programming, memory safety, performance | Memory safety critical for TUI performance, excellent async support, strong typing prevents runtime errors |
| TUI Framework | Ratatui | 0.25+ | Terminal user interface rendering | Most mature Rust TUI library, active development, excellent widget ecosystem |
| Async Runtime | Tokio | 1.35+ | Async database operations, non-blocking I/O | Industry standard for Rust async, required for responsive TUI during database operations |
| Terminal Abstraction | Crossterm | 0.27+ | Cross-platform terminal control | Ratatui dependency, handles terminal differences across platforms |
| PostgreSQL Driver | SQLx | 0.7+ | Async PostgreSQL connectivity | Compile-time checked queries, excellent async support, connection pooling |
| MySQL Driver | SQLx | 0.7+ | Async MySQL/MariaDB connectivity | Same interface as PostgreSQL driver, maintains consistency across SQL databases |
| SQLite Driver | SQLx | 0.7+ | Async SQLite file database access | Local file support, same SQLx interface for consistency |
| Redis Driver | redis-rs | 0.24+ | Async Redis key-value operations | Most mature Redis client for Rust, async/await support |
| Configuration | serde + toml | 1.0+ / 0.8+ | Configuration file parsing and serialization | Human-readable config format, strong typing for validation |
| Error Handling | thiserror + anyhow | 1.0+ / 1.0+ | Structured error types and error propagation | Ergonomic error handling essential for database operations |
| Logging | tracing + tracing-subscriber | 0.1+ / 0.3+ | Structured logging and observability | Async-aware logging, essential for debugging TUI and database issues |
| Security | keyring + sha2 | 2.0+ / 0.10+ | Credential storage and hashing | OS-native credential storage, secure password handling |
| Testing Framework | tokio-test + sqlx-test | 0.4+ / 0.7+ | Async testing and database test utilities | Async test support, database integration testing |
| Build Tool | Cargo | 1.75+ | Rust package management and building | Native Rust tooling, workspace support for modular architecture |
| CLI Parsing | clap | 4.0+ | Command line argument processing | Feature-rich CLI parsing for debug options and config paths |
| File Watching | notify | 6.0+ | SQL file change detection | Auto-reload SQL files when modified externally |
| Syntax Highlighting | syntect | 5.0+ | SQL syntax highlighting in terminal | Terminal-compatible syntax highlighting for query editor |
| Data Export | csv + serde_json | 1.3+ / 1.0+ | CSV and JSON export functionality | Standard serialization formats for query result export |
| Development Tools | cargo-watch | 8.0+ | Auto-rebuild on file changes | Essential for fast development iteration with TUI hot-reload |
| Benchmarking | criterion | 0.5+ | Performance testing and regression detection | Validate startup time and scrolling performance requirements |
| Dependency Management | cargo-deny | 0.14+ | License compliance and security scanning | Ensure all dependencies meet security and legal requirements |

## Single Binary Strategy

**Feature Flag Configuration:**
```toml
[features]
default = ["postgresql"]
postgresql = ["sqlx/postgres", "sqlx/runtime-tokio-rustls"]
mysql = ["sqlx/mysql", "sqlx/runtime-tokio-rustls"]
sqlite = ["sqlx/sqlite", "sqlx/runtime-tokio-rustls"]
redis = ["redis/tokio-comp"]
all-databases = ["postgresql", "mysql", "sqlite", "redis"]
syntax-highlighting = ["syntect"]
file-watching = ["notify"]
full = ["all-databases", "syntax-highlighting", "file-watching"]
```

**Build Optimization:**
- **Custom Builds**: `cargo install lazytables --features mysql,sqlite` (only needed databases)
- **Full Build**: `cargo install lazytables --features full` (all capabilities)
- **Development**: `cargo watch -x "run --features full"` (fast iteration with hot-reload)

**Installation Simplicity:**
```bash
# Default PostgreSQL support
cargo install lazytables

# All databases
cargo install lazytables --features all-databases

# Development setup
cargo install cargo-watch
make dev  # Uses cargo-watch for auto-reload
```
