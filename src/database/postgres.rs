// FilePath: src/database/postgres.rs

use crate::{
    core::error::Result,
    database::{Connection, ConnectionConfig},
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
}
