# LazyTables

[![Crates.io](https://img.shields.io/crates/v/lazytables.svg)](https://crates.io/crates/lazytables)
[![License: WTFPL](https://img.shields.io/badge/License-WTFPL-brightgreen.svg)](http://www.wtfpl.net/about/)
![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

**"Because life's too short for clicking around in database GUIs"**

Fast, terminal-based SQL database viewer and editor with vim-style navigation. Built in Rust for developers who live in the terminal.

---

## ✨ Why LazyTables?

- ⚡ **Lightning Fast** - Built in Rust for maximum performance
- ⌨️ **Keyboard-First** - Vim-style navigation, never touch the mouse
- 🎯 **Six-Pane Layout** - Efficient workspace optimized for database work
- 🔍 **Smart Query Editor** - Syntax highlighting, auto-completion, execute at cursor
- 🔐 **Secure** - Encrypted credential storage with AES-GCM
- 🎨 **Beautiful TUI** - Elegant interface built with Ratatui
- 🚀 **Zero Config** - Works out of the box with sensible defaults

---

## 📦 Installation

### Quick Install (Recommended)

```bash
cargo install lazytables
```

### Faster Install with Pre-built Binaries

```bash
cargo install cargo-binstall
cargo binstall lazytables
```

### From Source

```bash
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables
cargo install --path .
```

**Prerequisites**: Rust 1.70+, Terminal with 256 color support, macOS or Linux

📖 [Detailed Installation Guide](docs/user/installation.md)

---

## 🚀 Quick Start

```bash
lazytables
```

First time:
1. Press `a` to add a connection
2. Press `Enter` to connect
3. Press `2` to browse tables
4. Press `?` for help

Navigate: `1-6` for direct pane access, `Tab`/`Shift+Tab` to cycle.

---

## 🎉 What's New in v0.2.3

**Released:** October 14, 2025

### 🐛 Critical Bug Fix
- **Fixed number key input during cell editing** - Number keys 1-6 now work correctly when editing numerical fields

### ✨ New Features
- **Database disconnect** - Press `x` to disconnect from active database
- **Abort connection tests** - Press `Ctrl+C` to abort ongoing connection tests
- **Enhanced schema viewing** - Comprehensive metadata: columns, indexes, foreign keys, constraints, statistics

### 🏗️ Code Quality
- Major refactoring: reorganized ~2000 lines into dedicated handler modules
- Improved maintainability and code organization

📖 [Full Release Notes](docs/releases/v0.2.3.md) | [Changelog](CHANGELOG.md)

---

## 🗄️ Supported Databases

| Database | Status | Icon |
|----------|--------|------|
| **PostgreSQL** | ✅ Full Support | 🐘 |
| **MySQL** | ✅ Full Support | 🐬 |
| **SQLite** | ✅ Full Support | 📁 |
| **MariaDB** | ✅ Full Support | 🔱 |

**Coming Soon**: Oracle, Redis, MongoDB, DB2, ClickHouse

---

## ✨ Key Features

**Six-Pane Layout** - Connections, Tables, Details, Query Results, SQL Editor, SQL Files

**Schema Viewing** - Toggle Data/Schema views with `t`: columns, indexes, foreign keys, constraints, statistics

**Vim Navigation** - `h/j/k/l`, `gg/G`, `0/$`, number keys `1-6` for instant pane switching

**Query Editor** - Execute at cursor (`Ctrl+Enter`), syntax highlighting, auto-completion, file management

**Security** - Encrypted credential storage with AES-GCM

---

## 📚 Documentation

- [Installation Guide](docs/user/installation.md)
- [Key Bindings](docs/user/key-bindings.md)
- [Configuration](docs/user/configuration.md)
- [Guides & Workflows](docs/user/guides.md)
- [Troubleshooting](docs/user/troubleshooting.md)
- [Contributing](CONTRIBUTING.md) | [Architecture](docs/dev/architecture.md) | [Release Notes](docs/release_notes/)

---

## 🎮 Essential Key Bindings

### Global Navigation
- `1-6` - Jump directly to pane by number
- `Tab` / `Shift+Tab` - Cycle through panes
- `?` - Toggle help overlay
- `q` - Quit application

### Connection Management
- `a` - Add new connection
- `e` - Edit connection
- `d` - Delete connection
- `Enter` - Connect to database

### Query Execution
- `Ctrl+Enter` - Execute SQL at cursor
- `Ctrl+S` - Save current query
- `Ctrl+N` - New timestamped query file
- `i` - Enter insert mode (Query Editor only)

### Table Operations
- `t` - Toggle Data/Schema view
- `r` - Refresh data
- `h/j/k/l` - Navigate cells
- `gg/G` - Jump to first/last row

📖 [Complete Key Bindings Reference](docs/user/key-bindings.md)

---

## 🎯 Status Indicators

**Connection:** `✓` Connected | `—` Not connected | `✗` Failed | `⟳` Connecting

**Database:** `🐘` PostgreSQL | `🐬` MySQL/MariaDB | `📁` SQLite

---

## 🤝 Contributing

[Report Bugs](https://github.com/yuyudhan/LazyTables/issues) | [Request Features](https://github.com/yuyudhan/LazyTables/discussions) | [Submit PRs](CONTRIBUTING.md)

## 📜 License

WTFPL - Do What The Fuck You Want To Public License
