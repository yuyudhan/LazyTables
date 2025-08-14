# LazyTables Code Correction Plan

## Executive Summary

The LazyTables codebase has evolved into a monolithic structure with significant architectural debt. This plan outlines critical improvements needed to transform it into a maintainable, extensible, and performant terminal database management tool.

## Critical Issues Analysis

### 1. God Object Anti-Pattern - AppState (1,635 lines)

**Current Problem:**
The `AppState` struct in `src/app/state.rs` manages 35+ responsibilities including UI state, database connections, file operations, and user input - violating the Single Responsibility Principle.

**Impact:**
- Impossible to unit test individual features
- Changes to one feature affect unrelated code
- High memory footprint even for simple operations
- Difficult to understand state flow

**Solution:**
Decompose into focused state managers using the State Pattern:

```rust
pub struct AppState {
    pub ui: UIStateManager,
    pub database: DatabaseStateManager,
    pub editor: EditorStateManager,
    pub connections: ConnectionManager,
    pub commands: CommandHistory,
}

pub trait StateManager {
    fn handle_event(&mut self, event: AppEvent) -> Result<()>;
    fn get_state(&self) -> StateSnapshot;
    fn restore_state(&mut self, snapshot: StateSnapshot);
}
```

### 2. Massive Event Handler (1,380+ lines)

**Current Problem:**
Single `handle_key_event` method with 6+ levels of nested match statements mixing input parsing with business logic.

**Impact:**
- Cannot add new keybindings without modifying core code
- Impossible to test individual commands
- No support for custom keymaps
- No undo/redo capability

**Solution:**
Implement Command Pattern with Input Mapping:

```rust
pub struct InputMapper {
    keymaps: HashMap<Mode, HashMap<KeyEvent, CommandId>>,
}

pub struct CommandExecutor {
    commands: HashMap<CommandId, Box<dyn Command>>,
    history: CommandHistory,
}

pub trait Command {
    fn execute(&self, context: &mut ExecutionContext) -> Result<()>;
    fn undo(&self, context: &mut ExecutionContext) -> Result<()>;
    fn description(&self) -> &str;
}
```

### 3. UI Rendering Monolith (1,175 lines)

**Current Problem:**
All rendering logic in single file with hardcoded styles, repeated patterns, and no component reusability.

**Impact:**
- Cannot create custom UI layouts
- Theme changes require modifying dozens of locations
- No component testing possible
- Poor rendering performance

**Solution:**
Component-based UI architecture:

```rust
pub trait UIComponent: Send + Sync {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme);
    fn handle_event(&mut self, event: ComponentEvent) -> EventResult;
    fn get_constraints(&self) -> Constraints;
}

pub struct ComponentTree {
    root: Box<dyn UIComponent>,
    focus_manager: FocusManager,
    theme: Arc<Theme>,
}
```

### 4. Database Adapter Inconsistencies

**Current Problem:**
Each database adapter reimplements similar logic with different error handling and connection patterns.

**Impact:**
- Bugs fixed in one adapter don't propagate to others
- No consistent transaction support
- Connection pooling varies between databases
- Different feature sets per database

**Solution:**
Unified Database Abstraction Layer:

```rust
pub trait DatabaseConnection: Send + Sync {
    async fn execute_query(&self, query: Query) -> Result<QueryResult>;
    async fn begin_transaction(&self) -> Result<Transaction>;
    async fn get_metadata(&self, object: &DatabaseObject) -> Result<Metadata>;
}

pub struct ConnectionPool<T: DatabaseConnection> {
    connections: Vec<T>,
    config: PoolConfig,
}

pub struct QueryBuilder {
    dialect: SqlDialect,
    // Unified query building with dialect support
}
```

### 5. Missing Plugin System

**Current Problem:**
No plugin architecture despite being mentioned in documentation.

**Impact:**
- Cannot extend functionality without modifying core
- No community contribution pathway
- Features tightly coupled to core application

**Solution:**
Implement Plugin Architecture:

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> Version;
    fn init(&mut self, context: PluginContext) -> Result<()>;
    fn register_commands(&self) -> Vec<Box<dyn Command>>;
    fn register_components(&self) -> Vec<Box<dyn UIComponent>>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    loader: PluginLoader,
}
```

## Refactoring Plan

### Phase 1: Core Architecture (Week 1-2)

#### 1.1 State Management Refactoring
- [ ] Extract UIStateManager from AppState
- [ ] Extract DatabaseStateManager from AppState  
- [ ] Extract EditorStateManager from AppState
- [ ] Implement StateManager trait
- [ ] Add state persistence/restoration

#### 1.2 Command System Implementation
- [ ] Define Command trait and CommandId enum
- [ ] Implement CommandExecutor with history
- [ ] Convert key handlers to commands
- [ ] Add undo/redo support
- [ ] Create keybinding configuration

#### 1.3 Event System
- [ ] Define AppEvent enum hierarchy
- [ ] Implement EventBus with type-safe subscriptions
- [ ] Convert direct state mutations to events
- [ ] Add event logging/debugging

### Phase 2: UI Component System (Week 3-4)

#### 2.1 Component Framework
- [ ] Define UIComponent trait
- [ ] Create base components (Button, Input, List, Table)
- [ ] Implement ComponentTree with focus management
- [ ] Add component styling system

#### 2.2 Theme System
- [ ] Centralize all colors/styles into Theme struct
- [ ] Implement theme loading from TOML
- [ ] Add runtime theme switching
- [ ] Create default themes (dark, light, high-contrast)

#### 2.3 Layout System
- [ ] Create Layout trait for custom layouts
- [ ] Implement common layouts (Split, Grid, Stack)
- [ ] Add responsive layout support
- [ ] Create layout configuration system

### Phase 3: Database Abstraction (Week 5-6)

#### 3.1 Connection Management
- [ ] Define DatabaseConnection trait
- [ ] Implement connection pooling
- [ ] Add connection lifecycle management
- [ ] Create connection configuration

#### 3.2 Query Builder
- [ ] Implement type-safe query builder
- [ ] Add dialect support (PostgreSQL, MySQL, SQLite)
- [ ] Create prepared statement caching
- [ ] Add query optimization hints

#### 3.3 Transaction Support
- [ ] Define Transaction trait
- [ ] Implement transaction management
- [ ] Add savepoint support
- [ ] Create transaction event logging

### Phase 4: Plugin System (Week 7-8)

#### 4.1 Plugin Infrastructure
- [ ] Define Plugin trait
- [ ] Implement PluginLoader with dynamic loading
- [ ] Create plugin sandbox/security
- [ ] Add plugin configuration

#### 4.2 Plugin API
- [ ] Export public API for plugins
- [ ] Create plugin documentation
- [ ] Build example plugins
- [ ] Add plugin marketplace support

### Phase 5: Performance Optimization (Week 9-10)

#### 5.1 Rendering Optimization
- [ ] Implement differential rendering
- [ ] Add component-level caching
- [ ] Create render batching
- [ ] Profile and optimize hot paths

#### 5.2 Data Management
- [ ] Implement virtual scrolling for large datasets
- [ ] Add data pagination at UI level
- [ ] Create LRU cache for query results
- [ ] Optimize memory usage

#### 5.3 Async Operations
- [ ] Convert blocking operations to async
- [ ] Add operation cancellation
- [ ] Implement progress reporting
- [ ] Create background task system

## File Structure Reorganization

```
src/
├── core/
│   ├── app.rs              # Application lifecycle
│   ├── config.rs           # Configuration management
│   ├── error.rs            # Error types
│   └── event.rs            # Event system
├── state/
│   ├── mod.rs              # State management traits
│   ├── ui.rs               # UI state
│   ├── database.rs         # Database state
│   ├── editor.rs           # Editor state
│   └── connection.rs       # Connection state
├── commands/
│   ├── mod.rs              # Command system
│   ├── database.rs         # Database commands
│   ├── navigation.rs       # Navigation commands
│   ├── editing.rs          # Editing commands
│   └── registry.rs         # Command registry
├── ui/
│   ├── mod.rs              # UI system
│   ├── components/
│   │   ├── mod.rs          # Component traits
│   │   ├── basic/          # Basic components
│   │   ├── composite/      # Composite components
│   │   └── layout/         # Layout components
│   ├── theme/
│   │   ├── mod.rs          # Theme system
│   │   ├── colors.rs       # Color definitions
│   │   └── styles.rs       # Style definitions
│   └── renderer.rs         # Rendering system
├── database/
│   ├── mod.rs              # Database traits
│   ├── connection.rs       # Connection management
│   ├── query.rs            # Query builder
│   ├── transaction.rs      # Transaction support
│   └── adapters/
│       ├── postgres.rs     # PostgreSQL adapter
│       ├── mysql.rs        # MySQL adapter
│       └── sqlite.rs       # SQLite adapter
├── plugins/
│   ├── mod.rs              # Plugin system
│   ├── loader.rs           # Plugin loading
│   ├── api.rs              # Plugin API
│   └── sandbox.rs          # Plugin security
└── main.rs                 # Entry point
```

## Configuration Management

### Centralized Configuration Structure

```toml
# ~/.config/lazytables/config.toml

[general]
auto_connect = false
default_database = "postgresql"
history_size = 1000

[ui]
theme = "dark"
show_line_numbers = true
highlight_current_line = true

[keybindings]
quit = "q"
help = "?"
connect = ["Enter", "Space"]
save = ":w"

[database]
connection_timeout = 30
query_timeout = 300
pool_size = 5

[plugins]
enabled = true
directory = "~/.lazytables/plugins"
auto_load = ["vim-mode", "sql-formatter"]
```

### Theme Configuration

```toml
# ~/.config/lazytables/themes/dark.toml

[colors]
background = "#0d0d0d"
foreground = "#ffffff"
selection = "#74c7ec"
cursor = "#cba6f7"
error = "#f38ba8"
warning = "#f9e2af"
success = "#a6e3a1"

[components.table]
border = "#313244"
header_bg = "#1e1e2e"
header_fg = "#cba6f7"
row_even = "#11111b"
row_odd = "#0d0d0d"

[components.input]
background = "#1e1e2e"
border = "#313244"
border_focused = "#74c7ec"
```

## Testing Strategy

### Unit Testing
- Test individual commands in isolation
- Test state managers independently
- Test UI components with mock renderers
- Test database adapters with mock connections

### Integration Testing
- Test command execution flow
- Test state synchronization
- Test UI event handling
- Test database operations

### Performance Testing
- Benchmark rendering performance
- Test with large datasets (1M+ rows)
- Profile memory usage
- Test plugin loading performance

## Migration Guide

### For Current Codebase

1. **Start with non-breaking changes:**
   - Extract constants and magic numbers
   - Create type aliases for common patterns
   - Add documentation to existing code

2. **Incremental refactoring:**
   - Extract methods from large functions
   - Create traits for common behavior
   - Move related code into modules

3. **Gradual architecture migration:**
   - Implement new features using new architecture
   - Migrate existing features one at a time
   - Maintain backward compatibility during transition

## Task for Claude

### High-Level Implementation Task

```markdown
## Task: Refactor LazyTables Architecture

### Objective
Transform the monolithic LazyTables codebase into a modular, extensible, and maintainable architecture following the CODE_CORRECTION_PLAN.md.

### Phase 1 Requirements (Priority: Critical)

1. **State Management Refactoring**
   - Split AppState into focused state managers (UIState, DatabaseState, EditorState, ConnectionState)
   - Implement StateManager trait with save/restore capabilities
   - Create state synchronization through event system
   - Ensure no breaking changes to existing functionality

2. **Command System Implementation**
   - Extract all key handling logic into discrete Command implementations
   - Create CommandRegistry with dynamic command registration
   - Implement CommandExecutor with undo/redo support
   - Build InputMapper for configurable keybindings
   - Maintain current keybinding behavior as default

3. **Event System Creation**
   - Design AppEvent hierarchy for all application events
   - Implement EventBus with type-safe publish/subscribe
   - Convert direct state mutations to event dispatching
   - Add event logging for debugging

### Phase 2 Requirements (Priority: High)

4. **UI Component System**
   - Define UIComponent trait with render and event handling
   - Extract repeated UI patterns into reusable components
   - Create ComponentTree for hierarchical rendering
   - Implement focus management system

5. **Theme System Centralization**
   - Move all hardcoded colors/styles to Theme struct
   - Implement theme loading from TOML configuration
   - Add runtime theme switching capability
   - Create at least 3 themes: dark, light, high-contrast

6. **Configuration Management**
   - Create unified configuration structure
   - Implement configuration loading from ~/.config/lazytables
   - Add configuration validation and defaults
   - Support configuration hot-reloading

### Phase 3 Requirements (Priority: Medium)

7. **Database Abstraction Layer**
   - Create DatabaseConnection trait for all adapters
   - Implement QueryBuilder with dialect support
   - Add connection pooling with configurable parameters
   - Ensure consistent error handling across adapters

8. **Plugin System Architecture**
   - Define Plugin trait with lifecycle methods
   - Implement PluginLoader with dynamic library loading
   - Create plugin API for extending functionality
   - Build example plugin demonstrating capabilities

### Implementation Guidelines

- **Preserve all existing functionality** - no features should be removed
- **Maintain backward compatibility** where possible
- **Write comprehensive tests** for new components
- **Document all public APIs** with examples
- **Use Rust best practices** including proper error handling
- **Optimize for performance** - lazy loading, caching, async operations
- **Follow the file structure** outlined in CODE_CORRECTION_PLAN.md

### Success Criteria

1. All existing features continue to work
2. Code files reduced to <500 lines each
3. No more than 3 levels of nesting in any function
4. 80%+ test coverage for new code
5. Plugin system can load external plugins
6. Theme can be changed at runtime
7. Configuration fully externalized
8. Performance improved or maintained

### Deliverables

1. Refactored codebase following new architecture
2. Comprehensive test suite
3. Updated documentation
4. Example plugin implementation
5. Migration guide for existing users
```

## Conclusion

This correction plan addresses the fundamental architectural issues in LazyTables while providing a clear path forward. The refactoring should be done incrementally to maintain stability while improving the codebase. The end result will be a professional-grade, extensible terminal database management tool that can grow with community contributions through its plugin system.