// FilePath: src/database/app_state.rs

use crate::config::Config;
use crate::core::error::Result;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::path::PathBuf;

/// Application state stored in local SQLite database
#[derive(Debug, Clone)]
pub struct AppStateDb {
    /// Connection pool for the app state database
    pool: Option<SqlitePool>,
}

/// Represents the current active connection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveConnectionState {
    /// Connection ID that is currently active
    pub connection_id: Option<String>,
    /// Connection name for display
    pub connection_name: Option<String>,
    /// Database type of active connection
    pub database_type: Option<String>,
    /// Timestamp when connection became active
    pub connected_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl AppStateDb {
    /// Create a new app state database instance
    pub fn new() -> Self {
        Self { pool: None }
    }

    /// Initialize the application state database
    pub async fn initialize() -> Result<Self> {
        let db_path = Self::database_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create connection string
        let database_url = format!("sqlite:{}", db_path.display());

        // Create connection pool
        let pool = SqlitePool::connect(&database_url).await?;

        let app_db = Self { pool: Some(pool) };

        // Initialize schema
        app_db.create_schema().await?;

        Ok(app_db)
    }

    /// Get the path to the application state database
    pub fn database_path() -> PathBuf {
        Config::data_dir().join("app_state.db")
    }

    /// Create the database schema if it doesn't exist
    async fn create_schema(&self) -> Result<()> {
        if let Some(ref pool) = self.pool {
            // Create active_connection table to track which connection is currently active
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS active_connection (
                    id INTEGER PRIMARY KEY,
                    connection_id TEXT,
                    connection_name TEXT,
                    database_type TEXT,
                    connected_at DATETIME,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )
                "#,
            )
            .execute(pool)
            .await?;

            // Create connection_sessions table to track connection history
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS connection_sessions (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    connection_id TEXT NOT NULL,
                    connection_name TEXT NOT NULL,
                    database_type TEXT NOT NULL,
                    connected_at DATETIME NOT NULL,
                    disconnected_at DATETIME,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )
                "#,
            )
            .execute(pool)
            .await?;

            // Create sql_file_activity table to track SQL file usage per connection
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS sql_file_activity (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    connection_id TEXT NOT NULL,
                    file_path TEXT NOT NULL,
                    file_name TEXT NOT NULL,
                    last_opened DATETIME,
                    last_modified DATETIME,
                    open_count INTEGER DEFAULT 0,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(connection_id, file_path)
                )
                "#,
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Set the currently active connection
    pub async fn set_active_connection(
        &self,
        connection_id: &str,
        connection_name: &str,
        database_type: &str,
    ) -> Result<()> {
        if let Some(ref pool) = self.pool {
            let now = chrono::Utc::now();

            // First, clear any existing active connection
            sqlx::query("DELETE FROM active_connection")
                .execute(pool)
                .await?;

            // Insert the new active connection
            sqlx::query(
                r#"
                INSERT INTO active_connection (connection_id, connection_name, database_type, connected_at)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(connection_id)
            .bind(connection_name)
            .bind(database_type)
            .bind(now)
            .execute(pool)
            .await?;

            // Record this session in the history
            sqlx::query(
                r#"
                INSERT INTO connection_sessions (connection_id, connection_name, database_type, connected_at)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(connection_id)
            .bind(connection_name)
            .bind(database_type)
            .bind(now)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Clear the currently active connection
    pub async fn clear_active_connection(&self) -> Result<()> {
        if let Some(ref pool) = self.pool {
            // Update the last session's disconnected_at time
            sqlx::query(
                r#"
                UPDATE connection_sessions
                SET disconnected_at = ?
                WHERE disconnected_at IS NULL
                "#,
            )
            .bind(chrono::Utc::now())
            .execute(pool)
            .await?;

            // Clear the active connection
            sqlx::query("DELETE FROM active_connection")
                .execute(pool)
                .await?;
        }

        Ok(())
    }

    /// Get the currently active connection
    pub async fn get_active_connection(&self) -> Result<Option<ActiveConnectionState>> {
        if let Some(ref pool) = self.pool {
            let row = sqlx::query(
                "SELECT connection_id, connection_name, database_type, connected_at FROM active_connection LIMIT 1"
            )
            .fetch_optional(pool)
            .await?;

            if let Some(row) = row {
                return Ok(Some(ActiveConnectionState {
                    connection_id: row.get("connection_id"),
                    connection_name: row.get("connection_name"),
                    database_type: row.get("database_type"),
                    connected_at: row.get("connected_at"),
                }));
            }
        }

        Ok(None)
    }

    /// Check if a specific connection is currently active
    pub async fn is_connection_active(&self, connection_id: &str) -> Result<bool> {
        if let Some(active) = self.get_active_connection().await? {
            Ok(active.connection_id.as_deref() == Some(connection_id))
        } else {
            Ok(false)
        }
    }

    /// Record SQL file activity for the active connection
    pub async fn record_sql_file_activity(
        &self,
        connection_id: &str,
        file_path: &str,
        file_name: &str,
    ) -> Result<()> {
        if let Some(ref pool) = self.pool {
            let now = chrono::Utc::now();

            // Use INSERT OR REPLACE to update existing records or create new ones
            sqlx::query(
                r#"
                INSERT INTO sql_file_activity (connection_id, file_path, file_name, last_opened, last_modified, open_count, updated_at)
                VALUES (?, ?, ?, ?, ?, 1, ?)
                ON CONFLICT(connection_id, file_path) DO UPDATE SET
                    last_opened = excluded.last_opened,
                    last_modified = excluded.last_modified,
                    open_count = open_count + 1,
                    updated_at = excluded.updated_at
                "#,
            )
            .bind(connection_id)
            .bind(file_path)
            .bind(file_name)
            .bind(now)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Get SQL file activity for a specific connection
    pub async fn get_sql_file_activity(&self, connection_id: &str) -> Result<Vec<SqlFileActivity>> {
        if let Some(ref pool) = self.pool {
            let rows = sqlx::query(
                r#"
                SELECT file_path, file_name, last_opened, last_modified, open_count
                FROM sql_file_activity
                WHERE connection_id = ?
                ORDER BY last_opened DESC
                "#,
            )
            .bind(connection_id)
            .fetch_all(pool)
            .await?;

            let activities = rows
                .into_iter()
                .map(|row| SqlFileActivity {
                    file_path: row.get("file_path"),
                    file_name: row.get("file_name"),
                    last_opened: row.get("last_opened"),
                    last_modified: row.get("last_modified"),
                    open_count: row.get("open_count"),
                })
                .collect();

            return Ok(activities);
        }

        Ok(Vec::new())
    }

    /// Get connection session history
    pub async fn get_connection_history(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<ConnectionSession>> {
        if let Some(ref pool) = self.pool {
            let query = if let Some(limit) = limit {
                format!(
                    "SELECT connection_id, connection_name, database_type, connected_at, disconnected_at
                     FROM connection_sessions
                     ORDER BY connected_at DESC
                     LIMIT {}",
                    limit
                )
            } else {
                "SELECT connection_id, connection_name, database_type, connected_at, disconnected_at
                 FROM connection_sessions
                 ORDER BY connected_at DESC"
                    .to_string()
            };

            let rows = sqlx::query(&query).fetch_all(pool).await?;

            let sessions = rows
                .into_iter()
                .map(|row| ConnectionSession {
                    connection_id: row.get("connection_id"),
                    connection_name: row.get("connection_name"),
                    database_type: row.get("database_type"),
                    connected_at: row.get("connected_at"),
                    disconnected_at: row.get("disconnected_at"),
                })
                .collect();

            return Ok(sessions);
        }

        Ok(Vec::new())
    }
}

/// SQL file activity record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlFileActivity {
    pub file_path: String,
    pub file_name: String,
    pub last_opened: Option<chrono::DateTime<chrono::Utc>>,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub open_count: i64,
}

/// Connection session record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSession {
    pub connection_id: String,
    pub connection_name: String,
    pub database_type: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub disconnected_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for AppStateDb {
    fn default() -> Self {
        Self::new()
    }
}
