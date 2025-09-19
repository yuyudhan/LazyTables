// FilePath: src/database/query_history.rs

use crate::core::error::{LazyTablesError, Result};
use crate::database::DatabaseType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::path::PathBuf;

/// Query history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryHistoryEntry {
    pub id: i64,
    pub query_text: String,
    pub database_type: DatabaseType,
    pub database_name: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub executed_at: DateTime<Utc>,
    pub execution_time_ms: Option<i64>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Query history manager for local SQLite storage
pub struct QueryHistoryManager {
    pool: Option<SqlitePool>,
    db_path: PathBuf,
}

impl QueryHistoryManager {
    /// Create a new query history manager
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            LazyTablesError::Config("Could not determine home directory".to_string())
        })?;

        let lazytables_dir = home_dir.join(".lazytables");
        std::fs::create_dir_all(&lazytables_dir)?;

        let db_path = lazytables_dir.join("query_history.db");

        Ok(Self {
            pool: None,
            db_path,
        })
    }

    /// Initialize the database connection and create tables
    pub async fn initialize(&mut self) -> Result<()> {
        let database_url = format!("sqlite:{}", self.db_path.display());

        let pool = SqlitePool::connect(&database_url).await
            .map_err(|e| LazyTablesError::Config(format!("Failed to connect to query history database: {}", e)))?;

        // Create the query_history table with database context
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS query_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                query_text TEXT NOT NULL,
                database_type TEXT NOT NULL,
                database_name TEXT,
                executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                execution_time_ms INTEGER,
                success BOOLEAN DEFAULT 1,
                error_message TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| LazyTablesError::Config(format!("Failed to create query_history table: {}", e)))?;

        // Create index for efficient querying by database type
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_query_history_database_type
            ON query_history(database_type, executed_at DESC)
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| LazyTablesError::Config(format!("Failed to create index: {}", e)))?;

        self.pool = Some(pool);
        Ok(())
    }

    /// Add a query to history
    pub async fn add_query(
        &self,
        query_text: &str,
        database_type: DatabaseType,
        database_name: Option<&str>,
        execution_time_ms: Option<i64>,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<i64> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            LazyTablesError::Config("Query history database not initialized".to_string())
        })?;

        let row = sqlx::query(
            r#"
            INSERT INTO query_history
            (query_text, database_type, database_name, execution_time_ms, success, error_message)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(query_text)
        .bind(database_type.display_name())
        .bind(database_name)
        .bind(execution_time_ms)
        .bind(success)
        .bind(error_message)
        .fetch_one(pool)
        .await
        .map_err(|e| LazyTablesError::Config(format!("Failed to add query to history: {}", e)))?;

        Ok(row.get(0))
    }

    /// Get query history with optional database type filter
    pub async fn get_history(
        &self,
        database_type_filter: Option<DatabaseType>,
        limit: Option<i64>,
    ) -> Result<Vec<QueryHistoryEntry>> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            LazyTablesError::Config("Query history database not initialized".to_string())
        })?;

        let (query, params): (String, Vec<String>) = match database_type_filter {
            Some(db_type) => (
                "SELECT * FROM query_history WHERE database_type = ? ORDER BY executed_at DESC LIMIT ?".to_string(),
                vec![db_type.display_name().to_string(), limit.unwrap_or(50).to_string()]
            ),
            None => (
                "SELECT * FROM query_history ORDER BY executed_at DESC LIMIT ?".to_string(),
                vec![limit.unwrap_or(50).to_string()]
            ),
        };

        let mut query_builder = sqlx::query(&query);
        for param in &params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder
            .fetch_all(pool)
            .await
            .map_err(|e| LazyTablesError::Config(format!("Failed to fetch query history: {}", e)))?;

        let mut entries = Vec::new();
        for row in rows {
            let database_type_str: String = row.get("database_type");
            let database_type = match database_type_str.as_str() {
                "postgres" => DatabaseType::PostgreSQL,
                "mysql" => DatabaseType::MySQL,
                "mariadb" => DatabaseType::MariaDB,
                "sqlite" => DatabaseType::SQLite,
                "oracle" => DatabaseType::Oracle,
                "redis" => DatabaseType::Redis,
                "mongodb" => DatabaseType::MongoDB,
                _ => continue, // Skip unknown database types
            };

            let executed_at_str: String = row.get("executed_at");
            let executed_at = DateTime::parse_from_rfc3339(&executed_at_str)
                .unwrap_or_else(|_| DateTime::parse_from_str(&executed_at_str, "%Y-%m-%d %H:%M:%S%.f").unwrap_or_default())
                .with_timezone(&Utc);

            entries.push(QueryHistoryEntry {
                id: row.get("id"),
                query_text: row.get("query_text"),
                database_type,
                database_name: row.get("database_name"),
                executed_at,
                execution_time_ms: row.get("execution_time_ms"),
                success: row.get("success"),
                error_message: row.get("error_message"),
            });
        }

        Ok(entries)
    }

    /// Get recent queries for a specific database type
    pub async fn get_recent_queries(
        &self,
        database_type: DatabaseType,
        limit: i64,
    ) -> Result<Vec<String>> {
        let entries = self.get_history(Some(database_type), Some(limit)).await?;
        Ok(entries.into_iter().map(|e| e.query_text).collect())
    }

    /// Search query history by text content
    pub async fn search_history(
        &self,
        search_term: &str,
        database_type_filter: Option<DatabaseType>,
        limit: Option<i64>,
    ) -> Result<Vec<QueryHistoryEntry>> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            LazyTablesError::Config("Query history database not initialized".to_string())
        })?;

        let (query, params): (String, Vec<String>) = match database_type_filter {
            Some(db_type) => (
                "SELECT * FROM query_history WHERE query_text LIKE ? AND database_type = ? ORDER BY executed_at DESC LIMIT ?".to_string(),
                vec![
                    format!("%{}%", search_term),
                    db_type.display_name().to_string(),
                    limit.unwrap_or(50).to_string()
                ]
            ),
            None => (
                "SELECT * FROM query_history WHERE query_text LIKE ? ORDER BY executed_at DESC LIMIT ?".to_string(),
                vec![
                    format!("%{}%", search_term),
                    limit.unwrap_or(50).to_string()
                ]
            ),
        };

        let mut query_builder = sqlx::query(&query);
        for param in &params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder
            .fetch_all(pool)
            .await
            .map_err(|e| LazyTablesError::Config(format!("Failed to search query history: {}", e)))?;

        let mut entries = Vec::new();
        for row in rows {
            let database_type_str: String = row.get("database_type");
            let database_type = match database_type_str.as_str() {
                "postgres" => DatabaseType::PostgreSQL,
                "mysql" => DatabaseType::MySQL,
                "mariadb" => DatabaseType::MariaDB,
                "sqlite" => DatabaseType::SQLite,
                "oracle" => DatabaseType::Oracle,
                "redis" => DatabaseType::Redis,
                "mongodb" => DatabaseType::MongoDB,
                _ => continue,
            };

            let executed_at_str: String = row.get("executed_at");
            let executed_at = DateTime::parse_from_rfc3339(&executed_at_str)
                .unwrap_or_else(|_| DateTime::parse_from_str(&executed_at_str, "%Y-%m-%d %H:%M:%S%.f").unwrap_or_default())
                .with_timezone(&Utc);

            entries.push(QueryHistoryEntry {
                id: row.get("id"),
                query_text: row.get("query_text"),
                database_type,
                database_name: row.get("database_name"),
                executed_at,
                execution_time_ms: row.get("execution_time_ms"),
                success: row.get("success"),
                error_message: row.get("error_message"),
            });
        }

        Ok(entries)
    }

    /// Remove duplicate queries (keep most recent)
    pub async fn deduplicate_queries(&self) -> Result<usize> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            LazyTablesError::Config("Query history database not initialized".to_string())
        })?;

        let result = sqlx::query(
            r#"
            DELETE FROM query_history
            WHERE id NOT IN (
                SELECT MIN(id) FROM query_history
                GROUP BY query_text, database_type
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| LazyTablesError::Config(format!("Failed to deduplicate queries: {}", e)))?;

        Ok(result.rows_affected() as usize)
    }

    /// Clear old query history (keep only recent entries)
    pub async fn clear_old_history(&self, keep_count: i64) -> Result<usize> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            LazyTablesError::Config("Query history database not initialized".to_string())
        })?;

        let result = sqlx::query(
            r#"
            DELETE FROM query_history
            WHERE id NOT IN (
                SELECT id FROM query_history
                ORDER BY executed_at DESC
                LIMIT ?
            )
            "#,
        )
        .bind(keep_count)
        .execute(pool)
        .await
        .map_err(|e| LazyTablesError::Config(format!("Failed to clear old history: {}", e)))?;

        Ok(result.rows_affected() as usize)
    }
}

impl Default for QueryHistoryManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            pool: None,
            db_path: PathBuf::from("query_history.db"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_query_history_creation() -> Result<()> {
        let dir = tempdir().unwrap();
        std::env::set_var("TMPDIR", dir.path());
        let db_path = dir.path().join("test_history.db");

        let mut manager = QueryHistoryManager {
            pool: None,
            db_path,
        };

        manager.initialize().await?;

        let id = manager.add_query(
            "SELECT * FROM users",
            DatabaseType::PostgreSQL,
            Some("test_db"),
            Some(150),
            true,
            None,
        ).await?;

        assert!(id > 0);

        let history = manager.get_history(None, Some(10)).await?;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].query_text, "SELECT * FROM users");
        assert_eq!(history[0].database_type, DatabaseType::PostgreSQL);

        Ok(())
    }

    #[tokio::test]
    async fn test_database_type_filtering() -> Result<()> {
        let dir = tempdir().unwrap();
        std::env::set_var("TMPDIR", dir.path());
        let db_path = dir.path().join("test_filter.db");

        let mut manager = QueryHistoryManager {
            pool: None,
            db_path,
        };

        manager.initialize().await?;

        // Add queries for different database types
        manager.add_query(
            "SELECT * FROM postgres_table",
            DatabaseType::PostgreSQL,
            Some("pg_db"),
            Some(100),
            true,
            None,
        ).await?;

        manager.add_query(
            "SELECT * FROM mysql_table",
            DatabaseType::MySQL,
            Some("mysql_db"),
            Some(200),
            true,
            None,
        ).await?;

        // Test filtering
        let pg_history = manager.get_history(Some(DatabaseType::PostgreSQL), Some(10)).await?;
        assert_eq!(pg_history.len(), 1);
        assert_eq!(pg_history[0].database_type, DatabaseType::PostgreSQL);

        let mysql_history = manager.get_history(Some(DatabaseType::MySQL), Some(10)).await?;
        assert_eq!(mysql_history.len(), 1);
        assert_eq!(mysql_history[0].database_type, DatabaseType::MySQL);

        let all_history = manager.get_history(None, Some(10)).await?;
        assert_eq!(all_history.len(), 2);

        Ok(())
    }
}