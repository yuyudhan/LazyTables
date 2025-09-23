// FilePath: src/database/connection.rs

use crate::config::Config;
use crate::core::error::Result;
use crate::security::{PasswordManager, PasswordSource};
use serde::{Deserialize, Serialize};
use std::fs;

/// Database type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    MariaDB,
    SQLite,
    Oracle,
    Redis,
    MongoDB,
}

impl DatabaseType {
    /// Get display name for the database type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::PostgreSQL => "postgres",
            Self::MySQL => "mysql",
            Self::MariaDB => "mariadb",
            Self::SQLite => "sqlite",
            Self::Oracle => "oracle",
            Self::Redis => "redis",
            Self::MongoDB => "mongodb",
        }
    }
}

/// SSL/TLS mode for database connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SslMode {
    Disable,
    Allow,
    Prefer,
    Require,
    VerifyCA,
    VerifyFull,
}

/// Connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    /// Not connected
    Disconnected,
    /// Currently connecting
    Connecting,
    /// Successfully connected
    Connected,
    /// Connection failed
    Failed(String), // Error message
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl Default for SslMode {
    fn default() -> Self {
        Self::Prefer
    }
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Unique identifier for this connection
    pub id: String,
    /// Display name for the connection
    pub name: String,
    /// Database type
    pub database_type: DatabaseType,
    /// Host address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Database name (optional for some database types)
    pub database: Option<String>,
    /// Username for authentication
    pub username: String,
    /// Password source (environment variable or encrypted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_source: Option<PasswordSource>,
    /// Legacy password field (for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// SSL/TLS configuration
    pub ssl_mode: SslMode,
    /// Connection timeout in seconds
    pub timeout: Option<u64>,
    /// Connection status (not persisted, always starts as Disconnected)
    #[serde(skip)]
    pub status: ConnectionStatus,
}

impl ConnectionConfig {
    /// Create a new connection configuration
    pub fn new(
        name: String,
        database_type: DatabaseType,
        host: String,
        port: u16,
        username: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            database_type,
            host,
            port,
            database: None,
            username,
            password: None,
            password_source: None,
            ssl_mode: SslMode::default(),
            timeout: Some(30),
            status: ConnectionStatus::default(),
        }
    }

    /// Get connection display string (e.g., "jatayu (postgres)")
    pub fn display_string(&self) -> String {
        format!("{} ({})", self.name, self.database_type.display_name())
    }

    /// Get status display text
    pub fn status_text(&self) -> &str {
        match &self.status {
            ConnectionStatus::Disconnected => "Disconnected",
            ConnectionStatus::Connecting => "Connecting...",
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Failed(_) => "Failed",
        }
    }

    /// Get status indicator symbol
    pub fn status_symbol(&self) -> &str {
        match &self.status {
            ConnectionStatus::Disconnected => "—", // Em dash for disconnected
            ConnectionStatus::Connecting => "⟳",   // Rotation symbol for connecting
            ConnectionStatus::Connected => "✓",    // Check mark for connected
            ConnectionStatus::Failed(_) => "✗",    // X mark for failed
        }
    }

    /// Check if connection is currently connected
    pub fn is_connected(&self) -> bool {
        matches!(self.status, ConnectionStatus::Connected)
    }

    /// Check if connection is currently connecting
    pub fn is_connecting(&self) -> bool {
        matches!(self.status, ConnectionStatus::Connecting)
    }

    /// Check if connection failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, ConnectionStatus::Failed(_))
    }

    /// Get error message if connection failed
    pub fn get_error(&self) -> Option<&String> {
        if let ConnectionStatus::Failed(ref error) = self.status {
            Some(error)
        } else {
            None
        }
    }

    /// Resolve the actual password for this connection
    /// Takes an optional encryption key for encrypted passwords
    pub fn resolve_password(&self, encryption_key: Option<&str>) -> Result<String> {
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

    /// Set password using a source (environment variable or encrypted)
    pub fn set_password_source(&mut self, source: PasswordSource) {
        self.password_source = Some(source);
        // Clear legacy password field when using new source
        self.password = None;
    }

    /// Set plain text password (deprecated, for backward compatibility)
    pub fn set_plain_password(&mut self, password: String) {
        self.password = Some(password);
        self.password_source = None;
    }

    /// Check if this connection requires an encryption key
    pub fn requires_encryption_key(&self) -> bool {
        self.password_source
            .as_ref()
            .map(PasswordManager::requires_encryption_key)
            .unwrap_or(false)
    }

    /// Get hint for encrypted password if available
    pub fn get_password_hint(&self) -> Option<String> {
        self.password_source
            .as_ref()
            .and_then(PasswordManager::get_hint)
    }

    /// Migrate plain text password to encrypted
    pub fn migrate_to_encrypted_password(
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

/// Container for all saved connections
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectionStorage {
    pub connections: Vec<ConnectionConfig>,
    /// Version for future migration compatibility
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

impl ConnectionStorage {
    /// Load connections from storage
    pub fn load() -> Result<Self> {
        let path = Config::connections_path();

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let storage: ConnectionStorage = toml::from_str(&contents)?;
            Ok(storage)
        } else {
            Ok(Self::default())
        }
    }

    /// Save connections to storage
    pub fn save(&self) -> Result<()> {
        let path = Config::connections_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    /// Add a new connection
    pub fn add_connection(&mut self, connection: ConnectionConfig) -> Result<()> {
        // Check for duplicate names
        if self.connections.iter().any(|c| c.name == connection.name) {
            return Err(crate::core::error::LazyTablesError::ConnectionExists(
                connection.name,
            ));
        }

        self.connections.push(connection);
        self.save()
    }

    /// Remove a connection by ID
    pub fn remove_connection(&mut self, id: &str) -> Result<()> {
        self.connections.retain(|c| c.id != id);
        self.save()
    }

    /// Update a connection
    pub fn update_connection(&mut self, connection: ConnectionConfig) -> Result<()> {
        if let Some(index) = self.connections.iter().position(|c| c.id == connection.id) {
            self.connections[index] = connection;
            self.save()
        } else {
            Err(crate::core::error::LazyTablesError::ConnectionNotFound(
                connection.id,
            ))
        }
    }

    /// Get connection by ID
    pub fn get_connection(&self, id: &str) -> Option<&ConnectionConfig> {
        self.connections.iter().find(|c| c.id == id)
    }

    /// Get mutable connection by ID
    pub fn get_connection_mut(&mut self, id: &str) -> Option<&mut ConnectionConfig> {
        self.connections.iter_mut().find(|c| c.id == id)
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

/// Database-specific capabilities and features
#[derive(Debug, Clone)]
pub struct DatabaseCapabilities {
    pub supports_schemas: bool,
    pub supports_transactions: bool,
    pub supports_foreign_keys: bool,
    pub supports_json: bool,
    pub supports_arrays: bool,
    pub supports_stored_procedures: bool,
    pub supports_triggers: bool,
    pub supports_views: bool,
    pub supports_materialized_views: bool,
    pub supports_window_functions: bool,
    pub supports_cte: bool, // Common Table Expressions
    pub max_identifier_length: usize,
    pub max_query_length: Option<usize>,
    pub supported_isolation_levels: Vec<String>,
}

/// Health status for connection monitoring
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub last_error: Option<String>,
    pub database_version: Option<String>,
    pub active_connections: u32,
    pub max_connections: u32,
    pub uptime_seconds: Option<u64>,
}

/// Server information for database instance
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub version: String,
    pub build_info: Option<String>,
    pub server_name: Option<String>,
    pub charset: Option<String>,
    pub timezone: Option<String>,
    pub uptime_seconds: Option<u64>,
    pub current_database: Option<String>,
    pub current_user: Option<String>,
}

/// Connection pool status
#[derive(Debug, Clone)]
pub struct PoolStatus {
    pub size: u32,
    pub active: u32,
    pub idle: u32,
    pub waiting: u32,
    pub max_size: u32,
    pub min_size: u32,
}

/// Formatted error with recovery suggestions
#[derive(Debug, Clone)]
pub struct FormattedError {
    pub original_error: String,
    pub user_message: String,
    pub error_code: Option<String>,
    pub recovery_suggestions: Vec<String>,
    pub is_connection_error: bool,
    pub is_syntax_error: bool,
    pub is_permission_error: bool,
}
