// FilePath: src/database/connection.rs

use crate::core::error::Result;
use crate::config::Config;
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
    /// Password (stored encrypted, not serialized in plain text)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// SSL/TLS configuration
    pub ssl_mode: SslMode,
    /// Connection timeout in seconds
    pub timeout: Option<u64>,
    /// Whether this connection is currently active
    #[serde(default)]
    pub is_connected: bool,
}

impl ConnectionConfig {
    /// Create a new connection configuration
    pub fn new(name: String, database_type: DatabaseType, host: String, port: u16, username: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            database_type,
            host,
            port,
            database: None,
            username,
            password: None,
            ssl_mode: SslMode::default(),
            timeout: Some(30),
            is_connected: false,
        }
    }

    /// Get connection display string (e.g., "jatayu (postgres)")
    pub fn display_string(&self) -> String {
        format!("{} ({})", self.name, self.database_type.display_name())
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
            return Err(crate::core::error::LazyTablesError::ConnectionExists(connection.name));
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
            Err(crate::core::error::LazyTablesError::ConnectionNotFound(connection.id))
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

    /// Disconnect from the database
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Get connection config
    fn config(&self) -> &ConnectionConfig;
}

