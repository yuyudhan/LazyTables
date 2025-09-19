# LazyTables UI/UX Specification

This document defines the user experience goals, information architecture, user flows, and visual design specifications for LazyTables' terminal user interface. It serves as the foundation for TUI design and frontend development, ensuring a cohesive and user-centered experience optimized for keyboard-driven workflows.

## Overall UX Goals & Principles

### Target User Personas
- **Terminal Power User:** Database professionals comfortable with vim-style navigation and keyboard-driven workflows who value speed and efficiency over visual polish
- **Database Developer:** Software engineers who prefer terminal-based tools for database operations and expect familiar keyboard shortcuts and modal interaction patterns
- **System Administrator:** Operations professionals who work primarily in terminal environments and need reliable, fast database access without GUI dependencies

### Usability Goals
- **Learning Efficiency:** Users familiar with vim or terminal tools can become productive within 10 minutes
- **Operational Speed:** Expert users can execute common database tasks (connect, query, browse) 50% faster than GUI tools
- **Terminal Integration:** Seamless operation in SSH sessions, tmux/screen environments, and various terminal emulators
- **Accessibility:** Full functionality available via keyboard with screen reader compatibility

### Design Principles
1. **Keyboard-First Design** - Every function accessible via keyboard shortcuts, minimal mouse dependency
2. **Spatial Awareness** - Fixed six-pane layout provides consistent spatial relationships and navigation patterns
3. **Modal Interaction** - Vim-style modes (normal, insert, visual) provide clear context for different operations
4. **Performance Over Polish** - Prioritize responsiveness and efficiency over visual effects
5. **Terminal Native** - Embrace character-based display constraints rather than fighting them

### Change Log
| Date | Version | Description | Author |
|------|---------|-------------|---------|
| 2025-01-19 | 1.0 | Initial TUI-specific specification | UX Expert |

## Information Architecture (IA)

### Terminal Layout Structure

LazyTables uses a fixed six-pane terminal layout optimized for database operations. Each pane occupies specific terminal regions and serves distinct database workflow functions.

```
┌─────────────────┬─────────────────┬───┐
│   Connections   │  Query Results  │ S │
│    (Top Left)   │   (Top Right)   │ Q │
├─────────────────┼─────────────────┤ L │
│  Tables/Views   │                 │   │
│  (Middle Left)  │                 │ F │
├─────────────────┤  SQL Editor     │ i │
│ Table Details   │ (Bottom Right)  │ l │
│ (Bottom Left)   │                 │ e │
└─────────────────┴─────────────────┤ s │
                                    └───┘
```

### Pane Relationships & Focus Flow
- **Spatial Navigation:** Ctrl+h/j/k/l moves focus between adjacent panes
- **Sequential Navigation:** Tab cycles through panes in logical workflow order
- **Context Dependencies:** Table selection updates Details pane; query execution updates Results pane

### Key Bindings Per Pane
- **Connections:** j/k navigate, Enter connect, c create new
- **Tables/Views:** j/k navigate, Enter select table, / search
- **Table Details:** Scroll with j/k, shows metadata for selected table
- **Query Results:** Arrow keys navigate cells, PgUp/PgDn for large datasets
- **SQL Editor:** Standard text editing, Ctrl+Enter execute query
- **SQL Files:** j/k navigate files, Enter load, n new file

## User Flows

### Flow 1: Database Connection Setup

**User Goal:** Connect to a database to begin work
**Entry Points:** Application startup, 'c' key in Connections pane
**Success Criteria:** Successfully connected and database objects visible

**Flow Diagram:**
```
Start Application
       ↓
Connections Pane Focus
       ↓
Press 'c' (New Connection)
       ↓
Connection Type Modal
   ↓           ↓
PostgreSQL   MySQL/SQLite
   ↓           ↓
Connection Details Form
   ↓
Test Connection
   ↓         ↓
Success    Error
   ↓         ↓
Save       Retry/Fix
   ↓
Load Database Objects
   ↓
Focus Tables Pane
```

**Edge Cases & Error Handling:**
- Invalid credentials → Clear error message in status bar, return to form
- Network timeout → Retry prompt with connection status indicator
- Unsupported database version → Warning with compatibility info
- Missing driver → Installation guidance in help text

**Notes:** Modal forms require 'i' to enter insert mode for text fields, ESC to return to navigation mode

### Flow 2: Table Exploration & Query

**User Goal:** Explore table structure and execute queries
**Entry Points:** After connection, from Tables pane navigation
**Success Criteria:** View table data and metadata successfully

**Flow Diagram:**
```
Tables Pane (j/k navigate)
       ↓
Select Table (Enter)
       ↓
Table Details Populate
       ↓
Move to SQL Editor (Ctrl+l)
       ↓
Write Query ('i' for insert mode)
       ↓
Execute (Ctrl+Enter)
       ↓
Results Display
   ↓         ↓
Navigate   Export
Results    Data
```

**Edge Cases & Error Handling:**
- Large tables → Pagination indicators, virtual scrolling
- Query syntax errors → Inline error highlighting, suggestion hints
- Long-running queries → Progress indicator, cancel option (Ctrl+C)
- No results → Clear "0 rows" message with query validation tips

**Notes:** Query editor supports multi-statement execution at cursor position

## Component Library / Design System

### Design System Approach
Custom TUI component library built on Ratatui framework, optimized for database workflows and vim-style navigation.

### Core Components

#### Table/Grid Component
**Purpose:** Display tabular data with keyboard navigation
**Variants:**
- Data grid (query results)
- List view (connections, tables)
- Detail view (table metadata)

**States:**
- Normal (browseable)
- Selected row/cell (highlighted)
- Loading (spinner/progress)
- Error (red borders, error text)

**Visual Representation:**
```
┌─────────────┬─────────────┬─────────────┐
│ Column 1 ▲  │ Column 2    │ Column 3    │
├─────────────┼─────────────┼─────────────┤
│ Data 1      │ Data 2      │ Data 3      │
│ > Selected  │ Data 4      │ Data 5      │ ← Selected row
│ Data 7      │ Data 8      │ Data 9      │
└─────────────┴─────────────┴─────────────┘
```

#### Input/Form Component
**Purpose:** Text input with vim-style modal editing
**Variants:**
- Single line (connection strings)
- Multi-line (SQL editor)
- Dropdown/Select (database types)

**States:**
- Normal mode (navigation, ESC to enter)
- Insert mode ('i' to enter, ESC to exit)
- Error state (red border, validation message)

**Visual Representation:**
```
Normal Mode:
┌─────────────────────────────────┐
│ [Database URL]                  │
└─────────────────────────────────┘

Insert Mode:
┌─────────────────────────────────┐
│ postgres://user:pass@host/db█   │ ← Cursor
└─────────────────────────────────┘
[INSERT] ← Mode indicator
```

#### Modal/Dialog Component
**Purpose:** Focused interactions without losing context
**Variants:**
- Help modal (? key)
- Connection setup
- Confirmation dialogs

**States:**
- Open (focused, backdrop dimmed)
- Closing animation (if supported)

**Visual Representation:**
```
    ┌─────────────────────────────┐
    │ ┌─ New Connection ────────┐ │
    │ │                        │ │
    │ │ Database Type:         │ │
    │ │ > PostgreSQL           │ │
    │ │   MySQL                │ │
    │ │   SQLite               │ │
    │ │                        │ │
    │ │    [Cancel] [Next]     │ │
    │ └────────────────────────┘ │
    └─────────────────────────────┘
```

#### Status/Message Component
**Purpose:** Contextual feedback without interrupting workflow
**Variants:**
- Status bar (bottom screen)
- Inline messages (within panes)
- Loading indicators

**States:**
- Info (blue/normal)
- Success (green)
- Warning (yellow)
- Error (red)

**Visual Representation:**
```
┌─────────────────────────────────────────┐
│ Mode: NORMAL | DB: postgres | Table: users | 15 rows selected │
└─────────────────────────────────────────┘
```

**Usage Guidelines:**
- All components support keyboard navigation
- Color coding follows terminal conventions (red=error, green=success, etc.)
- Focus indicators use reverse video or bright colors
- Loading states use ASCII spinners: `|/-\` rotation

## Branding & Style Guide

### Visual Identity
**Brand Guidelines:** Minimalist, efficiency-focused aesthetic that respects terminal conventions while providing clear visual hierarchy

### Color Palette

| Color Type | ANSI Code | RGB Equivalent | Usage |
|------------|-----------|----------------|--------|
| Primary | Bright Blue (94) | #5F87FF | Active selections, primary actions |
| Secondary | Cyan (96) | #5FAFFF | Secondary elements, metadata |
| Accent | Bright Green (92) | #87FF5F | Success states, confirmations |
| Success | Green (32) | #5F875F | Positive feedback, connected status |
| Warning | Yellow (33) | #FFFF5F | Cautions, important notices |
| Error | Bright Red (91) | #FF5F5F | Errors, destructive actions |
| Neutral | White/Gray (37/90) | #BCBCBC/#626262 | Text, borders, backgrounds |

### Typography

#### Character Set Strategy
- **Primary:** Terminal default monospace font (system dependent)
- **Decoration:** Unicode box-drawing characters (─ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼)
- **Icons:** ASCII symbols and select Unicode glyphs (▲ ▼ ► ◄ ● ○ ✓ ✗)

#### Text Hierarchy

| Element | Style | Weight | Usage |
|---------|-------|--------|--------|
| Titles | UPPERCASE | Bold | Pane headers, modal titles |
| Headers | Title Case | Bold | Section headers, column names |
| Labels | Sentence case | Normal | Field labels, status text |
| Data | As-is | Normal | Database content, user input |
| Help | lowercase | Dim/Italic | Hints, keyboard shortcuts |

### Iconography
**Icon Library:** ASCII + Unicode subset optimized for database operations
- Connection: ○ (disconnected) ● (connected)
- Tables: ┌─┐ (table structure)
- Status: ✓ (success) ✗ (error) ⚠ (warning)
- Navigation: ▲▼◄► (directional indicators)
- Loading: |/-\ (rotating animation)

**Usage Guidelines:** Icons must render consistently across terminal emulators, fallback to ASCII if Unicode unavailable

### Spacing & Layout
**Grid System:** Character-based grid (80x24 minimum, responsive to terminal size)
- Standard terminal: 80 columns × 24 rows
- Wide terminal: Optimize for 120+ columns
- Tall terminal: Utilize extra rows for data display

**Spacing Scale:**
- Tight: 0 characters (adjacent elements)
- Normal: 1 character (standard spacing)
- Loose: 2 characters (section separation)
- Wide: 3+ characters (major section breaks)

### Border and Frame Styles
```
Simple:     +---+
            |   |
            +---+

Unicode:    ┌───┐
            │   │
            └───┘

Double:     ╔═══╗
            ║   ║
            ╚═══╝
```

## Accessibility Requirements

### Compliance Target
**Standard:** Terminal Accessibility Guidelines + WCAG 2.1 AA principles adapted for character-based interfaces

### Key Requirements

**Visual:**
- Color contrast ratios: Minimum 4.5:1 for normal text, 3:1 for large text (tested against common terminal background colors)
- Focus indicators: Reverse video, bright colors, or Unicode symbols (█ ▓ ▒) to ensure 7:1 contrast
- Text sizing: Respect terminal font scaling, provide zoom alternatives for data grids

**Interaction:**
- Keyboard navigation: 100% keyboard accessible (already inherent in TUI design)
- Screen reader support: Proper terminal buffer management, clear focus announcements, structured output
- Touch targets: N/A for terminal interface, but ensure clear key binding documentation

**Content:**
- Alternative text: Descriptive labels for ASCII art, Unicode symbols, and visual indicators
- Heading structure: Clear hierarchical structure using consistent formatting and spacing
- Form labels: Explicit labels for all input fields, clear input mode indicators

### Screen Reader Compatibility

**Terminal Output Structure:**
```
LazyTables Database Tool
Status: Connected to PostgreSQL database 'myapp'
Active Pane: Tables List (2 of 6 panes)
Current Selection: users table (3 of 45 tables)

Tables List:
- customers (147 rows)
- orders (523 rows)
- users (89 rows) [SELECTED]
- products (234 rows)

Navigation: Use j/k to move, Enter to select, Ctrl+l to move to next pane
```

**Focus Announcements:**
- Pane changes: "Entered SQL Editor pane, insert mode available with 'i' key"
- Selection changes: "Selected users table, 89 rows, 8 columns"
- Mode changes: "Entered insert mode, type ESC to return to navigation"

### Color Accessibility

**High Contrast Mode Support:**
- Detect terminal high contrast settings
- Provide alternative color schemes for different backgrounds (dark/light/high contrast)
- Ensure information isn't conveyed through color alone (use symbols + color)

**Colorblind Considerations:**
- Avoid red/green only distinctions for success/error states
- Use symbols alongside colors: ✓ (success), ✗ (error), ⚠ (warning)
- Test with common colorblind terminal color schemes

### Testing Strategy

**Automated Testing:**
- Terminal screen reader simulation in CI pipeline
- Color contrast validation against common terminal themes
- Keyboard navigation path verification

**Manual Testing:**
- Screen reader testing with Orca, NVDA, JAWS (where terminal support available)
- High contrast terminal theme testing
- Keyboard-only navigation verification
- Testing with users who have visual impairments

## Responsiveness Strategy

### Terminal Size Categories

| Category | Width Range | Height Range | Target Use Cases |
|----------|-------------|--------------|------------------|
| Minimum | 80 cols | 24 rows | Standard terminal, SSH sessions |
| Standard | 120 cols | 30 rows | Modern terminal defaults |
| Wide | 160+ cols | 30+ rows | Ultrawide monitors, split screens |
| Tall | 80-120 cols | 40+ rows | Vertical monitors, tmux panes |

### Adaptive Layout Patterns

**Minimum Terminal (80x24):**
```
┌──────────────┬──────────────┬─┐
│ Connections  │ Query Rslt   │S│
│ (20 cols)    │ (58 cols)    │Q│
├──────────────┼──────────────┤L│
│ Tables (20)  │              │ │
├──────────────┤   SQL Edit   │F│
│ Details (20) │   (58 cols)  │i│
│              │              │l│
└──────────────┴──────────────┴─┘
```

**Wide Terminal (160x30):**
```
┌─────────────┬─────────────────────────┬─────────────┬───┐
│ Connections │     Query Results       │ Table Detls │ S │
│ (30 cols)   │       (90 cols)         │ (35 cols)   │ Q │
├─────────────┼─────────────────────────┼─────────────┤ L │
│ Tables/Vws  │                         │ Enhanced    │   │
│ (30 cols)   │     SQL Editor          │ Metadata    │ F │
│             │     (90 cols)           │ (35 cols)   │ i │
│             │                         │             │ l │
└─────────────┴─────────────────────────┴─────────────┴───┘
```

### Adaptation Strategies

**Layout Changes:**
- Below 80 cols: Hide SQL Files pane, access via modal
- Above 140 cols: Expand Table Details pane, show more metadata
- Below 20 rows: Collapse panes vertically, use tab switching
- Above 35 rows: Expand SQL Editor and Results areas

**Navigation Changes:**
- Small terminals: Enhanced keyboard shortcuts for hidden elements
- Large terminals: Mouse support for resize handles between panes
- Narrow terminals: Horizontal scrolling indicators for wide data

**Content Priority:**
- Always prioritize SQL Editor and Query Results (core functionality)
- Scale down connection/table lists to minimum viable sizes
- Hide decorative elements (borders, extra spacing) on small terminals
- Truncate long text with ellipsis (...) and expand on focus

**Interaction Changes:**
- Small screens: Modal overlays for detailed views
- Large screens: Inline expansion of details
- Dynamic help text based on available space
- Context-sensitive status bar content

## Animation & Micro-interactions

### Motion Principles

**Terminal-Specific Motion Guidelines:**
- **Functional over decorative** - Animations should provide feedback or indicate state changes
- **Performance conscious** - Avoid rapid updates that can cause flicker or performance issues
- **Accessibility first** - Provide static alternatives for users with motion sensitivity
- **Character-based transitions** - Work within discrete character grid limitations

### Key Animations

**Loading Indicators:**
- **Spinner Animation:** `|/-\` rotation (Duration: 200ms per frame, Easing: linear)
- **Progress Bars:** `[████▒▒▒▒▒▒]` filling animation (Duration: proportional to progress, Easing: linear)
- **Pulse Effect:** `● ○ ● ○` for connection status (Duration: 1000ms cycle, Easing: ease-in-out)

**Focus Transitions:**
- **Pane Focus:** Border highlight change (Duration: instant, Easing: none)
- **Selection Movement:** `>` indicator movement (Duration: instant, Easing: none)
- **Modal Fade:** Gradual background dimming using shade characters (Duration: 150ms, Easing: ease-out)

**Data Updates:**
- **Table Refresh:** Row-by-row update with brief highlight (Duration: 50ms per row, Easing: linear)
- **Status Changes:** Color transition for connection indicators (Duration: 200ms, Easing: ease-in)
- **Error Flash:** Brief red highlight then return to normal (Duration: 500ms, Easing: ease-out)

**Query Execution:**
- **Typing Indicator:** Cursor blink in SQL editor (Duration: 500ms cycle, Easing: step)
- **Execution Progress:** `Executing query...` with animated dots (Duration: 300ms per dot, Easing: linear)
- **Results Populating:** Rows appearing from top to bottom (Duration: 25ms per row, Easing: linear)

### Terminal Animation Examples

**Loading Spinner Sequence:**
```
Frame 1: | Connecting...
Frame 2: / Connecting...
Frame 3: - Connecting...
Frame 4: \ Connecting...
```

**Progress Bar Animation:**
```
[          ] 0%
[██        ] 20%
[████      ] 40%
[██████    ] 60%
[████████  ] 80%
[██████████] 100%
```

**Modal Fade Effect:**
```
Normal:    │ Regular text │
Dimmed:    ░ Regular text ░
Background: ▓ Regular text ▓
```

### Performance Considerations

**Frame Rate Targets:**
- Loading animations: 4-8 FPS (sufficient for terminal feedback)
- Status updates: Instant (no animation delay)
- Data population: 20-40 FPS (smooth enough for perceived responsiveness)

**Resource Management:**
- Limit concurrent animations to prevent terminal overwhelm
- Use character replacement rather than full screen redraws
- Batch updates when possible to reduce flicker
- Provide animation disable option for performance-critical environments

## Performance Considerations

### Performance Goals

**Rendering Performance:**
- **Initial Load:** < 100ms (application startup to first screen)
- **Pane Refresh:** < 50ms (individual pane content update)
- **Keystroke Response:** < 16ms (immediate feedback for navigation)
- **Query Execution:** First results visible within 100ms, pagination for larger datasets

**Memory Efficiency:**
- **Base Memory:** < 10MB (application without data)
- **Data Handling:** < 50MB for 10K rows displayed
- **Connection Pool:** < 5MB per active database connection
- **Background Memory:** < 2MB when minimized/background

**Network Performance (SSH/Remote):**
- **Screen Updates:** Minimize character changes per update
- **Bandwidth Usage:** < 1KB/sec for normal navigation
- **Latency Tolerance:** Remain usable at 200ms+ network latency
- **Compression:** Support terminal compression when available

### Design Strategies for Performance

**Efficient Rendering:**
- **Incremental Updates:** Only redraw changed characters, not full screen
- **Viewport Virtualization:** Render only visible rows in large datasets
- **Double Buffering:** Prepare screen updates off-screen to prevent flicker
- **Dirty Region Tracking:** Track and update only modified pane regions

**Memory Management:**
- **Lazy Loading:** Load data on-demand as user navigates
- **Data Pagination:** Limit in-memory dataset size, fetch additional data as needed
- **Connection Pooling:** Reuse database connections efficiently
- **String Interning:** Cache common strings (column names, data types)

**Network Optimization:**
- **Batch Updates:** Group multiple character changes into single terminal update
- **Smart Refresh:** Avoid redundant screen updates during rapid navigation
- **Compression:** Use terminal escape sequence compression where supported
- **Local Caching:** Cache query results and metadata to reduce database calls

**Database Query Optimization:**
- **Result Limiting:** Default to LIMIT 1000 for initial queries
- **Streaming Results:** Display results as they arrive for long-running queries
- **Query Caching:** Cache frequently executed queries and metadata
- **Connection Reuse:** Maintain persistent connections to avoid reconnection overhead

## Next Steps

### Immediate Actions

1. **Stakeholder Review and Validation**
   - Present specification to development team for technical feasibility review
   - Validate terminal performance targets against Rust/Ratatui capabilities
   - Confirm accessibility requirements align with project constraints

2. **Terminal Prototype Development**
   - Create interactive prototype of key pane layouts using Ratatui
   - Test responsive behavior across different terminal sizes (80x24 to 160x40)
   - Validate keyboard navigation patterns with vim-style movement

3. **User Testing with Terminal Environment**
   - Test with database professionals who use terminal-based tools daily
   - Validate six-pane layout efficiency against current terminal database workflows
   - Gather feedback on vim-style navigation vs traditional TUI patterns

4. **Technical Architecture Alignment**
   - Review specification with backend database connection architecture
   - Ensure UI patterns support planned database adapter implementations
   - Validate performance targets against planned query execution patterns

5. **Accessibility Validation**
   - Test specification compliance with terminal screen readers (Orca, NVDA)
   - Validate color schemes against colorblind accessibility requirements
   - Confirm keyboard navigation patterns meet accessibility standards

### Design Handoff Checklist

- [x] All user flows documented with terminal-specific considerations
- [x] Component inventory complete with ASCII/Unicode visual examples
- [x] Accessibility requirements defined for terminal environment
- [x] Responsive strategy clear for various terminal sizes
- [x] TUI-specific brand guidelines incorporated (colors, typography, icons)
- [x] Performance goals established with terminal-specific metrics

### Success Metrics

**User Experience:**
- Task completion time for common database operations (connect, query, browse)
- User preference ratings compared to existing TUI database tools
- Accessibility compliance validation with actual assistive technology users

**Technical Performance:**
- Rendering performance meets specified targets (<50ms pane refresh)
- Memory usage stays within defined limits (<50MB for typical usage)
- Network efficiency for SSH sessions (<1KB/sec normal navigation)

This comprehensive TUI-specific UI/UX specification provides the foundation for developing LazyTables' terminal interface while respecting the constraints and opportunities of character-based display environments.