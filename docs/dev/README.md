# LazyTables Developer Documentation

Welcome to the LazyTables developer documentation! This directory contains comprehensive guides for contributing to and working with LazyTables.

## Documentation Index

### 📚 Core Documentation

1. **[001 - Getting Started](001-getting-started.md)**
   - Development environment setup
   - Prerequisites and dependencies
   - First development session
   - Troubleshooting common issues

2. **[002 - Architecture](002-architecture.md)**
   - Design philosophy and principles
   - Four-pane layout system
   - Component architecture
   - Performance considerations

3. **[003 - Development Commands](003-development-commands.md)**
   - Complete Makefile reference
   - Cargo command alternatives
   - Environment variables
   - Performance testing commands

4. **[004 - Project Structure](004-project-structure.md)**
   - Repository organization
   - Source code structure
   - Module dependencies
   - File naming conventions

### 🗄️ Database & Testing

5. **[005 - Database Support](005-database-support.md)**
   - Current and planned database support
   - Database adapter architecture
   - Implementation guidelines
   - Connection management

6. **[006 - Testing](006-testing.md)**
   - Testing philosophy and strategy
   - Unit, integration, and TUI tests
   - Test database setup
   - Performance testing

### 🤝 Contributing

7. **[007 - Contributing](007-contributing.md)**
   - Contribution workflow
   - Coding standards
   - Pull request guidelines
   - Community guidelines

## Quick Start

New to LazyTables development? Start here:

1. **Set up your environment**: [001 - Getting Started](001-getting-started.md)
2. **Understand the architecture**: [002 - Architecture](002-architecture.md)
3. **Learn the project structure**: [004 - Project Structure](004-project-structure.md)
4. **Read the contribution guide**: [007 - Contributing](007-contributing.md)

## Development Workflow

```bash
# 1. Initial setup
git clone git@github.com:yuyudhan/LazyTables.git
cd LazyTables
make install-deps

# 2. Start development
make dev              # Auto-reload development mode

# 3. Run tests
make test            # Run all tests
make lint            # Check code quality

# 4. Test with database
make db-up           # Start PostgreSQL
make db-down         # Stop PostgreSQL
```

## Key Resources

- **[Project README](../../README.md)**: User-facing documentation
- **[PRD.md](../../PRD.md)**: Product Requirements Document
- **[CLAUDE.md](../../CLAUDE.md)**: AI assistant instructions
- **[GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)**: Bug reports and features
- **[GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)**: Community discussions

## Architecture Overview

LazyTables uses a **four-pane terminal UI** built with Rust and Ratatui:

```
┌─────────────┬─────────────────────────────┐
│ Connections │                             │
├─────────────┤                             │
│ Tables/     │        Main Content         │
│ Views       │          Area               │
├─────────────┤                             │
│ Table       │                             │
│ Details     │                             │
└─────────────┴─────────────────────────────┘
```

**Navigation**: Vim-style with `h/j/k/l` and `Ctrl+h/j/k/l` for pane switching

**Performance Goals**:
- Startup: < 100ms
- Query display: < 50ms for first results
- Scrolling: 60 FPS
- Memory: < 50MB base usage

## Current Status

🚧 **Active Development**: LazyTables is in early development with PostgreSQL support being implemented first.

**Next Priorities**:
1. Complete PostgreSQL adapter
2. Implement basic TUI navigation
3. Add MySQL and SQLite support
4. Develop query editor functionality

## Getting Help

- **New to the project?** Start with [Getting Started](001-getting-started.md)
- **Architecture questions?** See [Architecture](002-architecture.md)
- **Want to contribute?** Read [Contributing](007-contributing.md)
- **Found a bug?** Check [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
- **Need support?** Join [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)

---

**Ready to contribute?** We'd love your help making LazyTables the best terminal database tool! 🚀