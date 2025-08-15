// FilePath: src/database/sqlite.rs

use crate::core::error::{LazyTablesError, Result};
use crate::database::{
    connection::ConnectionConfig, Connection, DataType, TableColumn, TableMetadata,
};
use async_trait::async_trait;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
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
