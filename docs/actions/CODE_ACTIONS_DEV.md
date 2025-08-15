# LazyTables Code Actions - Development Tasks

## Overview

Specific, actionable tasks to transform LazyTables from a monolithic codebase into a modular, maintainable, and extensible application. Each task includes exact file locations, line numbers, and concrete implementation steps.

---

## CRITICAL: Files Doing Too Much (>500 lines)

### Task 1: Break Down Monster Event Handler in app/mod.rs

**File:** `src/app/mod.rs` (1,591 lines)
**Problem:** Single `handle_key_event()` function spans 849 lines (192-1041)

**Actions:**

1. Create `src/app/handlers/` directory
2. Extract to separate files:
    - `src/app/handlers/normal_mode.rs` - Lines 359-846
    - `src/app/handlers/query_mode.rs` - Lines 847-1003
    - `src/app/handlers/command_mode.rs` - Lines 1004-1041
3. Create `KeyEventHandler` trait:

    ```rust
    pub trait KeyEventHandler {
        async fn handle(&mut self, key: KeyEvent, state: &mut AppState) -> Result<()>;
    }
    ```

4. Replace 849-line function with 20-line dispatcher

**Impact:** Reduces app/mod.rs by 53%

---

### Task 2: Split God Object AppState

**File:** `src/app/state.rs` (1,438 lines)
**Problem:** Mixing UI state, database operations, and business logic

**Actions:**

1. Already has `src/state/ui.rs` - MOVE remaining UI methods (lines 276-400)
2. Already has `src/state/database.rs` - MOVE database methods:
    - `execute_query()` - Lines 1029-1076
    - `delete_table_row()` - Lines 1084-1089
    - `load_table_data()` - Lines 950-1028
    - `update_table_cell()` - Lines 1077-1083
3. Create `src/state/query_editor.rs` - MOVE:
    - All query editing methods - Lines 600-750
    - `get_statement_under_cursor()` - Lines 751-820
4. Keep only coordination logic in main state.rs

**Impact:** Splits 1,438 lines into 4 files of ~350 lines each

---

## CRITICAL: Code Duplication Issues

### Task 3: Eliminate Insert Mode Duplication

**Problem:** Same insert mode pattern in 3 components
**Locations:**

- `src/ui/components/connection_modal.rs:49`
- `src/ui/components/table_editor.rs:28`
- `src/ui/components/table_creator.rs:350`

**Actions:**

1. Create `src/ui/traits/insert_mode.rs`:

    ```rust
    pub trait InsertMode {
        fn enter_insert_mode(&mut self);
        fn exit_insert_mode(&mut self);
        fn is_insert_mode(&self) -> bool;
        fn handle_insert_char(&mut self, c: char);
        fn handle_backspace(&mut self);
    }
    ```

2. Implement trait for all 3 components
3. Remove duplicate fields and methods
4. Update key handlers to use trait methods

**Impact:** Removes 81 lines of duplicate code

---

### Task 4: Extract Modal Key Handler Pattern

**Problem:** 3 nearly identical modal key handlers (564 lines total)
**Locations:**

- `src/app/mod.rs:1043-1234` - handle_connection_modal_key_event
- `src/app/mod.rs:1236-1409` - handle_table_creator_key_event
- `src/app/mod.rs:1411-1583` - handle_table_editor_key_event

**Actions:**

1. Create `src/app/handlers/modal_handler.rs`
2. Extract common pattern:

    ```rust
    pub struct ModalKeyHandler<T: ModalState> {
        handle_escape: fn(&mut T),
        handle_tab: fn(&mut T),
        handle_enter: fn(&mut T) -> Result<()>,
    }
    ```

3. Implement for each modal type
4. Replace 564 lines with ~100 lines

**Impact:** Removes 464 lines of duplicate code

---

### Task 5: Unify Database Connection Pattern

**Problem:** Identical connection structs in 3 database modules
**Locations:**

- `src/database/postgres.rs:13-30`
- `src/database/mysql.rs:13-30`
- `src/database/sqlite.rs:13-30`

**Actions:**

1. Create `src/database/connection_base.rs`:

    ```rust
    pub struct ConnectionBase<T> {
        config: ConnectionConfig,
        pool: Option<T>,
    }
    ```

2. Update each adapter to use base
3. Extract common methods to trait default implementations

**Impact:** Removes 45 lines of duplicate code

---

## HIGH: Reusability Improvements

### Task 6: Create Centralized Error Formatting

**Problem:** 21 instances of identical error formatting
**Pattern:** `.error(format!("Failed to {}: {}", action, error))`

**Actions:**

1. Create `src/core/error_formatter.rs`:

    ```rust
    pub trait ErrorFormatter {
        fn format_error(&self, action: &str, error: impl Display) -> String;
        fn format_success(&self, action: &str) -> String;
    }
    ```

2. Add to ToastManager:

    ```rust
    pub fn error_with_action(&mut self, action: &str, error: impl Display)
    ```

3. Replace all 21 occurrences

**Impact:** Consistent error messages, 42 lines removed

---

### Task 7: Extract Table Rendering Logic

**Problem:** Table rendering logic scattered across 4 files
**Locations:**

- `src/ui/mod.rs:650-754` - draw_tabular_output
- `src/ui/components/table_viewer.rs:487-928`
- `src/ui/components/table_creator.rs:500-650`
- `src/ui/components/table_editor.rs:400-550`

**Actions:**

1. Create `src/ui/widgets/table.rs`
2. Extract common `TableWidget` struct
3. Implement builder pattern for configuration
4. Reuse in all 4 locations

**Impact:** Removes ~300 lines of duplicate rendering

---

## HIGH: Dead Code Removal

### Task 8: Remove Duplicate Mode Enum

**Location:** `src/app/mod.rs:18-26`
**Problem:** Internal Mode enum duplicates UI state modes

**Actions:**

1. Remove Mode enum definition
2. Use `crate::app::state::QueryEditMode` directly
3. Update all 15 references

**Impact:** Removes confusion and 9 lines

---

### Task 9: Remove Unused Vim Command Fields

**Problem:** Vim command fields exist but aren't properly used
**Locations:**

- `src/state/ui.rs:30-31` - vim_command_buffer, in_vim_command
- Partial implementation in query mode

**Actions:**

1. Either complete vim command implementation OR
2. Remove fields and simplify to basic editing

**Impact:** Clarifies intent, removes 50+ lines of partial code

---

## MEDIUM: Organization Improvements

### Task 10: Create Plugin System Foundation

**Problem:** No plugin system exists despite being planned

**Actions:**

1. Create `src/plugins/mod.rs`:

    ```rust
    pub trait Plugin {
        fn name(&self) -> &str;
        fn version(&self) -> &str;
        fn on_load(&mut self, context: &mut PluginContext) -> Result<()>;
        fn on_command(&mut self, cmd: &str, context: &mut PluginContext) -> Result<()>;
    }
    ```

2. Create `src/plugins/loader.rs` for dynamic loading
3. Add plugin discovery in `~/.lazytables/plugins/`
4. Create example plugin in `plugins/example_logger/`

**Impact:** Enables extensibility

---

### Task 11: Implement Lazy Loading for Tables

**Problem:** Loading entire tables into memory (see table_viewer.rs)
**Location:** `src/ui/components/table_viewer.rs:50-70`

**Actions:**

1. Add to `TableTab`:

    ```rust
    pub viewport_start: usize,
    pub viewport_size: usize,
    pub total_rows: usize,
    pub cached_rows: HashMap<Range<usize>, Vec<Vec<String>>>,
    ```

2. Implement `load_viewport()` method
3. Modify render to only show viewport
4. Add prefetching for smooth scrolling

**Impact:** Handles 1M+ row tables efficiently

---

### Task 12: Extract SQL Parsing Logic

**Problem:** SQL parsing logic mixed with UI in multiple places
**Locations:**

- `src/app/state.rs:751-820` - get_statement_under_cursor
- `src/ui/mod.rs:896-920` - SQL highlighting attempt

**Actions:**

1. Create `src/sql/parser.rs`
2. Implement proper SQL tokenizer
3. Extract statement detection logic
4. Add SQL syntax highlighting support

**Impact:** Reusable SQL handling, better highlighting

---

### Task 13: Create Component Registry

**Problem:** UI components are hardcoded, not extensible
**Location:** `src/ui/mod.rs` - All draw\_\* methods

**Actions:**

1. Create `src/ui/registry.rs`:

    ```rust
    pub trait UIComponent {
        fn id(&self) -> &str;
        fn render(&self, frame: &mut Frame, area: Rect, state: &AppState);
        fn handle_event(&mut self, event: KeyEvent) -> Result<()>;
    }
    ```

2. Register all panes as components
3. Allow dynamic component addition

**Impact:** Enables UI plugins

---

### Task 14: Implement Proper Transactions

**Problem:** No transaction support for multi-step operations
**Location:** Database operations throughout `state.rs`

**Actions:**

1. Add to Connection trait:

    ```rust
    async fn begin_transaction(&mut self) -> Result<Transaction>;
    async fn commit(&mut self, tx: Transaction) -> Result<()>;
    async fn rollback(&mut self, tx: Transaction) -> Result<()>;
    ```

2. Wrap multi-step operations in transactions
3. Add rollback on errors

**Impact:** Data integrity

---

### Task 15: Extract Configuration Management

**Problem:** Configuration scattered across codebase
**Locations:** Various hardcoded values and paths

**Actions:**

1. Expand `src/config/mod.rs`
2. Add sections for:
    - Keybindings
    - UI preferences
    - Database defaults
    - Plugin settings
3. Load from `~/.lazytables/config.toml`
4. Support hot-reloading

**Impact:** User customization

---

## LOW: Performance & Quality

### Task 16: Add Render Caching

**Problem:** Full re-render on every frame
**Location:** `src/ui/mod.rs:79-125`

**Actions:**

1. Add dirty flags to state
2. Cache rendered widgets
3. Only re-render changed panes
4. Implement differential rendering

**Impact:** 50% reduction in CPU usage

---

### Task 17: Implement Connection Pooling

**Problem:** Creating new connections per operation
**Location:** Database modules

**Actions:**

1. Add proper connection pool management
2. Implement connection health checks
3. Add automatic reconnection
4. Configure pool sizes

**Impact:** Better resource usage

---

### Task 18: Add Comprehensive Tests

**Problem:** No test coverage
**Locations:** Entire codebase

**Actions:**

1. Add unit tests for state management
2. Add integration tests for database operations
3. Add UI snapshot tests
4. Mock database connections for testing

**Impact:** Confidence in refactoring

---

