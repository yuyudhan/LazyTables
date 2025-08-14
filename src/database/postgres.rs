// FilePath: src/database/postgres.rs

use crate::{
    core::error::Result,
    database::{Connection, ConnectionConfig, TableMetadata},
};
use sqlx::{postgres::PgPool, Pool, Postgres, Row};

/// PostgreSQL connection implementation
pub struct PostgresConnection {
    config: ConnectionConfig,
    pool: Option<Pool<Postgres>>,
}

impl PostgresConnection {
    /// Create a new PostgreSQL connection
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config, pool: None }
    }
}

#[async_trait::async_trait]
impl Connection for PostgresConnection {
    async fn connect(&mut self) -> Result<()> {
        let connection_string = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.config.username,
            self.config.password.as_deref().unwrap_or(""),
            self.config.host,
            self.config.port,
            self.config.database.as_deref().unwrap_or("postgres")
        );

        let pool = PgPool::connect(&connection_string).await?;
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

impl PostgresConnection {
    /// Query all tables from the connected database
    pub async fn get_tables(&self) -> Result<Vec<String>> {
        if let Some(pool) = &self.pool {
            let query = "
                SELECT table_name 
                FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_type = 'BASE TABLE'
                ORDER BY table_name
            ";

            let rows = sqlx::query(query).fetch_all(pool).await?;

            let tables: Vec<String> = rows
                .iter()
                .map(|row| row.get::<String, _>("table_name"))
                .collect();

            Ok(tables)
        } else {
            Err(crate::core::error::LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }

    /// Execute SQL statement (for DDL operations like CREATE TABLE)
    pub async fn execute_sql(&self, sql: &str) -> Result<()> {
        if let Some(pool) = &self.pool {
            sqlx::query(sql).execute(pool).await?;
            Ok(())
        } else {
            Err(crate::core::error::LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }

    /// Get table columns information
    pub async fn get_table_columns(
        &self,
        table_name: &str,
    ) -> Result<Vec<crate::ui::components::ColumnDefinition>> {
        use crate::ui::components::{ColumnDefinition, PostgresDataType};

        if let Some(pool) = &self.pool {
            let query = "
                SELECT 
                    c.column_name,
                    c.data_type,
                    c.character_maximum_length,
                    c.numeric_precision,
                    c.numeric_scale,
                    c.is_nullable,
                    c.column_default,
                    CASE 
                        WHEN pk.column_name IS NOT NULL THEN true 
                        ELSE false 
                    END as is_primary_key,
                    CASE 
                        WHEN u.column_name IS NOT NULL THEN true 
                        ELSE false 
                    END as is_unique
                FROM information_schema.columns c
                LEFT JOIN (
                    SELECT kcu.column_name
                    FROM information_schema.table_constraints tc
                    JOIN information_schema.key_column_usage kcu
                        ON tc.constraint_name = kcu.constraint_name
                        AND tc.table_schema = kcu.table_schema
                    WHERE tc.table_name = $1
                        AND tc.constraint_type = 'PRIMARY KEY'
                        AND tc.table_schema = 'public'
                ) pk ON c.column_name = pk.column_name
                LEFT JOIN (
                    SELECT kcu.column_name
                    FROM information_schema.table_constraints tc
                    JOIN information_schema.key_column_usage kcu
                        ON tc.constraint_name = kcu.constraint_name
                        AND tc.table_schema = kcu.table_schema
                    WHERE tc.table_name = $1
                        AND tc.constraint_type = 'UNIQUE'
                        AND tc.table_schema = 'public'
                ) u ON c.column_name = u.column_name
                WHERE c.table_name = $1
                    AND c.table_schema = 'public'
                ORDER BY c.ordinal_position
            ";

            let rows = sqlx::query(query).bind(table_name).fetch_all(pool).await?;

            let columns: Vec<ColumnDefinition> = rows
                .iter()
                .map(|row| {
                    let column_name: String = row.get("column_name");
                    let data_type_str: String = row.get("data_type");
                    let char_max_length: Option<i32> = row.get("character_maximum_length");
                    let is_nullable: String = row.get("is_nullable");
                    let column_default: Option<String> = row.get("column_default");
                    let is_primary_key: bool = row.get("is_primary_key");
                    let is_unique: bool = row.get("is_unique");

                    // Map PostgreSQL data types to our enum
                    let data_type = match data_type_str.as_str() {
                        "smallint" => PostgresDataType::SmallInt,
                        "integer" => PostgresDataType::Integer,
                        "bigint" => PostgresDataType::BigInt,
                        "numeric" | "decimal" => PostgresDataType::Numeric,
                        "real" => PostgresDataType::Real,
                        "double precision" => PostgresDataType::DoublePrecision,
                        "text" => PostgresDataType::Text,
                        "character varying" => {
                            PostgresDataType::CharacterVarying(char_max_length.map(|v| v as u32))
                        }
                        "character" => {
                            PostgresDataType::Character(char_max_length.map(|v| v as u32))
                        }
                        "boolean" => PostgresDataType::Boolean,
                        "date" => PostgresDataType::Date,
                        "timestamp without time zone" => PostgresDataType::Timestamp,
                        "timestamp with time zone" => PostgresDataType::TimestampWithTimeZone,
                        "json" => PostgresDataType::Json,
                        "jsonb" => PostgresDataType::Jsonb,
                        "uuid" => PostgresDataType::Uuid,
                        _ => PostgresDataType::Custom(data_type_str),
                    };

                    ColumnDefinition {
                        name: column_name,
                        data_type,
                        is_nullable: is_nullable == "YES",
                        is_primary_key,
                        is_unique,
                        default_value: column_default,
                        check_constraint: None,
                        references: None,
                    }
                })
                .collect();

            Ok(columns)
        } else {
            Err(crate::core::error::LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }

    /// Get table row count
    pub async fn get_table_row_count(&self, table_name: &str) -> Result<usize> {
        if let Some(pool) = &self.pool {
            let query = format!("SELECT COUNT(*) as count FROM {table_name}");
            let row = sqlx::query(&query).fetch_one(pool).await?;
            let count: i64 = row.get("count");
            Ok(count as usize)
        } else {
            Err(crate::core::error::LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }

    /// Get detailed table metadata
    pub async fn get_table_metadata(&self, table_name: &str) -> Result<TableMetadata> {
        if let Some(pool) = &self.pool {
            // Get basic table info including size
            let size_query = "
                SELECT 
                    pg_size_pretty(pg_total_relation_size($1::regclass)) as total_size,
                    pg_size_pretty(pg_relation_size($1::regclass)) as table_size,
                    pg_size_pretty(pg_indexes_size($1::regclass)) as indexes_size
            ";

            let size_row = sqlx::query(size_query)
                .bind(format!("public.{}", table_name))
                .fetch_one(pool)
                .await?;

            let total_size: String = size_row.get("total_size");
            let table_size: String = size_row.get("table_size");
            let indexes_size: String = size_row.get("indexes_size");

            // Get row count
            let row_count = self.get_table_row_count(table_name).await?;

            // Get column count
            let col_count_query = "
                SELECT COUNT(*) as count 
                FROM information_schema.columns 
                WHERE table_name = $1 AND table_schema = 'public'
            ";

            let col_row = sqlx::query(col_count_query)
                .bind(table_name)
                .fetch_one(pool)
                .await?;

            let column_count: i64 = col_row.get("count");

            // Get primary key columns
            let pk_query = "
                SELECT kcu.column_name
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                WHERE tc.table_name = $1
                    AND tc.constraint_type = 'PRIMARY KEY'
                    AND tc.table_schema = 'public'
                ORDER BY kcu.ordinal_position
            ";

            let pk_rows = sqlx::query(pk_query)
                .bind(table_name)
                .fetch_all(pool)
                .await?;

            let primary_keys: Vec<String> = pk_rows
                .iter()
                .map(|row| row.get::<String, _>("column_name"))
                .collect();

            // Get foreign keys
            let fk_query = "
                SELECT 
                    kcu.column_name,
                    ccu.table_name AS foreign_table,
                    ccu.column_name AS foreign_column
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                JOIN information_schema.constraint_column_usage ccu
                    ON tc.constraint_name = ccu.constraint_name
                    AND tc.table_schema = ccu.table_schema
                WHERE tc.table_name = $1
                    AND tc.constraint_type = 'FOREIGN KEY'
                    AND tc.table_schema = 'public'
            ";

            let fk_rows = sqlx::query(fk_query)
                .bind(table_name)
                .fetch_all(pool)
                .await?;

            let foreign_keys: Vec<String> = fk_rows
                .iter()
                .map(|row| {
                    let column: String = row.get("column_name");
                    let foreign_table: String = row.get("foreign_table");
                    let foreign_column: String = row.get("foreign_column");
                    format!("{} → {}.{}", column, foreign_table, foreign_column)
                })
                .collect();

            // Get indexes
            let index_query = "
                SELECT 
                    indexname,
                    indexdef
                FROM pg_indexes
                WHERE tablename = $1
                    AND schemaname = 'public'
            ";

            let index_rows = sqlx::query(index_query)
                .bind(table_name)
                .fetch_all(pool)
                .await?;

            let indexes: Vec<String> = index_rows
                .iter()
                .map(|row| row.get::<String, _>("indexname"))
                .collect();

            // Get table comment if any
            let comment_query = "
                SELECT obj_description($1::regclass, 'pg_class') as comment
            ";

            let comment_row = sqlx::query(comment_query)
                .bind(format!("public.{}", table_name))
                .fetch_one(pool)
                .await?;

            let comment: Option<String> = comment_row.get("comment");

            Ok(TableMetadata {
                table_name: table_name.to_string(),
                row_count,
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
            Err(crate::core::error::LazyTablesError::Connection(
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
            let columns_query = "
                SELECT column_name 
                FROM information_schema.columns 
                WHERE table_name = $1 AND table_schema = 'public'
                ORDER BY ordinal_position
            ";

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
                .map(|col| format!("\"{col}\"::text"))
                .collect::<Vec<_>>()
                .join(", ");

            let query = format!(
                "SELECT {select_list} FROM {table_name} ORDER BY 1 LIMIT {limit} OFFSET {offset}"
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
            Err(crate::core::error::LazyTablesError::Connection(
                "Not connected to database".to_string(),
            ))
        }
    }
}
