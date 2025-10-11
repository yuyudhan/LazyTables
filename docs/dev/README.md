# LazyTables Development Documentation

This directory contains technical documentation, specifications, and development notes for LazyTables.

## 📚 Documentation Index

### Architecture & Implementation

#### [UI_BLOCKING_FIXES.md](UI_BLOCKING_FIXES.md)
Comprehensive analysis of UI blocking operations and async architecture implementation.

**Topics covered:**
- Async file I/O module (`src/io/async_fs.rs`)
- ConnectionStorage async conversion
- Event-driven background tasks
- Timeout protection implementation
- Performance improvements

**Status:** Phase 1 Complete - Foundation established

---

### Feature Specifications

#### [features.txt](features.txt)
Complete list of planned features with detailed specifications.

**21 features organized by category:**
1. User configuration & customization
2. Text editing & VIM integration
3. Database connection experience
4. Data manipulation & management
5. Productivity & workflow enhancements

#### [specs.txt](specs.txt)
Technical implementation specifications organized by development phases.

**6 implementation phases:**
- Phase 1: Core infrastructure & configuration
- Phase 2: VIM editor enhancements
- Phase 3: Connection management improvements
- Phase 4: UI/UX enhancements
- Phase 5: Data management features
- Phase 6: Advanced features & optimization

---

### Development Tasks

#### [tasks.md](tasks.md)
Current development task list with priorities and completion status.

**Sections:**
- Recurring tasks (ongoing maintenance)
- Active development (in progress)
- Completed tasks (archived)
- Notes and patterns

#### [tasks.txt](tasks.txt)
Additional task tracking and notes.

#### [tasks_completed.txt](tasks_completed.txt)
Archive of completed development tasks.

#### [repeated_tasks.txt](repeated_tasks.txt)
Recurring maintenance and QA tasks.

---

### Future Planning

#### [future.txt](future.txt)
Long-term vision and planned features beyond v1.0.

**Topics:**
- Additional database support (MongoDB, Redis, Oracle)
- Advanced query features
- Plugin system
- Performance optimizations
- Cloud integration possibilities

---

## 🛠️ Development Setup

### Prerequisites

```bash
# Install Rust 1.70+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development dependencies
make install-deps
```

### Development Workflow

```bash
# Run with auto-reload
make dev

# Run tests
make test

# Run linting
make lint

# Format code
make format

# Check code (format + lint, CI-friendly)
make check

# Build release
make build
```

### Project Structure

```
src/
├── app/              # Main application logic
├── commands/         # Command pattern implementations
├── database/         # Database adapters (PostgreSQL, MySQL, SQLite)
├── ui/               # UI components and rendering
│   ├── components/   # Reusable UI components
│   └── theme/        # Theme system
├── state/            # Application state management
├── config/           # Configuration handling
├── security/         # Encryption and credential management
├── io/               # Async I/O operations
└── logging.rs        # Logging infrastructure
```

### Key Architectural Patterns

#### Event-Driven Architecture
- `EventHandler` → `App::handle_event()` → Command execution → State update → UI redraw
- No polling, all updates triggered by events
- Async operations send events when complete

#### Command Pattern
- All user actions represented as commands
- Commands return `CommandResult` (success, error, requires confirmation)
- Clear separation between action and execution

#### Database Abstraction
- `Connection` trait defines interface for all databases
- `AdapterFactory` creates appropriate adapter
- Async operations throughout

#### Async File I/O
- All file operations use `src/io/async_fs.rs`
- 5-second timeout protection
- Non-blocking with event notifications

---

## 📊 Current Development Phase

**Version:** v0.2.0-beta.1 (Beta testing)

**Focus Areas:**
1. ✅ Async architecture complete
2. ✅ Vim command mode implemented
3. ✅ SQL autocomplete working
4. ⏳ Beta testing and bug fixes
5. ⏳ Performance optimization
6. ⏳ Integration tests

**Next Milestone:** v0.2.0 stable (late October 2025)

---

## 🧪 Testing Strategy

### Unit Tests
- Inline with module code using `#[cfg(test)]`
- Use `tempfile` for filesystem tests
- Use `pretty_assertions` for better output

### Integration Tests
- Database adapter tests in module files
- Require database connections for full coverage
- Use environment variables for connection strings

### Manual Testing
- Terminal compatibility testing
- Database version compatibility
- Performance under load
- Edge case discovery

**Run tests:**
```bash
cargo test --all-features
```

---

## 📝 Documentation Standards

### Code Documentation
- Public APIs must have doc comments
- Use examples in doc comments where helpful
- Keep comments concise and accurate

### File Headers
- Include file path in comment at top
- Document module purpose

### Commit Messages
Use conventional commits with emojis:
- ✨ `:sparkles:` - New feature
- 🐛 `:bug:` - Bug fix
- 📝 `:memo:` - Documentation
- 🎨 `:art:` - UI/UX improvements
- ♻️ `:recycle:` - Refactoring

---

## 🚀 Release Process

1. **Feature Development** (on `development` branch)
2. **Testing & QA** (local + CI)
3. **Documentation Update** (README, CHANGELOG, release notes)
4. **Version Bump** (Cargo.toml)
5. **PR to main** (with review)
6. **Tag Release** (`git tag v0.x.x`)
7. **GitHub Release** (automated via Actions)
8. **Publish to crates.io** (automated)

---

## 🤝 Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for:
- Bug report templates
- Feature request guidelines
- Pull request process
- Code style requirements

---

## 📧 Questions?

- **Issues**: [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)
- **Email**: ankurkumarpandey@gmail.com

---

**Last Updated:** 2025-10-11 (v0.2.0-beta.1)
