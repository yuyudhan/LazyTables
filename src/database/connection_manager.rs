// FilePath: src/database/connection_manager.rs

use crate::core::error::{LazyTablesError, Result};
use crate::database::{connection::Connection, ConnectionConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Connection manager that maintains persistent database connections
/// to prevent the connection churning issue where connections are
/// constantly created and destroyed for each operation
/// Trait for database connections stored in the connection manager
/// This trait needs to be object-safe, so all methods use &self and no generics
#[async_trait::async_trait]
pub trait ManagedConnection: Send + Sync + std::fmt::Debug {
    async fn execute_raw_query(&self, query: &str) -> Result<(Vec<String>, Vec<Vec<String>>)>;
    async fn get_table_data(
        &self,
        table_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Vec<String>>>;
    async fn get_table_columns(
        &self,
        table_name: &str,
    ) -> Result<Vec<crate::database::TableColumn>>;
    async fn get_table_metadata(&self, table_name: &str) -> Result<crate::database::TableMetadata>;
    async fn list_database_objects(&self) -> Result<crate::database::DatabaseObjectList>;
    fn is_connected(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct ConnectionManager {
    /// Active connections keyed by connection ID
    connections: Arc<Mutex<HashMap<String, Arc<Mutex<Box<dyn ManagedConnection>>>>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Establish a persistent connection to a database
    /// This replaces the problematic pattern of creating/destroying connections per operation
    pub async fn connect(&self, config: &ConnectionConfig) -> Result<()> {
        let mut connections = self.connections.lock().await;

        // Check if we already have an active connection
        if let Some(existing_conn) = connections.get(&config.id) {
            let conn = existing_conn.lock().await;
            if conn.is_connected() {
                return Ok(()); // Already connected
            }
            // Remove stale connection
            drop(conn);
            connections.remove(&config.id);
        }

        // Create new connection based on database type
        let connection: Box<dyn ManagedConnection> = match config.database_type {
            crate::database::DatabaseType::PostgreSQL => {
                let mut pg_conn =
                    crate::database::postgres::PostgresConnection::new(config.clone());
                // Establish the connection
                Connection::connect(&mut pg_conn).await?;
                Box::new(pg_conn)
            }
            _ => {
                return Err(LazyTablesError::Connection(format!(
                    "Database type {} not supported yet",
                    config.database_type.display_name()
                )))
            }
        };

        // Store the connected instance
        tracing::error!("Storing connection with ID: '{}'", config.id);
        connections.insert(config.id.clone(), Arc::new(Mutex::new(connection)));
        tracing::error!(
            "Connection manager now has {} connections",
            connections.len()
        );

        Ok(())
    }

    /// Get a reference to an active connection
    pub async fn get_connection(
        &self,
        connection_id: &str,
    ) -> Result<Arc<Mutex<Box<dyn ManagedConnection>>>> {
        let connections = self.connections.lock().await;

        // Error-level logging to help diagnose connection issues (visible in production)
        tracing::error!("Looking for connection ID: '{}'", connection_id);
        tracing::error!(
            "Available connections: {:?}",
            connections.keys().collect::<Vec<_>>()
        );

        connections.get(connection_id).cloned().ok_or_else(|| {
            LazyTablesError::Connection(format!(
                "Connection not found or not active. Requested: '{}', Available: {:?}",
                connection_id,
                connections.keys().collect::<Vec<_>>()
            ))
        })
    }

    /// Disconnect from a specific database
    pub async fn disconnect(&self, connection_id: &str) -> Result<()> {
        let mut connections = self.connections.lock().await;

        if let Some(_connection_ref) = connections.remove(connection_id) {
            // The connection will be dropped automatically when removed from the map
            // Individual connection cleanup happens in the Drop trait
        }

        Ok(())
    }

    /// Disconnect from all databases
    pub async fn disconnect_all(&self) -> Result<()> {
        let mut connections = self.connections.lock().await;

        // Simply clear all connections - they will be automatically dropped
        connections.clear();

        Ok(())
    }

    /// Check if a connection is active and healthy
    pub async fn is_connected(&self, connection_id: &str) -> bool {
        let connections = self.connections.lock().await;

        if let Some(connection_ref) = connections.get(connection_id) {
            let connection = connection_ref.lock().await;
            connection.is_connected()
        } else {
            false
        }
    }

    /// Execute a raw SQL query using the persistent connection
    pub async fn execute_raw_query(
        &self,
        connection_id: &str,
        query: &str,
    ) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        let connection_ref = self.get_connection(connection_id).await?;
        let connection = connection_ref.lock().await;
        connection.execute_raw_query(query).await
    }

    /// Get table data using the persistent connection
    pub async fn get_table_data(
        &self,
        connection_id: &str,
        table_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Vec<String>>> {
        let connection_ref = self.get_connection(connection_id).await?;
        let connection = connection_ref.lock().await;
        connection.get_table_data(table_name, limit, offset).await
    }

    /// Get table columns using the persistent connection
    pub async fn get_table_columns(
        &self,
        connection_id: &str,
        table_name: &str,
    ) -> Result<Vec<crate::database::TableColumn>> {
        let connection_ref = self.get_connection(connection_id).await?;
        let connection = connection_ref.lock().await;
        connection.get_table_columns(table_name).await
    }

    /// Get table metadata using the persistent connection
    pub async fn get_table_metadata(
        &self,
        connection_id: &str,
        table_name: &str,
    ) -> Result<crate::database::TableMetadata> {
        let connection_ref = self.get_connection(connection_id).await?;
        let connection = connection_ref.lock().await;
        connection.get_table_metadata(table_name).await
    }

    /// List database objects using the persistent connection
    pub async fn list_database_objects(
        &self,
        connection_id: &str,
    ) -> Result<crate::database::DatabaseObjectList> {
        let connection_ref = self.get_connection(connection_id).await?;
        let connection = connection_ref.lock().await;
        connection.list_database_objects().await
    }

    /// Check if a connection is healthy by trying to execute a simple query
    pub async fn health_check(&self, connection_id: &str) -> Result<bool> {
        match self.execute_raw_query(connection_id, "SELECT 1").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

// Ensure ConnectionManager is thread-safe
unsafe impl Send for ConnectionManager {}
unsafe impl Sync for ConnectionManager {}
