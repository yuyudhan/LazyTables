// FilePath: src/database/postgres.rs

use crate::{
    core::error::Result,
    database::{Connection, ConnectionInfo},
};
use sqlx::{postgres::PgPool, Pool, Postgres};

/// PostgreSQL connection implementation
pub struct PostgresConnection {
    info: ConnectionInfo,
    pool: Option<Pool<Postgres>>,
}

impl PostgresConnection {
    /// Create a new PostgreSQL connection
    pub fn new(info: ConnectionInfo) -> Self {
        Self { info, pool: None }
    }
}

#[async_trait::async_trait]
impl Connection for PostgresConnection {
    async fn connect(&mut self) -> Result<()> {
        let connection_string = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.info.username,
            self.info.password.as_deref().unwrap_or(""),
            self.info.host,
            self.info.port,
            self.info.database
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

    fn info(&self) -> &ConnectionInfo {
        &self.info
    }
}

