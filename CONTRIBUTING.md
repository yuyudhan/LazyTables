# Contributing to LazyTables

Thank you for your interest in contributing to LazyTables! This document provides guidelines for contributing to the project, especially during the beta phase.

## 🚧 Beta Testing

LazyTables is currently in **beta** (v0.2.0-beta.1). We're actively seeking feedback from users to help us reach a stable 1.0 release.

### How to Help

1. **Use LazyTables** in your daily workflow
2. **Report bugs** you encounter
3. **Suggest improvements** based on your experience
4. **Test edge cases** with different databases and data types
5. **Contribute code** for features or fixes

## 🐛 Reporting Issues

When reporting a bug, please include:

### Bug Report Template

```markdown
**Description**
A clear description of what went wrong.

**To Reproduce**
Steps to reproduce the behavior:
1. Navigate to '...'
2. Press key '...'
3. Perform action '...'
4. See error

**Expected Behavior**
What you expected to happen.

**Actual Behavior**
What actually happened.

**Environment**
- LazyTables version: [e.g., 0.2.0-beta.1]
- OS: [e.g., macOS 14.0, Ubuntu 22.04]
- Database: [e.g., PostgreSQL 15.2]
- Terminal: [e.g., iTerm2, Alacritty]

**Screenshots/Logs**
If applicable, press Ctrl+B to open debug view and copy relevant logs.

**Additional Context**
Any other context about the problem.
```

## 💡 Feature Requests

We welcome feature suggestions! Please:

1. **Check existing issues** to avoid duplicates
2. **Describe the use case** - why would this be useful?
3. **Provide examples** - how should it work?
4. **Consider vim philosophy** - LazyTables emphasizes keyboard-driven workflows

### Feature Request Template

```markdown
**Feature Description**
A clear description of the feature you'd like to see.

**Use Case**
Why would this feature be useful? What problem does it solve?

**Proposed Implementation**
How do you envision this working? (Optional but helpful)

**Alternatives Considered**
Are there other ways to achieve the same goal?

**Additional Context**
Any other relevant information.
```

## 🔧 Development Setup

### Prerequisites

- Rust 1.70+ with cargo
- Git
- A supported database (PostgreSQL, MySQL, or SQLite) for testing

### Getting Started

```bash
# Clone the repository
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables

# Install development dependencies
make install-deps

# Run in development mode with auto-reload
make dev

# Or run normally
cargo run
```

### Development Commands

```bash
make dev          # Run with auto-reload (requires cargo-watch)
make test         # Run all tests
make lint         # Run clippy linter
make format       # Auto-format code
make format-check # Check formatting without changes
make check        # Run format-check and clippy (CI-friendly)
make build        # Build release binary
```

## 📝 Code Contributions

### Branch Workflow

1. **Fork** the repository
2. **Create a feature branch** from `development`:
   ```bash
   git checkout development
   git pull origin development
   git checkout -b feature/your-feature-name
   ```
3. **Make your changes** with clear, focused commits
4. **Run tests** and linting:
   ```bash
   make test
   make check
   ```
5. **Push to your fork** and create a Pull Request to `development`

### Commit Message Guidelines

We use conventional commit messages with emojis:

- ✨ `:sparkles:` - New feature
- 🐛 `:bug:` - Bug fix
- 📝 `:memo:` - Documentation
- 🎨 `:art:` - UI/UX improvements
- ♻️ `:recycle:` - Refactoring
- ⚡ `:zap:` - Performance improvement
- 🔧 `:wrench:` - Configuration changes
- 🚀 `:rocket:` - Deployment/release

Example:
```
✨ Add SQL autocomplete for table names

- Implement table name completion in query editor
- Add context-aware suggestions based on current database
- Include unit tests for completion logic
```

### Code Style

- **Rust formatting**: Use `rustfmt` (run `make format`)
- **Linting**: Code must pass `clippy` with no warnings (run `make lint`)
- **Testing**: Add tests for new functionality
- **Documentation**: Update relevant docs (README, CLAUDE.md, help text)

### Pull Request Process

1. **Ensure all tests pass** locally
2. **Update documentation** if needed (README, help system, CLAUDE.md)
3. **Describe your changes** clearly in the PR description
4. **Reference related issues** using GitHub keywords (Fixes #123, Relates to #456)
5. **Wait for review** - maintainers will review and provide feedback
6. **Address feedback** if requested

### PR Template

```markdown
**Description**
Clear description of what this PR does.

**Type of Change**
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

**Related Issues**
Fixes #(issue number)
Relates to #(issue number)

**Testing**
How has this been tested?

**Checklist**
- [ ] Tests pass (`make test`)
- [ ] Linting passes (`make check`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (for significant changes)
```

## 🎯 Development Priorities

For v0.2.0-beta to v1.0 stable, we're focusing on:

### Critical (Must Fix)
- [ ] Async file I/O conversion completion
- [ ] Missing keybindings from WIP docs
- [ ] Fix all clippy warnings
- [ ] Comprehensive manual QA testing

### Important (Should Fix)
- [ ] Database version info queries
- [ ] Page scrolling implementation (Ctrl+d/u)
- [ ] Security audit of credential encryption
- [ ] Integration tests for database operations

### Nice to Have
- [ ] Demo GIF/video for README
- [ ] Extended user guide documentation
- [ ] Performance benchmarking
- [ ] Additional database support (MongoDB, Redis)

## 📚 Documentation

### Files to Update When Making Changes

- **README.md** - User-facing features and installation
- **CLAUDE.md** - Development guidelines and architecture (for significant changes)
- **docs/dev/** - Technical documentation and specifications
- **Help system** - In-app help text (src/ui/help.rs)

### Documentation Standards

- Use clear, concise language
- Provide examples where helpful
- Keep keyboard shortcuts up to date
- Maintain consistency with existing docs

## 🤝 Code of Conduct

### Our Standards

- **Be respectful** of different viewpoints and experiences
- **Accept constructive criticism** gracefully
- **Focus on what's best** for the community
- **Show empathy** towards other community members

### Unacceptable Behavior

- Harassment, discrimination, or offensive comments
- Personal attacks or trolling
- Publishing others' private information
- Other conduct which could reasonably be considered inappropriate

## 📧 Contact

- **Issues**: [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)

## 🙏 Recognition

Contributors will be recognized in:
- Release notes for their contributions
- GitHub contributors page
- Special thanks in major release announcements

---

**Thank you for contributing to LazyTables!** Your help is invaluable in making this the best terminal-based database tool for developers.
