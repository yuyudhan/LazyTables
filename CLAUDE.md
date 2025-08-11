# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LazyTables is a terminal-based SQL database viewer and editor designed for developers who value keyboard-driven workflows. Built with Rust and featuring vim motions throughout, it provides a fast, intuitive interface for database management without leaving the terminal.

**Current Status**: Early development phase - only PRD documentation exists. No implementation has begun.

## Technical Stack

- **Language**: Rust (for performance, memory safety, and strong CLI tooling)
- **TUI Framework**: Ratatui (most mature and feature-rich TUI library)
- **Database Drivers**: Native Rust crates (e.g., sqlx for async connections)
- **License**: WTFPL (What The Fuck Public License)
- **Supported Platforms**: macOS, Linux (No Windows support)

## Architecture Overview

LazyTables will use a fixed four-pane layout optimized for database navigation:

1. **Connections Pane** (Top Left): Manage database connections
2. **Tables/Views Pane** (Middle Left): Navigate database objects  
3. **Table Details Pane** (Bottom Left): Display metadata about selected table
4. **Main Content Area** (Right): Primary workspace for viewing/editing data

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

## Configuration Structure (Future)

```
~/.config/lazytables/
├── config.toml       # Main configuration
├── plugins/          # Plugin directory
├── themes/           # Theme files
└── connections.toml  # Saved connections (encrypted)
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