# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LazyTables is a terminal-based SQL database viewer and editor designed for developers who value keyboard-driven workflows. Built with Rust and featuring vim motions throughout, it provides a fast, intuitive interface for database management without leaving the terminal.

**Current Status**: Active development phase - core UI framework and SQL query editor functionality implemented. Basic navigation, connection management, split-pane layout, and contextual messaging system are working.

## Technical Stack

- **Language**: Rust 1.70+ (for performance, memory safety, and strong CLI tooling)
- **TUI Framework**: Ratatui 0.29 + Crossterm 0.28 (terminal rendering and event handling)
- **Async Runtime**: Tokio 1.41 (async database operations and event handling)
- **Database**: SQLx 0.8 (async database driver supporting Postgres, MySQL, SQLite)
- **Security**: AES-GCM 0.10 + Argon2 0.5 (credential encryption and key derivation)
- **Logging**: Tracing 0.1 + Tracing-subscriber 0.3 (structured logging)
- **Error Handling**: color-eyre 0.6, thiserror 2.0, anyhow 1.0
- **CLI Parsing**: Clap 4.5 (command-line argument parsing)
- **Serialization**: Serde 1.0 + serde_json 1.0 + TOML 0.8 (config and data)
- **License**: WTFPL (What The Fuck Public License)
- **Supported Platforms**: macOS, Linux (No Windows support)

## Architecture Overview

LazyTables uses a fixed six-pane layout optimized for database navigation and SQL querying. Each pane is numbered for direct keyboard access:

1. **[1] Connections Pane** (Top Left): Manage database connections
2. **[2] Tables/Views Pane** (Middle Left): Navigate database objects
3. **[3] Table Details Pane** (Bottom Left): Display metadata about selected table
4. **[4] Query Results Area** (Top Right): Display tabular query results
5. **[5] SQL Query Editor** (Bottom Left): Write, edit, and execute SQL queries
6. **[6] SQL Files Browser** (Bottom Right Column): Thin column to browse and load saved SQL files

**Navigation**: Press 1-6 to jump directly to any pane, or use Tab/Shift+Tab to cycle through panes.

The application follows vim-style navigation with multiple modes:
- **Normal Mode**: Navigation and commands (default)
- **Insert Mode**: Direct cell editing
- **Visual Mode**: Row/column selection
- **Query Mode**: SQL query composition
- **Command Mode**: Complex operations

## Development Commands

```bash
# Development
make dev              # Run with cargo watch for auto-reload (requires cargo-watch)
make run              # Run debug build
make run-debug        # Run with debug logging enabled
make build            # Build release binary
cargo run             # Run directly with cargo

# Testing & Quality
make test             # Run all tests
cargo test            # Run all tests (same as make test)
cargo test test_name  # Run specific test by name
cargo test --package lazytables --lib database::postgres::tests  # Run tests in specific module
make lint             # Run clippy linter with warnings as errors
make format           # Auto-format code with rustfmt
make format-check     # Check formatting without modifying files
make check            # Run format check and clippy (CI-friendly)

# Installation
make install          # Install LazyTables via cargo
make uninstall        # Remove LazyTables from system
make install-deps     # Install development dependencies (cargo-watch, cargo-audit)

# Cleanup
make clean            # Clean build artifacts and target/ directory
```

**Important notes:**
- The application requires a terminal environment to run. It will not work when piped or run in non-interactive mode.
- Use `make dev` for development with auto-reload (requires `cargo install cargo-watch`)
- Tests may require database connections; some can be run with environment variables for connection strings

## Database Support Roadmap

### Phase 0 - Foundation (MVP)
- PostgreSQL (full support with all core features)

### Phase 1 - Core Databases
- MySQL, MariaDB, SQLite
- Oracle, DB2, ClickHouse
- Redis (key-value store support)

### Phase 2 - Extended Support
- MongoDB (document database support)
- Additional databases based on community demand

## Implemented Features (Current)

### Core UI Framework
- ✅ Six-pane layout with proper focus management
- ✅ Vim-style navigation with h/j/k/l movement
- ✅ Modal system for help and command entry
- ✅ Theme support with default dark theme
- ✅ Status bar with mode indicators
- ✅ Contextual messaging system with state-aware guidance

### SQL Query Editor & File Management
- ✅ Horizontal split layout: query editor with thin files column on right
- ✅ SQL files browser with current file indicator
- ✅ Basic text editing with cursor navigation
- ✅ File management (save, load, create new queries)
- ✅ Cursor-based SQL statement execution (Ctrl+Enter)
- ✅ Query mode for full-screen SQL editing
- ✅ Directory structure creation on startup

### Connection Management
- ✅ Two-step connection creation process (database type → configuration)
- ✅ Connection string support with auto-parsing for PostgreSQL, MySQL, SQLite
- ✅ Individual field input with database-specific defaults
- ✅ Dynamic UI based on selected database type
- ✅ Connection storage in ~/.lazytables directory
- ✅ Elegant modal with step-by-step guidance
- ✅ Display format showing "name (database_type)"
- ✅ Secure credential storage (encrypted)

### Navigation & Key Bindings
- ✅ Six-pane numbered navigation with number keys 1-6 for direct pane access
- ✅ Tab/Shift+Tab for cycling through panes
- ✅ Direct key bindings per pane (a=add, e=edit, d=delete, etc.)
- ✅ SQL file browser navigation (j/k to navigate, Enter to load)
- ✅ Query editor with vim-style insert mode (ONLY pane with insert mode)
- ✅ Help system with '?' key
- ✅ Context-aware help messages and state indicators in all panes

### Enhanced Schema Viewing
- ✅ Toggle between Data and Schema view with 't' key in Table Viewer
- ✅ Comprehensive read-only schema display with 5 sections:
  - **Columns**: Name, type, nullable status, primary key indicators (🔑)
  - **Indexes**: Index name, type (BTREE/HASH/etc), unique/primary markers, columns, size
  - **Foreign Keys**: Constraint name, column relationships (col → table(col)), ON DELETE/UPDATE actions
  - **Constraints**: CHECK, UNIQUE, and other constraints with definitions
  - **Table Statistics**: Row count, table size, index size, total size, last vacuum/analyze timestamps, owner
- ✅ Scrollable navigation with j/k, gg/G, Ctrl+d/u
- ✅ Color-coded information using theme colors for better readability
- ✅ Graceful handling when metadata is unavailable

## Key Features to Implement

### UI Pane Update System (In Progress)
- Event-driven pane synchronization with <100ms latency
- Update queue with priority and deduplication
- Virtual scrolling for >1000 rows, pagination at 10K
- Loading state indicators per pane
- Error display in status bar with inline context
- Manual refresh: Ctrl+R (focused), Shift+Ctrl+R (all)
- Dependency tracking between panes
- State preservation during updates (cursor, scroll)

### Core Functionality
- Connection management with secure credential storage
- Lazy loading for large datasets with virtual scrolling
- In-place cell editing with vim-style insert mode
- Full-screen query editor with syntax highlighting
- Plugin system for extensibility

### Navigation System
- Vim motions throughout (`h/j/k/l`, `gg/G`, `0/$`, etc.)
- Leader key commands with `Space` as leader
- Context-sensitive help system
- Pane navigation with number keys `1-6` for direct access
- Tab/Shift+Tab for cycling through panes

### Input Mode System (Simplified Architecture)
- **Direct key bindings in all panes** - no insert mode needed except Query Editor
- **SQL Query Editor** - ONLY pane with vim-style insert mode:
  - Press 'i', 'a', 'o', or 'O' to enter insert mode
  - Press ESC to exit insert mode and return to normal mode
  - All vim motions available in normal mode (h/j/k/l, w/b/e, 0/$, gg/G, etc.)
  - Visual feedback shows current mode ([INSERT] or [NORMAL])
- **Forms and Modals** (Connection creation/editing):
  - Direct typing in all text fields - NO insert mode required
  - Tab/Shift+Tab to navigate between fields
  - ESC to cancel and close modal
  - Arrow keys for dropdown navigation
- **All Other Panes**: Direct action keys (a=add, e=edit, d=delete, /=search, etc.)

### Per-Pane Key Bindings Reference

#### [1] Connections Pane (Direct Bindings)
- **a** → Add new connection
- **e** → Edit selected connection
- **d** → Delete connection (with confirmation)
- **Enter** → Connect to selected database
- **r** → Refresh connection list
- **/** → Enter search mode
- **j/k** or **↑/↓** → Navigate connections

#### [2] Tables Pane (Direct Bindings)
- **Enter** → Open table for viewing
- **r** → Refresh tables list
- **/** → Enter search mode
- **j/k** or **↑/↓** → Navigate tables
- **g** → First press of gg (jump to top)
- **G** → Jump to last table

#### [3] Details Pane (Read-Only, Navigation Only)
- **j/k** or **↑/↓** → Scroll up/down
- **Ctrl+d/u** → Page down/up
- **gg** → Jump to top
- **G** → Jump to bottom

#### [4] Query Results / Table Viewer (Has Edit Mode)
- **i** or **Enter** → Start editing current cell
- **d** → Delete current row (with confirmation)
- **/** → Enter search mode
- **h/j/k/l** or **arrows** → Navigate cells
- **Ctrl+r** → Refresh table data
- **ESC** → Exit edit/search mode

#### [5] SQL Query Editor (VIM-STYLE - Only Pane with Insert Mode)
**Normal Mode** (default):
- **i** → Insert mode at cursor
- **a** → Insert mode after cursor
- **o** → New line below + insert mode
- **O** → New line above + insert mode
- **h/j/k/l** → Navigate cursor
- **w/b/e** → Word motions
- **0/$** → Line start/end
- **gg/G** → File start/end
- **Ctrl+Enter** → Execute query at cursor

**Insert Mode**:
- **ESC** → Return to normal mode
- **Type normally** → Edit text
- **Ctrl+Enter** → Execute query

#### [6] SQL Files Pane (Direct Bindings)
- **Enter** → Load selected SQL file
- **n** → Create new file
- **r** → Rename file
- **d** → Delete file (with confirmation)
- **/** → Enter search mode
- **j/k** or **↑/↓** → Navigate files

### Global Key Bindings (Work Everywhere)
- **1-6** → Jump directly to pane by number
- **Tab** → Cycle to next pane
- **Shift+Tab** → Cycle to previous pane
- **?** → Toggle help overlay
- **q** → Quit application (with confirmation)
- **Ctrl+B** → Toggle debug view

### Performance Requirements
- Startup time: < 100ms
- Query execution: Display first results within 50ms
- Scrolling: 60 FPS smooth scrolling
- Memory usage: < 50MB base

## Configuration Structure

```
~/.config/lazytables/
└── config.toml       # Main configuration file

~/.lazytables/
├── README.md         # Data directory documentation
├── config.toml       # Legacy config location (deprecated)
├── connections.json  # Database connection definitions
├── connections/      # Individual connection files
├── sql_files/        # Saved SQL query files
│   └── sample_queries.sql  # Sample SQL queries
├── logs/             # Application log files
│   └── lazytables.log
└── backups/          # Backup files
```

## Architecture Patterns

### Main Application Flow
The application follows an event-driven architecture:
1. **main.rs**: Entry point that initializes logging, config, terminal, and creates the App instance
2. **App::run()** (src/app/mod.rs): Main event loop that draws UI and handles events
3. Event flow: `EventHandler` → `App::handle_event()` → Command execution → State update → UI redraw

### State Management
State is split into two main structures (src/state/):
- **DatabaseState** (database.rs): Connection info, tables, metadata, query results
- **UIState** (ui.rs): Focused pane, selection indices, scroll offsets, modal states

Both are aggregated in **AppState** (src/app/state.rs) along with UI component states like:
- `query_editor: QueryEditor` - SQL editor component state
- `table_viewer_state: TableViewerState` - Table viewer state with tabs
- `connection_modal_state: ConnectionModalState` - Connection creation/editing
- `toast_manager: ToastManager` - Notification system

### Command Pattern
Commands are implemented using the Command pattern (src/commands/):
- **CommandId** enum defines all available commands
- **CommandContext** provides access to state and config
- **CommandResult** indicates success, error, or requires confirmation
- Commands are organized in modules: basic, connection, editing, navigation, query

### Database Abstraction
Database drivers follow the adapter pattern (src/database/):
- **Connection** trait defines the interface all database adapters must implement
- **AdapterFactory** creates the appropriate adapter based on DatabaseType
- Adapters: PostgreSQLAdapter, MySQLAdapter, SQLiteAdapter
- **ConnectionManager** handles connection pooling and lifecycle

Key types:
- **TableMetadata**: Comprehensive table information (columns, indexes, FKs, sizes)
- **DatabaseObject**: Represents tables, views, materialized views with schema info
- **QueryHistoryManager**: Tracks query execution history per connection

### UI Component Architecture
The UI is rendered through **UI::draw()** (src/ui/mod.rs) which delegates to specialized components:

Core panes:
- **Connections Pane**: Shows connection list with status indicators (src/ui/mod.rs:295)
- **Tables Pane**: Database-adaptive table/view browser (src/ui/components/tables_pane.rs)
- **Details Pane**: Comprehensive table metadata display (src/ui/mod.rs:446)
- **Query Results**: Table viewer with tabs (src/ui/components/table_viewer.rs)
- **SQL Files**: File browser for saved queries (src/ui/mod.rs:939)
- **Query Editor**: Syntax-highlighted SQL editor (src/ui/components/query_editor.rs)

Modals and overlays:
- **Connection Modal**: Two-step connection creation (src/ui/components/connection_modal.rs)
- **Help System**: Context-aware help overlay (src/ui/help.rs)
- **Debug View**: Live log viewer (src/ui/components/debug_view.rs)

### Key Architectural Decisions

1. **Async Database Operations**: All database operations use `sqlx` with `tokio` runtime for non-blocking I/O
2. **Theme System**: Colors and styles loaded from TOML files (src/themes/, src/ui/theme/)
3. **Secure Credentials**: Passwords encrypted using AES-GCM with Argon2 key derivation (src/security/)
4. **Modal System**: Modals are overlays rendered after main UI with dimmed background
5. **Event-Driven Updates**: State changes trigger UI redraws, no polling required

### Common Development Patterns

**Adding a new database adapter:**
1. Create new file in `src/database/` (e.g., `oracle.rs`)
2. Implement the `Connection` trait with all required methods
3. Add variant to `DatabaseType` enum in `src/database/connection.rs`
4. Update `AdapterFactory::create_adapter()` in `src/database/factory.rs`

**Adding a new UI component:**
1. Create component module in `src/ui/components/`
2. Define component state structure
3. Implement `render()` function taking `Frame`, state, area, and theme
4. Add component state to `AppState` if stateful
5. Call render function from `UI::draw()` method

**Adding a new command:**
1. Add variant to `CommandId` enum in `src/commands/mod.rs`
2. Implement command logic in appropriate module (basic, connection, etc.)
3. Add key binding in `App::handle_key_event()` in `src/app/mod.rs`
4. Update help system text in `src/ui/help.rs` if user-facing

## Testing Strategy

Run tests with: `make test` or `cargo test --all-features`

Test organization:
- Unit tests inline with module code using `#[cfg(test)]`
- Integration tests for database adapters in module tests
- No separate test directories to avoid clutter
- Use `tempfile` crate for filesystem tests
- Use `pretty_assertions` for better test failure output

**Important testing guidelines:**
- Never corrupt the terminal view with debug outputs during tests
- Don't create unnecessary extra test file clutter
- Mock database connections when possible to avoid requiring live databases
- Use `#[tokio::test]` for async tests

## Development Notes

- All database operations are async - use `.await` and `#[tokio::main]` or `#[tokio::test]`
- Logging uses `tracing` crate - use `tracing::info!()`, `tracing::error!()`, etc.
- Error handling uses `thiserror` for custom errors, `anyhow` for application errors
- Terminal state must be restored on panic - handled by `terminal::install_panic_hook()`
- Never use `println!` or `dbg!` - they corrupt the TUI; use `tracing::debug!()` instead
- Configuration files use TOML format (see src/config/mod.rs)
- SQL files are stored per-connection in `~/.lazytables/sql_files/<connection_name>/`
- Always keep the help area up to date whenever we change the any of the keybindings.
- Always use run cargo run, cargo check or cargo build commands to see if there are any errors and correct them, code should be clean always.
- **IMPORTANT ARCHITECTURE NOTE**: The app uses a simplified mode system:
  - NO global Mode enum - removed for simplicity
  - Each pane has direct key bindings (a/e/d for actions, j/k for navigation, / for search)
  - ONLY the SQL Query Editor has vim-style insert mode (managed by QueryEditor component)
  - Forms/modals accept direct typing - NO insert mode required
  - Key events are routed per-pane via dedicated handler methods
- When implementing on one functionality if you think you are writing code that might break another functionality you should double check what will break, avoid or ensure the other functionlity or functionalities are working.
- **SCHEMA MODIFICATION PHILOSOPHY**: LazyTables focuses on viewing and querying data, NOT schema editing:
  - Table creation and schema editing features have been intentionally removed (as of 2025-10)
  - Most users only need to view schema information, not modify it
  - Power users who need schema modifications should use SQL directly via the Query Editor
  - Schema view is read-only and comprehensive, showing columns, indexes, foreign keys, constraints, and statistics
  - This design decision reduces complexity and aligns with the tool's core mission as a data viewer/query tool
  - Keep the markdown content crisp and clean. Don't bloat it with random verbiage. Ensure clarity and focus on the user.
- When releasing new versions create the changelog and release notes etc properly for that version.
- Always run the @scripts/create-release-files.sh to create the release files when creating the builds.