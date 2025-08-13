# 002 - Architecture

This document outlines the overall architecture and design principles of LazyTables.

## Design Philosophy

LazyTables is built around these core principles:

- **Vim-First**: Every interaction should feel natural to vim users
- **Performance**: Sub-100ms startup, 60fps scrolling, minimal memory usage
- **Simplicity**: Four-pane layout that doesn't overwhelm
- **Extensibility**: Plugin system for future customization
- **Terminal Native**: Designed specifically for terminal environments

## Application Architecture

### Four-Pane Layout

LazyTables uses a fixed four-pane layout optimized for database workflows:

```
┌─────────────┬─────────────────────────────┐
│ Connections │                             │
├─────────────┤                             │
│ Tables/     │        Main Content         │
│ Views       │          Area               │
├─────────────┤                             │
│ Table       │                             │
│ Details     │                             │
└─────────────┴─────────────────────────────┘
```

1. **Connections Pane** (Top Left): Database connection management
2. **Tables/Views Pane** (Middle Left): Navigate database objects  
3. **Table Details Pane** (Bottom Left): Metadata about selected table
4. **Main Content Area** (Right): Query editor and results viewer

### Navigation System

#### Multiple Modes

- **Normal Mode** (default): Navigation and commands
- **Insert Mode**: Direct cell editing
- **Visual Mode**: Row/column selection
- **Query Mode**: SQL query composition
- **Command Mode**: Complex operations

#### Key Binding System

```rust
// Example key binding structure
pub enum KeyBinding {
    // Navigation
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    
    // Pane switching
    SwitchPaneUp,
    SwitchPaneDown,
    SwitchPaneLeft,
    SwitchPaneRight,
    
    // Mode switching
    EnterInsertMode,
    EnterVisualMode,
    EnterCommandMode,
    EnterQueryMode,
    
    // Actions
    Quit,
    AddConnection,
    DeleteConnection,
}
```

## Component Architecture

### Core Components

```
LazyTables Application
├── Terminal Management
├── Event System
├── State Management
├── UI Rendering
├── Database Layer
└── Configuration
```

### State Management

The application uses a centralized state management system:

```rust
pub struct AppState {
    pub current_mode: Mode,
    pub active_pane: PaneType,
    pub connections: Vec<Connection>,
    pub selected_connection: Option<ConnectionId>,
    pub selected_database: Option<String>,
    pub selected_table: Option<String>,
    pub query_buffer: String,
    pub results: QueryResults,
}
```

### Event System

Events flow through the system in a predictable pattern:

1. **Input Capture**: Terminal input is captured
2. **Event Processing**: Raw input converted to application events
3. **State Update**: Events modify application state
4. **UI Update**: State changes trigger UI re-render

```rust
pub enum AppEvent {
    KeyPress(KeyEvent),
    DatabaseEvent(DatabaseEvent),
    UIEvent(UIEvent),
    Quit,
}
```

## Database Layer

### Adapter Pattern

Database connections use an adapter pattern for extensibility:

```rust
pub trait DatabaseAdapter {
    async fn connect(&self, config: &ConnectionConfig) -> Result<Connection>;
    async fn list_databases(&self, conn: &Connection) -> Result<Vec<String>>;
    async fn list_tables(&self, conn: &Connection, db: &str) -> Result<Vec<Table>>;
    async fn execute_query(&self, conn: &Connection, query: &str) -> Result<QueryResult>;
}
```

### Connection Management

Connections are managed asynchronously with connection pooling:

- Lazy connection establishment
- Automatic reconnection on failure
- Connection health monitoring
- Secure credential storage

## UI Architecture

### Ratatui Integration

LazyTables uses [Ratatui](https://ratatui.rs/) for terminal UI:

- **Widgets**: Custom widgets for database-specific needs
- **Layouts**: Constraint-based layout system
- **Themes**: Configurable color schemes
- **Events**: Non-blocking event handling

### Widget Hierarchy

```
App
├── ConnectionsPane
│   ├── ConnectionList
│   └── AddConnectionDialog
├── TablesPane
│   ├── DatabaseList  
│   └── TableList
├── TableDetailsPane
│   ├── SchemaView
│   └── MetadataView
└── MainContentArea
    ├── QueryEditor
    └── ResultsView
        ├── TableView
        └── StatusBar
```

## Performance Considerations

### Memory Management

- **Lazy Loading**: Only load visible data
- **Virtual Scrolling**: Handle large datasets efficiently
- **Connection Pooling**: Reuse database connections
- **Result Caching**: Cache recent query results

### Startup Optimization

- **Minimal Dependencies**: Keep binary size small
- **Lazy Initialization**: Only initialize what's needed
- **Configuration Loading**: Fast config parsing
- **UI Rendering**: Efficient initial render

### Runtime Performance

- **Async Operations**: Non-blocking database operations
- **Event Batching**: Batch UI updates
- **Efficient Rendering**: Only re-render changed areas
- **Memory Pooling**: Reuse allocated memory

## Security Architecture

### Credential Management

- **Encrypted Storage**: Credentials encrypted at rest
- **Memory Safety**: Clear sensitive data from memory
- **OS Integration**: Use OS keychain when available
- **Connection Validation**: Validate connections before storing

### SQL Injection Prevention

- **Parameterized Queries**: Use prepared statements
- **Input Validation**: Sanitize user input
- **Permission Checking**: Validate database permissions
- **Audit Logging**: Log security-relevant events

## Plugin System (Future)

The architecture is designed to support future plugin development:

```rust
pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, app: &mut App) -> Result<()>;
    fn handle_event(&mut self, event: &AppEvent) -> Result<Option<AppEvent>>;
}
```

## Error Handling

LazyTables uses comprehensive error handling:

```rust
#[derive(Debug, thiserror::Error)]
pub enum LazyTablesError {
    #[error("Database connection failed: {0}")]
    DatabaseConnection(#[from] DatabaseError),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Terminal error: {0}")]
    Terminal(#[from] crossterm::ErrorKind),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Configuration System

Configuration is loaded from multiple sources in order of precedence:

1. Command line arguments
2. Environment variables  
3. User configuration file (`~/.config/lazytables/config.toml`)
4. System configuration file
5. Built-in defaults

This architecture ensures LazyTables is both performant and extensible while maintaining a clean, vim-like user experience.