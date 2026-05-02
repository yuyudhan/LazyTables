// FilePath: src/database/connection.rs

// Re-exports core database types from lazytables-core and defines the Connection
// trait, plus extension traits for ConnectionConfig and ConnectionStorage that
// depend on crate-specific imports (PasswordManager, Config, async_fs, toml).

use crate::config::Config;
use crate::core::error::Result;
use crate::security::PasswordManager;

// Re-export all types from lazytables-core::database
pub use lazytables_core::database::*;

/// Extension trait for ConnectionConfig methods that depend on crate-specific imports
pub trait ConnectionConfigExt {
    /// Resolve the actual password for this connection
    fn resolve_password(&self, encryption_key: Option<&str>) -> Result<String>;

    /// Migrate plain text password to encrypted
    fn migrate_to_encrypted_password(
        &mut self,
        encryption_key: &str,
        hint: Option<String>,
    ) -> Result<()>;
}

impl ConnectionConfigExt for ConnectionConfig {
    /// Resolve the actual password for this connection
    /// Takes an optional encryption key for encrypted passwords
    fn resolve_password(&self, encryption_key: Option<&str>) -> Result<String> {
        // First check if we have a password source
        if let Some(ref source) = self.password_source {
            PasswordManager::resolve_password(source, encryption_key)
                .map_err(crate::core::error::LazyTablesError::PasswordError)
        } else if let Some(ref password) = self.password {
            // Fall back to legacy plain text password field
            Ok(password.clone())
        } else {
            Err(crate::core::error::LazyTablesError::PasswordError(
                "No password configured".to_string(),
            ))
        }
    }

    /// Migrate plain text password to encrypted
    fn migrate_to_encrypted_password(
        &mut self,
        encryption_key: &str,
        hint: Option<String>,
    ) -> Result<()> {
        if let Some(ref password) = self.password {
            let source = PasswordManager::migrate_to_encrypted(password, encryption_key, hint)
                .map_err(crate::core::error::LazyTablesError::PasswordError)?;
            self.set_password_source(source);
            Ok(())
        } else {
            Err(crate::core::error::LazyTablesError::PasswordError(
                "No plain text password to migrate".to_string(),
            ))
        }
    }
}

/// Extension trait for ConnectionStorage methods that depend on crate-specific imports
#[async_trait::async_trait]
pub trait ConnectionStorageExt {
    /// Load connections from storage asynchronously (non-blocking)
    async fn load() -> Result<ConnectionStorage>;

    /// Save connections to storage asynchronously (non-blocking)
    async fn save(&self) -> Result<()>;

    /// Add a new connection asynchronously
    async fn add_connection(&mut self, connection: ConnectionConfig) -> Result<()>;

    /// Remove a connection by ID asynchronously
    async fn remove_connection(&mut self, id: &str) -> Result<()>;

    /// Update a connection asynchronously
    async fn update_connection(&mut self, connection: ConnectionConfig) -> Result<()>;
}

#[async_trait::async_trait]
impl ConnectionStorageExt for ConnectionStorage {
    /// Load connections from storage asynchronously (non-blocking)
    async fn load() -> Result<ConnectionStorage> {
        let path = Config::connections_path();

        if path.exists() {
            let contents = crate::io::async_fs::read_to_string(&path).await?;
            let storage: ConnectionStorage =
                toml::from_str(&contents).map_err(crate::core::error::from_toml_de_error)?;
            Ok(storage)
        } else {
            Ok(ConnectionStorage::default())
        }
    }

    /// Save connections to storage asynchronously (non-blocking)
    async fn save(&self) -> Result<()> {
        let path = Config::connections_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            crate::io::async_fs::create_dir_all(parent).await?;
        }

        let contents =
            toml::to_string_pretty(self).map_err(crate::core::error::from_toml_ser_error)?;
        crate::io::async_fs::write(path, contents).await?;
        Ok(())
    }

    /// Add a new connection asynchronously
    async fn add_connection(&mut self, connection: ConnectionConfig) -> Result<()> {
        // Check for duplicate names
        if self.connections.iter().any(|c| c.name == connection.name) {
            return Err(crate::core::error::LazyTablesError::ConnectionExists(
                connection.name,
            ));
        }

        self.connections.push(connection);
        self.save().await
    }

    /// Remove a connection by ID asynchronously
    async fn remove_connection(&mut self, id: &str) -> Result<()> {
        self.connections.retain(|c| c.id != id);
        self.save().await
    }

    /// Update a connection asynchronously
    async fn update_connection(&mut self, connection: ConnectionConfig) -> Result<()> {
        if let Some(index) = self.connections.iter().position(|c| c.id == connection.id) {
            self.connections[index] = connection;
            self.save().await
        } else {
            Err(crate::core::error::LazyTablesError::ConnectionNotFound(
                connection.id,
            ))
        }
    }
}

/// Database connection trait
#[async_trait::async_trait]
pub trait Connection: Send + Sync {
    /// Connect to the database
    async fn connect(&mut self) -> Result<()>;

    /// Connect to the database with an encryption key for encrypted passwords
    async fn connect_with_key(&mut self, encryption_key: Option<&str>) -> Result<()>;

    /// Disconnect from the database
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Get connection config
    fn config(&self) -> &ConnectionConfig;

    // Query execution capabilities (AC1 requirement)
    /// Execute a raw SQL query and return columns and data
    async fn execute_raw_query(&self, query: &str) -> Result<(Vec<String>, Vec<Vec<String>>)>;

    // Metadata operations (AC1 & AC2 requirements)
    /// List all tables in the current database
    async fn list_tables(&self) -> Result<Vec<String>>;

    /// List all database objects (tables, views, etc.)
    async fn list_database_objects(&self) -> Result<crate::database::DatabaseObjectList>;

    /// List all schemas in the database (PostgreSQL) or databases (MySQL)
    /// Returns empty vector for databases that don't support schemas (SQLite)
    async fn list_schemas(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    /// Get detailed metadata for a specific table
    async fn get_table_metadata(&self, table_name: &str) -> Result<crate::database::TableMetadata>;

    /// Get column information for a table
    async fn get_table_columns(
        &self,
        table_name: &str,
    ) -> Result<Vec<crate::database::TableColumn>>;

    /// Get table data with pagination
    async fn get_table_data(
        &self,
        table_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Vec<String>>>;

    // Database-specific capabilities (AC1 & AC2 requirement)
    /// Get database-specific capabilities and features
    async fn get_database_capabilities(&self) -> Result<DatabaseCapabilities>;

    /// Test connection health with database-specific checks
    async fn health_check(&self) -> Result<HealthStatus>;

    /// Get database version and server information
    async fn get_server_info(&self) -> Result<ServerInfo>;

    // Connection pooling support (AC4 requirement)
    /// Get current pool statistics
    fn get_pool_status(&self) -> Option<PoolStatus>;

    /// Get maximum connections supported by this adapter
    fn max_connections(&self) -> u32;

    /// Get current active connections count
    fn active_connections(&self) -> u32;

    // Database-specific error handling (AC5 requirement)
    /// Convert database-specific error to user-friendly message with recovery suggestions
    fn format_error(&self, error: &str) -> FormattedError;

    /// Get database-specific keywords for syntax highlighting
    fn get_keywords(&self) -> Vec<String>;

    /// Get database-specific functions for autocomplete
    fn get_functions(&self) -> Vec<String>;
}
