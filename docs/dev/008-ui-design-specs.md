# 008 - UI Design Specifications

This document contains the detailed UI design specifications for LazyTables, including layout, navigation patterns, and component behavior.

## Overview

LazyTables is a terminal-based SQL database viewer and editor built as a "Lazygit alternative for managing SQL tables." Inspired by TablePlus, it aims to create a simple, elegant terminal tool that eliminates the clumsiness of PGAdmin and phpMyAdmin.

## Application Layout

### Four-Pane Layout System

LazyTables uses a fixed four-pane layout optimized for database workflows:

```
┌─────────────────────────┬─────────────────────────────────────┐
│                         │                                     │
│     Connections         │                                     │
│                         │                                     │
├─────────────────────────┤            Main Content             │
│                         │               Area                  │
│    Tables/Views         │        (Query Editor +              │
│                         │         Results Display)            │
├─────────────────────────┤                                     │
│                         │                                     │
│    Table Details        │                                     │
│                         │                                     │
└─────────────────────────┴─────────────────────────────────────┘
```

### Layout Proportions

- **Left Sidebar**: 20% of screen width
- **Main Content Area**: 80% of screen width
- **Sidebar Panes**: Each occupies roughly 33% of sidebar height
- **Main Content**: Split between query editor (20%) and results (80%)

## Component Specifications

### Connections Pane (Top Left)

**Purpose**: Manage database connections
**Location**: Top-left, 20% width × 33% height

**Features**:
- List all configured database connections
- Show connection status indicators
- Connection management actions

**Key Bindings**:
```bash
c                   # Focus this pane
j, k                # Navigate connection list
Enter               # Connect/disconnect
a                   # Add new connection
d                   # Delete connection (with confirmation)
e                   # Edit connection
r                   # Refresh connection status
```

**Visual Indicators**:
- 🟢 Connected (green indicator)
- ⚪ Disconnected (gray indicator)
- 🔴 Error (red indicator)

### Tables/Views Pane (Middle Left)

**Purpose**: Browse database objects
**Location**: Middle-left, 20% width × 33% height

**Features**:
- Display tables and views for selected database
- Show table types and schemas
- Navigate database structure

**Key Bindings**:
```bash
t                   # Focus this pane
j, k                # Navigate table list
Enter               # Select table
s                   # Select/activate table
/                   # Search tables
```

**Display Logic**:
- Show message "No database selected" when no database is active
- Update table list based on selected database
- Group by schema when applicable

### Table Details Pane (Bottom Left)

**Purpose**: Display metadata about selected table
**Location**: Bottom-left, 20% width × 33% height

**Features**:
- Column information and data types
- Indexes and constraints
- Table statistics
- Foreign key relationships

**Key Bindings**:
```bash
d                   # Focus this pane
j, k                # Navigate detail items
Tab                 # Switch between detail categories
Enter               # Show detailed information
```

**Content Categories**:
- Columns (name, type, nullable, default)
- Indexes (name, type, columns)
- Constraints (primary key, foreign keys, checks)
- Statistics (row count, size, last modified)

### Main Content Area (Right)

**Purpose**: Primary workspace for queries and results
**Location**: Right side, 80% width × 100% height

**Sub-components**:
1. **Query Editor** (top 20%)
2. **Results Display** (bottom 80%)

#### Query Editor

**Features**:
- SQL syntax highlighting
- Multi-line query support
- Query history
- Auto-completion (future)

**Key Bindings**:
```bash
q                   # Focus query editor
Ctrl+Enter          # Execute query
Ctrl+S              # Save query
Ctrl+O              # Open query file
i                   # Enter insert mode
Esc                 # Return to normal mode
```

#### Results Display

**Features**:
- Tabular data display
- Vim-style navigation
- Large dataset support with lazy loading
- Export capabilities (future)

**Key Bindings**:
```bash
o                   # Focus output/results area
h, j, k, l          # Navigate table cells
gg, G               # Jump to first/last row
Page Up/Down        # Page through results
Home/End            # Jump to first/last column
```

## Global Navigation

### Pane Switching

**Directional Navigation**:
```bash
Ctrl+h              # Move to left pane
Ctrl+j              # Move to pane below
Ctrl+k              # Move to pane above
Ctrl+l              # Move to right pane
```

**Direct Pane Access**:
```bash
c                   # Jump to Connections pane
t                   # Jump to Tables pane
d                   # Jump to Details pane
m                   # Jump to Main content area
q                   # Jump to Query editor
o                   # Jump to Output/results
```

**Cycling**:
```bash
Tab                 # Next pane (clockwise)
Shift+Tab           # Previous pane (counter-clockwise)
```

### Visibility Toggle (Future Feature)

Original specification included visibility toggles:
```bash
C                   # Toggle connections pane visibility
T                   # Toggle tables pane visibility
D                   # Toggle details pane visibility
Q                   # Toggle query editor visibility
O                   # Toggle results visibility
```

*Note: This feature is deferred to maintain simplicity in initial implementation.*

## Status Bar Design

### Layout

The status bar follows a Powerlevel10k-inspired theme with segments:

```
[FOCUS] [CONNECTION] [DATABASE] [TABLE] ························ [TIME] [DATE]
```

### Left Side Segments

1. **Focus Indicator**: Shows currently active pane
   - Examples: `CONNECTIONS`, `TABLES`, `QUERY`, `RESULTS`

2. **Connection Status**: Active connection name
   - Examples: `Local Dev`, `Production DB`, `No Connection`

3. **Database Status**: Current database
   - Examples: `myapp_dev`, `production`, `No DB Active`

4. **Table Status**: Selected table
   - Examples: `users`, `orders`, `No Table Active`

### Right Side Segments

1. **Time**: Current time with seconds
   - Format: `14:32:45`

2. **Date**: Current date
   - Format: `2024-01-15`

## Notification System

### Location and Behavior

- **Position**: Top-right corner of terminal
- **Display Duration**: 3 seconds (configurable)
- **Stacking**: Multiple notifications stack vertically
- **Auto-dismiss**: Each notification vanishes after its lifetime

### Notification Types

- **Info**: Blue background, informational messages
- **Success**: Green background, operation completed
- **Warning**: Yellow background, warnings and cautions  
- **Error**: Red background, errors and failures

### Example Notifications

```
┌─────────────────────────────────────┐
│ ✓ Connected to Production DB        │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ ⚠ Query returned 10,000+ rows       │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ ✗ Connection failed: Invalid auth   │
└─────────────────────────────────────┘
```

## Modal Interactions

### Connection Dialog

**Trigger**: Press `a` in Connections pane
**Purpose**: Add new database connection

**Fields**:
- Connection name
- Database type (dropdown)
- Host and port
- Username and password
- Database name (optional)
- SSL settings

**Actions**:
- Test connection before saving
- Save and connect
- Cancel without saving

### Confirmation Dialogs

**Delete Connection**:
```
Delete connection "Local Dev"? [y/N]
```

**Quit Application**:
```
Quit LazyTables? Unsaved queries will be lost [y/N]
(Triggered by :q or :quit command only)
```

## Responsive Design

### Terminal Resize Behavior

- **Maintain proportions**: Keep 20%/80% split between sidebar and main area
- **Minimum dimensions**: Gracefully handle terminals smaller than 80×24
- **Content reflow**: Adjust content display based on available space
- **Scrolling**: Add scrollbars when content exceeds pane dimensions

### Adaptive Layout

**Small terminals (< 100 characters)**:
- Reduce sidebar to minimum viable width
- Stack panes vertically if necessary
- Hide less critical information

**Large terminals (> 120 characters)**:
- Maintain design proportions
- Utilize extra space for content, not padding
- Consider showing additional information

## Focus Indicators

### Visual Feedback

- **Active pane**: Highlighted border (colored)
- **Inactive panes**: Dimmed border (gray)
- **Selected items**: Background highlighting
- **Cursor position**: Clear cursor indicators

### Color Scheme

**Default theme colors**:
- Active border: Bright blue (`#61AFEF`)
- Inactive border: Gray (`#5C6370`)
- Selected item: Dark blue background (`#2C313C`)
- Text: Light gray (`#ABB2BF`)
- Keywords: Purple (`#C678DD`)

## Accessibility Considerations

### Keyboard-Only Operation

- Every feature accessible via keyboard
- No mouse dependency
- Clear focus indicators
- Logical tab order

### Screen Reader Support

- Meaningful labels for all UI elements
- Screen reader announcements for state changes
- Alternative text for visual indicators

## Performance Requirements

### Rendering Performance

- **Target**: 60 FPS smooth scrolling
- **Startup**: < 100ms application launch
- **Query results**: First results visible within 50ms
- **Navigation**: Instant pane switching

### Memory Efficiency

- **Base usage**: < 50MB RAM
- **Large datasets**: Lazy loading with virtual scrolling
- **Connection pooling**: Efficient database connection management

This specification serves as the foundation for LazyTables' user interface implementation, ensuring consistent behavior and visual design across all components.