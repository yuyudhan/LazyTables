# 004 - Project Structure

This document explains the organization of the LazyTables codebase and the purpose of each directory and file.

## Repository Overview

```
LazyTables/
├── docs/                    # Documentation
│   ├── dev/                # Developer documentation (this directory)
│   ├── feature-specs.md    # Feature specifications
│   └── rough.md           # Development notes
├── src/                    # Source code
│   ├── app/               # Application state and logic
│   ├── cli.rs             # Command line interface
│   ├── config/            # Configuration management  
│   ├── core/              # Core functionality
│   ├── database/          # Database connections and adapters
│   ├── event/             # Event handling system
│   ├── terminal.rs        # Terminal setup and management
│   ├── themes/            # Theme engine and color schemes
│   ├── ui/                # User interface components
│   ├── lib.rs             # Library root
│   ├── main.rs            # Application entry point
│   └── logging.rs         # Logging configuration
├── tests/                  # Integration tests
├── examples/              # Usage examples
├── target/                # Build artifacts (gitignored)
├── docker/                # Docker-related files
├── Cargo.toml            # Rust project configuration
├── Cargo.lock            # Dependency lock file
├── Makefile              # Development commands
├── Dockerfile            # Container definition
├── docker-compose.yml    # Multi-container setup
├── CLAUDE.md             # AI assistant instructions
├── PRD.md                # Product Requirements Document
├── README.md             # User-facing documentation
├── LICENSE               # WTFPL license
└── MORE_NOTES.md         # Additional development notes
```

## Source Code Structure (`src/`)

### Application Layer (`app/`)

```
src/app/
├── mod.rs              # Module exports
└── state.rs            # Application state management
```

**Purpose**: Central application state and business logic
- `state.rs`: Manages the global application state, including current mode, active pane, database connections, and user selections

### CLI Layer (`cli.rs`)

**Purpose**: Command line argument parsing and application entry point configuration
- Defines command line options and flags
- Handles help text and version information
- Routes CLI arguments to appropriate application functions

### Configuration (`config/`)

```
src/config/
└── mod.rs              # Configuration loading and management
```

**Purpose**: Application configuration system
- Loads settings from multiple sources (CLI, env vars, config files)
- Manages user preferences and database connection settings
- Handles configuration validation and defaults

### Core Functionality (`core/`)

```
src/core/
├── mod.rs              # Core module exports
└── error.rs            # Error types and handling
```

**Purpose**: Fundamental types and error handling
- `error.rs`: Defines all error types used throughout the application
- Common utilities and shared functionality
- Type definitions used across multiple modules

### Database Layer (`database/`)

```
src/database/
├── mod.rs              # Database module exports
├── connection.rs       # Connection management
└── postgres.rs         # PostgreSQL adapter
```

**Purpose**: Database connectivity and operations
- `connection.rs`: Connection pooling, health checks, and lifecycle management
- `postgres.rs`: PostgreSQL-specific implementation of database operations
- Future: Additional adapter files for MySQL, SQLite, etc.

### Event System (`event/`)

```
src/event/
└── mod.rs              # Event definitions and handling
```

**Purpose**: Application event system
- Defines event types (keyboard, database, UI events)
- Event routing and processing
- Event-driven architecture support

### Terminal Management (`terminal.rs`)

**Purpose**: Terminal initialization and management
- Sets up terminal for TUI operation
- Handles terminal restoration on exit
- Manages terminal capabilities and features

### Themes (`themes/`)

```
src/themes/
└── mod.rs              # Theme definitions and management
```

**Purpose**: Visual theming system
- Color scheme definitions
- Theme loading and switching
- Integration with UI components

### User Interface (`ui/`)

```
src/ui/
├── components/         # Reusable UI components
│   └── mod.rs
├── layout/            # Layout management
│   └── mod.rs  
├── widgets/           # Custom widgets
│   └── mod.rs
└── mod.rs             # UI module exports
```

**Purpose**: All user interface code
- `components/`: Reusable UI components (dialogs, forms, lists)
- `layout/`: Layout management and pane organization
- `widgets/`: Custom TUI widgets for database-specific needs

### Library Root (`lib.rs`)

**Purpose**: Library entry point and public API
- Exposes public functions and types
- Module organization and re-exports
- Library-wide configuration

### Application Entry (`main.rs`)

**Purpose**: Binary entry point
- Initializes logging and configuration
- Sets up terminal and event handling
- Starts the main application loop

### Logging (`logging.rs`)

**Purpose**: Logging configuration and setup
- Configures log levels and output
- Sets up structured logging
- Debug and troubleshooting support

## Directory Purposes

### Documentation (`docs/`)

- **`dev/`**: Developer-focused documentation (architecture, setup, contributing)
- **`feature-specs.md`**: Detailed feature specifications and requirements
- **`rough.md`**: Development notes and brainstorming

### Tests (`tests/`)

```
tests/
├── integration/        # Integration tests
├── fixtures/          # Test data and fixtures
└── common/            # Shared test utilities
```

**Purpose**: Automated testing
- Integration tests for end-to-end functionality
- Database adapter testing
- UI component testing

### Examples (`examples/`)

**Purpose**: Usage examples and demonstrations
- Code examples for library usage
- Configuration examples
- Demo scripts and data

### Docker (`docker/`)

**Purpose**: Container-related files
- Development environment containers
- Multi-stage build configurations
- Database setup for testing

## File Naming Conventions

### Rust Files
- **`mod.rs`**: Module definition and exports
- **`lib.rs`**: Library root
- **`main.rs`**: Binary entry point
- **Snake_case**: All other Rust files use snake_case

### Documentation
- **`README.md`**: Primary documentation file in each directory
- **`XXX-topic.md`**: Numbered documentation files (001-setup.md)
- **Kebab-case**: Multi-word documentation files use kebab-case

### Configuration
- **`Cargo.toml`**: Rust project configuration
- **`Makefile`**: Development commands
- **`Dockerfile`**: Container definition
- **`docker-compose.yml`**: Multi-container orchestration

## Module Dependencies

```
main.rs
└── lib.rs
    ├── app/ (depends on core, database, ui, event)
    ├── cli.rs (depends on config, core)
    ├── config/ (depends on core)
    ├── core/ (no dependencies - foundational)
    ├── database/ (depends on core, config)
    ├── event/ (depends on core)
    ├── terminal.rs (depends on core)
    ├── themes/ (depends on core)
    ├── ui/ (depends on core, app, themes, event)
    └── logging.rs (depends on core)
```

**Design Principles**:
- **Core module** has no dependencies and provides foundational types
- **Database layer** is independent of UI concerns
- **UI components** depend on app state but not directly on database
- **Event system** mediates between UI and business logic

## Adding New Modules

When adding new functionality:

1. **Determine the appropriate layer** (app, core, database, ui, etc.)
2. **Create the module directory** if it doesn't exist
3. **Add `mod.rs`** with proper exports
4. **Update parent `mod.rs`** to include the new module
5. **Follow naming conventions** for consistency
6. **Document the purpose** in this file

## Integration Points

### Database Integration
- New database adapters go in `src/database/`
- Implement the `DatabaseAdapter` trait
- Add connection configuration to `config/`

### UI Integration
- New components go in `src/ui/components/`
- New widgets go in `src/ui/widgets/`
- Follow existing patterns for state management

### Event Integration
- Define new events in `src/event/`
- Add event handlers in appropriate modules
- Maintain event flow documentation

This structure supports the modular, maintainable architecture needed for LazyTables' growth and extensibility.