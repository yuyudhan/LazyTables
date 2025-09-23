// FilePath: src/database/postgres.rs

use crate::core::error::{LazyTablesError, Result};
use crate::database::{
    connection::ConnectionConfig, Connection, DataType, TableColumn, TableMetadata,
};
use async_trait::async_trait;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Column, Row};

/// PostgreSQL database connection implementation
pub struct PostgresConnection {
    config: ConnectionConfig,
    pub pool: Option<PgPool>,
}

impl PostgresConnection {
    /// Create a new PostgreSQL connection instance
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config, pool: None }
    }

    /// Build PostgreSQL connection string
    fn build_connection_string(&self, encryption_key: Option<&str>) -> Result<String> {
        let host = &self.config.host;
        let port = self.config.port;
        let database = self.config.database.as_deref().unwrap_or("postgres");
        let username = &self.config.username;

        // Try to resolve password from various sources
        let password = self
            .config
            .resolve_password(encryption_key)
            .unwrap_or_default();

        if !password.is_empty() {
            Ok(format!(
                "postgresql://{username}:{password}@{host}:{port}/{database}"
            ))
        } else {
            Ok(format!("postgresql://{username}@{host}:{port}/{database}"))
        }
    }
}

#[async_trait]
impl Connection for PostgresConnection {
    async fn connect(&mut self) -> Result<()> {
        // Use connect_with_key with None for backward compatibility
        self.connect_with_key(None).await
    }

    async fn connect_with_key(&mut self, encryption_key: Option<&str>) -> Result<()> {
        let connection_string = self.build_connection_string(encryption_key)?;

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await
            .map_err(|e| {
                LazyTablesError::Connection(format!("Failed to connect to PostgreSQL: {e}"))
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

    fn is_connected(&self) -> bool {
        self.pool.is_some()
    }

    fn config(&self) -> &ConnectionConfig {
        &self.config
    }

    // Query execution capabilities (AC1 requirement)
    async fn execute_raw_query(&self, query: &str) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        PostgresConnection::execute_raw_query(self, query).await
    }

    // Metadata operations (AC1 & AC2 requirements)
    async fn list_tables(&self) -> Result<Vec<String>> {
        PostgresConnection::list_tables(self).await
    }

    async fn list_database_objects(&self) -> Result<crate::database::DatabaseObjectList> {
        PostgresConnection::list_database_objects(self).await
    }

    async fn get_table_metadata(&self, table_name: &str) -> Result<crate::database::TableMetadata> {
        PostgresConnection::get_table_metadata(self, table_name).await
    }

    async fn get_table_columns(
        &self,
        table_name: &str,
    ) -> Result<Vec<crate::database::TableColumn>> {
        PostgresConnection::get_table_columns(self, table_name).await
    }

    async fn get_table_data(
        &self,
        table_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Vec<String>>> {
        PostgresConnection::get_table_data(self, table_name, limit, offset).await
    }

    // Database-specific capabilities (AC1 & AC2 requirement)
    async fn get_database_capabilities(&self) -> Result<crate::database::DatabaseCapabilities> {
        Ok(crate::database::DatabaseCapabilities {
            supports_schemas: true,
            supports_transactions: true,
            supports_foreign_keys: true,
            supports_json: true,
            supports_arrays: true,
            supports_stored_procedures: true,
            supports_triggers: true,
            supports_views: true,
            supports_materialized_views: true,
            supports_window_functions: true,
            supports_cte: true,
            max_identifier_length: 63, // PostgreSQL identifier limit
            max_query_length: None,    // No specific limit
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

                    Ok(crate::database::HealthStatus {
                        is_healthy: true,
                        response_time_ms: response_time,
                        last_error: None,
                        database_version: None, // TODO: Get version
                        active_connections: pool.size(),
                        max_connections: 5,   // Hard-coded from pool config
                        uptime_seconds: None, // TODO: Get uptime
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
            Ok(crate::database::ServerInfo {
                version: "PostgreSQL 14+".to_string(), // TODO: Get actual version
                build_info: None,
                server_name: Some("PostgreSQL".to_string()),
                charset: Some("UTF8".to_string()),
                timezone: None,       // TODO: Get timezone
                uptime_seconds: None, // TODO: Get uptime
                current_database: self.config.database.clone(),
                current_user: Some(self.config.username.clone()),
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
            active: pool.size(),
            idle: 0,
            waiting: 0,
            max_size: 5,
            min_size: 0,
        })
    }

    fn max_connections(&self) -> u32 {
        5
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

        let user_message = if error_lower.contains("permission denied")
            || error_lower.contains("authentication failed")
        {
            is_permission_error = true;
            recovery_suggestions.push("Check username and password".to_string());
            recovery_suggestions
                .push("Verify user has permission to access the database".to_string());
            "Access denied. Please check your credentials."
        } else if error_lower.contains("database") && error_lower.contains("does not exist") {
            is_connection_error = true;
            recovery_suggestions.push("Check database name spelling".to_string());
            recovery_suggestions.push("Ensure database exists on the server".to_string());
            "Database not found. Please verify the database name."
        } else if error_lower.contains("connection refused")
            || error_lower.contains("could not connect")
        {
            is_connection_error = true;
            recovery_suggestions.push("Check if PostgreSQL server is running".to_string());
            recovery_suggestions.push("Verify host and port are correct".to_string());
            recovery_suggestions.push("Check firewall settings".to_string());
            "Cannot connect to PostgreSQL server. Please check server status and connection details."
        } else if error_lower.contains("syntax error") {
            is_syntax_error = true;
            recovery_suggestions.push("Check SQL syntax for typos".to_string());
            recovery_suggestions
                .push("Refer to PostgreSQL documentation for correct syntax".to_string());
            "SQL syntax error. Please check your query for syntax mistakes."
        } else if error_lower.contains("relation") && error_lower.contains("does not exist") {
            recovery_suggestions.push("Check table name spelling".to_string());
            recovery_suggestions.push("Use \\dt to list available tables".to_string());
            "Table not found. Please verify the table name."
        } else {
            recovery_suggestions.push("Check PostgreSQL error log for details".to_string());
            recovery_suggestions.push("Consult PostgreSQL documentation".to_string());
            "PostgreSQL database error occurred."
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
            "SEQUENCE".to_string(),
            "PRIMARY".to_string(),
            "KEY".to_string(),
            "FOREIGN".to_string(),
            "REFERENCES".to_string(),
            "UNIQUE".to_string(),
            "SERIAL".to_string(),
            "BIGSERIAL".to_string(),
            "ARRAY".to_string(),
            "JSONB".to_string(),
            "EXPLAIN".to_string(),
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
            "CURRENT_DATE".to_string(),
            "CURRENT_TIME".to_string(),
            "DATE_TRUNC".to_string(),
            "EXTRACT".to_string(),
            "AGE".to_string(),
            "COALESCE".to_string(),
            "NULLIF".to_string(),
            "ARRAY_AGG".to_string(),
            "STRING_AGG".to_string(),
            "GENERATE_SERIES".to_string(),
            "RANDOM".to_string(),
            "ROUND".to_string(),
            "FLOOR".to_string(),
            "CEIL".to_string(),
        ]
    }
}

impl PostgresConnection {
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
    #[allow(dead_code)]
    pub async fn list_databases(&self) -> Result<Vec<String>> {
        if let Some(pool) = &self.pool {
            let rows = sqlx::query("SELECT datname FROM pg_database WHERE datistemplate = false")
                .fetch_all(pool)
                .await
                .map_err(|e| {
                    LazyTablesError::Connection(format!("Failed to list databases: {e}"))
                })?;

            let databases = rows
                .iter()
                .map(|row| row.get::<String, _>("datname"))
                .collect();

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
            let query = "
                SELECT table_name
                FROM information_schema.tables
                WHERE table_schema = 'public'
                AND table_type = 'BASE TABLE'
                ORDER BY table_name
            ";

            let rows = sqlx::query(query)
                .fetch_all(pool)
                .await
                .map_err(|e| LazyTablesError::Connection(format!("Failed to list tables: {e}")))?;

            let tables = rows
                .iter()
                .map(|row| row.get::<String, _>("table_name"))
                .collect();

            Ok(tables)
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    /// List all database objects (tables, views, etc.)
    pub async fn list_database_objects(&self) -> Result<crate::database::DatabaseObjectList> {
        use crate::database::{DatabaseObject, DatabaseObjectList, DatabaseObjectType};

        if let Some(pool) = &self.pool {
            let mut result = DatabaseObjectList::default();

            // Query for all objects with proper error handling for permissions
            let query = "
                WITH object_info AS (
                    SELECT
                        n.nspname AS schema_name,
                        c.relname AS object_name,
                        CASE
                            WHEN c.relkind = 'r' THEN 'table'
                            WHEN c.relkind = 'v' THEN 'view'
                            WHEN c.relkind = 'm' THEN 'matview'
                            WHEN c.relkind = 'f' THEN 'foreign'
                        END AS object_type,
                        pg_catalog.obj_description(c.oid, 'pg_class') AS comment,
                        CASE
                            WHEN c.relkind IN ('r', 'm') THEN c.reltuples::BIGINT
                            ELSE NULL
                        END AS row_count,
                        CASE
                            WHEN c.relkind IN ('r', 'm') THEN pg_total_relation_size(c.oid)
                            ELSE NULL
                        END AS size_bytes
                    FROM pg_catalog.pg_class c
                    LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                    WHERE c.relkind IN ('r', 'v', 'm', 'f')
                        AND n.nspname NOT IN ('pg_catalog', 'information_schema')
                        AND n.nspname NOT LIKE 'pg_toast%'
                        AND n.nspname NOT LIKE 'pg_temp%'
                )
                SELECT * FROM object_info
                ORDER BY schema_name, object_type, object_name
            ";

            match sqlx::query(query).fetch_all(pool).await {
                Ok(rows) => {
                    for row in rows {
                        let schema: String = row.get("schema_name");
                        let name: String = row.get("object_name");
                        let obj_type: String = row.get("object_type");
                        let comment: Option<String> = row.get("comment");
                        let row_count: Option<i64> = row.get("row_count");
                        let size_bytes: Option<i64> = row.get("size_bytes");

                        let object_type = match obj_type.as_str() {
                            "table" => DatabaseObjectType::Table,
                            "view" => DatabaseObjectType::View,
                            "matview" => DatabaseObjectType::MaterializedView,
                            "foreign" => DatabaseObjectType::ForeignTable,
                            _ => continue,
                        };

                        let obj = DatabaseObject {
                            name,
                            schema: Some(schema),
                            object_type: object_type.clone(),
                            row_count,
                            size_bytes,
                            comment,
                        };

                        // Sort into appropriate lists
                        match object_type {
                            DatabaseObjectType::Table => result.tables.push(obj),
                            DatabaseObjectType::View => result.views.push(obj),
                            DatabaseObjectType::MaterializedView => {
                                result.materialized_views.push(obj)
                            }
                            DatabaseObjectType::ForeignTable => result.foreign_tables.push(obj),
                            _ => {}
                        }
                    }

                    result.total_count = result.tables.len()
                        + result.views.len()
                        + result.materialized_views.len()
                        + result.foreign_tables.len();
                }
                Err(e) => {
                    // Check for permission errors
                    let error_msg = e.to_string();
                    if error_msg.contains("permission denied")
                        || error_msg.contains("insufficient privilege")
                    {
                        result.error =
                            Some("Insufficient permissions to list database objects".to_string());
                    } else {
                        result.error = Some(format!("Failed to list objects: {}", e));
                    }
                }
            }

            Ok(result)
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    /// List all schemas in the database
    pub async fn list_schemas(&self) -> Result<Vec<String>> {
        if let Some(pool) = &self.pool {
            let query = "
                SELECT nspname AS schema_name
                FROM pg_catalog.pg_namespace
                WHERE nspname NOT IN ('pg_catalog', 'information_schema')
                    AND nspname NOT LIKE 'pg_toast%'
                    AND nspname NOT LIKE 'pg_temp%'
                ORDER BY nspname
            ";

            let rows = sqlx::query(query).fetch_all(pool).await.map_err(|e| {
                LazyTablesError::Connection(format!("Failed to list schemas: {}", e))
            })?;

            let schemas = rows
                .iter()
                .map(|row| row.get::<String, _>("schema_name"))
                .collect();

            Ok(schemas)
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    /// List database objects filtered by schema
    pub async fn list_database_objects_in_schema(
        &self,
        schema_name: Option<&str>,
    ) -> Result<crate::database::DatabaseObjectList> {
        use crate::database::{DatabaseObject, DatabaseObjectList, DatabaseObjectType};

        if let Some(pool) = &self.pool {
            let mut result = DatabaseObjectList::default();

            // Build schema filter
            let schema_filter = if let Some(schema) = schema_name {
                format!("AND n.nspname = '{}'", schema.replace("'", "''"))
            } else {
                "AND n.nspname NOT IN ('pg_catalog', 'information_schema')
                 AND n.nspname NOT LIKE 'pg_toast%'
                 AND n.nspname NOT LIKE 'pg_temp%'"
                    .to_string()
            };

            let query = format!(
                "
                WITH object_info AS (
                    SELECT
                        n.nspname AS schema_name,
                        c.relname AS object_name,
                        CASE
                            WHEN c.relkind = 'r' THEN 'table'
                            WHEN c.relkind = 'v' THEN 'view'
                            WHEN c.relkind = 'm' THEN 'matview'
                            WHEN c.relkind = 'f' THEN 'foreign'
                        END AS object_type,
                        pg_catalog.obj_description(c.oid, 'pg_class') AS comment,
                        CASE
                            WHEN c.relkind IN ('r', 'm') THEN c.reltuples::BIGINT
                            ELSE NULL
                        END AS row_count,
                        CASE
                            WHEN c.relkind IN ('r', 'm') THEN pg_total_relation_size(c.oid)
                            ELSE NULL
                        END AS size_bytes
                    FROM pg_catalog.pg_class c
                    LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                    WHERE c.relkind IN ('r', 'v', 'm', 'f')
                        {}
                )
                SELECT * FROM object_info
                ORDER BY schema_name, object_type, object_name
            ",
                schema_filter
            );

            match sqlx::query(&query).fetch_all(pool).await {
                Ok(rows) => {
                    for row in rows {
                        let schema: String = row.get("schema_name");
                        let name: String = row.get("object_name");
                        let obj_type: String = row.get("object_type");
                        let comment: Option<String> = row.get("comment");
                        let row_count: Option<i64> = row.get("row_count");
                        let size_bytes: Option<i64> = row.get("size_bytes");

                        let object_type = match obj_type.as_str() {
                            "table" => DatabaseObjectType::Table,
                            "view" => DatabaseObjectType::View,
                            "matview" => DatabaseObjectType::MaterializedView,
                            "foreign" => DatabaseObjectType::ForeignTable,
                            _ => continue,
                        };

                        let obj = DatabaseObject {
                            name,
                            schema: Some(schema),
                            object_type: object_type.clone(),
                            row_count,
                            size_bytes,
                            comment,
                        };

                        // Sort into appropriate lists
                        match object_type {
                            DatabaseObjectType::Table => result.tables.push(obj),
                            DatabaseObjectType::View => result.views.push(obj),
                            DatabaseObjectType::MaterializedView => {
                                result.materialized_views.push(obj)
                            }
                            DatabaseObjectType::ForeignTable => result.foreign_tables.push(obj),
                            _ => {}
                        }
                    }

                    result.total_count = result.tables.len()
                        + result.views.len()
                        + result.materialized_views.len()
                        + result.foreign_tables.len();
                }
                Err(e) => {
                    // Check for permission errors
                    let error_msg = e.to_string();
                    if error_msg.contains("permission denied")
                        || error_msg.contains("insufficient privilege")
                    {
                        result.error =
                            Some("Insufficient permissions to list database objects".to_string());
                    } else {
                        result.error = Some(format!("Failed to list objects: {}", e));
                    }
                }
            }

            Ok(result)
        } else {
            Err(LazyTablesError::Connection(
                "No active connection".to_string(),
            ))
        }
    }

    /// Get metadata for a specific table
    pub async fn get_table_metadata(&self, table_name: &str) -> Result<TableMetadata> {
        if let Some(pool) = &self.pool {
            // Parse schema and table name
            let (schema, table) = if table_name.contains('.') {
                let parts: Vec<&str> = table_name.splitn(2, '.').collect();
                (parts[0], parts[1])
            } else {
                ("public", table_name)
            };

            // First, determine the object type
            let type_query = "SELECT c.relkind::text as relkind
                FROM pg_catalog.pg_class c
                JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                WHERE n.nspname = $1 AND c.relname = $2";

            let type_row = sqlx::query(type_query)
                .bind(schema)
                .bind(table)
                .fetch_optional(pool)
                .await?;

            let is_view = if let Some(row) = type_row {
                let relkind: String = row.get("relkind");
                matches!(relkind.as_str(), "v" | "m") // v = view, m = materialized view
            } else {
                false
            };

            // Get row count (skip for regular views)
            let row_count = if !is_view {
                let count_query = format!(
                    "SELECT COUNT(*) FROM {}.{}",
                    schema.replace("'", "''"),
                    table.replace("'", "''")
                );
                match sqlx::query(&count_query).fetch_one(pool).await {
                    Ok(row) => row.get::<i64, _>(0),
                    Err(_) => 0, // Default to 0 if we can't get count
                }
            } else {
                0 // Views don't have direct row counts
            };

            // Get column count
            let columns_query = "SELECT COUNT(*) FROM information_schema.columns
                                WHERE table_schema = $1 AND table_name = $2";
            let col_row = sqlx::query(columns_query)
                .bind(schema)
                .bind(table)
                .fetch_one(pool)
                .await?;
            let column_count: i64 = col_row.get(0);

            // Get table size
            let size_query = "SELECT 
                pg_size_pretty(pg_total_relation_size($1)) as total_size,
                pg_size_pretty(pg_table_size($1)) as table_size,
                pg_size_pretty(pg_indexes_size($1)) as indexes_size,
                pg_total_relation_size($1) as total_bytes,
                pg_table_size($1) as table_bytes,
                pg_indexes_size($1) as index_bytes";

            // Get size (skip for regular views as they don't have physical storage)
            let (total_size, table_size, indexes_size) = if !is_view {
                let qualified_name =
                    format!("{}.{}", schema.replace("'", "''"), table.replace("'", "''"));

                match sqlx::query(size_query)
                    .bind(&qualified_name)
                    .fetch_one(pool)
                    .await
                {
                    Ok(row) => (
                        row.get::<i64, _>("total_bytes"),
                        row.get::<i64, _>("table_bytes"),
                        row.get::<i64, _>("index_bytes"),
                    ),
                    Err(_) => (0, 0, 0), // Default to 0 if size query fails
                }
            } else {
                (0, 0, 0) // Views don't have physical storage
            };

            // Get primary keys
            let pk_query = "SELECT a.attname 
                           FROM pg_index i
                           JOIN pg_attribute a ON a.attrelid = i.indrelid
                           AND a.attnum = ANY(i.indkey)
                           WHERE i.indrelid = $1::regclass
                           AND i.indisprimary";

            let qualified_name =
                format!("{}.{}", schema.replace("'", "''"), table.replace("'", "''"));

            let pk_rows: Vec<sqlx::postgres::PgRow> = sqlx::query(pk_query)
                .bind(&qualified_name)
                .fetch_all(pool)
                .await
                .unwrap_or_default();

            let primary_keys: Vec<String> = pk_rows
                .iter()
                .map(|row| row.get::<String, _>("attname"))
                .collect();

            // Get foreign keys
            let fk_query = "
                SELECT 
                    kcu.column_name || ' → ' || ccu.table_name || '.' || ccu.column_name as fk_info
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                JOIN information_schema.constraint_column_usage ccu
                    ON ccu.constraint_name = tc.constraint_name
                    AND ccu.table_schema = tc.table_schema
                WHERE tc.constraint_type = 'FOREIGN KEY' 
                    AND tc.table_name = $1
                    AND tc.table_schema = $2";

            let fk_rows: Vec<sqlx::postgres::PgRow> = sqlx::query(fk_query)
                .bind(table)
                .bind(schema)
                .fetch_all(pool)
                .await
                .unwrap_or_default();

            let foreign_keys: Vec<String> = fk_rows
                .iter()
                .map(|row| row.get::<String, _>("fk_info"))
                .collect();

            // Get indexes
            let index_query = "
                SELECT indexname 
                FROM pg_indexes 
                WHERE tablename = $1
                AND schemaname = $2";

            let index_rows: Vec<sqlx::postgres::PgRow> = sqlx::query(index_query)
                .bind(table)
                .bind(schema)
                .fetch_all(pool)
                .await
                .unwrap_or_default();

            let indexes: Vec<String> = index_rows
                .iter()
                .map(|row| row.get::<String, _>("indexname"))
                .collect();

            // Get table comment
            let comment_query = "SELECT obj_description($1::regclass, 'pg_class') as comment";

            let comment_row = match sqlx::query(comment_query)
                .bind(&qualified_name)
                .fetch_one(pool)
                .await
            {
                Ok(row) => row,
                Err(_) => {
                    return Ok(TableMetadata {
                        table_name: table_name.to_string(),
                        row_count: row_count as usize,
                        column_count: column_count as usize,
                        total_size,
                        table_size,
                        indexes_size,
                        primary_keys,
                        foreign_keys,
                        indexes,
                        comment: None,
                    })
                }
            };

            let comment: Option<String> = comment_row.get("comment");

            Ok(TableMetadata {
                table_name: table_name.to_string(),
                row_count: row_count as usize,
                column_count: column_count as usize,
                total_size,
                table_size,
                indexes_size,
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

    /// Get column information for a table
    pub async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>> {
        if let Some(pool) = &self.pool {
            let query = "SELECT 
                c.column_name,
                c.data_type,
                c.is_nullable,
                c.column_default,
                CASE 
                    WHEN pk.column_name IS NOT NULL THEN true 
                    ELSE false 
                END as is_primary_key
                FROM information_schema.columns c
                LEFT JOIN (
                    SELECT kcu.column_name
                    FROM information_schema.table_constraints tc
                    JOIN information_schema.key_column_usage kcu
                        ON tc.constraint_name = kcu.constraint_name
                        AND tc.table_schema = kcu.table_schema
                    WHERE tc.constraint_type = 'PRIMARY KEY'
                        AND tc.table_name = $1
                        AND tc.table_schema = 'public'
                ) pk ON c.column_name = pk.column_name
                WHERE c.table_schema = 'public' 
                AND c.table_name = $1
                ORDER BY c.ordinal_position";

            let rows = sqlx::query(query).bind(table_name).fetch_all(pool).await?;

            let columns = rows
                .iter()
                .map(|row| {
                    let column_name: String = row.get("column_name");
                    let data_type_str: String = row.get("data_type");
                    let is_nullable: String = row.get("is_nullable");
                    let column_default: Option<String> = row.get("column_default");
                    let is_primary_key: bool = row.get("is_primary_key");

                    TableColumn {
                        name: column_name,
                        data_type: parse_postgres_type(&data_type_str),
                        is_nullable: is_nullable == "YES",
                        default_value: column_default,
                        is_primary_key,
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
            // Use the same qualified name logic as get_table_data
            let qualified_name = if table_name.contains('.') {
                table_name.to_string()
            } else {
                format!("public.{}", table_name)
            };

            let query = format!("SELECT COUNT(*) FROM {qualified_name}");
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
            // Parse schema and table name
            let (schema, table) = if table_name.contains('.') {
                let parts: Vec<&str> = table_name.splitn(2, '.').collect();
                (parts[0], parts[1])
            } else {
                ("public", table_name)
            };

            // Get column names first to maintain order
            let columns_query = "
                SELECT column_name
                FROM information_schema.columns
                WHERE table_name = $1 AND table_schema = $2
                ORDER BY ordinal_position
            ";

            let column_rows = sqlx::query(columns_query)
                .bind(table)
                .bind(schema)
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
                .map(|col| format!("\"{col}\"::text"))
                .collect::<Vec<_>>()
                .join(", ");

            let qualified_name = if table_name.contains('.') {
                table_name.to_string()
            } else {
                format!("public.{}", table_name)
            };

            let query = format!(
                "SELECT {select_list} FROM {qualified_name} ORDER BY 1 LIMIT {limit} OFFSET {offset}"
            );

            let rows = sqlx::query(&query).fetch_all(pool).await?;

            let mut result = Vec::new();
            for row in rows {
                let mut row_data = Vec::new();
                for (idx, _col_name) in column_names.iter().enumerate() {
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

impl PostgresConnection {
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

/// Parse PostgreSQL data type string to internal DataType enum
fn parse_postgres_type(type_str: &str) -> DataType {
    match type_str {
        "integer" | "int4" => DataType::Integer,
        "bigint" | "int8" => DataType::BigInt,
        "smallint" | "int2" => DataType::SmallInt,
        "numeric" | "decimal" => DataType::Decimal,
        "real" | "float4" => DataType::Float,
        "double precision" | "float8" => DataType::Double,
        "boolean" | "bool" => DataType::Boolean,
        "text" => DataType::Text,
        "character varying" | "varchar" => DataType::Varchar(None),
        "character" | "char" => DataType::Char(None),
        "date" => DataType::Date,
        "time" | "time without time zone" => DataType::Time,
        "timestamp" | "timestamp without time zone" | "timestamp with time zone" => {
            DataType::Timestamp
        }
        "json" | "jsonb" => DataType::Json,
        "uuid" => DataType::Uuid,
        "bytea" => DataType::Bytea,
        s if s.starts_with("ARRAY") => DataType::Text, // Simplified for now
        _ => DataType::Text,
    }
}
