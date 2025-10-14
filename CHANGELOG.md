# Changelog

All notable changes to LazyTables will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3] - 2025-10-14

Major bug fixes, code refactoring, and user experience improvements.

### Added
- **Database disconnect functionality** - Press 'x' in Connections pane to disconnect from active database
- **Connection test abort capability** - Press Ctrl+C during connection testing to abort ongoing test
- **Enhanced schema viewing** with comprehensive metadata display including:
  - Columns with primary key indicators (🔑)
  - Indexes with type, uniqueness, and size information
  - Foreign keys with ON DELETE/UPDATE actions
  - Table constraints (CHECK, UNIQUE, etc.)
  - Table statistics (row count, sizes, vacuum/analyze timestamps)
- **Toggle between Data and Schema views** using 't' key in table viewer
- **Informative empty states** in query results pane when no data is available
- **Progressive pane enablement** - panes are only enabled when relevant data is available

### Changed
- **Major code refactoring** - Event handlers reorganized into dedicated modules for better maintainability:
  - `src/app/handlers/global.rs` - Global key bindings
  - `src/app/handlers/connections.rs` - Connection pane handlers
  - `src/app/handlers/tables.rs` - Tables pane handlers
  - `src/app/handlers/query_results.rs` - Query results handlers
  - `src/app/handlers/query_editor.rs` - SQL editor handlers
  - `src/app/handlers/sql_files.rs` - SQL files browser handlers
  - `src/app/handlers/overlays.rs` - Modal and overlay handlers
- **Improved visual feedback** for disabled panes with dimmed borders and informative messages
- **Enhanced delete confirmation modal** with better visibility using solid background colors
- **Streamlined documentation** with clearer structure and focused content
- **Schema modification philosophy** - Removed table creation and schema editing features to focus on data viewing and querying

### Fixed
- **Critical: Number key input blocked during cell editing** - Number keys 1-6 now work correctly when editing numerical fields in table cells
- **Help modal keyboard navigation** - Fixed event routing for proper help system interaction
- **SQLite connection URL format** - Corrected database connection string format for SQLite
- **Security improvements** in database adapters with better error handling

### Technical
- Reduced `src/app/mod.rs` from ~2600 lines to ~600 lines by modularizing event handlers
- Improved code organization with clear separation of concerns
- Added state refactoring planning documentation
- Enhanced error handling in database adapters (PostgreSQL, MySQL, MariaDB, SQLite)
- Removed orphaned and unimplemented code for better maintainability

### Documentation
- Updated development notes with completed work items
- Documented enhanced schema viewing capabilities
- Added design philosophy regarding schema modifications
- Improved help text accuracy across all panes
- Enhanced WIP documentation with audit checklist

## [0.2.1] - 2025-10-11

Stable release of v0.2.0-beta.1 with all async improvements and UX enhancements.

### Added
- **Vim-style command mode** in SQL query editor (`:w` to save, `:q` to quit, `:wq` to save and quit)
- **SQL autocomplete** with keyword suggestions and context-aware completions
- **Numbered pane navigation** - Press 1-6 to jump directly to any pane
- **Animated loading indicators** with timer display for connection attempts
- **Table browser with collapsible groups** for better navigation
- **Tab navigation in table viewer** using Shift+H/L to switch between open tables
- **Non-blocking database connections** with visual feedback during connection attempts
- **Async file I/O infrastructure** with timeout protection (5-second default)

### Changed
- **Fully async architecture** - All database and file operations now non-blocking
- **Unified view hierarchy system** for better state management
- **Event-driven overlay routing** for improved modal and overlay handling
- **Connection modal UX improvements** with step-by-step guidance
- **Help system** updated with numbered navigation and new features

### Fixed
- Tab key behavior in SQL editor insert mode (no longer switches panes during text editing)
- Async runtime panics in database and file operations
- Connection testing UI responsiveness (no longer freezes during testing)
- Table navigation and scrolling edge cases
- Various UI blocking operations converted to async
- Clippy warnings for CI compliance

### Technical
- Migrated all command file operations to async I/O
- Converted ConnectionStorage to fully async operations
- Refactored application initialization to async architecture
- Added comprehensive timeout protection for all I/O operations
- Improved error handling with better user feedback

### Documentation
- Updated development notes and project guidelines
- Added comprehensive UI blocking fixes documentation
- Enhanced work-in-progress planning documents

## [0.2.0-beta.1] - 2025-10-11

### Added
- **Vim-style command mode** in SQL query editor (`:w` to save, `:q` to quit, `:wq` to save and quit)
- **SQL autocomplete** with keyword suggestions and context-aware completions
- **Numbered pane navigation** - Press 1-6 to jump directly to any pane
- **Animated loading indicators** with timer display for connection attempts
- **Table browser with collapsible groups** for better navigation
- **Tab navigation in table viewer** using Shift+H/L to switch between open tables
- **Non-blocking database connections** with visual feedback during connection attempts
- **Async file I/O infrastructure** with timeout protection (5-second default)

### Changed
- **Fully async architecture** - All database and file operations now non-blocking
- **Unified view hierarchy system** for better state management
- **Event-driven overlay routing** for improved modal and overlay handling
- **Connection modal UX improvements** with step-by-step guidance
- **Help system** updated with numbered navigation and new features

### Fixed
- Tab key behavior in SQL editor insert mode (no longer switches panes during text editing)
- Async runtime panics in database and file operations
- Connection testing UI responsiveness (no longer freezes during testing)
- Table navigation and scrolling edge cases
- Various UI blocking operations converted to async

### Technical
- Migrated all command file operations to async I/O
- Converted ConnectionStorage to fully async operations
- Refactored application initialization to async architecture
- Added comprehensive timeout protection for all I/O operations
- Improved error handling with better user feedback

### Documentation
- Updated development notes and project guidelines
- Added comprehensive UI blocking fixes documentation
- Enhanced work-in-progress planning documents

## [0.1.7] - 2025-09-27

### Added
- **cargo-binstall compatibility** for fast binary installation without compilation
- Pre-built binaries for macOS (Intel & Apple Silicon) and Linux (x86_64 & aarch64)
- Enhanced GitHub Actions workflow for automated cross-platform releases

### Changed
- Fixed archive naming format for cargo-binstall compatibility
- Improved release automation with proper tar.gz packaging
- Updated installation documentation with cargo-binstall instructions

### Technical
- Version synchronization between git tags and Cargo.toml
- Cross-platform binary generation in CI/CD pipeline

## [0.1.6] - 2025-09-27

### Added
- Comprehensive README.md with complete key bindings reference
- Detailed installation guide (crates.io, source, cargo-binstall)
- Database support status documentation (PostgreSQL, MySQL, SQLite)
- Configuration directory explanations and troubleshooting guide
- User-friendly sections: tips, workflows, and common patterns
- Feature specifications document (21 planned features)
- Development roadmap with 6 implementation phases

### Changed
- Professional markdown structure with proper tables and hierarchy
- Enhanced six-pane layout documentation
- Improved context-aware help system documentation
- Clearer status indicators and display format explanations

### Documentation
- Complete overhaul focused on user experience and accessibility
- Clear technical specifications for future development
- Comprehensive key bindings for all panes and modes

## [0.1.5] - 2025-09-26

### Technical
- Version bump for cargo-binstall testing
- No functional changes

## [0.1.4] - 2025-09-24

### Added
- **Debug view** with Ctrl+B hotkey and full-screen overlay
- Real-time logging system visible in debug view
- Persistent connection management system
- Database cell editing capabilities

### Changed
- Dark theme fixes and UI improvements
- Debug view state management and controls

### Fixed
- Various UI consistency issues
- Connection state persistence

## [0.1.3] - 2025-08-15

### Added
- **Secure password management** with AES-256-GCM encryption
- Support for environment variables in connection strings
- **SQL file management** with per-database organization
- Execute SQL queries under cursor with Ctrl+Enter
- Connection-specific SQL file directories
- Single active connection enforcement
- Connection deletion with confirmation modal
- Query results displayed in table viewer tabs

### Changed
- Removed vim-style insert mode from connection modal for better UX
- Updated help text for all panes with accurate keybindings
- Added confirmation modals for destructive actions

### Fixed
- All clippy warnings and code formatting issues
- Zero compilation warnings

### Technical
- Rust-based terminal UI with Ratatui
- PostgreSQL support (MySQL, SQLite support added in later versions)
- Secure credential storage with Argon2 key derivation
- Cross-platform support (macOS, Linux)

---

## Version History

- **0.2.3** - Bug fixes, code refactoring, and UX improvements
- **0.2.1** - Stable release with async architecture and major UX improvements
- **0.2.0-beta.1** - Major async refactor and UX improvements (Beta Release)
- **0.1.7** - cargo-binstall compatibility
- **0.1.6** - Documentation excellence
- **0.1.5** - Testing release
- **0.1.4** - Debug view and connection management
- **0.1.3** - First stable release

[Unreleased]: https://github.com/yuyudhan/LazyTables/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/yuyudhan/LazyTables/compare/v0.2.1...v0.2.3
[0.2.1]: https://github.com/yuyudhan/LazyTables/compare/v0.1.7...v0.2.1
[0.2.0-beta.1]: https://github.com/yuyudhan/LazyTables/compare/v0.1.7...v0.2.0-beta.1
[0.1.7]: https://github.com/yuyudhan/LazyTables/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/yuyudhan/LazyTables/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/yuyudhan/LazyTables/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/yuyudhan/LazyTables/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/yuyudhan/LazyTables/releases/tag/v0.1.3
