// FilePath: src/database/postgres.rs

use crate::{
    core::error::Result,
    database::{Connection, ConnectionConfig},
};
use sqlx::{postgres::PgPool, Pool, Postgres};

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

