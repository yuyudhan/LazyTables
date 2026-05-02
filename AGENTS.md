# AGENTS.md - AI Coding Agent Reference

## Critical Rules

1. **Code MUST compile** - Run `cargo check` or `cargo build` after every change. Never leave the codebase in a broken state.
2. **Clean logs are CRITICAL** - Run `just lint` after changes. Resolve all clippy warnings/errors properly — don't suppress them.
3. **NO stubbed code** - Write complete implementations unless explicitly asked for a stub.
4. **Never use `println!` or `dbg!`** - They corrupt the TUI. Use `tracing::debug!()`, `tracing::info!()`, etc.
5. **500-line limit** - Refactor files exceeding this into modules.
6. **No hardcoded paths** - Use `$HOME`, env vars, `dirs` crate, or relative paths.
7. **No secrets in code** - Never commit API keys, tokens, or credentials.
8. **Terminal must be restored** - Any panic or error path must restore terminal state. Use `terminal::install_panic_hook()`.
9. **Don't break other features** - Before implementing, check what else might break. Verify adjacent functionality still works.
10. **Keep help in sync** - Update `src/ui/help.rs` whenever key bindings change.

## Multi-Agent Workflow

When multiple AI agents work on this codebase simultaneously:

1. **Own your changes** - Focus exclusively on warnings and errors from YOUR code changes. If you see errors in files you didn't touch, ignore them — another agent is handling those.

2. **Quality standards** - Before considering your task complete:
   - Code compiles cleanly (`cargo check`)
   - Clippy passes (`just lint`)
   - Your changes are logically correct, not just syntactically valid
   - Related tests pass (`cargo test`)

3. **Stay in your lane** - If you see warnings/errors from files you didn't modify, leave them alone.

4. **Coordinate on shared files** - Be cautious when modifying shared files like `mod.rs`, `state.rs`, or `app/mod.rs`. Make surgical changes and verify you didn't break existing functionality.

5. **NO git worktrees without explicit permission** - Do not use `isolation: "worktree"` unless the user explicitly asks for it.

## Documentation & Libraries

Always use context7 MCP tools when you need code generation, setup steps, or library/API documentation. Use context7 to resolve library IDs and fetch up-to-date docs — especially for Ratatui, SQLx, Crossterm, and Tokio.

## Build/Test/Run Commands

```bash
# Development
just dev              # Run with cargo watch for auto-reload (requires cargo-watch)
just run              # Run debug build
just run-debug        # Run with debug logging enabled
just build            # Build release binary
cargo run             # Run directly with cargo
cargo check           # Fast compilation check (no binary output)

# Testing & Quality
just test             # Run all tests
cargo test            # Run all tests (same as just test)
cargo test test_name  # Run specific test by name
cargo test test_name -- --nocapture  # Run with output (for debugging)
cargo test --package lazytables --lib database::postgres::tests  # Run module tests
just lint             # Run clippy linter with warnings as errors
cargo clippy --all-targets --all-features -- -D warnings  # Full clippy
just format           # Auto-format code with rustfmt
just format-check     # Check formatting without modifying files
just check            # Run format check and clippy (CI-friendly)
just quality          # Full quality check (format, lint, test, build)
just quick-lint       # Quick check (format + lint only)
just fix              # Auto-fix formatting and lint issues
./scripts/quality.sh  # Direct quality script (supports --fix, --quick)

# Installation
just install          # Install LazyTables via cargo
just install-deps     # Install development dependencies (cargo-watch, cargo-audit)

# Diagnostics
just doctor           # Check that required tools are installed

# Cleanup
just clean            # Clean build artifacts and target/ directory
```

**Important notes:**
- The application requires a terminal environment. It will not work when piped or run in non-interactive mode.
- Use `just dev` for development with auto-reload (requires `cargo install cargo-watch`)
- Tests may require database connections; some need environment variables for connection strings

## Code Style - Rust

### Imports

Group: std → external crates → `crate::`

```rust
// FilePath: src/module/file.rs

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::config::Config;
use crate::core::error::{LazyTablesError, Result};
```

### Naming & Types

- **Functions/variables**: `snake_case` | **Types/structs/enums**: `PascalCase` | **Constants**: `SCREAMING_SNAKE_CASE`
- **Files/modules**: `snake_case` (e.g., `connection_manager.rs`)
- Explicit types on public APIs

### Error Handling

```rust
use crate::core::error::{LazyTablesError, Result};

// Use thiserror for custom errors in src/core/error.rs
// Use anyhow for application-level errors
// Use color_eyre for main error handling
// Return Result<T> (alias for std::result::Result<T, LazyTablesError>)
// Provide user-friendly error messages with recovery suggestions
```

### Logging

#### When to Use Each Level

**Use DEBUG for:**
- Internal processing steps and intermediate results
- Database query details and execution flow
- Input validation results
- Key event routing and state transitions
- Performance timing for sub-operations

**Use INFO for:**
- Connection established/disconnected
- Query execution completed
- File operations (save, load, create)
- Significant state changes
- User-initiated action outcomes

**Use WARN for:**
- Recoverable errors (retry succeeded, fallback used)
- Deprecated feature usage
- Performance degradation (slow queries)
- Resource limits approaching

**Use ERROR for:**
- Connection failures
- Query execution failures
- File I/O errors
- Unrecoverable errors requiring user attention

```rust
// DEBUG: Internal detail
tracing::debug!("Routing key event to pane {}", pane_id);

// INFO: User-facing outcome
tracing::info!("Connected to database: {}", connection_name);

// WARN: Recoverable issue
tracing::warn!("Connection pool near capacity: {}/{}", current, max);

// ERROR: Operation failure
tracing::error!("Failed to execute query: {}", error);
```

### Async Patterns

- All database operations must be async
- Use `#[tokio::main]` for main, `#[tokio::test]` for tests
- Use `async_trait` for async trait methods
- Prefer `tokio::sync::mpsc` for channels

### Struct Patterns

- Builder pattern with `with_*` methods for optional configuration
- Implement `Default` where appropriate
- Use `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields

### Module Organization (500-line limit per file)

- Public re-exports in `mod.rs`: `pub use submodule::Type;`
- Use `prelude` module for commonly used types
- Keep handlers in dedicated `handlers/` subdirectory

## Architecture

```
src/
    main.rs             # Entry point: logging, config, terminal init
    app/                # Application core
        mod.rs          # Main event loop (App::run), event handling
        state.rs        # AppState aggregating all state
        handlers/       # Per-pane key event handlers
    state/              # State management
        database.rs     # DatabaseState: connections, tables, metadata, results
        ui.rs           # UIState: focus, selections, scroll, modals
    database/           # Database abstraction layer
        connection.rs   # Connection trait, DatabaseType enum, ConnectionConfig
        connection_manager.rs  # Connection pooling and lifecycle
        factory.rs      # AdapterFactory
        postgres.rs     # PostgreSQL adapter
        mysql.rs        # MySQL adapter
        sqlite.rs       # SQLite adapter
    ui/                 # UI rendering
        mod.rs          # UI::draw() - main render dispatch
        components/     # Reusable UI components
            connection_modal.rs  # Connection creation/editing modal
            query_editor.rs      # SQL editor with vim modes
            table_viewer.rs      # Table data/schema viewer
            tables_pane.rs       # Database object browser
            debug_view.rs        # Live log viewer
        help.rs         # Context-aware help overlay
        theme/          # Theme system
    commands/           # Command pattern implementation
        mod.rs          # CommandId enum, CommandContext, CommandResult
    config/             # Configuration (TOML)
    security/           # AES-GCM encryption, Argon2 key derivation
    core/               # Core types
        error.rs        # LazyTablesError, Result type alias
```

### Main Application Flow

1. **main.rs**: Initialize logging, config, terminal, create App instance
2. **App::run()**: Main event loop — draw UI, handle events
3. Event flow: `EventHandler` → `App::handle_event()` → per-pane handler → state update → UI redraw

### Key Architectural Decisions

1. **Simplified mode system** — NO global Mode enum. Each pane has direct key bindings. ONLY the SQL Query Editor has vim-style insert mode (managed by QueryEditor component). Forms/modals accept direct typing.
2. **Async database operations** — All DB ops use `sqlx` with `tokio` runtime for non-blocking I/O.
3. **Theme system** — Colors and styles loaded from TOML files (`src/themes/`, `src/ui/theme/`).
4. **Secure credentials** — Passwords encrypted using AES-GCM with Argon2 key derivation (`src/security/`).
5. **Event-driven updates** — State changes trigger UI redraws, no polling.
6. **Schema is read-only** — LazyTables focuses on viewing and querying data, NOT schema editing. Power users use the Query Editor for DDL.

## Common Workflows

### Adding a New Database Adapter

1. Create new file in `src/database/` (e.g., `oracle.rs`)
2. Implement the `Connection` trait with all required methods
3. Add variant to `DatabaseType` enum in `src/database/connection.rs`
4. Update `AdapterFactory::create_adapter()` in `src/database/factory.rs`
5. Run `cargo check` to verify compilation
6. Add tests in the module

### Adding a New UI Component

1. Create component module in `src/ui/components/`
2. Define component state structure
3. Implement `render()` function taking `Frame`, state, area, and theme
4. Add component state to `AppState` if stateful
5. Call render function from `UI::draw()` method

### Adding a New Command

1. Add variant to `CommandId` enum in `src/commands/mod.rs`
2. Implement command logic in appropriate module (basic, connection, etc.)
3. Add key binding in `App::handle_key_event()` in `src/app/mod.rs`
4. Update help system text in `src/ui/help.rs` if user-facing

### Debugging Issues

1. Run `just run-debug` to enable debug logging
2. Check logs at `~/.lazytables/logs/lazytables.log`
3. Use `Ctrl+B` in-app to toggle debug view
4. Never use `println!` — it corrupts the TUI

## Testing Strategy

Run tests with: `just test` or `cargo test`

- Unit tests inline with module code using `#[cfg(test)]`
- Integration tests for database adapters in module tests
- No separate test directories to avoid clutter
- Use `tempfile` crate for filesystem tests
- Use `pretty_assertions` for better test failure output
- Use `#[tokio::test]` for async tests
- Mock database connections when possible to avoid requiring live databases
- Never corrupt the terminal with debug outputs during tests

## Configuration Structure

```
~/.config/lazytables/
└── config.toml          # Main configuration file

~/.lazytables/
├── connections.json     # Database connection definitions
├── connections/         # Individual connection files
├── sql_files/           # Saved SQL query files (per-connection subdirectories)
├── logs/                # Application log files
└── backups/             # Backup files
```

## Before Committing

1. Code compiles: `cargo check` shows no errors
2. Linter passes: `just lint` (clippy with `-D warnings`)
3. Formatting clean: `just format-check`
4. No `println!`, `dbg!`, or debug statements left behind
5. Help text updated if key bindings changed
6. Tests pass: `just test`

Or run `just quality` to check all of the above in one step.

## Releasing

- Create changelog and release notes for the version
- Run `scripts/create-release-files.sh` to create release archives
- See script for platform-specific build options
