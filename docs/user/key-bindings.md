# Complete Key Bindings Reference

LazyTables uses vim-style navigation throughout the application. This guide provides a complete reference of all key bindings organized by pane and context.

## Table of Contents

- [Global Commands](#global-commands)
- [Navigation](#navigation)
- [Pane-Specific Bindings](#pane-specific-bindings)
  - [Connections Pane](#1-connections-pane)
  - [Tables Pane](#2-tables-pane)
  - [Details Pane](#3-details-pane)
  - [Query Results / Table Viewer](#4-query-results--table-viewer)
  - [SQL Query Editor](#5-sql-query-editor)
  - [SQL Files Browser](#6-sql-files-browser)
- [Search Modes](#search-modes)
- [Modal Windows](#modal-windows)

---

## Global Commands

These keys work from anywhere in the application:

| Key | Action |
|-----|--------|
| `q` | Quit LazyTables (with confirmation) |
| `?` | Toggle context-aware help overlay |
| `:` | Enter command mode |
| `Ctrl+B` | Toggle debug view for logs |

## Navigation

### Pane Navigation

LazyTables has 6 panes that you can navigate between:

| Key | Action |
|-----|--------|
| `1` | Jump to Connections Pane |
| `2` | Jump to Tables Pane |
| `3` | Jump to Details Pane |
| `4` | Jump to Query Results |
| `5` | Jump to SQL Query Editor |
| `6` | Jump to SQL Files Browser |
| `Tab` | Cycle to next pane |
| `Shift+Tab` | Cycle to previous pane |

### Directional Pane Navigation

| Key | Action |
|-----|--------|
| `Ctrl+h` | Focus pane to the left |
| `Ctrl+j` | Focus pane below |
| `Ctrl+k` | Focus pane above |
| `Ctrl+l` | Focus pane to the right |

### Data Operations

| Key | Action |
|-----|--------|
| `Ctrl+Enter` | Execute SQL query at cursor |
| `Ctrl+S` | Save current SQL query |
| `Ctrl+O` | Refresh current view |
| `Ctrl+N` | Create new timestamped query file |

---

## Pane-Specific Bindings

### [1] Connections Pane

Manage database connections in this pane.

#### List Navigation
| Key | Action |
|-----|--------|
| `j` or `↓` | Move down in connection list |
| `k` or `↑` | Move up in connection list |
| `gg` | Jump to first connection |
| `G` | Jump to last connection |

#### Connection Actions
| Key | Action |
|-----|--------|
| `Enter` or `Space` | Connect to selected database |
| `x` | Disconnect from current connection |
| `a` | Add new connection (opens modal) |
| `e` | Edit selected connection |
| `d` | Delete connection (with confirmation) |
| `/` | Enter search mode to filter connections |
| `r` | Refresh connection list |

#### Connection Modal

When creating or editing a connection:

| Key | Action |
|-----|--------|
| `Enter` | Save and test connection |
| `←` or `→` | Navigate between form steps |
| `Tab` | Next form field |
| `Shift+Tab` | Previous form field |
| `i` | Enter insert mode in text field |
| `ESC` | Cancel modal / Exit insert mode |
| `Ctrl+T` | Toggle connection method (string vs fields) |

---

### [2] Tables Pane

Navigate and manage database tables and views.

#### List Navigation
| Key | Action |
|-----|--------|
| `j` or `↓` | Move down in table list |
| `k` or `↑` | Move up in table list |
| `gg` | Jump to first table |
| `G` | Jump to last table |

#### Table Actions
| Key | Action |
|-----|--------|
| `Enter` or `Space` | Open table for viewing |
| `n` | Create new table (when connected) |
| `e` | Edit table structure |
| `/` | Enter search mode to filter tables |
| `r` | Refresh table list |

---

### [3] Details Pane

View detailed information about the selected table (read-only).

#### Scrolling
| Key | Action |
|-----|--------|
| `j` or `↓` | Scroll down one line |
| `k` or `↑` | Scroll up one line |
| `Ctrl+D` | Scroll down half page |
| `Ctrl+U` | Scroll up half page |
| `gg` | Jump to top |
| `G` | Jump to bottom |

#### Actions
| Key | Action |
|-----|--------|
| `Enter` or `Space` | Load detailed metadata for table |
| `r` | Refresh metadata |

---

### [4] Query Results / Table Viewer

View and interact with query results or table data.

#### Cell Navigation
| Key | Action |
|-----|--------|
| `h` or `←` | Move left one column |
| `j` or `↓` | Move down one row |
| `k` or `↑` | Move up one row |
| `l` or `→` | Move right one column |
| `0` | Jump to first column |
| `$` | Jump to last column |
| `gg` | Jump to first row |
| `G` | Jump to last row |

#### Page Navigation
| Key | Action |
|-----|--------|
| `Ctrl+D` | Scroll down half page |
| `Ctrl+U` | Scroll up half page |

#### Data Operations
| Key | Action |
|-----|--------|
| `i` or `Enter` | Enter edit mode for current cell |
| `Enter` | Save cell changes (in edit mode) |
| `ESC` | Cancel cell edit |
| `dd` | Delete current row (with confirmation) |
| `yy` | Copy row data in CSV format |

#### View Controls
| Key | Action |
|-----|--------|
| `t` | Toggle between Data and Schema view |
| `r` | Refresh / Reload table data |
| `/` | Enter search mode |
| `n` | Jump to next search match |
| `N` | Jump to previous search match |

#### Tab Management
| Key | Action |
|-----|--------|
| `S` | Switch to previous tab |
| `D` | Switch to next tab |
| `x` | Close current tab |

---

### [5] SQL Query Editor

Write and execute SQL queries. This is the only pane with vim-style insert mode.

#### Normal Mode (Default)

##### Cursor Movement
| Key | Action |
|-----|--------|
| `h` | Move left one character |
| `j` | Move down one line |
| `k` | Move up one line |
| `l` | Move right one character |
| `w` | Move to next word start |
| `b` | Move to previous word start |
| `e` | Move to next word end |
| `0` | Move to line start |
| `$` | Move to line end |
| `gg` | Move to file start |
| `G` | Move to file end |

##### Entering Insert Mode
| Key | Action |
|-----|--------|
| `i` | Enter insert mode at cursor |
| `a` | Enter insert mode after cursor |
| `o` | New line below + enter insert mode |
| `O` | New line above + enter insert mode |

##### Query Execution
| Key | Action |
|-----|--------|
| `Ctrl+Enter` | Execute query at cursor |

##### Modes
| Key | Action |
|-----|--------|
| `i` | Enter full-screen Query mode |
| `:` | Enter command mode |

#### Insert Mode

| Key | Action |
|-----|--------|
| `ESC` | Exit insert mode (return to normal) |
| `Ctrl+Enter` | Execute query |
| `Tab` | Accept auto-completion suggestion |
| `↑` or `↓` | Navigate completion suggestions |
| `Enter` | Insert new line |
| `Backspace` | Delete character before cursor |

#### Query Mode (Full-Screen)

When you press `i` in the Query Editor, you enter full-screen Query mode:

| Key | Action |
|-----|--------|
| `ESC` | Exit Query mode / Exit insert mode |
| `i` | Enter insert mode (for editing) |
| `q` | Quit with confirmation |
| `h/j/k/l` | Cursor navigation (normal mode) |
| `w/b/e` | Word navigation (normal mode) |
| `0/$` | Line start/end (normal mode) |
| `gg/G` | File start/end (normal mode) |

#### Command Mode

Press `:` in normal mode to enter command mode:

| Command | Action |
|---------|--------|
| `:w` | Save current query |
| `:q` | Quit with confirmation |
| `:q!` | Force quit without saving |
| `:wq` | Save and quit |

---

### [6] SQL Files Browser

Browse and manage saved SQL query files.

#### List Navigation
| Key | Action |
|-----|--------|
| `j` or `↓` | Move down in file list |
| `k` or `↑` | Move up in file list |
| `gg` | Jump to first file |
| `G` | Jump to last file |

#### File Operations
| Key | Action |
|-----|--------|
| `Enter` or `Space` | Load selected SQL file into editor |
| `c` | Copy / Duplicate file |
| `d` | Delete file (with confirmation) |
| `r` | Rename file |
| `Ctrl+N` | Create new timestamped query file |
| `/` | Enter search mode to filter files |

#### Editor Actions
| Key | Action |
|-----|--------|
| `i` | Enter Query mode for editing |

---

## Search Modes

Most panes support search functionality for quick filtering:

### Activating Search
Press `/` in any searchable pane to activate search mode.

### Search Controls
| Key | Action |
|-----|--------|
| Type | Filter results in real-time |
| `↑` or `k` | Navigate up in filtered results |
| `↓` or `j` | Navigate down in filtered results |
| `Enter` | Select highlighted result |
| `ESC` | Exit search mode |

### Finding in Results
In Query Results / Table Viewer:

| Key | Action |
|-----|--------|
| `/` | Start search |
| `n` | Jump to next match |
| `N` | Jump to previous match |
| `ESC` | Clear search highlights |

---

## Modal Windows

### Connection Modal

Used when creating or editing database connections.

| Key | Action |
|-----|--------|
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `←` or `→` | Navigate between steps |
| `i` | Enter insert mode (text fields) |
| `ESC` | Cancel / Exit insert mode |
| `Enter` | Confirm / Next step / Save |
| `Ctrl+T` | Toggle connection method |

### Confirmation Dialogs

When confirming destructive actions (delete, disconnect):

| Key | Action |
|-----|--------|
| `y` or `Enter` | Confirm action |
| `n` or `ESC` | Cancel action |
| `Tab` | Toggle between options |

---

## Tips for Efficient Navigation

1. **Direct Pane Access**: Use number keys `1-6` to jump directly to any pane
2. **Vim Motions**: Most vim motions work throughout (gg, G, w, b, 0, $)
3. **Context Help**: Press `?` in any pane for context-specific help
4. **Search Everything**: Use `/` liberally to filter long lists
5. **Tab Management**: Use `S`/`D` to quickly switch between open tables

---

## Understanding Insert Mode

LazyTables follows vim-style input patterns:

- **SQL Query Editor** is the ONLY pane with traditional vim insert mode
- **Forms and modals** accept direct typing in text fields (no insert mode needed)
- **All other panes** use direct key bindings (a/e/d for actions, j/k for navigation)

Visual indicators show when you're in insert mode:
- `[INSERT]` indicator in status bar
- Cursor style change (block → line)

---

**Need more help?** Press `?` in any pane for context-aware assistance, or check out the [Guides](guides.md) for common workflows.
