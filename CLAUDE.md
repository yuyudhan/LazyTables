# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LazyTables is a terminal-based SQL database viewer and editor designed for developers who value keyboard-driven workflows. Built with Rust and featuring vim motions throughout, it provides a fast, intuitive interface for database management without leaving the terminal.

**Current Status**: Active development phase - core UI framework and SQL query editor functionality implemented. Basic navigation, connection management, split-pane layout, and contextual messaging system are working.

## Technical Stack

- **Language**: Rust (for performance, memory safety, and strong CLI tooling)
- **TUI Framework**: Ratatui (most mature and feature-rich TUI library)
- **Database Drivers**: Native Rust crates (e.g., sqlx for async connections)
- **License**: WTFPL (What The Fuck Public License)
- **Supported Platforms**: macOS, Linux (No Windows support)

## Architecture Overview

LazyTables uses a fixed six-pane layout optimized for database navigation and SQL querying:

1. **Connections Pane** (Top Left): Manage database connections
2. **Tables/Views Pane** (Middle Left): Navigate database objects  
3. **Table Details Pane** (Bottom Left): Display metadata about selected table
4. **Query Results Area** (Top Right): Display tabular query results
5. **SQL Query Editor** (Bottom Left): Write, edit, and execute SQL queries
6. **SQL Files Browser** (Bottom Right Column): Thin column to browse and load saved SQL files

The application follows vim-style navigation with multiple modes:
- **Normal Mode**: Navigation and commands (default)
- **Insert Mode**: Direct cell editing
- **Visual Mode**: Row/column selection
- **Query Mode**: SQL query composition
- **Command Mode**: Complex operations

## Development Commands

```bash
# Development
make dev              # Run with cargo watch for auto-reload
make run              # Run debug build
make build            # Build release binary
cargo run             # Run directly with cargo

# Testing & Quality
make test             # Run all tests
make lint             # Run clippy linter
make format           # Auto-format code with rustfmt
make check            # Run format check and clippy

# Database operations
make db-up            # Start test PostgreSQL in Docker
make db-down          # Stop test PostgreSQL

# Docker
make docker-build     # Build Docker image
make docker-dev       # Run in Docker container

# Cleanup
make clean            # Clean build artifacts
```

**Note**: The application requires a terminal environment to run. It will not work when piped or run in non-interactive mode.

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
- ✅ Six-pane directional navigation with Ctrl+h/j/k/l (true spatial movement) and Tab/Shift+Tab
- ✅ SQL file browser navigation (j/k to navigate, Enter to load)
- ✅ Query editor key bindings (Ctrl+S/O/N for save/refresh/new)
- ✅ Mode switching (Normal/Insert/Visual/Command/Query modes)
- ✅ Help system with '?' key
- ✅ Context-aware help messages and state indicators in all panes

## Key Features to Implement

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
- Pane navigation with `Ctrl+h/j/k/l`

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

## Repository Structure (Planned)

```
lazytables/
├── src/
│   ├── core/         # Core functionality
│   ├── ui/           # TUI components
│   ├── adapters/     # Database adapters
│   ├── plugins/      # Plugin system
│   └── themes/       # Theme engine
├── plugins/          # Built-in plugins
├── themes/           # Default themes
├── docs/             # Documentation
├── tests/            # Test suite
└── examples/         # Usage examples
```

## Development Notes

- Follow Rust best practices with proper error handling
- Use async/await patterns for database operations
- Implement comprehensive logging for debugging
- Design for horizontal scaling and plugin extensibility
- Maintain vim-style consistency throughout the interface
- Prioritize performance and memory efficiency
- Use proper security practices for credential management

## Testing Strategy

- Unit tests for all core modules
- Integration tests for database adapters
- TUI testing for user interface components
- Performance benchmarks for large datasets
- Security testing for credential handling