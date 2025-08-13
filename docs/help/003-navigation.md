# 003 - Navigation

Master LazyTables' vim-inspired navigation system to work efficiently with your databases.

## Navigation Philosophy

LazyTables follows vim's navigation principles:
- **Keyboard-first**: Every action can be performed without a mouse
- **Modal interface**: Different modes for different types of interactions
- **Efficient movement**: Get where you need to go with minimal keystrokes
- **Consistent patterns**: Same navigation works across all panes

## Movement Basics

### Within Panes

All panes use consistent vim-style movement:

```bash
# Basic movement
h                   # Move left
j                   # Move down  
k                   # Move up
l                   # Move right

# Word movement
w                   # Next word
b                   # Previous word
e                   # End of word

# Line movement
0                   # Beginning of line
^                   # First non-blank character
$                   # End of line

# Document movement
gg                  # Go to top
G                   # Go to bottom
Ctrl+u              # Page up
Ctrl+d              # Page down
```

### Between Panes

Switch focus between the four main panes:

```bash
# Directional pane switching
Ctrl+h              # Move to left pane
Ctrl+j              # Move to pane below
Ctrl+k              # Move to pane above
Ctrl+l              # Move to right pane

# Cycle through panes
Tab                 # Next pane (clockwise)
Shift+Tab           # Previous pane (counter-clockwise)

# Direct pane access
c                   # Jump to Connections pane
t                   # Jump to Tables pane
d                   # Jump to Details pane
m                   # Jump to Main content area
```

### Pane Focus Visual Indicators

- **Active pane**: Highlighted border (usually colored)
- **Inactive panes**: Dimmed border
- **Pane titles**: Show current focus state
- **Status bar**: Displays active pane name

## Navigation Modes

### Normal Mode (Default)

Primary navigation and command mode:

```bash
# Current mode shown in status bar as: "NORMAL"
# Available actions:
j, k                # Navigate lists
Enter               # Select/activate item
Space               # Toggle selection
/                   # Search within pane
n                   # Next search result
N                   # Previous search result
:                   # Enter command mode
```

### Insert Mode

For text input in query editor and forms:

```bash
# Enter insert mode
i                   # Insert at cursor
I                   # Insert at beginning of line
a                   # Append after cursor
A                   # Append at end of line
o                   # Open new line below
O                   # Open new line above

# Exit insert mode
Esc                 # Return to normal mode
Ctrl+C              # Alternative exit (interrupt)
```

### Visual Mode

For selecting text and data:

```bash
# Enter visual mode
v                   # Character-wise selection
V                   # Line-wise selection
Ctrl+v              # Block/column selection

# Selection actions
y                   # Copy (yank) selection
d                   # Delete selection
c                   # Change (delete and enter insert mode)

# Exit visual mode
Esc                 # Return to normal mode
v                   # Toggle back to normal mode
```

### Command Mode

For executing complex commands:

```bash
# Enter command mode
:                   # Opens command prompt

# Common commands
:help               # Show help
:quit or :q         # Quit application (only way to quit)
:write or :w        # Save current query
:export             # Export data
:connect            # Connect to database
:disconnect         # Disconnect from database

# Exit command mode
Enter               # Execute command
Esc                 # Cancel and return to normal
```

## Pane-Specific Navigation

### Connections Pane

Manage database connections:

```bash
# Navigation
j, k                # Move through connection list
Enter               # Connect/disconnect
Space               # Toggle connection details

# Actions
a                   # Add new connection
e                   # Edit selected connection
d                   # Delete connection (with confirmation)
r                   # Refresh connection status
t                   # Test connection

# Connection states
# [●] Connected (green indicator)
# [○] Disconnected (gray indicator)  
# [!] Error (red indicator)
```

### Tables Pane

Browse database objects:

```bash
# Navigation
j, k                # Move through table list
h, l                # Expand/collapse schema folders
Enter               # Select table and load data
Space               # Preview table structure

# Filtering and search
/                   # Search tables by name
f                   # Filter by table type (table, view, etc.)
s                   # Sort by name, size, or modified date

# Actions
r                   # Refresh table list
i                   # Show table information
c                   # Show table creation SQL
```

### Table Details Pane

View table metadata:

```bash
# Tab navigation within details
Tab                 # Switch between detail tabs:
                    #   - Columns
                    #   - Indexes  
                    #   - Constraints
                    #   - Triggers
                    #   - Statistics

# Column details navigation
j, k                # Move through columns
Enter               # Show column details popup
Space               # Toggle column visibility in main view

# Actions
c                   # Copy column definition
s                   # Generate SELECT statement
i                   # Generate INSERT template
```

### Main Content Area

Query editor and results:

```bash
# Mode switching within main area
q                   # Enter query mode (editor)
r                   # View results mode
e                   # Edit mode (for data editing)

# Query editor (when in query mode)
Ctrl+Enter          # Execute query
Ctrl+S              # Save query
Ctrl+O              # Open query file
Ctrl+N              # New query
Ctrl+Z              # Undo
Ctrl+Y              # Redo

# Results navigation (when in results mode)  
j, k                # Scroll rows
h, l                # Scroll columns
Page Up/Down        # Page through results
Home/End            # Jump to first/last column
gg, G               # Jump to first/last row

# Data editing (when in edit mode)
Enter               # Edit selected cell
Tab                 # Move to next cell
Shift+Tab           # Move to previous cell
Ctrl+S              # Save changes
Esc                 # Cancel editing
```

## Advanced Navigation Patterns

### Quick Database Exploration

Efficient workflow for exploring a new database:

```bash
1. c               # Focus connections
2. j/k             # Select database  
3. Enter           # Connect
4. t               # Focus tables
5. /users          # Search for "users" table
6. Enter           # Select table
7. m               # Focus main area to see data
```

### Query Development Workflow

Streamlined query writing and testing:

```bash
1. m               # Focus main area
2. q               # Enter query mode
3. # Write query...
4. Ctrl+Enter      # Execute
5. r               # Switch to results mode
6. # Review results...
7. q               # Back to query mode for edits
```

### Multi-Table Analysis

Working with related tables:

```bash
1. t               # Focus tables
2. /orders         # Find orders table
3. Enter           # Load orders data
4. d               # Check foreign keys in details
5. t               # Back to tables
6. /customers      # Find related customers table
7. Enter           # Load customers data
8. m               # Focus main area
9. q               # Write JOIN query
```

## Search and Filtering

### Global Search

Search across different contexts:

```bash
# In any pane
/pattern            # Search forward
?pattern            # Search backward
n                   # Next match
N                   # Previous match
*                   # Search for word under cursor
#                   # Search backward for word under cursor
```

### Advanced Search Patterns

```bash
# Regular expressions (when supported)
/user.*admin        # Find tables with "user" followed by "admin"
/^temp_             # Find tables starting with "temp_"
/_(id|key)$         # Find columns ending with "_id" or "_key"

# Case sensitivity
/pattern\c          # Case-insensitive search
/pattern\C          # Case-sensitive search
```

### Filtering

Context-aware filtering in each pane:

```bash
# Tables pane
:filter table       # Show only tables (not views)
:filter view        # Show only views
:filter system      # Show system tables
:clear-filter       # Clear all filters

# Results filtering  
:where status='active'    # Filter results
:limit 100               # Limit number of rows
:order-by created_at     # Sort results
```

## Bookmarks and Jumps

### Jump List

LazyTables maintains a jump list for quick navigation:

```bash
Ctrl+O              # Jump to previous location
Ctrl+I              # Jump to next location
:jumps              # Show jump list
```

### Marks

Set marks for quick navigation:

```bash
m{a-z}              # Set local mark (a through z)
'{a-z}              # Jump to mark
``                  # Jump to last position
'.                  # Jump to last change
```

## Customizing Navigation

### Key Remapping

Customize key bindings in config:

```toml
[keys]
# Pane navigation
pane_left = "h"
pane_right = "l"
pane_up = "k" 
pane_down = "j"

# Custom shortcuts
quick_connect = "C"
quick_query = "Q"
toggle_details = "D"
```

### Navigation Preferences

```toml
[navigation]
wrap_around = true          # Wrap at list boundaries
vim_mode = true            # Use vim navigation
mouse_support = false      # Disable mouse navigation
page_size = 20            # Items per page
```

## Troubleshooting Navigation

### Common Issues

**Navigation not working**:
- Ensure you're in Normal mode (press `Esc`)
- Check if terminal supports the key combinations
- Verify terminal focus (click terminal window)

**Pane switching not working**:
- Some terminals capture `Ctrl+h/j/k/l`
- Try `Tab/Shift+Tab` instead
- Use direct pane shortcuts (`c`, `t`, `d`, `m`)

**Vim navigation feels unfamiliar**:
- Enable practice mode: `:set practice-mode on`
- Use arrow keys initially while learning
- Reference built-in help: `?`

### Performance Tips

For large datasets:
- Use lazy loading (enabled by default)
- Limit result sizes with `:limit N`
- Use server-side filtering when possible
- Enable pagination for big tables

## Important: No Accidental Exits

LazyTables prevents accidental application exits by requiring you to use **command mode** to quit:

- **'q' key alone**: Does NOT quit the application (prevents accidental exits)
- **':q' command**: Properly quits the application (like vim)
- **':quit' command**: Alternative quit command

This design prevents the frustration of accidentally losing your work when you meant to press a different key.

## Navigation Cheat Sheet

Keep this reference handy:

```bash
# Essential Movement
h/j/k/l             # Basic movement
gg/G                # Top/bottom
0/$                 # Line start/end
Ctrl+h/j/k/l        # Switch panes
c/t/d/m             # Direct pane access

# Modes
i                   # Insert mode
v                   # Visual mode
:                   # Command mode
Esc                 # Normal mode

# Actions
Enter               # Select/activate
Space               # Toggle/preview
/                   # Search
:q                  # Quit (only way to exit)
?                   # Help
```

Master these navigation patterns and you'll work with databases more efficiently than ever before! 🚀