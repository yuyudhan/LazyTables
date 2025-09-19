// FilePath: src/database/sqlite.rs

use crate::core::error::{LazyTablesError, Result};
use crate::database::{
    connection::ConnectionConfig, Connection, DataType, TableColumn, TableMetadata,
};
use async_trait::async_trait;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::{Column, Row};
use std::path::Path;

/// SQLite database connection implementation
pub struct SqliteConnection {
    config: ConnectionConfig,
    pool: Option<SqlitePool>,
}

impl SqliteConnection {
    /// Create a new SQLite connection instance
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config, pool: None }
    }

    /// Build SQLite connection string
    fn build_connection_string(&self) -> String {
        // For SQLite, we use the database field as the file path
        let db_path = self.config.database.as_deref().unwrap_or(":memory:");

        // Ensure the path exists if it's not in-memory
        if db_path != ":memory:" {
            if let Some(parent) = Path::new(db_path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
        }

        format!("sqlite://{db_path}")
    }
}

#[async_trait]
impl Connection for SqliteConnection {
    async fn connect(&mut self) -> Result<()> {
        // SQLite doesn't use passwords, so just call connect_with_key with None
        self.connect_with_key(None).await
    }

    async fn connect_with_key(&mut self, _encryption_key: Option<&str>) -> Result<()> {
        // SQLite doesn't use passwords, so ignore encryption_key
        let connection_string = self.build_connection_string();

        let pool = SqlitePoolOptions::new()
            .max_connections(1) // SQLite works best with single connection
            .connect(&connection_string)
            .await
            .map_err(|e| {
                LazyTablesError::Connection(format!("Failed to connect to SQLite: {e}"))
            })?;

        // Enable foreign key constraints
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await?;

        self.pool = Some(pool);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(pool) = self.pool.take() {
            pool.close().await;
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.pool.is_some()
    }

    fn config(&self) -> &ConnectionConfig {
        &self.config
    }

    // Query execution capabilities (AC1 requirement)
    async fn execute_raw_query(&self, query: &str) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        SqliteConnection::execute_raw_query(self, query).await
    }

    // Metadata operations (AC1 & AC2 requirements)
    async fn list_tables(&self) -> Result<Vec<String>> {
        SqliteConnection::list_tables(self).await
    }

    async fn list_database_objects(&self) -> Result<crate::database::DatabaseObjectList> {
        // SQLite basic implementation - convert tables to database objects
        let tables = SqliteConnection::list_tables(self).await?;
        let mut result = crate::database::DatabaseObjectList::default();

        for table_name in tables {
            let obj = crate::database::DatabaseObject {
                name: table_name,
                schema: Some("main".to_string()),
                object_type: crate::database::DatabaseObjectType::Table,
                row_count: None,
                size_bytes: None,
                comment: None,
            };
            result.tables.push(obj);
        }
        result.total_count = result.tables.len();
        Ok(result)
    }

    async fn get_table_metadata(&self, table_name: &str) -> Result<crate::database::TableMetadata> {
        SqliteConnection::get_table_metadata(self, table_name).await
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<crate::database::TableColumn>> {
        SqliteConnection::get_table_columns(self, table_name).await
    }

    async fn get_table_data(&self, table_name: &str, limit: usize, offset: usize) -> Result<Vec<Vec<String>>> {
        SqliteConnection::get_table_data(self, table_name, limit, offset).await
    }

    // Database-specific capabilities (AC1 & AC2 requirement)
    async fn get_database_capabilities(&self) -> Result<crate::database::DatabaseCapabilities> {
        Ok(crate::database::DatabaseCapabilities {
            supports_schemas: false, // SQLite has limited schema support
            supports_transactions: true,
            supports_foreign_keys: true,
            supports_json: true, // SQLite 3.38+
            supports_arrays: false,
            supports_stored_procedures: false,
            supports_triggers: true,
            supports_views: true,
            supports_materialized_views: false,
            supports_window_functions: true,
            supports_cte: true,
            max_identifier_length: 1000, // SQLite identifier limit is very high
            max_query_length: Some(1_000_000), // 1MB limit
            supported_isolation_levels: vec![
                "DEFERRED".to_string(),
                "IMMEDIATE".to_string(),
                "EXCLUSIVE".to_string(),
            ],
        })
    }

    async fn health_check(&self) -> Result<crate::database::HealthStatus> {
        let start = std::time::Instant::now();

        if let Some(pool) = &self.pool {
            match sqlx::query("SELECT 1").fetch_one(pool).await {
                Ok(_) => {
                    let response_time = start.elapsed().as_millis() as u64;

                    Ok(crate::database::HealthStatus {
                        is_healthy: true,
                        response_time_ms: response_time,
                        last_error: None,
                        database_version: None, // TODO: Get SQLite version
                        active_connections: 1, // SQLite is single connection
                        max_connections: 1,
                        uptime_seconds: None, // SQLite doesn't have uptime
                    })
                }
                Err(e) => {
                    Ok(crate::database::HealthStatus {
                        is_healthy: false,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        last_error: Some(e.to_string()),
                        database_version: None,
                        active_connections: 0,
                        max_connections: 1,
                        uptime_seconds: None,
                    })
                }
            }
        } else {
            Ok(crate::database::HealthStatus {
                is_healthy: false,
                response_time_ms: 0,
                last_error: Some("No active connection".to_string()),
                database_version: None,
                active_connections: 0,
                max_connections: 1,
                uptime_seconds: None,
            })
        }
    }

    async fn get_server_info(&self) -> Result<crate::database::ServerInfo> {
        if let Some(_pool) = &self.pool {
            Ok(crate::database::ServerInfo {
                version: "SQLite 3.x".to_string(), // TODO: Get actual version
                build_info: None,
                server_name: Some("SQLite".to_string()),
                charset: Some("UTF-8".to_string()),
                timezone: None, // SQLite doesn't have timezone
                uptime_seconds: None, // SQLite doesn't have uptime
                current_database: self.config.database.clone(),
                current_user: None, // SQLite doesn't have users
            })
        } else {
            Err(LazyTablesError::Connection("No active connection".to_string()))
        }
    }

    // Connection pooling support (AC4 requirement)
    fn get_pool_status(&self) -> Option<crate::database::PoolStatus> {
        self.pool.as_ref().map(|_pool| crate::database::PoolStatus {
            size: 1,
            active: 1,
            idle: 0,
            waiting: 0,
            max_size: 1,
            min_size: 1,
        })
    }

    fn max_connections(&self) -> u32 {
        1 // SQLite is single connection
    }

    fn active_connections(&self) -> u32 {
        if self.pool.is_some() { 1 } else { 0 }
    }

    // Database-specific error handling (AC5 requirement)
    fn format_error(&self, error: &str) -> crate::database::FormattedError {
        let error_lower = error.to_lowercase();
        let mut recovery_suggestions = Vec::new();
        let mut is_connection_error = false;
        let mut is_syntax_error = false;
        let is_permission_error = false;

        let user_message = if error_lower.contains("no such table") {
            recovery_suggestions.push("Check table name spelling".to_string());
            recovery_suggestions.push("Use .tables to list available tables".to_string());
            "Table not found. Please verify the table name."
        } else if error_lower.contains("syntax error") {
            is_syntax_error = true;
            recovery_suggestions.push("Check SQL syntax for typos".to_string());
            recovery_suggestions.push("Refer to SQLite documentation for correct syntax".to_string());
            "SQL syntax error. Please check your query for syntax mistakes."
        } else if error_lower.contains("database is locked") {
            is_connection_error = true;
            recovery_suggestions.push("Close other connections to the database".to_string());
            recovery_suggestions.push("Check if another process is using the database".to_string());
            "Database is locked. Please close other connections."
        } else if error_lower.contains("no such file") || error_lower.contains("unable to open") {
            is_connection_error = true;
            recovery_suggestions.push("Check file path is correct".to_string());
            recovery_suggestions.push("Ensure directory exists and is writable".to_string());
            "Database file not found or cannot be opened."
        } else {
            recovery_suggestions.push("Check SQLite documentation".to_string());
            "SQLite database error occurred."
        };

        crate::database::FormattedError {
            original_error: error.to_string(),
            user_message: user_message.to_string(),
            error_code: None,
            recovery_suggestions,
            is_connection_error,
            is_syntax_error,
            is_permission_error,
        }
    }

    fn get_keywords(&self) -> Vec<String> {
        vec![
            "SELECT".to_string(), "FROM".to_string(), "WHERE".to_string(), "INSERT".to_string(),
            "UPDATE".to_string(), "DELETE".to_string(), "CREATE".to_string(), "DROP".to_string(),
            "ALTER".to_string(), "TABLE".to_string(), "INDEX".to_string(), "VIEW".to_string(),
            "DATABASE".to_string(), "TRIGGER".to_string(), "PRIMARY".to_string(), "KEY".to_string(),
            "FOREIGN".to_string(), "REFERENCES".to_string(), "UNIQUE".to_string(), "AUTOINCREMENT".to_string(),
            "PRAGMA".to_string(), "EXPLAIN".to_string(), "ANALYZE".to_string(), "VACUUM".to_string(),
        ]
    }

    fn get_functions(&self) -> Vec<String> {
        vec![
            "COUNT".to_string(), "SUM".to_string(), "AVG".to_string(), "MIN".to_string(), "MAX".to_string(),
            "LENGTH".to_string(), "SUBSTR".to_string(), "UPPER".to_string(), "LOWER".to_string(),
            "DATETIME".to_string(), "DATE".to_string(), "TIME".to_string(), "STRFTIME".to_string(),
            "COALESCE".to_string(), "IFNULL".to_string(), "NULLIF".to_string(), "CASE".to_string(),
            "RANDOM".to_string(), "ROUND".to_string(), "ABS".to_string(), "TRIM".to_string(),
        ]
    }
}

impl SqliteConnection {
    /// Test the connection by running a simple query
    pub async fn test_connection(&self) -> Result<()> {
        if let Some(pool) = &self.pool {
            sqlx::query("SELECT 1")
                .fetch_one(pool)
                .await
                .map_err(|e| LazyTablesError::Connection(format!("Connection test failed: {e}")))?;
            Ok(())
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    /// List all databases (SQLite doesn't have multiple databases)
    #[allow(dead_code)]
    pub async fn list_databases(&self) -> Result<Vec<String>> {
        // SQLite doesn't have multiple databases in the same connection
        // Return the current database name
        if self.pool.is_some() {
            let db_name = self
                .config
                .database
                .as_deref()
                .unwrap_or("main")
                .to_string();
            Ok(vec![db_name])
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    /// List all tables in the database
    pub async fn list_tables(&self) -> Result<Vec<String>> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query(
                "SELECT name FROM sqlite_master 
                 WHERE type='table' 
                 AND name NOT LIKE 'sqlite_%'
                 ORDER BY name",
            )
            .fetch_all(pool)
            .await
            .map_err(|e| LazyTablesError::Connection(format!("Failed to list tables: {e}")))?;

            let tables = rows
                .iter()
                .map(|row| row.get::<String, _>("name"))
                .collect();

            Ok(tables)
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    /// Get metadata for a specific table
    pub async fn get_table_metadata(&self, table_name: &str) -> Result<TableMetadata> {
        if let Some(pool) = &self.pool {
            // Get row count
            let count_query = format!("SELECT COUNT(*) FROM \"{table_name}\"");
            let count_row = sqlx::query(&count_query)
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    LazyTablesError::Connection(format!("Failed to get row count: {e}"))
                })?;
            let row_count: i64 = count_row.get(0);

            // Get column info
            let pragma_query = format!("PRAGMA table_info(\"{table_name}\")");
            let col_rows = sqlx::query(&pragma_query).fetch_all(pool).await?;
            let column_count = col_rows.len();

            // Get primary keys
            let primary_keys: Vec<String> = col_rows
                .iter()
                .filter(|row| row.get::<i32, _>("pk") > 0)
                .map(|row| row.get::<String, _>("name"))
                .collect();

            // Get foreign keys
            let fk_query = format!("PRAGMA foreign_key_list(\"{table_name}\")");
            let fk_rows = sqlx::query(&fk_query).fetch_all(pool).await?;

            let foreign_keys: Vec<String> = fk_rows
                .iter()
                .map(|row| {
                    let from: String = row.get("from");
                    let table: String = row.get("table");
                    let to: String = row.get("to");
                    format!("{from} → {table}.{to}")
                })
                .collect();

            // Get indexes
            let index_query = format!("PRAGMA index_list(\"{table_name}\")");
            let index_rows = sqlx::query(&index_query).fetch_all(pool).await?;

            let indexes: Vec<String> = index_rows
                .iter()
                .map(|row| row.get::<String, _>("name"))
                .collect();

            // SQLite doesn't track table size in the same way
            // We can estimate based on page count
            let page_count_query =
                "SELECT COUNT(*) * (SELECT page_size FROM pragma_page_size()) as size 
                 FROM dbstat WHERE name = ?"
                    .to_string();

            let size = if let Ok(size_row) = sqlx::query(&page_count_query)
                .bind(table_name)
                .fetch_one(pool)
                .await
            {
                size_row.get::<Option<i64>, _>(0).unwrap_or(0)
            } else {
                0
            };

            Ok(TableMetadata {
                table_name: table_name.to_string(),
                row_count: row_count as usize,
                column_count,
                total_size: size,
                table_size: size,
                indexes_size: 0,
                primary_keys,
                foreign_keys,
                indexes,
                comment: None,
            })
        } else {
            Err(LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }

    /// Get column information for a table
    pub async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>> {
        if let Some(pool) = &self.pool {
            let query = format!("PRAGMA table_info(\"{table_name}\")");

            let rows = sqlx::query(&query).fetch_all(pool).await?;

            let columns = rows
                .iter()
                .map(|row| {
                    let column_name: String = row.get("name");
                    let data_type_str: String = row.get("type");
                    let not_null: i32 = row.get("notnull");
                    let default_value: Option<String> = row.get("dflt_value");
                    let is_pk: i32 = row.get("pk");

                    TableColumn {
                        name: column_name,
                        data_type: parse_sqlite_type(&data_type_str),
                        is_nullable: not_null == 0,
                        default_value,
                        is_primary_key: is_pk > 0,
                    }
                })
                .collect();

            Ok(columns)
        } else {
            Err(LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }

    /// Get the row count for a table
    pub async fn get_table_row_count(&self, table_name: &str) -> Result<usize> {
        if let Some(pool) = &self.pool {
            let query = format!("SELECT COUNT(*) FROM \"{table_name}\"");
            let row = sqlx::query(&query).fetch_one(pool).await?;
            let count: i64 = row.get(0);
            Ok(count as usize)
        } else {
            Err(LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }

    /// Get table data with pagination
    pub async fn get_table_data(
        &self,
        table_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Vec<String>>> {
        if let Some(pool) = &self.pool {
            // Get column names first to maintain order
            let pragma_query = format!("PRAGMA table_info(\"{table_name}\")");
            let column_rows = sqlx::query(&pragma_query).fetch_all(pool).await?;

            let column_names: Vec<String> = column_rows
                .iter()
                .map(|row| row.get::<String, _>("name"))
                .collect();

            if column_names.is_empty() {
                return Ok(Vec::new());
            }

            // Build SELECT query with all columns
            let select_list = column_names
                .iter()
                .map(|col| format!("\"{col}\""))
                .collect::<Vec<_>>()
                .join(", ");

            let query =
                format!("SELECT {select_list} FROM \"{table_name}\" LIMIT {limit} OFFSET {offset}");

            let rows = sqlx::query(&query).fetch_all(pool).await?;

            let mut result = Vec::new();
            for row in rows {
                let mut row_data = Vec::new();
                for (idx, _col_name) in column_names.iter().enumerate() {
                    // Try to get the value as string, handle NULL values
                    let value: Option<String> = row.try_get(idx).ok();
                    row_data.push(value.unwrap_or_else(|| "NULL".to_string()));
                }
                result.push(row_data);
            }

            Ok(result)
        } else {
            Err(LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }

    /// Execute a raw SQL query and return columns and rows
    pub async fn execute_raw_query(&self, query: &str) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        if let Some(pool) = &self.pool {
            // Try to execute the query
            let rows = sqlx::query(query).fetch_all(pool).await?;

            if rows.is_empty() {
                return Ok((Vec::new(), Vec::new()));
            }

            // Get column information from the first row
            let first_row = &rows[0];
            let columns = first_row.columns();

            let column_names: Vec<String> =
                columns.iter().map(|col| col.name().to_string()).collect();

            // Extract data from all rows
            let mut result_rows = Vec::new();
            for row in &rows {
                let mut row_data = Vec::new();
                for col in columns {
                    // Try to get value as string
                    let value: Option<String> = row.try_get(col.ordinal()).ok();
                    row_data.push(value.unwrap_or_else(|| "NULL".to_string()));
                }
                result_rows.push(row_data);
            }

            Ok((column_names, result_rows))
        } else {
            Err(LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }
}

/// Parse SQLite data type string to internal DataType enum
fn parse_sqlite_type(type_str: &str) -> DataType {
    let type_upper = type_str.to_uppercase();

    // SQLite has flexible typing, so we check for common patterns
    if type_upper.contains("INT") {
        DataType::Integer
    } else if type_upper.contains("REAL")
        || type_upper.contains("FLOAT")
        || type_upper.contains("DOUBLE")
    {
        DataType::Float
    } else if type_upper.contains("DECIMAL") || type_upper.contains("NUMERIC") {
        DataType::Decimal
    } else if type_upper.contains("CHAR")
        || type_upper.contains("TEXT")
        || type_upper.contains("CLOB")
    {
        DataType::Text
    } else if type_upper.contains("BLOB") {
        DataType::Bytea
    } else if type_upper.contains("DATE") {
        DataType::Date
    } else if type_upper.contains("TIME") {
        DataType::Timestamp
    } else if type_upper.contains("BOOL") {
        DataType::Boolean
    } else {
        // Default to text for unknown types (SQLite's default behavior)
        DataType::Text
    }
}
