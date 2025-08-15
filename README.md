# LazyTables

**"Because life's too short for clicking around in database GUIs"**

A fast, terminal-based SQL database viewer and editor designed for developers who value keyboard-driven workflows. Built with Rust and featuring vim motions throughout, LazyTables provides an intuitive interface for database management without ever leaving your terminal.

## Why LazyTables?

Stop wrestling with clunky GUI database tools. LazyTables brings the power and efficiency of terminal-based workflows to database management:

- **Lightning Fast**: Built with Rust for maximum performance
- **Vim-Style Navigation**: Navigate databases like you navigate code
- **Keyboard-First**: Never reach for your mouse again
- **Cross-Platform**: Works on macOS and Linux

## Installation

### Prerequisites
- **Rust 1.70+** (cargo will be installed with Rust)
- Terminal with 256 color support
- macOS or Linux (Windows not supported)

### Install from crates.io
```bash
cargo install lazytables
```

### Install from source
```bash
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables
cargo install --path .
```

### Install pre-built binary with cargo-binstall
For faster installation without compilation:
```bash
# Install cargo-binstall if you haven't already
cargo install cargo-binstall

# Install lazytables using pre-built binaries
cargo binstall lazytables
```

## Quick Start

1. **Launch LazyTables**:
   ```bash
   lazytables
   ```

2. **Add a database connection** by pressing `a` in the connections pane

3. **Navigate** using vim-style keys:
   - `h/j/k/l` - Move within panes
   - `Ctrl+h/j/k/l` - Switch between panes
   - `Tab/Shift+Tab` - Cycle through panes

4. **Execute queries** with `Space z q` to enter query mode

5. **Quit** with `q`

## Key Features

### Six-Pane Layout
- **Connections** - Manage database connections
- **Tables/Views** - Browse database objects  
- **Table Details** - View metadata and schema
- **Query Results** - Display query output
- **SQL Editor** - Write and execute queries
- **SQL Files** - Browse saved queries

### Vim-Style Navigation
- `h/j/k/l` - Move cursor
- `gg/G` - Jump to top/bottom
- `Ctrl+h/j/k/l` - Switch panes
- `i` - Enter insert mode for editing
- `v` - Visual selection mode
- `:` - Command mode
- `q` - Quit application

### Database Support
- **PostgreSQL** (full support)
- **MySQL, MariaDB, SQLite** (coming soon)
- More databases planned

## Development

### Build and run locally
```bash
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables
make dev  # Run in development mode with auto-reload
```

### Run tests
```bash
make test
```

### Other commands
```bash
make build   # Build release binary
make lint    # Run clippy linter
make format  # Format code with rustfmt
make clean   # Clean build artifacts
```

## Contributing

We welcome contributions! Please see our [development guide](docs/dev/README.md) for details on:
- Architecture overview
- Coding standards
- Testing procedures
- How to submit pull requests

## Support

- **Issues**: [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)

## License

WTFPL - Do What The Fuck You Want To Public License

---

**Ready to ditch the GUI?** Install LazyTables and experience database management the terminal way. 🚀