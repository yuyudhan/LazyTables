# LazyTables Developer Guide

Technical documentation for LazyTables contributors and developers.

## 📚 Documentation Index

### Getting Started
1. **[Getting Started](001-getting-started.md)** - Development environment setup
2. **[Architecture](002-architecture.md)** - System design and architecture
3. **[Development Commands](003-development-commands.md)** - Useful make commands
4. **[Project Structure](004-project-structure.md)** - Codebase organization

### Implementation
5. **[Database Support](005-database-support.md)** - Database adapter implementation
6. **[Testing](006-testing.md)** - Testing guidelines and strategies
7. **[Contributing](007-contributing.md)** - How to contribute
8. **[UI Design](008-ui-design-specs.md)** - Terminal UI specifications

## 🚀 Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables

# 2. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Install development dependencies
make install-deps

# 4. Run in development mode
make dev

# 5. Run tests
make test
```

## 🏗️ Architecture Overview

LazyTables uses a **six-pane terminal UI** built with Rust and Ratatui:

```
┌─────────────┬──────────────────┬──────┐
│ Connections │  Query Results   │ SQL  │
├─────────────┼──────────────────┤ Files│
│ Tables/     ├──────────────────┤      │
│ Views       │  SQL Editor      │      │
├─────────────┤                  │      │
│ Table       │                  │      │
│ Details     │                  │      │
└─────────────┴──────────────────┴──────┘
```

### Key Components

- **TUI Framework**: Ratatui with crossterm backend
- **Database Layer**: SQLx with async support
- **State Management**: Application state with event-driven updates
- **Security**: AES-GCM encryption for credentials
- **Configuration**: TOML-based configuration

## 🛠️ Development Workflow

### 1. Feature Development

```bash
# Create feature branch
git checkout -b feature/your-feature

# Run in watch mode
make dev

# Test your changes
make test
make lint
```

### 2. Database Testing

```bash
# Start test database
make db-up

# View logs
make db-logs

# Reset database
make db-reset

# Stop database
make db-down
```

### 3. Code Quality

```bash
# Format code
make format

# Run linter
make lint

# Run all checks
make check
```

## 📊 Performance Goals

- **Startup**: < 100ms
- **Query Display**: < 50ms for first results
- **Scrolling**: 60 FPS smooth
- **Memory**: < 50MB base usage
- **Large Datasets**: Virtual scrolling for millions of rows

## 🔧 Key Technologies

- **Rust 1.70+** - Systems programming language
- **Ratatui** - Terminal UI framework
- **SQLx** - Async SQL toolkit
- **Tokio** - Async runtime
- **Crossterm** - Cross-platform terminal manipulation

## 📝 Coding Standards

### Rust Best Practices
- Use `Result<T, E>` for error handling
- Prefer `&str` over `String` for function parameters
- Use `clippy` for linting
- Write doc comments for public APIs
- Keep functions small and focused

### File Organization
```rust
// FilePath: src/module/file.rs

// Module imports
use crate::prelude::*;

// External imports
use external_crate::Type;

// Module implementation
impl Module {
    // Public methods first
    pub fn public_method() {}
    
    // Private methods last
    fn private_method() {}
}
```

### Commit Messages
Follow conventional commits:
```
feat: add PostgreSQL connection pooling
fix: resolve cursor position in SQL editor
docs: update installation guide
test: add integration tests for MySQL adapter
```

## 🧪 Testing Strategy

### Test Types
- **Unit Tests**: Individual functions and methods
- **Integration Tests**: Database adapters and TUI components
- **End-to-End Tests**: Full application workflows

### Running Tests
```bash
# All tests
make test

# Specific module
cargo test database::

# With output
cargo test -- --nocapture
```

## 🤝 Contributing

1. **Fork** the repository
2. **Create** a feature branch
3. **Implement** your feature with tests
4. **Ensure** all tests pass
5. **Submit** a pull request

See [Contributing Guide](007-contributing.md) for details.

## 📚 Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Ratatui Documentation](https://ratatui.rs/)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [Vim Motion Reference](https://vim.rtorr.com/)

## 🆘 Getting Help

- **Issues**: [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)
- **Documentation**: This guide and inline code comments

---

**Ready to contribute?** Start with [Getting Started](001-getting-started.md)!