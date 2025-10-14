# LazyTables Release Notes

This directory contains detailed release notes for all versions of LazyTables.

## Latest Release

### [v0.2.3](v0.2.3.md) - 2025-10-14 ✅ STABLE
**Bug Fixes & Code Refactoring**

Maintenance release with critical bug fixes, major code refactoring for improved maintainability, and enhanced user experience features.

**Highlights:**
- 🐛 Fixed critical number key input bug during cell editing
- 🏗️ Major code refactoring (~2000 lines into dedicated modules)
- 🔌 Database disconnect functionality (press 'x')
- 🛑 Abort connection tests with Ctrl+C
- 📊 Enhanced schema viewing with comprehensive metadata
- ✨ Improved visual feedback and empty states

---

## Recent Stable Releases

### [v0.2.2](v0.2.2.md) - 2025-10-12
**Bug Fixes & Navigation Enhancements**

Critical fixes for SQLite connections and enhanced vim-style navigation with page scrolling and jump commands.

### [v0.2.1](v0.2.1.md) - 2025-10-11
**Async Architecture & UX Improvements**

Stable release of v0.2.0-beta.1 with fully async operations, vim command mode, SQL autocomplete, and numbered pane navigation.

---

## Earlier Stable Releases

### [v0.1.7](v0.1.7.md) - 2025-09-27
**cargo-binstall Compatibility**

Enables fast binary installation without compilation. Pre-built binaries for macOS (Intel & Apple Silicon) and Linux (x86_64 & ARM64).

**Key Features:**
- cargo-binstall support for 5-15 second installation
- Multi-platform pre-built binaries
- Enhanced GitHub Actions workflow
- Same functionality as v0.1.6

### [v0.1.6](v0.1.6.md) - 2025-09-27
**Comprehensive Documentation & User Experience**

Major documentation overhaul with complete key bindings reference, installation guides, and user workflows.

**Key Features:**
- Complete README.md transformation
- Comprehensive key bindings reference
- Feature specifications (21 planned features)
- Development roadmap (6 phases)
- User-friendly documentation structure

### [v0.1.5](v0.1.5.md) - 2025-09-26
**Version Bump**

Technical release for cargo-binstall testing. No functional changes.

### [v0.1.4](v0.1.4.md) - 2025-09-24
**Debug View & Connection Management**

Introduced debugging capabilities and improved connection management.

**Key Features:**
- Full-screen debug view (Ctrl+B)
- Real-time logging system
- Persistent connection management
- Database cell editing capabilities
- Dark theme fixes

### [v0.1.3](v0.1.3.md) - 2025-08-15
**First Stable Release**

Initial stable release with secure credential management and SQL file organization.

**Key Features:**
- AES-256-GCM password encryption
- SQL file management per database
- Query execution with Ctrl+Enter
- PostgreSQL support
- Cross-platform (macOS, Linux)

---

## Version Timeline

```
v0.2.3 (2025-10-14) ← LATEST STABLE
    ↑
v0.2.2 (2025-10-12)
    ↑
v0.2.1 (2025-10-11)
    ↑
v0.2.0-beta.1 (2025-10-11) ← BETA
    ↑
v0.1.7 (2025-09-27)
    ↑
v0.1.6 (2025-09-27)
    ↑
v0.1.5 (2025-09-26)
    ↑
v0.1.4 (2025-09-24)
    ↑
v0.1.3 (2025-08-15) ← FIRST STABLE
```

## Release Types

### Stable Releases
Production-ready versions recommended for daily use:
- v0.2.3, v0.2.2, v0.2.1, v0.1.7, v0.1.6, v0.1.4, v0.1.3

### Beta Releases
Feature-complete but seeking feedback before stable:
- v0.2.0-beta.1

### Technical Releases
Version bumps for testing purposes:
- v0.1.5

## How to Choose a Version

### For Production Use
Use **v0.2.3** (latest stable):
```bash
cargo install lazytables
```
or
```bash
cargo binstall lazytables
```

### For Development
Use **development branch**:
```bash
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables
git checkout development
cargo run
```

## Migration Guides

All versions maintain backward compatibility. No manual migration required when upgrading:
- Configuration files remain compatible
- Connection data preserved
- SQL files unchanged

## Contributing

Found an issue? Want to contribute?
- See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines
- Report bugs at [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
- Join discussions at [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)

## License

All versions released under WTFPL (Do What The Fuck You Want To Public License)

---

For a complete version history, see [CHANGELOG.md](../../CHANGELOG.md)
