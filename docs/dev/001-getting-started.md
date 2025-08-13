# 001 - Getting Started

This guide covers the initial setup and first steps for LazyTables development.

## Prerequisites

- **Rust 1.70+** and Cargo
- **Docker** (for database testing)
- **Git**
- Terminal with 256 color support

## Initial Setup

### 1. Clone the Repository

```bash
git clone git@github.com:yuyudhan/LazyTables.git
cd LazyTables
```

### 2. Install Development Dependencies

```bash
make install-deps
```

This installs:
- `cargo-watch` - For auto-reload during development
- `cargo-audit` - For security auditing dependencies

### 3. Verify Setup

```bash
# Test that everything compiles
cargo check

# Run the application (will show help/error since no DB connected)
cargo run

# Run tests
cargo test
```

## First Development Session

### Start Development Mode

```bash
# Auto-reload on file changes
make dev
```

This uses `cargo watch` to automatically recompile and restart the application when you modify source files.

### Alternative: Manual Run

```bash
# Run debug build
make run

# Or directly with cargo
cargo run
```

## Testing Database Connection

### Start Test Database

```bash
# Start PostgreSQL in Docker
make db-up
```

This creates a test PostgreSQL instance:
- Host: `localhost`
- Port: `5432`
- User: `lazytables`
- Password: `lazytables`
- Database: `test_db`

### Stop Test Database

```bash
make db-down
```

## Troubleshooting

### Common Issues

1. **Rust version too old**
   ```bash
   rustup update stable
   ```

2. **Missing cargo-watch**
   ```bash
   cargo install cargo-watch
   ```

3. **Docker not running**
   - Make sure Docker Desktop is running
   - Check with: `docker ps`

4. **Port 5432 already in use**
   ```bash
   # Stop existing PostgreSQL
   brew services stop postgresql
   # Or kill process using port
   lsof -ti:5432 | xargs kill
   ```

### Getting Help

- Check [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues) for known problems
- Join discussions on [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)
- Review other dev documentation files for specific topics

## Next Steps

1. Read [002-architecture.md](002-architecture.md) to understand the codebase structure
2. Review [003-development-commands.md](003-development-commands.md) for available commands
3. Check [004-project-structure.md](004-project-structure.md) for code organization
4. See [007-contributing.md](007-contributing.md) before making changes