# LazyTables

[![Crates.io](https://img.shields.io/crates/v/lazytables.svg)](https://crates.io/crates/lazytables)
[![License: WTFPL](https://img.shields.io/badge/License-WTFPL-brightgreen.svg)](http://www.wtfpl.net/about/)
![Status: Beta](https://img.shields.io/badge/status-beta-yellow)

**"Because life's too short for clicking around in database GUIs"**

A fast, terminal-based SQL database viewer and editor designed for developers who value keyboard-driven workflows. Built with Rust and featuring vim motions throughout, LazyTables provides an intuitive interface for database management without ever leaving your terminal.

## 🚧 Beta Status (v0.2.0-beta.1)

LazyTables is currently in **beta**. The core features are stable and ready for daily use, but we're seeking feedback before the v0.2.0 stable release.

**What works:**
- ✅ All three database types (PostgreSQL, MySQL, SQLite)
- ✅ Fully async architecture (no UI blocking)
- ✅ Vim command mode and SQL autocomplete
- ✅ Numbered pane navigation
- ✅ Connection management with animated loading
- ✅ 82 tests passing

**What we're polishing:**
- ⏳ Some advanced features in progress
- ⏳ Additional testing on edge cases
- ⏳ Performance optimizations

[Report issues](https://github.com/yuyudhan/LazyTables/issues) | [Read beta notes](docs/release_notes/v0.2.0-beta.1.md) | [See roadmap](#roadmap-to-stable-v020)

## Why LazyTables?

Stop wrestling with clunky GUI database tools. LazyTables brings the power and efficiency of terminal-based workflows to database management:

- **Lightning Fast**: Built with Rust for maximum performance
- **Vim-Style Navigation**: Navigate databases like you navigate code
- **Keyboard-First**: Never reach for your mouse again
- **Cross-Platform**: Works on macOS and Linux
- **Six-Pane Layout**: Efficient workspace for database management
- **Smart Query Editor**: Full vim-style editing with syntax highlighting and auto-completion

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

4. **Execute queries** with `Ctrl+Enter` or enter query mode with `i`

5. **Get help** by pressing `?` for context-aware assistance

6. **Quit** with `q`

## Database Support

### Currently Supported
- **PostgreSQL** - Full support with all features
- **MySQL** - Full support (stable)
- **SQLite** - Full support (stable)

### Planned Support
- **MariaDB** - Coming soon
- **Oracle** - Planned
- **Redis** - Key-value store support planned
- **MongoDB** - Document database support planned

## Key Features

### Six-Pane Layout
LazyTables uses a fixed six-pane layout optimized for database workflow:

1. **Connections Pane** (Top Left) - Manage database connections with status indicators
2. **Tables/Views Pane** (Middle Left) - Browse database objects with search and filtering
3. **Table Details Pane** (Bottom Left) - View detailed metadata, schema, and relationships
4. **Query Results Area** (Top Right) - Display tabular query results with navigation
5. **SQL Query Editor** (Bottom Left) - Full-featured vim-style SQL editor
6. **SQL Files Browser** (Bottom Right) - Browse and manage saved SQL files

### Advanced Query Editor
- **Vim-style editing** with multiple modes (Normal, Insert, Visual, Command)
- **Syntax highlighting** for SQL queries
- **Smart auto-completion** with database context awareness
- **Execute at cursor** - Run specific SQL statements with `Ctrl+Enter`
- **File management** - Save, load, and organize SQL queries
- **Query history** - Track and reuse previous queries

### Table Management & Navigation
- **Data view mode** - Browse table data with cell editing
- **Schema view mode** - Inspect table structure and metadata
- **Search and filter** - Find data quickly across tables
- **Row operations** - Edit, delete, and copy rows
- **Tab management** - Work with multiple tables simultaneously

## Complete Key Bindings Reference

### Global Commands
| Key | Action |
|-----|--------|
| `q` | Quit LazyTables |
| `?` | Toggle help guide |
| `:` | Enter command mode |
| `Ctrl+B` | Toggle debug view |

### Navigation
| Key | Action |
|-----|--------|
| `Tab` | Next pane |
| `Shift+Tab` | Previous pane |
| `Ctrl+h` | Focus left pane |
| `Ctrl+j` | Focus down pane |
| `Ctrl+k` | Focus up pane |
| `Ctrl+l` | Focus right pane |

### Data Operations
| Key | Action |
|-----|--------|
| `Ctrl+Enter` | Execute SQL at cursor |
| `Ctrl+S` | Save current query |
| `Ctrl+O` | Refresh current view |
| `Ctrl+N` | New timestamped query |

### Connections Pane
| Key | Action |
|-----|--------|
| `j/k` | Navigate up/down connections |
| `Enter/Space` | Connect to selected database |
| `x` | Disconnect current connection |
| `a` | Add new connection |
| `e` | Edit selected connection |
| `d` | Delete connection (with confirmation) |
| `/` | Start search mode |

#### Connection Modal
| Key | Action |
|-----|--------|
| `Enter` | Save/Test connection |
| `←/→` | Navigate form steps |
| `Tab/Shift+Tab` | Navigate form fields |
| `i` | Enter insert mode (text fields) |
| `ESC` | Cancel modal/exit insert |
| `Ctrl+T` | Toggle connection method |

### Tables Pane
| Key | Action |
|-----|--------|
| `j/k` | Navigate up/down tables |
| `gg/G` | Jump to first/last table |
| `Enter/Space` | Open table for viewing |
| `n` | Create new table (when connected) |
| `e` | Edit table structure |
| `/` | Start search mode |

### Table Details Pane
| Key | Action |
|-----|--------|
| `j/k` | Scroll up/down |
| `Enter/Space` | Load detailed metadata |
| `r` | Refresh metadata |

### Table Viewer (Query Results)
| Key | Action |
|-----|--------|
| `h/j/k/l` | Navigate table cells |
| `gg/G` | Jump to first/last row |
| `0/$` | Jump to first/last column |
| `Ctrl+D/U` | Page down/up through data |
| `i` | Enter edit mode for current cell |
| `Enter` | Save cell changes |
| `ESC` | Cancel cell edit |
| `/` | Start search mode |
| `n/N` | Navigate to next/previous match |
| `dd` | Delete current row |
| `yy` | Copy row data (CSV format) |
| `t` | Toggle between Data and Schema view |
| `r` | Refresh/reload table data |
| `x` | Close current tab |
| `S/D` | Switch to previous/next tab |

### SQL Files Pane
| Key | Action |
|-----|--------|
| `j/k` | Navigate up/down files |
| `Enter/Space` | Load selected SQL file |
| `c` | Copy/duplicate file |
| `d` | Delete file (with confirmation) |
| `r` | Rename file |
| `i` | Enter Query mode for editing |
| `Ctrl+N` | Create new timestamped query |
| `/` | Start search mode |

### Query Editor
| Key | Action |
|-----|--------|
| `i` | Enter full-screen Query mode |
| `h/j/k/l` | Navigate cursor (normal mode) |
| `Ctrl+Enter` | Execute query at cursor |

#### Query Mode (Full-screen)
| Key | Action |
|-----|--------|
| `ESC` | Exit Query mode / Exit insert mode |
| `i` | Enter insert mode for text editing |
| `q` | Quit with confirmation |
| `h/j/k/l` | Cursor navigation (vim keys) |
| `w/b/e` | Word navigation |
| `0/$` | Line start/end |
| `gg/G` | File start/end |

##### Insert Mode
| Key | Action |
|-----|--------|
| `Tab` | Accept auto-completion suggestion |
| `↑/↓` | Navigate suggestions |
| `Enter` | Insert new line |
| `Backspace` | Delete character |
| `ESC` | Exit insert mode |

##### Command Mode
| Key | Action |
|-----|--------|
| `:` | Enter vim command mode |
| `:w` | Save current query |
| `:q` | Quit with confirmation |
| `:q!` | Force quit without saving |
| `:wq` | Save and quit |

### Search Modes
Most panes support search functionality:
- Type to filter results in real-time
- `↑/↓` or `j/k` to navigate results
- `Enter` to select highlighted result
- `ESC` to exit search mode

### Insert Mode Requirements
LazyTables follows vim-style input patterns:
- **All text input fields require pressing 'i' to enter insert mode**
- **Press ESC to exit insert mode and return to normal navigation**
- **Arrow keys are used for dropdown/list navigation**
- **Visual feedback shows when in insert mode (e.g., "[INSERT]" indicator)**

This applies to:
- Connection creation/editing forms
- Table creation forms
- Query editing windows
- File renaming operations
- All text input fields

## Status Indicators

### Connection Status
- `✓` Connected to database
- `—` Not connected
- `✗` Connection failed
- `⟳` Connecting in progress

### Display Format
Connections show as: `[Icon] [Status] Name (type) [DB: name] Status`

Database type icons:
- `🐘` PostgreSQL
- `🐬` MySQL
- `📁` SQLite

## Configuration

LazyTables stores its configuration and data in the following locations:

### Configuration Directory
```
~/.config/lazytables/
└── config.toml       # Main configuration file
```

### Data Directory
```
~/.lazytables/
├── README.md         # Data directory documentation
├── connections.json  # Database connection definitions (encrypted)
├── connections/      # Individual connection files
├── sql_files/        # Saved SQL query files
│   └── sample_queries.sql  # Sample SQL queries
├── logs/             # Application log files
│   └── lazytables.log
└── backups/          # Backup files
```

### Connection Storage
- **Secure encryption** for database credentials
- **Connection files** stored individually for better organization
- **Auto-backup** of connection configurations
- **Legacy support** for existing connection formats

## Tips for New Users

### Getting Started
1. **Start with the help system** - Press `?` in any pane for context-specific help
2. **Practice navigation** - Use `Tab` and `Ctrl+h/j/k/l` to move between panes
3. **Learn the modes** - Normal mode for navigation, Insert mode for editing
4. **Use the built-in help** - Every pane has detailed guidance

### Productivity Tips
- **Query at cursor**: Place cursor on any SQL statement and press `Ctrl+Enter`
- **Save frequently used queries**: Use `Ctrl+S` to save queries to files
- **Search everything**: Most panes support `/` for searching
- **Use vim motions**: `gg`, `G`, `0`, `$`, `w`, `b` work throughout the app
- **Tab management**: Work with multiple tables using `S`/`D` to switch tabs

### Common Workflows
1. **Database exploration**: Connect → Browse tables → View details → Query data
2. **Query development**: Write in editor → Execute with `Ctrl+Enter` → Save with `Ctrl+S`
3. **Data editing**: Open table → Navigate with `h/j/k/l` → Edit with `i` → Save with `Enter`
4. **File management**: Browse SQL files → Load with `Enter` → Edit → Save

## Troubleshooting

### Connection Issues
- Check connection credentials and host/port
- Verify database is running and accessible
- Look at status indicators for specific error messages
- Use debug mode (`Ctrl+B`) for detailed connection logs

### Performance
- LazyTables is optimized for large datasets with virtual scrolling
- Query results are paginated automatically for >10K rows
- Use filters and search to work with large tables efficiently

### Keyboard Navigation
- Remember that text input requires `i` to enter insert mode
- Press `ESC` to return to normal mode from any input state
- Use `?` in any pane for context-specific help
- Arrow keys work for dropdown navigation, `h/j/k/l` for vim-style movement

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

## Known Limitations (Beta)

As a beta release, some features are still being polished:

### In Progress
- Some file I/O operations being converted to async
- Database version info queries for some databases
- Page scrolling (Ctrl+d/u) in some panes
- Minor clippy warnings being addressed

### Planned for Stable
- Table and column name autocomplete (keywords work now)
- More comprehensive error messages
- Additional integration tests
- Performance benchmarks

**These limitations don't affect daily use**, but we're actively working on them. [See detailed list](docs/release_notes/v0.2.0-beta.1.md#known-limitations)

## Roadmap to Stable v0.2.0

We're targeting **late October 2025** for the stable release. Here's our plan:

### Week 1-2: Bug Fixes (Current)
- ✅ Core beta release published
- 🔄 Addressing reported bugs
- 🔄 Completing async file I/O
- 🔄 Fixing clippy warnings

### Week 3-4: Polish
- ⏳ Enhanced error messages
- ⏳ Additional integration tests
- ⏳ Performance optimization
- ⏳ Security audit

### Stable Release
- ⏳ All beta issues resolved
- ⏳ External testing complete
- ⏳ Documentation finalized
- ⏳ Performance benchmarks published

**Help us get there faster:** [Contribute](#contributing) | [Report bugs](https://github.com/yuyudhan/LazyTables/issues) | [Join discussions](https://github.com/yuyudhan/LazyTables/discussions)

## Contributing

We welcome contributions, especially during beta! Please see:
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Beta testing guide and contribution guidelines
- **[docs/dev/README.md](docs/dev/README.md)** - Technical documentation and architecture
- **Bug reports** - Use [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
- **Feature requests** - Start a [Discussion](https://github.com/yuyudhan/LazyTables/discussions)

## Support

- **Issues**: [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)
- **Documentation**: Built-in help system (press `?` in any pane)

## License

WTFPL - Do What The Fuck You Want To Public License

---

**Ready to ditch the GUI?** Install LazyTables and experience database management the terminal way. 🚀

*Pro tip: Press `?` after installation to explore the comprehensive help system built right into the application.*