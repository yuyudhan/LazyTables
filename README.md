# LazyTables

**"Because life's too short for clicking around in database GUIs"**

A fast, terminal-based SQL database viewer and editor designed for developers who value keyboard-driven workflows. Built with Rust and featuring vim motions throughout, LazyTables provides an intuitive interface for database management without ever leaving your terminal.

🚀 **Active Development** - LazyTables is continuously evolving with new features and improvements to enhance your database management experience. We welcome contributions and feedback from the community!

## Why LazyTables?

Stop wrestling with clunky GUI database tools. LazyTables brings the power and efficiency of terminal-based workflows to database management:

- **Lightning Fast**: Built with Rust for maximum performance
- **Vim-Style Navigation**: Navigate databases like you navigate code
- **Four-Pane Layout**: Efficient workspace designed for productivity
- **Keyboard-First**: Never reach for your mouse again
- **Cross-Platform**: Works seamlessly on macOS and Linux

## Installation

### macOS

#### Using Homebrew
```bash
brew tap yuyudhan/lazytables
brew install lazytables
```

#### Build from source
```bash
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables
cargo build --release
sudo cp target/release/lazytables /usr/local/bin/
```

### Linux

#### Ubuntu/Debian (package repository coming soon)
```bash
curl -fsSL https://repo.lazytables.dev/setup.sh | sudo bash
sudo apt install lazytables
```

#### Arch Linux (AUR)
```bash
yay -S lazytables-git
```

#### Build from source
```bash
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables
cargo build --release
sudo cp target/release/lazytables /usr/local/bin/
```

### Prerequisites

- **Rust 1.70+** (for building from source)
- **Terminal** with 256 color support
- Supported on **macOS** and **Linux** (Windows not supported)

## Quick Start

1. **Install LazyTables** using one of the methods above
2. **Launch the application**:
   ```bash
   lazytables
   ```
3. **Add your first connection** by pressing `a` in the connections pane
4. **Navigate** using vim-style keys (`h/j/k/l`) and switch panes with `Ctrl+h/j/k/l`
5. **Quit** anytime with `q`

## Key Features

### Four-Pane Workspace
- **Connections** (top-left): Manage database connections
- **Tables/Views** (middle-left): Browse database objects
- **Table Details** (bottom-left): View metadata and schema
- **Content Area** (right): Query editor and results viewer

### Vim-Style Navigation
- `h/j/k/l` - Move within active pane
- `Ctrl+h/j/k/l` - Switch between panes  
- `Tab/Shift+Tab` - Cycle through panes
- `gg/G` - Jump to top/bottom of lists
- `q` - Quit application

### Multiple Modes
- **Normal Mode** (default) - Navigate and execute commands
- **Insert Mode** (`i`) - Edit data directly in cells
- **Visual Mode** (`v`) - Select rows and columns
- **Command Mode** (`:`) - Execute complex operations
- **Query Mode** (`Space z q`) - Write and execute SQL

### Database Support
- **PostgreSQL** (full support)
- **MySQL, MariaDB, SQLite** (planned)
- **Oracle, DB2, ClickHouse** (planned)
- **Redis, MongoDB** (future releases)

## Development & Contribution

Interested in contributing? We'd love your help! See our [development guide](docs/dev/README.md) for:

- Development setup and commands
- Architecture overview
- Coding standards and guidelines
- Testing procedures
- Contribution workflow

### Quick Development Setup

```bash
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables
make dev  # Start development mode with auto-reload
```

## Support & Community

- **Issues**: Report bugs and request features on [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
- **Discussions**: Join conversations on [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)
- **Contributions**: See [docs/dev/README.md](docs/dev/README.md) for contribution guidelines

## License

WTFPL - Do What The Fuck You Want To Public License

---

**Ready to ditch the GUI?** Install LazyTables and experience database management the terminal way. 🚀