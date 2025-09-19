# API Specification

LazyTables uses **internal Rust trait-based APIs** rather than HTTP REST APIs, since this is a single-process TUI application. The "API" is the interface between TUI components and database adapters.

## Database Adapter Trait Interface

**Purpose:** Unified async interface that all database adapters implement, enabling consistent TUI behavior across PostgreSQL, MySQL, SQLite, and Redis.

```rust
#[async_trait]
pub trait DatabaseAdapter: Send + Sync + Clone {
    type Connection: Send + Sync;
    type QueryResult: Send + Sync;
    type Error: Send + Sync + std::error::Error;

    // Connection Management
    async fn connect(&self, config: &ConnectionConfig) -> Result<Self::Connection, Self::Error>;
    async fn disconnect(&self, conn: Self::Connection) -> Result<(), Self::Error>;
    async fn test_connection(&self, config: &ConnectionConfig) -> Result<bool, Self::Error>;
    async fn get_connection_info(&self, conn: &Self::Connection) -> Result<ConnectionInfo, Self::Error>;

    // Schema Operations
    async fn list_databases(&self, conn: &Self::Connection) -> Result<Vec<DatabaseInfo>, Self::Error>;
    async fn list_tables(&self, conn: &Self::Connection, database: Option<&str>) -> Result<Vec<TableInfo>, Self::Error>;
    async fn get_table_schema(&self, conn: &Self::Connection, table: &TableRef) -> Result<TableSchema, Self::Error>;
    async fn get_table_indexes(&self, conn: &Self::Connection, table: &TableRef) -> Result<Vec<IndexInfo>, Self::Error>;

    // Query Operations
    async fn execute_query(&self, conn: &Self::Connection, query: &str) -> Result<Self::QueryResult, Self::Error>;
    async fn execute_statement(&self, conn: &Self::Connection, statement: &str) -> Result<u64, Self::Error>;
    async fn stream_query_results(&self, conn: &Self::Connection, query: &str) -> Result<QueryStream<Self::QueryResult>, Self::Error>;

    // Database-Specific Features
    fn supports_feature(&self, feature: DatabaseFeature) -> bool;
}
```

## Core Data Structures

```rust
// Connection configuration (stored in SQLite)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: Uuid,
    pub name: String,
    pub database_type: DatabaseType,
    pub connection_params: ConnectionParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    MariaDB,
    SQLite,
    Redis,
}

// Query results with pagination support
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Row>,
    pub affected_rows: Option<u64>,
    pub execution_time: Duration,
    pub has_more: bool,
    pub total_count: Option<u64>,
}

// Table metadata for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: Option<String>,
    pub table_type: TableType, // Table, View, MaterializedView
    pub row_count: Option<u64>,
    pub size_bytes: Option<u64>,
    pub comment: Option<String>,
}

// Database-specific features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseFeature {
    Schemas,
    Views,
    MaterializedViews,
    Triggers,
    StoredProcedures,
    FullTextSearch,
    JsonSupport,
    ArraySupport,
    TransactionSupport,
    StreamingResults,
}
```

## TUI to Adapter Communication Pattern

**Event-Driven Architecture:**
```rust
// TUI events that trigger database operations
#[derive(Debug, Clone)]
pub enum DatabaseEvent {
    ConnectToDatabase { connection_id: Uuid },
    RefreshTableList { connection_id: Uuid },
    ExecuteQuery { connection_id: Uuid, query: String },
    LoadTableData { connection_id: Uuid, table: TableRef, page: usize },
}

// Database adapter responses
#[derive(Debug, Clone)]
pub enum DatabaseResponse {
    ConnectionEstablished { connection_info: ConnectionInfo },
    TableListUpdated { tables: Vec<TableInfo> },
    QueryExecuted { result: QueryResult },
    TableDataLoaded { data: QueryResult },
    Error { error: DatabaseError },
}
```

## Async Message Passing

```rust
// Channel-based communication between TUI and adapters
pub struct DatabaseManager {
    event_sender: mpsc::UnboundedSender<DatabaseEvent>,
    response_receiver: broadcast::Receiver<DatabaseResponse>,
    adapters: HashMap<DatabaseType, Box<dyn DatabaseAdapter>>,
    active_connections: HashMap<Uuid, ActiveConnection>,
}

impl DatabaseManager {
    pub async fn handle_event(&mut self, event: DatabaseEvent) {
        match event {
            DatabaseEvent::ExecuteQuery { connection_id, query } => {
                if let Some(connection) = self.active_connections.get(&connection_id) {
                    let adapter = &self.adapters[&connection.database_type];

                    // Execute query asynchronously
                    tokio::spawn(async move {
                        let result = adapter.execute_query(&connection.handle, &query).await;
                        // Send response back to TUI
                        self.send_response(DatabaseResponse::QueryExecuted { result }).await;
                    });
                }
            }
            // ... other event handlers
        }
    }
}
```
