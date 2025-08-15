# LazyTables - Product Requirements Document

**_"Because life's too short for clicking around in database GUIs"_**

## Executive Summary

LazyTables is a terminal-based SQL database viewer and editor designed for
developers who value keyboard-driven workflows. Built with Rust and featuring
vim motions throughout, it provides a fast, intuitive interface for database
management without leaving the terminal.

## Product Vision

Create the most efficient and enjoyable terminal-based database management
tool that respects developers' muscle memory and workflow preferences, making
database interaction as seamless as text editing in vim.

## Core Principles

- **Keyboard-First**: Every action accessible via keyboard shortcuts
- **Vim-Native**: Consistent vim motions throughout the application
- **Speed**: Instant response times and lazy loading for large datasets
- **Simplicity**: Intuitive interface that doesn't require documentation for
  basic tasks
- **Extensibility**: Plugin architecture for custom workflows and database
  support

## Technical Stack

- **Language**: Rust (for performance, memory safety, and strong CLI tooling)
- **TUI Framework**: Ratatui (most mature and feature-rich TUI library)
- **Database Drivers**: Native Rust crates (e.g., sqlx for async connections)
- **License**: WTFPL (What The Fuck Public License)
- **Repository**: `git@github.com:yuyudhan/LazyTables.git`
- **Supported Platforms**: macOS, Linux (No Windows support - Windows users
  are not worth creating tools for)

## Database Support Roadmap

### Phase 0 - Foundation (MVP)

- PostgreSQL (full support with all core features)

### Phase 1 - Core Databases

- MySQL
- MariaDB
- SQLite
- Oracle
- DB2
- ClickHouse
- Redis (key-value store support)

### Phase 2 - Extended Support

- MongoDB (document database support)
- Additional databases based on community demand

## User Interface Design

### Layout Architecture

LazyTables uses a fixed four-pane layout optimized for database navigation and
data manipulation. The layout is intentionally non-configurable to maintain
consistency and simplicity across all user workflows.

## Visual Layout

```
┌────────────────────────────────────────────────────────────────────────┐
│                          LazyTables v1.0.0                             │
├─────────────────┬─────────────────┬───────────────────────────────────┤
│                 │                 │                                   │
│  Connections    │  Tables/Views   │         Main Content Area        │
│                 │                 │                                   │
│ ▶ production    │ ▼ public        │  ┌────┬──────┬──────┬──────────┐ │
│   localhost     │   ▶ users       │  │ id │ name │email │ created  │ │
│   staging_db    │   ▶ products    │  ├────┼──────┼──────┼──────────┤ │
│ ▼ development   │   ▶ orders      │  │  1 │ John │j@e.c │2024-01-15│ │
│   ▶ test_db1    │   ▶ payments    │  │  2 │ Jane │j@m.c │2024-01-16│ │
│   ▶ test_db2    │ ▼ analytics     │  │  3 │ Bob  │b@e.c │2024-01-17│ │
│   local_dev     │   events        │  │  4 │ Alice│a@e.c │2024-01-18│ │
│                 │   sessions      │  │  5 │ Eve  │e@e.c │2024-01-19│ │
│ [+] Add New     │   metrics       │  └────┴──────┴──────┴──────────┘ │
│                 │                 │                                   │
├─────────────────┼─────────────────┤  Showing 5 of 15,234 rows       │
│                 │                 │  Columns: 4 | Page 1/3047        │
│ Table Details   │ Schema: public  │                                   │
│                 │ Table: users    │  [F5] Refresh [Space e x] Export │
│ Rows: 15,234    │ Engine: InnoDB  │  [Space q] Query [i] Edit Cell   │
│ Size: 2.4 MB    │ Charset: utf8   │                                   │
│ Created: Jan'24 │                 │                                   │
│ Modified: Today │ Indexes: 3      │                                   │
│ Type: Table     │ Constraints: 2  │                                   │
│                 │ Relations: 5    │                                   │
└─────────────────┴─────────────────┴───────────────────────────────────┘
│ [NORMAL] Connected: production@localhost | Cell: B2 | Mode: Read-Only │
└────────────────────────────────────────────────────────────────────────┘
```

## Pane Specifications

### Left Section (25% width)

The left section contains three vertically stacked panes for database
navigation and metadata.

#### 1. Connections Pane (Top Left - 40% of left height)

**Purpose**: Manage and switch between database connections

**Content**:

- List of saved database connections
- Active connection highlighted with different color/symbol
- Connection status indicators (● connected, ○ disconnected, ⟳ connecting)
- Grouped by environment (production, staging, development, local)
- "[+] Add New" option at the bottom

**Features**:

- `Enter`: Connect to selected database
- `d`: Delete connection (with confirmation)
- `r`: Rename connection
- `e`: Edit connection details
- `n`: Create new connection
- `Space c c`: Quick connection switcher
- Visual indicators for connection type (PostgreSQL, MySQL, etc.)

**Visual Elements**:

```
▼ production        (expanded group)
  ● prod_main      (connected)
  ○ prod_replica   (disconnected)
▶ staging          (collapsed group)
```

#### 2. Tables/Views Pane (Middle Left - 40% of left height)

**Purpose**: Navigate database objects within the active connection

**Content**:

- Hierarchical tree view of database objects
- Schemas/databases as top-level items
- Tables, views, functions grouped under schemas
- Search/filter bar at top (activated with `/`)
- Object count badges
- Type indicators (T for table, V for view, F for function)

**Features**:

- `Enter`: Load table/view in main content area
- `Space t n`: Create new table
- `Space t d`: Drop selected table
- `Space t r`: Rename table
- `/`: Filter tables by name
- `*`: Show only tables with recent changes
- Collapsible schemas with `Space` or `Enter`

**Visual Elements**:

```
▼ public (134 objects)
  T users (15.2k rows)
  T products (8.5k rows)
  V active_users
  F calculate_total()
▶ analytics (47 objects)
```

#### 3. Table Details Pane (Bottom Left - 20% of left height)

**Purpose**: Display metadata about the currently selected table

**Content**:

- Table statistics (row count, size, last modified)
- Storage information (engine type, charset, collation)
- Structure summary (column count, index count)
- Relationship information (foreign keys in/out)
- Quick actions panel

**Features**:

- Dynamic content based on selected table
- Real-time statistics updates
- Click-through to detailed views
- Performance indicators (slow queries, missing indexes)

**Layout Example**:

```
Table: users
─────────────────
Rows: 15,234 (↑2.3%)
Size: 2.4 MB
Created: 2024-01-15
Modified: 2 hrs ago

Structure:
• Columns: 12
• Indexes: 3
• Constraints: 2
• Relations: 5 FK

[Details] [Structure]
```

### Right Section - Main Content Area (75% width)

#### Main Content Pane

**Purpose**: Primary workspace for viewing and editing data

**Content Modes**:

1. **Table View Mode** (default):

   - Spreadsheet-like grid for table data
   - Column headers with sort indicators
   - Row numbers on the left
   - Horizontal and vertical scrolling
   - Cell highlighting for active selection
   - Multi-cell selection support

2. **Query Mode** (`Space z q`):

   - Full-screen SQL editor
   - Syntax highlighting
   - Auto-completion
   - Query history sidebar
   - Result set display below editor

3. **Structure View** (`Space s`):
   - Column definitions
   - Index details
   - Constraints and triggers
   - Table DDL preview

**Features**:

- Virtual scrolling for large datasets
- Lazy loading with visual indicators
- Inline cell editing with `i`
- Column resizing with `<` and `>`
- Sort by column with `s` on header
- Filter columns with `f`

**Status Bar** (within main content):

```
Showing 1-50 of 15,234 rows | Page 1/305 | Filtered: No | Sort: id ASC
```

## Navigation Flow

### Pane Focus Management

**Active Pane Indicators**:

- Bright border color for active pane
- Dimmed border for inactive panes
- Status bar shows current pane context

**Keyboard Navigation**:

- `Ctrl+h`: Focus Connections pane (left-most)
- `Ctrl+j`: Focus Tables pane (middle-left)
- `Ctrl+k`: Focus Details pane (bottom-left)
- `Ctrl+l`: Focus Main Content (right)
- `Tab`: Cycle through panes clockwise
- `Shift+Tab`: Cycle counter-clockwise

### Contextual Navigation

**Within Connections Pane**:

- `j/k`: Navigate up/down through connections
- `h/l`: Collapse/expand connection groups
- `gg/G`: Jump to first/last connection

**Within Tables Pane**:

- `j/k`: Navigate through tables
- `h`: Collapse schema/go to parent
- `l`: Expand schema/enter
- `/`: Start filtering
- `n/N`: Next/previous search match

**Within Main Content**:

- `h/j/k/l`: Cell navigation
- `Ctrl+d/u`: Page down/up
- `0/$`: First/last column
- `gg/G`: First/last row
- `w/b`: Next/previous word in cell

## Responsive Behavior

### Minimum Terminal Size

- Minimum width: 120 columns
- Minimum height: 30 rows
- Below minimum: Show warning and simplified layout

### Pane Sizing Rules

1. **Connections Pane**:

   - Min: 20 columns
   - Max: 40 columns
   - Auto-adjust based on longest connection name

2. **Tables Pane**:

   - Min: 20 columns
   - Max: 40 columns
   - Truncate long table names with ellipsis

3. **Details Pane**:

   - Fixed height: 8 rows minimum
   - Expands with terminal height

4. **Main Content**:
   - Takes all remaining space
   - Minimum 60 columns for usability

### Overflow Handling

- **Horizontal Overflow**: Scrollable with indicators
- **Vertical Overflow**: Virtual scrolling with row count
- **Content Truncation**: Ellipsis (...) for long text
- **Tooltip**: Full content on hover/focus

## Color Coding & Visual Hierarchy

### Pane Colors (Dark Theme)

```
Connections Pane:
  Background: #1a1a1a
  Active Connection: #4a9eff (blue)
  Disconnected: #666666 (gray)
  Error: #ff4444 (red)

Tables Pane:
  Background: #1e1e1e
  Selected Table: #4a9eff (blue)
  Table Type Icons: #888888 (gray)
  Search Match: #ffaa00 (yellow)

Details Pane:
  Background: #161616
  Labels: #888888 (gray)
  Values: #ffffff (white)
  Warnings: #ffaa00 (yellow)

Main Content:
  Background: #0d0d0d
  Headers: #2d2d2d
  Selected Cell: #2a4a7a
  Edit Mode: #4a9eff (blue border)
  Grid Lines: #2a2a2a
```

## State Persistence

### Saved Layout State

- Last active pane
- Collapsed/expanded schemas
- Column widths in main content
- Sort preferences per table
- Filter settings
- Connection group states

### Session Restoration

On startup, LazyTables restores:

1. Last used connection (attempt reconnection)
2. Last viewed table
3. Scroll position in main content
4. Pane focus
5. Search/filter history

## Performance Optimizations

### Lazy Loading Strategy

1. **Connections**: Load on demand, cache connection tests
2. **Tables List**: Load first 50, then progressively
3. **Table Data**: Virtual scrolling with 100-row chunks
4. **Details**: Async load, cache for 5 minutes

### Rendering Optimizations

- Only render visible portions of each pane
- Debounce resize events (100ms)
- Cache rendered rows for quick scrolling
- Pre-fetch adjacent data chunks
- Use diff rendering for cell updates

## Error States

### Connection Pane Errors

```
▼ production
  ⚠ prod_main (timeout)
  ✗ prod_replica (auth failed)
```

### Table Loading Errors

Display in Main Content:

```
┌────────────────────────────┐
│                            │
│   ⚠ Failed to load table  │
│                            │
│   Error: Permission denied │
│                            │
│   [Retry] [Details]        │
│                            │
└────────────────────────────┘
```

## Quick Actions Overlay

Activated with `Space h` from any pane:

```
┌─────────────────────────────┐
│   Quick Actions             │
├─────────────────────────────┤
│ c c  Change Connection      │
│ t n  New Table              │
│ q    Query Mode             │
│ e x  Export Data            │
│ / ?  Search / Help          │
│                             │
│ Press key or ESC to cancel  │
└─────────────────────────────┘
```

This layout structure provides a comprehensive workspace for database
management while maintaining the simplicity and efficiency that vim users
expect.

### Layout Structure

```
┌─────────────────────────────────────────────────────────┐
│ [Connection: prod_db] [Database: users] [Table: accounts]│
├──────────────┬──────────────────────────────────────────┤
│              │                                          │
│  Navigation  │          Main Content Area               │
│              │                                          │
│  • Databases │  ┌──────┬──────┬──────┬──────┐         │
│  • Tables    │  │  id  │ name │email │ date │         │
│  • Views     │  ├──────┼──────┼──────┼──────┤         │
│  • Functions │  │  1   │ John │j@e.c │ 2024 │         │
│              │  │  2   │ Jane │j@m.c │ 2024 │         │
│              │  └──────┴──────┴──────┴──────┘         │
│              │                                          │
├──────────────┴──────────────────────────────────────────┤
│ [NORMAL] Rows: 1-10 of 15,234 | Cell: B2               │
└─────────────────────────────────────────────────────────┘
```

- **Fixed Layout**: No user-controlled pane splitting to maintain simplicity
- **Tree View**: Database schema browser on the left
- **Main Content**: Table data display on the right
- **Status Bar**: Connection info, mode indicator, and position

### Color Scheme & Theming

- **Default Theme**: Dark mode optimized for long coding sessions
- **Highlighting**: Active cell with subtle border, selected rows with
  background tint
- **Syntax Highlighting**: SQL keywords, strings, numbers in query editor
- **Status Indicators**: Color-coded connection status (green=connected,
  yellow=connecting, red=error)
- **Custom Themes**: User-configurable via `~/.config/lazytables/theme.toml`

## Navigation & Key Bindings

### Mode System

- **Normal Mode**: Navigation and commands (default)
- **Insert Mode**: Direct cell editing (press `i` to enter)
- **Visual Mode**: Row/column selection
- **Query Mode**: SQL query composition (`Space z q`)
- **Command Mode**: Complex operations (`:` prefix)

### Core Navigation (Normal Mode)

#### Vim Motions

- `h/j/k/l`: Move between cells
- `gg`: Go to first row
- `G`: Go to last row
- `0`: Go to first column
- `$`: Go to last column
- `w/b`: Next/previous word in cell content
- `{/}`: Navigate between tables

#### Pane Navigation

- `Ctrl+h`: Move to left pane
- `Ctrl+j`: Move to pane below
- `Ctrl+k`: Move to pane above
- `Ctrl+l`: Move to right pane

#### Search

- `/`: Search within current view
- `n/N`: Next/previous search result

### Leader Key Commands (Space as Leader)

#### Connection Management

- `Space c c`: Change connection
- `Space c d`: Change database
- `Space c r`: Refresh connection

#### Table Operations

- `Space t n`: New table
- `Space t d`: Drop table
- `Space t r`: Rename table
- `Space t`: Open table in new tab

#### Query Operations

- `Space z q`: Toggle full-screen query editor
- `Space q n`: New query tab
- `Space q r`: Run query (`Ctrl+Enter` in query mode)
- `Space q s`: Save query to file
- `Space q h`: Query history

#### Data Operations

- `Space e x`: Export data
- `Space e i`: Import data
- `Space b m`: Bookmark current view
- `Space b l`: List bookmarks
- `Space f f`: Open file browser for SQL scripts

### Editing Operations

- `i`: Edit current cell (enter insert mode)
- `ESC`: Exit insert mode
- `Enter`: Commit cell change
- `I`: Insert new row
- `A`: Append new column
- `dd`: Delete current row
- `yy`: Yank (copy) current row
- `p`: Paste row
- `u`: Undo
- `Ctrl+r`: Redo
- `v`: Visual mode (select cells)
- `V`: Visual line mode (select entire rows)
- `Ctrl+v`: Visual block mode (select columns)

### Help System

- `?`: Full-screen help overlay with scrollable content (navigate with
  `h/j/k/l`, exit with `q`)
- `K`: Context-sensitive help for current operation
- `:help [topic]`: Detailed help for specific topic

## Core Features

### 1. Connection Management

- **Quick Connect**: Recently used connections list
- **Connection Profiles**: Save and organize connections
- **SSH Tunneling**: Built-in SSH tunnel support
- **Connection Pool**: Efficient connection management
- **Auto-reconnect**: Automatic reconnection on timeout
- **Secure Storage**: Platform-specific secret stores (macOS Keychain, Linux
  Keyring)

### 2. Data Viewing

- **Lazy Loading**: Load data as needed for performance
- **Virtual Scrolling**: Handle millions of rows smoothly
- **Column Resizing**: Automatic and manual column width adjustment
- **Data Types**: Proper rendering for all SQL data types
- **NULL Handling**: Clear visual distinction for NULL values
- **Binary Data**: Hex viewer for binary columns

### 3. Data Editing

- **In-place Editing**: Direct cell modification with vim-style insert mode
- **Batch Operations**: Apply changes to multiple cells
- **Transaction Support**: Explicit commit/rollback
- **Validation**: Type checking before submission
- **Conflict Resolution**: Handle concurrent edits gracefully

### 4. Query Editor

- **Full-Screen Mode**: Distraction-free query composition
- **Multi-tab Support**: Multiple query tabs
- **Syntax Highlighting**: Full SQL syntax support
- **Auto-completion**: Table names, column names, SQL keywords
- **Query History**: Searchable history with favorites
- **Execution Plans**: Visual execution plan display
- **Result Sets**: Multiple result set handling

### 5. Schema Management

- **Visual Schema Editor**: Create/modify tables visually
- **Migration Tracking**: Track schema changes
- **Index Management**: View and manage indexes
- **Foreign Keys**: Visual relationship display
- **Constraints**: Easy constraint management

## Plugin System

### Architecture

Simple Rust crates compiled as shared libraries (.so files) that extend
functionality without complex APIs.

```rust
pub trait LazyTablesPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    fn execute(&self, context: PluginContext) -> Result<PluginResponse>;
    fn cleanup(&mut self) -> Result<()>;
}
```

### Configuration

Plugins defined in `~/.config/lazytables/plugins.toml`:

```toml
[[plugin]]
name = "goto_fk_definition"
path = "/home/user/lazy_plugins/goto_fk_def.so"
command = "Space g d"
```

### Plugin Capabilities

- **Custom Commands**: Register new leader key commands
- **Data Transformers**: Process data before display
- **Export Formats**: Add new export formats
- **Import Handlers**: Support additional import sources
- **Custom Visualizations**: Create specialized data views
- **Database Adapters**: Add support for new databases

## Theming System

### Theme Configuration

Located at `~/.config/lazytables/theme.toml`:

```toml
[theme]
name = "LazyDark"
author = "LazyTables Team"

[colors]
background = "#1e1e2e"
foreground = "#cdd6f4"
selection = "#45475a"
cursor = "#f5e0dc"
pane_background = "#181825"
primary_highlight = "#74c7ec"
selected_cell_background = "#45475a"
table_header_text = "#cba6f7"
status_bar_background = "#313244"

[syntax]
keyword = "#cba6f7"
string = "#a6e3a1"
number = "#fab387"
function = "#89b4fa"

[ui]
border = "#313244"
active_pane = "#74c7ec"
```

### Theme Features

- **Hot Reload**: Live theme switching
- **Color Support**: 256-color and true color support
- **Font Styles**: Bold, italic, underline support
- **Custom Icons**: Configurable icon sets
- **Theme Sharing**: Easy distribution of theme files

## Performance Requirements

- **Startup Time**: < 100ms
- **Query Execution**: Display first results within 50ms
- **Scrolling**: 60 FPS smooth scrolling
- **Memory Usage**: < 50MB base, efficient for large datasets
- **Connection Time**: < 1 second for local databases
- **UI Responsiveness**: Asynchronous data fetching to prevent UI freezing

## Installation & Distribution

### Package Managers

- **macOS**: `brew install lazytables`
- **Ubuntu/Debian**: `apt install lazytables`
- **Arch Linux**: `pacman -S lazytables`
- **Generic**: `cargo install lazytables`

### Configuration Locations

- **Config**: `~/.config/lazytables/config.toml`
- **Plugins**: `~/.config/lazytables/plugins/`
- **Themes**: `~/.config/lazytables/themes/`
- **Cache**: `~/.cache/lazytables/`

## Security & Error Handling

### Security

- **Credential Storage**: OS keychain integration (never plain text)
- **Encryption**: TLS/SSL support for connections
- **Audit Logging**: Optional query logging
- **Read-only Mode**: Safe browsing mode
- **SQL Injection**: Prevention in all user inputs

### Error Handling

- **Graceful Degradation**: Never crash, always recover
- **Clear Messages**: User-friendly error descriptions in status bar or
  temporary pop-ups
- **Recovery Suggestions**: Actionable error resolution
- **Error Logging**: Detailed logs for debugging
- **Retry Logic**: Automatic retry for transient errors

## Development Phases

### Phase 0 (Months 1-2): PostgreSQL MVP

- Core TUI framework with ratatui
- PostgreSQL connection and browsing
- Basic vim motions implementation
- Table viewing with pagination
- Simple cell editing with insert mode

### Phase 1 (Months 3-4): Multi-Database Support

- MySQL, MariaDB, SQLite implementation
- Oracle, DB2, ClickHouse adapters
- Redis key-value interface
- Unified database abstraction layer

### Phase 2 (Months 5-6): Advanced Features

- Plugin system implementation
- Theme support with hot reload
- Query editor with autocomplete
- Advanced editing capabilities
- Import/export functionality

### Phase 3 (Month 7): Polish & Release

- Performance optimization
- Documentation
- Package manager releases
- Community setup
- Marketing website

## Repository Structure

```
lazytables/
├── src/
│   ├── core/           # Core functionality
│   ├── ui/             # TUI components
│   ├── adapters/       # Database adapters
│   ├── plugins/        # Plugin system
│   └── themes/         # Theme engine
├── plugins/            # Built-in plugins
├── themes/             # Default themes
├── docs/               # Documentation
├── tests/              # Test suite
└── examples/           # Usage examples
```

## Success Metrics

- **Performance**: All operations under 100ms
- **Reliability**: 99.9% crash-free sessions
- **Adoption**: 10,000 active users in first year
- **Satisfaction**: > 4.5 stars on package managers
- **Community**: 100+ community plugins

## Future Enhancements

### Version 2.0

- **Collaboration**: Multi-user shared sessions
- **Cloud Sync**: Settings and bookmarks sync
- **Advanced Visualizations**: Charts and graphs
- **Script Runner**: Batch script execution
- **API Integration**: REST API for automation
- **Data Export**: CSV, JSON, SQL format support

### Version 3.0

- **AI Assistant**: Natural language to SQL
- **Performance Advisor**: Query optimization suggestions
- **Data Profiling**: Automatic data analysis
- **Version Control**: Git-like database versioning
- **Advanced Editing**: Bulk editing, rollbacks, DDL/DML script generation

## License

This project is licensed under the WTFPL (What The Fuck Public License),
giving users complete freedom to do whatever they want with the software.

---

_LazyTables - Making database management as smooth as vim navigation_

