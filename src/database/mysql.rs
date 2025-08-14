// FilePath: src/database/mysql.rs

use crate::core::error::{LazyTablesError, Result};
use crate::database::{
    connection::ConnectionConfig,
    Connection, DataType, TableColumn, TableMetadata,
};
use async_trait::async_trait;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::Row;

/// MySQL database connection implementation
pub struct MySqlConnection {
    config: ConnectionConfig,
    pool: Option<MySqlPool>,
}

impl MySqlConnection {
    /// Create a new MySQL connection instance
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config, pool: None }
    }

    /// Build MySQL connection string
    fn build_connection_string(&self) -> String {
        let host = &self.config.host;
        let port = self.config.port;
        let database = self.config.database.as_deref().unwrap_or("mysql");
        let username = &self.config.username;
        let password = self.config.password.as_deref().unwrap_or("");

        if !password.is_empty() {
            format!("mysql://{username}:{password}@{host}:{port}/{database}")
        } else {
            format!("mysql://{username}@{host}:{port}/{database}")
        }
    }
}

#[async_trait]
impl Connection for MySqlConnection {
    async fn connect(&mut self) -> Result<()> {
        let connection_string = self.build_connection_string();

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await
            .map_err(|e| {
                LazyTablesError::Connection(format!("Failed to connect to MySQL: {e}"))
            })?;

        self.pool = Some(pool);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(pool) = self.pool.take() {
            pool.close().await;
        }
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        if let Some(pool) = &self.pool {
            pool.acquire().await.is_ok()
        } else {
            false
        }
    }

    async fn test_connection(&self) -> Result<()> {
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

    async fn list_databases(&self) -> Result<Vec<String>> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query("SHOW DATABASES")
                .fetch_all(pool)
                .await
                .map_err(|e| LazyTablesError::Connection(format!("Failed to list databases: {e}")))?;

            let databases = rows
                .iter()
                .map(|row| row.get::<String, _>(0))
                .collect();

            Ok(databases)
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    async fn list_tables(&self) -> Result<Vec<String>> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query("SHOW TABLES")
                .fetch_all(pool)
                .await
                .map_err(|e| LazyTablesError::Connection(format!("Failed to list tables: {e}")))?;

            let tables = rows
                .iter()
                .map(|row| row.get::<String, _>(0))
                .collect();

            Ok(tables)
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    async fn get_table_metadata(&self, table_name: &str) -> Result<TableMetadata> {
        if let Some(pool) = &self.pool {
            // Get row count
            let count_query = format!("SELECT COUNT(*) FROM `{table_name}`");
            let count_row = sqlx::query(&count_query)
                .fetch_one(pool)
                .await
                .map_err(|e| LazyTablesError::Connection(format!("Failed to get row count: {e}")))?;
            let row_count: i64 = count_row.get(0);

            // Get column count
            let columns_query = "SELECT COUNT(*) FROM information_schema.columns 
                                WHERE table_schema = DATABASE() AND table_name = ?";
            let col_row = sqlx::query(columns_query)
                .bind(table_name)
                .fetch_one(pool)
                .await?;
            let column_count: i64 = col_row.get(0);

            // Get table size
            let size_query = "SELECT 
                data_length + index_length AS total_size,
                data_length AS table_size,
                index_length AS indexes_size
                FROM information_schema.tables 
                WHERE table_schema = DATABASE() AND table_name = ?";
            
            let size_row = sqlx::query(size_query)
                .bind(table_name)
                .fetch_one(pool)
                .await?;
            
            let total_size: Option<i64> = size_row.get(0);
            let table_size: Option<i64> = size_row.get(1);
            let indexes_size: Option<i64> = size_row.get(2);

            // Get primary keys
            let pk_query = "SELECT column_name 
                           FROM information_schema.key_column_usage 
                           WHERE table_schema = DATABASE() 
                           AND table_name = ? 
                           AND constraint_name = 'PRIMARY'
                           ORDER BY ordinal_position";
            
            let pk_rows = sqlx::query(pk_query)
                .bind(table_name)
                .fetch_all(pool)
                .await?;
            
            let primary_keys: Vec<String> = pk_rows
                .iter()
                .map(|row| row.get::<String, _>(0))
                .collect();

            // Get foreign keys
            let fk_query = "SELECT 
                CONCAT(column_name, ' → ', referenced_table_name, '.', referenced_column_name) as fk_info
                FROM information_schema.key_column_usage 
                WHERE table_schema = DATABASE() 
                AND table_name = ? 
                AND referenced_table_name IS NOT NULL";
            
            let fk_rows = sqlx::query(fk_query)
                .bind(table_name)
                .fetch_all(pool)
                .await?;
            
            let foreign_keys: Vec<String> = fk_rows
                .iter()
                .map(|row| row.get::<String, _>(0))
                .collect();

            // Get indexes
            let index_query = "SELECT DISTINCT index_name 
                              FROM information_schema.statistics 
                              WHERE table_schema = DATABASE() 
                              AND table_name = ? 
                              AND index_name != 'PRIMARY'";
            
            let index_rows = sqlx::query(index_query)
                .bind(table_name)
                .fetch_all(pool)
                .await?;
            
            let indexes: Vec<String> = index_rows
                .iter()
                .map(|row| row.get::<String, _>(0))
                .collect();

            // Get table comment
            let comment_query = "SELECT table_comment 
                                FROM information_schema.tables 
                                WHERE table_schema = DATABASE() 
                                AND table_name = ?";
            
            let comment_row = sqlx::query(comment_query)
                .bind(table_name)
                .fetch_one(pool)
                .await?;
            
            let comment: String = comment_row.get(0);
            let comment = if comment.is_empty() { None } else { Some(comment) };

            Ok(TableMetadata {
                table_name: table_name.to_string(),
                row_count: row_count as usize,
                column_count: column_count as usize,
                total_size: total_size.unwrap_or(0) as i64,
                table_size: table_size.unwrap_or(0) as i64,
                indexes_size: indexes_size.unwrap_or(0) as i64,
                primary_keys,
                foreign_keys,
                indexes,
                comment,
            })
        } else {
            Err(LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>> {
        if let Some(pool) = &self.pool {
            let query = "SELECT 
                column_name,
                data_type,
                is_nullable,
                column_default,
                column_key
                FROM information_schema.columns 
                WHERE table_schema = DATABASE() 
                AND table_name = ?
                ORDER BY ordinal_position";

            let rows = sqlx::query(query)
                .bind(table_name)
                .fetch_all(pool)
                .await?;

            let columns = rows
                .iter()
                .map(|row| {
                    let column_name: String = row.get("column_name");
                    let data_type_str: String = row.get("data_type");
                    let is_nullable: String = row.get("is_nullable");
                    let column_default: Option<String> = row.get("column_default");
                    let column_key: String = row.get("column_key");

                    TableColumn {
                        name: column_name,
                        data_type: parse_mysql_type(&data_type_str),
                        is_nullable: is_nullable == "YES",
                        default_value: column_default,
                        is_primary_key: column_key == "PRI",
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

    async fn get_table_row_count(&self, table_name: &str) -> Result<usize> {
        if let Some(pool) = &self.pool {
            let query = format!("SELECT COUNT(*) FROM `{table_name}`");
            let row = sqlx::query(&query)
                .fetch_one(pool)
                .await?;
            let count: i64 = row.get(0);
            Ok(count as usize)
        } else {
            Err(LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }

    async fn get_table_data(
        &self,
        table_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Vec<String>>> {
        if let Some(pool) = &self.pool {
            // Get column names first to maintain order
            let columns_query = "SELECT column_name 
                FROM information_schema.columns 
                WHERE table_schema = DATABASE() AND table_name = ?
                ORDER BY ordinal_position";

            let column_rows = sqlx::query(columns_query)
                .bind(table_name)
                .fetch_all(pool)
                .await?;

            let column_names: Vec<String> = column_rows
                .iter()
                .map(|row| row.get::<String, _>("column_name"))
                .collect();

            if column_names.is_empty() {
                return Ok(Vec::new());
            }

            // Build SELECT query with all columns
            let select_list = column_names
                .iter()
                .map(|col| format!("`{col}`"))
                .collect::<Vec<_>>()
                .join(", ");

            let query = format!(
                "SELECT {select_list} FROM `{table_name}` LIMIT {limit} OFFSET {offset}"
            );

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

/// Parse MySQL data type string to internal DataType enum
fn parse_mysql_type(type_str: &str) -> DataType {
    let type_lower = type_str.to_lowercase();
    
    match type_lower.as_str() {
        "tinyint" | "smallint" | "mediumint" | "int" | "integer" => DataType::Integer,
        "bigint" => DataType::BigInt,
        "decimal" | "numeric" | "dec" => DataType::Decimal,
        "float" => DataType::Float,
        "double" | "double precision" | "real" => DataType::Double,
        "bit" => DataType::Boolean,
        "char" | "varchar" => DataType::Text,
        "tinytext" | "text" | "mediumtext" | "longtext" => DataType::Text,
        "date" => DataType::Date,
        "time" => DataType::Time,
        "datetime" | "timestamp" => DataType::Timestamp,
        "year" => DataType::Integer,
        "binary" | "varbinary" | "blob" | "tinyblob" | "mediumblob" | "longblob" => DataType::Bytea,
        "json" => DataType::Json,
        "enum" | "set" => DataType::Text,
        _ => DataType::Text,
    }
}