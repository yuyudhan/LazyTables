# TUI Architecture

LazyTables implements a **six-pane fixed layout** with vim-style navigation and async state management optimized for terminal performance.

## Pane Layout Architecture

```
┌─ Connections ─┬─ Query Results ────────────────────┐
│ ● postgres-1  │ id │ name     │ email            │
│ ○ mysql-prod  │ 1  │ John Doe │ john@example.com │
│ ○ redis-cache │ 2  │ Jane     │ jane@example.com │
├─ Tables ──────┼─ SQL Query Editor ─────────────────┤
│ ▸ users       │ SELECT id, name, email            │
│ ▸ products    │ FROM users                        │
│ ▸ orders      │ WHERE created_at > '2024-01-01'   │
├─ Details ─────┼─ SQL Files ───┐                   │
│ Table: users  │ queries.sql   │                   │
│ Columns: 15   │ reports.sql   │                   │
│ Rows: 1.2M    │ > current.sql │                   │
└───────────────┴───────────────┴───────────────────┘
```

## Component State Management

```rust
#[derive(Debug, Clone)]
pub struct AppState {
    // Core state
    pub active_pane: PaneType,
    pub vim_mode: VimMode, // Normal, Insert, Visual, Command

    // Connection state
    pub connections: Vec<Connection>,
    pub active_connection_id: Option<Uuid>,
    pub connection_status: HashMap<Uuid, ConnectionStatus>,

    // Database state
    pub current_tables: Vec<TableInfo>,
    pub selected_table: Option<TableRef>,
    pub table_metadata_cache: HashMap<String, TableSchema>,

    // Query state
    pub query_editor: QueryEditorState,
    pub query_results: Option<QueryResult>,
    pub query_history: VecDeque<QueryHistoryItem>,

    // File state
    pub sql_files: Vec<SqlFile>,
    pub current_file: Option<PathBuf>,
    pub file_content: String,

    // UI state
    pub error_message: Option<String>,
    pub status_message: Option<String>,
    pub loading_operations: HashSet<OperationType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneType {
    Connections,
    Tables,
    Details,
    QueryResults,
    QueryEditor,
    SqlFiles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Command,
    Query, // Full-screen query editing
}
```

## Event-Driven Update System

```rust
#[derive(Debug, Clone)]
pub enum AppEvent {
    // Input events
    KeyPressed(KeyEvent),
    FocusChanged(PaneType),

    // Database events
    ConnectionSelected(Uuid),
    TableSelected(TableRef),
    QueryExecuted(String),

    // File events
    FileSelected(PathBuf),
    FileContentChanged(PathBuf, String),

    // System events
    DatabaseResponse(DatabaseResponse),
    ErrorOccurred(String),
    StatusUpdate(String),
}

// Update cycle: Event -> State Change -> UI Render
impl App {
    pub async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::KeyPressed(key) => {
                match (self.state.vim_mode, self.state.active_pane, key.code) {
                    // Vim navigation in normal mode
                    (VimMode::Normal, _, KeyCode::Char('h')) => self.navigate_left(),
                    (VimMode::Normal, _, KeyCode::Char('l')) => self.navigate_right(),
                    (VimMode::Normal, _, KeyCode::Char('j')) => self.navigate_down(),
                    (VimMode::Normal, _, KeyCode::Char('k')) => self.navigate_up(),

                    // Pane switching
                    (VimMode::Normal, _, KeyCode::Char('c')) => self.focus_pane(PaneType::Connections),
                    (VimMode::Normal, _, KeyCode::Char('t')) => self.focus_pane(PaneType::Tables),
                    (VimMode::Normal, _, KeyCode::Char('q')) => self.focus_pane(PaneType::QueryEditor),

                    // Mode switching
                    (VimMode::Normal, PaneType::QueryEditor, KeyCode::Char('i')) => {
                        self.state.vim_mode = VimMode::Insert;
                    },
                    (VimMode::Insert, _, KeyCode::Esc) => {
                        self.state.vim_mode = VimMode::Normal;
                    },

                    // Query execution
                    (_, PaneType::QueryEditor, KeyCode::F(5)) => {
                        self.execute_current_query().await?;
                    },

                    _ => {} // Unhandled key combinations
                }
            },

            AppEvent::DatabaseResponse(response) => {
                self.handle_database_response(response).await?;
            },

            _ => {} // Other event handlers
        }

        // Trigger UI re-render
        self.should_redraw = true;
        Ok(())
    }
}
```

## Performance Optimizations

**Virtual Scrolling:**
- Query results use viewport rendering for large datasets
- Only visible rows are rendered, supporting millions of results
- Smooth 60fps scrolling with lazy loading

**Async Operations:**
- All database operations are non-blocking
- Connection status updates in background
- Query execution with progress indication

**Memory Management:**
- Connection pooling prevents repeated connection overhead
- Table metadata caching reduces database round-trips
- Query result streaming for large datasets

**Terminal Optimization:**
- Minimal redraw cycles, only changed regions updated
- Efficient terminal buffer management
- Graceful degradation for limited terminal capabilities
