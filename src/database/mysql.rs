// FilePath: src/database/mysql.rs

use crate::core::error::{LazyTablesError, Result};
use crate::database::{
    connection::ConnectionConfig, Connection, DataType, TableColumn, TableMetadata,
};
use async_trait::async_trait;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::{Column, Row};

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
    fn build_connection_string(&self, encryption_key: Option<&str>) -> Result<String> {
        let host = &self.config.host;
        let port = self.config.port;
        let database = self.config.database.as_deref().unwrap_or("mysql");
        let username = &self.config.username;

        // Try to resolve password from various sources
        let password = self
            .config
            .resolve_password(encryption_key)
            .unwrap_or_default();

        if !password.is_empty() {
            Ok(format!(
                "mysql://{username}:{password}@{host}:{port}/{database}"
            ))
        } else {
            Ok(format!("mysql://{username}@{host}:{port}/{database}"))
        }
    }
}

#[async_trait]
impl Connection for MySqlConnection {
    async fn connect(&mut self) -> Result<()> {
        // Use connect_with_key with None for backward compatibility
        self.connect_with_key(None).await
    }

    async fn connect_with_key(&mut self, encryption_key: Option<&str>) -> Result<()> {
        let connection_string = self.build_connection_string(encryption_key)?;

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await
            .map_err(|e| LazyTablesError::Connection(format!("Failed to connect to MySQL: {e}")))?;

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
        MySqlConnection::execute_raw_query(self, query).await
    }

    // Metadata operations (AC1 & AC2 requirements)
    async fn list_tables(&self) -> Result<Vec<String>> {
        MySqlConnection::list_tables(self).await
    }

    async fn list_database_objects(&self) -> Result<crate::database::DatabaseObjectList> {
        // MySQL adapter doesn't have list_database_objects yet, implement basic version
        let tables = MySqlConnection::list_tables(self).await?;
        let mut result = crate::database::DatabaseObjectList::default();

        for table_name in tables {
            let obj = crate::database::DatabaseObject {
                name: table_name,
                schema: Some("default".to_string()),
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
        MySqlConnection::get_table_metadata(self, table_name).await
    }

    async fn get_table_columns(
        &self,
        table_name: &str,
    ) -> Result<Vec<crate::database::TableColumn>> {
        MySqlConnection::get_table_columns(self, table_name).await
    }

    async fn get_table_data(
        &self,
        table_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Vec<String>>> {
        MySqlConnection::get_table_data(self, table_name, limit, offset).await
    }

    // Database-specific capabilities (AC1 & AC2 requirement)
    async fn get_database_capabilities(&self) -> Result<crate::database::DatabaseCapabilities> {
        Ok(crate::database::DatabaseCapabilities {
            supports_schemas: false, // MySQL uses databases instead of schemas
            supports_transactions: true,
            supports_foreign_keys: true,
            supports_json: true,
            supports_arrays: false, // MySQL doesn't have native array types
            supports_stored_procedures: true,
            supports_triggers: true,
            supports_views: true,
            supports_materialized_views: false, // MySQL doesn't have materialized views
            supports_window_functions: true,
            supports_cte: true,
            max_identifier_length: 64,         // MySQL identifier limit
            max_query_length: Some(1_048_576), // 1MB default max_allowed_packet
            supported_isolation_levels: vec![
                "READ UNCOMMITTED".to_string(),
                "READ COMMITTED".to_string(),
                "REPEATABLE READ".to_string(),
                "SERIALIZABLE".to_string(),
            ],
        })
    }

    async fn health_check(&self) -> Result<crate::database::HealthStatus> {
        let start = std::time::Instant::now();

        if let Some(pool) = &self.pool {
            match sqlx::query("SELECT 1").fetch_one(pool).await {
                Ok(_) => {
                    let response_time = start.elapsed().as_millis() as u64;

                    // Get server info for additional health details
                    let version = self.get_mysql_version().await.ok();
                    let (active, max_conn) = self.get_connection_counts().await.unwrap_or((0, 0));

                    Ok(crate::database::HealthStatus {
                        is_healthy: true,
                        response_time_ms: response_time,
                        last_error: None,
                        database_version: version,
                        active_connections: active,
                        max_connections: max_conn,
                        uptime_seconds: self.get_uptime().await.ok(),
                    })
                }
                Err(e) => Ok(crate::database::HealthStatus {
                    is_healthy: false,
                    response_time_ms: start.elapsed().as_millis() as u64,
                    last_error: Some(e.to_string()),
                    database_version: None,
                    active_connections: 0,
                    max_connections: 0,
                    uptime_seconds: None,
                }),
            }
        } else {
            Ok(crate::database::HealthStatus {
                is_healthy: false,
                response_time_ms: 0,
                last_error: Some("No active connection".to_string()),
                database_version: None,
                active_connections: 0,
                max_connections: 0,
                uptime_seconds: None,
            })
        }
    }

    async fn get_server_info(&self) -> Result<crate::database::ServerInfo> {
        if let Some(_pool) = &self.pool {
            let version = self.get_mysql_version().await?;
            let charset = self.get_charset().await.ok();
            let timezone = self.get_timezone().await.ok();
            let uptime = self.get_uptime().await.ok();
            let current_db = self.config.database.clone();
            let current_user = Some(self.config.username.clone());

            Ok(crate::database::ServerInfo {
                version,
                build_info: None,
                server_name: Some("MySQL".to_string()),
                charset,
                timezone,
                uptime_seconds: uptime,
                current_database: current_db,
                current_user,
            })
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    // Connection pooling support (AC4 requirement)
    fn get_pool_status(&self) -> Option<crate::database::PoolStatus> {
        self.pool.as_ref().map(|pool| crate::database::PoolStatus {
            size: pool.size(),
            active: pool.size(), // SQLx doesn't expose detailed pool stats
            idle: 0,
            waiting: 0,
            max_size: 5, // Hard-coded from our pool configuration
            min_size: 0,
        })
    }

    fn max_connections(&self) -> u32 {
        5 // Current pool configuration
    }

    fn active_connections(&self) -> u32 {
        if let Some(pool) = &self.pool {
            pool.size()
        } else {
            0
        }
    }

    // Database-specific error handling (AC5 requirement)
    fn format_error(&self, error: &str) -> crate::database::FormattedError {
        let error_lower = error.to_lowercase();
        let mut recovery_suggestions = Vec::new();
        let mut is_connection_error = false;
        let mut is_syntax_error = false;
        let mut is_permission_error = false;
        let mut error_code = None;

        let user_message = if error_lower.contains("access denied") {
            is_permission_error = true;
            recovery_suggestions.push("Check username and password".to_string());
            recovery_suggestions
                .push("Verify user has permission to access the database".to_string());
            "Access denied. Please check your credentials."
        } else if error_lower.contains("unknown database") {
            is_connection_error = true;
            recovery_suggestions.push("Check database name spelling".to_string());
            recovery_suggestions.push("Ensure database exists on the server".to_string());
            "Database not found. Please verify the database name."
        } else if error_lower.contains("can't connect")
            || error_lower.contains("connection refused")
        {
            is_connection_error = true;
            recovery_suggestions.push("Check if MySQL server is running".to_string());
            recovery_suggestions.push("Verify host and port are correct".to_string());
            recovery_suggestions.push("Check firewall settings".to_string());
            "Cannot connect to MySQL server. Please check server status and connection details."
        } else if error_lower.contains("syntax error")
            || error_lower.contains("you have an error in your sql syntax")
        {
            is_syntax_error = true;
            error_code = Some("1064".to_string());
            recovery_suggestions.push("Check SQL syntax for typos".to_string());
            recovery_suggestions
                .push("Refer to MySQL documentation for correct syntax".to_string());
            "SQL syntax error. Please check your query for syntax mistakes."
        } else if error_lower.contains("table") && error_lower.contains("doesn't exist") {
            recovery_suggestions.push("Check table name spelling".to_string());
            recovery_suggestions.push("Use SHOW TABLES to list available tables".to_string());
            "Table not found. Please verify the table name."
        } else if error_lower.contains("duplicate entry") {
            recovery_suggestions.push("Check for unique constraint violations".to_string());
            recovery_suggestions.push("Use INSERT IGNORE or ON DUPLICATE KEY UPDATE".to_string());
            "Duplicate entry error. A unique constraint is being violated."
        } else {
            recovery_suggestions.push("Check MySQL error log for details".to_string());
            recovery_suggestions.push("Consult MySQL documentation".to_string());
            "MySQL database error occurred."
        };

        crate::database::FormattedError {
            original_error: error.to_string(),
            user_message: user_message.to_string(),
            error_code,
            recovery_suggestions,
            is_connection_error,
            is_syntax_error,
            is_permission_error,
        }
    }

    fn get_keywords(&self) -> Vec<String> {
        vec![
            "SELECT".to_string(),
            "FROM".to_string(),
            "WHERE".to_string(),
            "INSERT".to_string(),
            "UPDATE".to_string(),
            "DELETE".to_string(),
            "CREATE".to_string(),
            "DROP".to_string(),
            "ALTER".to_string(),
            "TABLE".to_string(),
            "INDEX".to_string(),
            "VIEW".to_string(),
            "DATABASE".to_string(),
            "SCHEMA".to_string(),
            "PROCEDURE".to_string(),
            "FUNCTION".to_string(),
            "TRIGGER".to_string(),
            "EVENT".to_string(),
            "PRIMARY".to_string(),
            "KEY".to_string(),
            "FOREIGN".to_string(),
            "REFERENCES".to_string(),
            "UNIQUE".to_string(),
            "AUTO_INCREMENT".to_string(),
            "ENGINE".to_string(),
            "CHARSET".to_string(),
            "COLLATE".to_string(),
            "SHOW".to_string(),
            "DESCRIBE".to_string(),
            "EXPLAIN".to_string(),
            "OPTIMIZE".to_string(),
            "ANALYZE".to_string(),
        ]
    }

    fn get_functions(&self) -> Vec<String> {
        vec![
            "COUNT".to_string(),
            "SUM".to_string(),
            "AVG".to_string(),
            "MIN".to_string(),
            "MAX".to_string(),
            "CONCAT".to_string(),
            "SUBSTRING".to_string(),
            "LENGTH".to_string(),
            "UPPER".to_string(),
            "LOWER".to_string(),
            "NOW".to_string(),
            "CURDATE".to_string(),
            "CURTIME".to_string(),
            "DATE_FORMAT".to_string(),
            "YEAR".to_string(),
            "MONTH".to_string(),
            "DAY".to_string(),
            "HOUR".to_string(),
            "IFNULL".to_string(),
            "COALESCE".to_string(),
            "CASE".to_string(),
            "IF".to_string(),
            "JSON_EXTRACT".to_string(),
            "JSON_OBJECT".to_string(),
            "JSON_ARRAY".to_string(),
            "RAND".to_string(),
            "ROUND".to_string(),
            "FLOOR".to_string(),
            "CEIL".to_string(),
        ]
    }
}

impl MySqlConnection {
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

    /// List all databases accessible to the user
    pub async fn list_databases(&self) -> Result<Vec<String>> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query("SHOW DATABASES")
                .fetch_all(pool)
                .await
                .map_err(|e| {
                    LazyTablesError::Connection(format!("Failed to list databases: {e}"))
                })?;

            let databases = rows.iter().map(|row| row.get::<String, _>(0)).collect();

            Ok(databases)
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    /// List all tables in the current database
    pub async fn list_tables(&self) -> Result<Vec<String>> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query("SHOW TABLES")
                .fetch_all(pool)
                .await
                .map_err(|e| LazyTablesError::Connection(format!("Failed to list tables: {e}")))?;

            let tables = rows.iter().map(|row| row.get::<String, _>(0)).collect();

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
            let count_query = format!("SELECT COUNT(*) FROM `{table_name}`");
            let count_row = sqlx::query(&count_query)
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    LazyTablesError::Connection(format!("Failed to get row count: {e}"))
                })?;
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

            let primary_keys: Vec<String> =
                pk_rows.iter().map(|row| row.get::<String, _>(0)).collect();

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

            let foreign_keys: Vec<String> =
                fk_rows.iter().map(|row| row.get::<String, _>(0)).collect();

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
            let comment = if comment.is_empty() {
                None
            } else {
                Some(comment)
            };

            Ok(TableMetadata::basic(
                table_name.to_string(),
                row_count as usize,
                column_count as usize,
                total_size.unwrap_or(0) as i64,
                table_size.unwrap_or(0) as i64,
                indexes_size.unwrap_or(0) as i64,
                primary_keys,
                foreign_keys,
                indexes,
                comment,
            ))
        } else {
            Err(LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }

    /// Get column information for a table
    pub async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>> {
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

            let rows = sqlx::query(query).bind(table_name).fetch_all(pool).await?;

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

    /// Get the row count for a table
    pub async fn get_table_row_count(&self, table_name: &str) -> Result<usize> {
        if let Some(pool) = &self.pool {
            let query = format!("SELECT COUNT(*) FROM `{table_name}`");
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

            let query =
                format!("SELECT {select_list} FROM `{table_name}` LIMIT {limit} OFFSET {offset}");

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

    /// Get MySQL version
    pub async fn get_mysql_version(&self) -> Result<String> {
        if let Some(pool) = &self.pool {
            let row = sqlx::query("SELECT VERSION()").fetch_one(pool).await?;
            Ok(row.get::<String, _>(0))
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    /// Get current character set
    pub async fn get_charset(&self) -> Result<String> {
        if let Some(pool) = &self.pool {
            let row = sqlx::query("SELECT @@character_set_database")
                .fetch_one(pool)
                .await?;
            Ok(row.get::<String, _>(0))
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    /// Get current timezone
    pub async fn get_timezone(&self) -> Result<String> {
        if let Some(pool) = &self.pool {
            let row = sqlx::query("SELECT @@time_zone").fetch_one(pool).await?;
            Ok(row.get::<String, _>(0))
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    /// Get server uptime in seconds
    pub async fn get_uptime(&self) -> Result<u64> {
        if let Some(pool) = &self.pool {
            let row = sqlx::query("SHOW STATUS LIKE 'Uptime'")
                .fetch_one(pool)
                .await?;
            let uptime_str: String = row.get("Value");
            Ok(uptime_str.parse().unwrap_or(0))
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    /// Get connection counts (active, max)
    pub async fn get_connection_counts(&self) -> Result<(u32, u32)> {
        if let Some(pool) = &self.pool {
            let threads_row = sqlx::query("SHOW STATUS LIKE 'Threads_connected'")
                .fetch_one(pool)
                .await?;
            let max_row = sqlx::query("SHOW VARIABLES LIKE 'max_connections'")
                .fetch_one(pool)
                .await?;

            let active_str: String = threads_row.get("Value");
            let max_str: String = max_row.get("Value");

            let active = active_str.parse().unwrap_or(0);
            let max_conn = max_str.parse().unwrap_or(0);

            Ok((active, max_conn))
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
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

/// Parse MySQL data type string to internal DataType enum
#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseType;

    #[test]
    fn test_mysql_connection_creation() {
        let config = ConnectionConfig::new(
            "test".to_string(),
            DatabaseType::MySQL,
            "localhost".to_string(),
            3306,
            "root".to_string(),
        );

        let connection = MySqlConnection::new(config);
        assert!(!connection.is_connected());
        assert_eq!(connection.config().database_type, DatabaseType::MySQL);
    }

    #[test]
    fn test_build_connection_string_with_password() {
        let mut config = ConnectionConfig::new(
            "test".to_string(),
            DatabaseType::MySQL,
            "localhost".to_string(),
            3306,
            "root".to_string(),
        );
        config.set_plain_password("password123".to_string());
        config.database = Some("testdb".to_string());

        let connection = MySqlConnection::new(config);
        let conn_str = connection.build_connection_string(None).unwrap();

        assert!(conn_str.contains("mysql://"));
        assert!(conn_str.contains("root:password123@"));
        assert!(conn_str.contains("localhost:3306"));
        assert!(conn_str.contains("/testdb"));
    }

    #[test]
    fn test_build_connection_string_without_password() {
        let config = ConnectionConfig::new(
            "test".to_string(),
            DatabaseType::MySQL,
            "localhost".to_string(),
            3306,
            "root".to_string(),
        );

        let connection = MySqlConnection::new(config);
        let conn_str = connection.build_connection_string(None).unwrap();

        assert!(conn_str.contains("mysql://"));
        assert!(conn_str.contains("root@"));
        assert!(!conn_str.contains(":@")); // No empty password
    }

    #[test]
    fn test_parse_mysql_types() {
        assert_eq!(parse_mysql_type("int"), DataType::Integer);
        assert_eq!(parse_mysql_type("bigint"), DataType::BigInt);
        assert_eq!(parse_mysql_type("varchar"), DataType::Text);
        assert_eq!(parse_mysql_type("text"), DataType::Text);
        assert_eq!(parse_mysql_type("json"), DataType::Json);
        assert_eq!(parse_mysql_type("datetime"), DataType::Timestamp);
        assert_eq!(parse_mysql_type("date"), DataType::Date);
        assert_eq!(parse_mysql_type("time"), DataType::Time);
        assert_eq!(parse_mysql_type("decimal"), DataType::Decimal);
        assert_eq!(parse_mysql_type("float"), DataType::Float);
        assert_eq!(parse_mysql_type("double"), DataType::Double);
        assert_eq!(parse_mysql_type("blob"), DataType::Bytea);
    }

    #[test]
    fn test_database_capabilities() {
        let config = ConnectionConfig::new(
            "test".to_string(),
            DatabaseType::MySQL,
            "localhost".to_string(),
            3306,
            "root".to_string(),
        );

        let connection = MySqlConnection::new(config);

        // Test this doesn't panic (can't test actual DB capabilities without connection)
        assert_eq!(connection.max_connections(), 5);
        assert_eq!(connection.active_connections(), 0);
        assert!(connection.get_pool_status().is_none());
    }

    #[test]
    fn test_format_error_access_denied() {
        let config = ConnectionConfig::new(
            "test".to_string(),
            DatabaseType::MySQL,
            "localhost".to_string(),
            3306,
            "root".to_string(),
        );

        let connection = MySqlConnection::new(config);
        let formatted = connection.format_error("Access denied for user 'root'@'localhost'");

        assert!(formatted.is_permission_error);
        assert!(!formatted.is_connection_error);
        assert!(!formatted.is_syntax_error);
        assert!(formatted.user_message.contains("Access denied"));
        assert!(!formatted.recovery_suggestions.is_empty());
    }

    #[test]
    fn test_format_error_syntax() {
        let config = ConnectionConfig::new(
            "test".to_string(),
            DatabaseType::MySQL,
            "localhost".to_string(),
            3306,
            "root".to_string(),
        );

        let connection = MySqlConnection::new(config);
        let formatted = connection.format_error("You have an error in your SQL syntax");

        assert!(!formatted.is_permission_error);
        assert!(!formatted.is_connection_error);
        assert!(formatted.is_syntax_error);
        assert!(formatted.user_message.contains("syntax error"));
        assert_eq!(formatted.error_code, Some("1064".to_string()));
    }

    #[test]
    fn test_format_error_connection() {
        let config = ConnectionConfig::new(
            "test".to_string(),
            DatabaseType::MySQL,
            "localhost".to_string(),
            3306,
            "root".to_string(),
        );

        let connection = MySqlConnection::new(config);
        let formatted = connection.format_error("Can't connect to MySQL server");

        assert!(!formatted.is_permission_error);
        assert!(formatted.is_connection_error);
        assert!(!formatted.is_syntax_error);
        assert!(formatted.user_message.contains("Cannot connect"));
    }

    #[test]
    fn test_keywords_and_functions() {
        let config = ConnectionConfig::new(
            "test".to_string(),
            DatabaseType::MySQL,
            "localhost".to_string(),
            3306,
            "root".to_string(),
        );

        let connection = MySqlConnection::new(config);
        let keywords = connection.get_keywords();
        let functions = connection.get_functions();

        assert!(keywords.contains(&"SELECT".to_string()));
        assert!(keywords.contains(&"AUTO_INCREMENT".to_string()));
        assert!(functions.contains(&"COUNT".to_string()));
        assert!(functions.contains(&"JSON_EXTRACT".to_string()));
    }
}

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
