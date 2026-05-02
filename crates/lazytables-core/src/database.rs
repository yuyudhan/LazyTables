// FilePath: crates/lazytables-core/src/database.rs

// Core database type definitions: DatabaseType, SslMode, ConnectionStatus,
// ConnectionConfig, and capability/status types shared across crates.

use crate::security::PasswordSource;
use serde::{Deserialize, Serialize};

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

impl Default for SslMode {
    fn default() -> Self {
        Self::Prefer
    }
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
    Failed(String),
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self::Disconnected
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
            ConnectionStatus::Disconnected => "\u{2014}",
            ConnectionStatus::Connecting => "\u{27f3}",
            ConnectionStatus::Connected => "\u{2713}",
            ConnectionStatus::Failed(_) => "\u{2717}",
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
            .map(PasswordSource::requires_encryption_key)
            .unwrap_or(false)
    }

    /// Get hint for encrypted password if available
    pub fn get_password_hint(&self) -> Option<String> {
        self.password_source
            .as_ref()
            .and_then(PasswordSource::get_hint)
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
    /// Get connection by ID
    pub fn get_connection(&self, id: &str) -> Option<&ConnectionConfig> {
        self.connections.iter().find(|c| c.id == id)
    }

    /// Get mutable connection by ID
    pub fn get_connection_mut(&mut self, id: &str) -> Option<&mut ConnectionConfig> {
        self.connections.iter_mut().find(|c| c.id == id)
    }
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
    pub supports_cte: bool,
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
