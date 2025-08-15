# 005 - Database Support

This document outlines the current and planned database support for LazyTables, including implementation details for database adapters.

## Current Support

### PostgreSQL ✅
**Status**: Primary focus, full implementation

- **Driver**: `sqlx` with PostgreSQL features
- **Connection**: Async connection pooling
- **Features**: 
  - Full schema introspection
  - Query execution and result display
  - Table metadata and constraints
  - Index information
  - Stored procedures and functions

## Planned Database Support

### Phase 1 - Core Databases

#### MySQL / MariaDB 🚧
**Status**: Planned for immediate implementation after PostgreSQL

- **Driver**: `sqlx` with MySQL features
- **Compatibility**: Both MySQL and MariaDB
- **Features**:
  - Schema introspection
  - Query execution
  - Stored procedures
  - Trigger support

#### SQLite 🚧
**Status**: Planned for Phase 1

- **Driver**: `sqlx` with SQLite features
- **Features**:
  - Local file databases
  - In-memory databases
  - Pragma commands
  - Attached databases

### Phase 2 - Enterprise Databases

#### Oracle Database 📋
**Status**: Planned

- **Driver**: TBD (potentially `oracle` crate)
- **Challenges**: Complex licensing, connection setup
- **Features**:
  - Schema introspection
  - PL/SQL support
  - Package and procedure browsing

#### IBM DB2 📋
**Status**: Planned

- **Driver**: ODBC-based connection
- **Features**:
  - Basic query support
  - Schema browsing
  - Catalog views

#### ClickHouse 📋
**Status**: Planned

- **Driver**: `clickhouse` crate
- **Features**:
  - Column-oriented queries
  - Performance optimizations
  - Large dataset handling

### Phase 3 - NoSQL and Specialized

#### Redis 📋
**Status**: Future consideration

- **Driver**: `redis` crate
- **Features**:
  - Key-value browsing
  - Data type specific views
  - Command execution
  - Cluster support

#### MongoDB 📋
**Status**: Future consideration

- **Driver**: `mongodb` crate
- **Features**:
  - Document browsing
  - Collection management
  - Aggregation pipeline
  - Index management

## Database Adapter Architecture

### Core Trait Definition

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait DatabaseAdapter: Send + Sync {
    /// Connect to the database
    async fn connect(&self, config: &ConnectionConfig) -> Result<Box<dyn Connection>>;
    
    /// Test connection health
    async fn health_check(&self, conn: &dyn Connection) -> Result<bool>;
    
    /// Get adapter information
    fn adapter_info(&self) -> AdapterInfo;
}

#[async_trait]
pub trait Connection: Send + Sync {
    /// List available databases/schemas
    async fn list_databases(&self) -> Result<Vec<DatabaseInfo>>;
    
    /// List tables in a database
    async fn list_tables(&self, database: &str) -> Result<Vec<TableInfo>>;
    
    /// Get table schema information
    async fn get_table_schema(&self, database: &str, table: &str) -> Result<TableSchema>;
    
    /// Execute a query
    async fn execute_query(&self, query: &str) -> Result<QueryResult>;
    
    /// Close the connection
    async fn close(&self) -> Result<()>;
}
```

### Configuration Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub name: String,
    pub database_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub database: Option<String>,
    pub username: String,
    pub password: String, // Encrypted in storage
    pub ssl: bool,
    pub connection_params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    MariaDB,
    SQLite,
    Oracle,
    DB2,
    ClickHouse,
    Redis,
    MongoDB,
}
```

### Result Types

```rust
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Row>,
    pub affected_rows: Option<u64>,
    pub execution_time: Duration,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Row {
    pub values: Vec<Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Date(chrono::NaiveDate),
    DateTime(chrono::DateTime<chrono::Utc>),
}
```

## Implementation Guidelines

### Adding a New Database Adapter

1. **Create adapter file**: `src/database/{database_name}.rs`
2. **Implement traits**: `DatabaseAdapter` and `Connection`
3. **Add configuration**: Update `DatabaseType` enum
4. **Add dependencies**: Update `Cargo.toml` with required drivers
5. **Add tests**: Create integration tests
6. **Update documentation**: Document the new adapter

### Example: PostgreSQL Adapter

```rust
// src/database/postgres.rs
use async_trait::async_trait;
use sqlx::{PgPool, Row};

pub struct PostgresAdapter;

#[async_trait]
impl DatabaseAdapter for PostgresAdapter {
    async fn connect(&self, config: &ConnectionConfig) -> Result<Box<dyn Connection>> {
        let connection_string = format!(
            "postgresql://{}:{}@{}:{}/{}",
            config.username,
            config.password,
            config.host,
            config.port,
            config.database.as_deref().unwrap_or("postgres")
        );
        
        let pool = PgPool::connect(&connection_string).await?;
        Ok(Box::new(PostgresConnection { pool }))
    }
    
    fn adapter_info(&self) -> AdapterInfo {
        AdapterInfo {
            name: "PostgreSQL".to_string(),
            version: "1.0.0".to_string(),
            features: vec![
                "schema_introspection".to_string(),
                "stored_procedures".to_string(),
                "transactions".to_string(),
            ],
        }
    }
}

struct PostgresConnection {
    pool: PgPool,
}

#[async_trait]
impl Connection for PostgresConnection {
    async fn list_databases(&self) -> Result<Vec<DatabaseInfo>> {
        let rows = sqlx::query("SELECT datname FROM pg_database WHERE datistemplate = false")
            .fetch_all(&self.pool)
            .await?;
            
        let databases = rows
            .into_iter()
            .map(|row| DatabaseInfo {
                name: row.get("datname"),
                schema_count: None,
                table_count: None,
            })
            .collect();
            
        Ok(databases)
    }
    
    async fn list_tables(&self, database: &str) -> Result<Vec<TableInfo>> {
        let query = r#"
            SELECT 
                table_name,
                table_type,
                table_schema
            FROM information_schema.tables 
            WHERE table_catalog = $1
            AND table_schema NOT IN ('information_schema', 'pg_catalog')
            ORDER BY table_schema, table_name
        "#;
        
        let rows = sqlx::query(query)
            .bind(database)
            .fetch_all(&self.pool)
            .await?;
            
        // Process rows into TableInfo structs...
        todo!("Implementation continues...")
    }
    
    // Other trait methods...
}
```

### Connection Management

#### Connection Pooling

```rust
pub struct ConnectionPool {
    pools: HashMap<String, Box<dyn Connection>>,
    config: PoolConfig,
}

impl ConnectionPool {
    pub async fn get_connection(&mut self, name: &str) -> Result<&dyn Connection> {
        if !self.pools.contains_key(name) {
            let config = self.load_connection_config(name)?;
            let adapter = self.create_adapter(&config.database_type);
            let connection = adapter.connect(&config).await?;
            self.pools.insert(name.to_string(), connection);
        }
        
        Ok(self.pools.get(name).unwrap().as_ref())
    }
}
```

#### Health Monitoring

```rust
impl ConnectionPool {
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();
        
        for (name, connection) in &self.pools {
            let health = connection.health_check().await.unwrap_or(false);
            results.insert(name.clone(), health);
        }
        
        results
    }
}
```

## Database-Specific Features

### PostgreSQL Specifics
- **Extensions**: Display installed extensions
- **Schemas**: Full schema support
- **JSONB**: Special handling for JSON columns
- **Arrays**: Array type visualization
- **Custom Types**: Enum and composite type support

### MySQL Specifics  
- **Storage Engines**: Display table storage engines
- **Partitioning**: Partition information
- **Replication**: Master/slave status
- **Character Sets**: Collation information

### SQLite Specifics
- **File Info**: Database file size and location
- **Pragmas**: Common pragma settings
- **Attached DBs**: Multiple database support
- **FTS**: Full-text search capabilities

## Performance Considerations

### Query Optimization
- **Prepared Statements**: Use for repeated queries
- **Result Streaming**: Handle large result sets
- **Connection Reuse**: Maintain persistent connections
- **Query Caching**: Cache metadata queries

### Memory Management
- **Lazy Loading**: Load data on demand
- **Result Pagination**: Limit result set sizes
- **Connection Limits**: Prevent connection exhaustion

## Testing Database Adapters

### Integration Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_postgres_connection() {
        let config = ConnectionConfig {
            name: "test".to_string(),
            database_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            database: Some("test_db".to_string()),
            username: "lazytables".to_string(),
            password: "lazytables".to_string(),
            ssl: false,
            connection_params: HashMap::new(),
        };
        
        let adapter = PostgresAdapter;
        let connection = adapter.connect(&config).await.unwrap();
        
        let databases = connection.list_databases().await.unwrap();
        assert!(!databases.is_empty());
    }
}
```

### Test Database Setup

Each database adapter should include:
- **Docker setup**: For consistent testing environments
- **Test data**: Sample schemas and data
- **CI integration**: Automated testing in pipelines

This architecture allows LazyTables to support multiple database types while maintaining consistent behavior and performance across all adapters.