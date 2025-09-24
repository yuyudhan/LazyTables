// FilePath: src/database/factory.rs

use crate::core::error::{LazyTablesError, Result};
use crate::database::{
    mysql::MySqlConnection, postgres::PostgresConnection, sqlite::SqliteConnection, Connection,
    ConnectionConfig, DatabaseType,
};

/// Factory for creating database adapter connections (AC3 requirement)
pub struct AdapterFactory;

impl AdapterFactory {
    /// Create a database connection based on the configuration
    /// Implements automatic adapter selection per AC3: "Database type detection and adapter selection works automatically"
    pub fn create_connection(config: ConnectionConfig) -> Result<Box<dyn Connection>> {
        match config.database_type {
            DatabaseType::PostgreSQL => Ok(Box::new(PostgresConnection::new(config))),
            DatabaseType::MySQL | DatabaseType::MariaDB => {
                Ok(Box::new(MySqlConnection::new(config)))
            }
            DatabaseType::SQLite => Ok(Box::new(SqliteConnection::new(config))),
            DatabaseType::Oracle => Err(LazyTablesError::Unsupported(
                "Oracle support not yet implemented".to_string(),
            )),
            DatabaseType::Redis => Err(LazyTablesError::Unsupported(
                "Redis support not yet implemented".to_string(),
            )),
            DatabaseType::MongoDB => Err(LazyTablesError::Unsupported(
                "MongoDB support not yet implemented".to_string(),
            )),
        }
    }

    /// Detect database type from connection string
    /// Implements automatic database type detection per AC3
    pub fn detect_database_type(connection_string: &str) -> Result<DatabaseType> {
        let lower = connection_string.to_lowercase();

        if lower.starts_with("postgresql://") || lower.starts_with("postgres://") {
            Ok(DatabaseType::PostgreSQL)
        } else if lower.starts_with("mysql://") {
            Ok(DatabaseType::MySQL)
        } else if lower.starts_with("mariadb://") {
            Ok(DatabaseType::MariaDB)
        } else if lower.starts_with("sqlite://")
            || lower.ends_with(".db")
            || lower.ends_with(".sqlite")
        {
            Ok(DatabaseType::SQLite)
        } else if lower.starts_with("oracle://") {
            Ok(DatabaseType::Oracle)
        } else if lower.starts_with("redis://") {
            Ok(DatabaseType::Redis)
        } else if lower.starts_with("mongodb://") {
            Ok(DatabaseType::MongoDB)
        } else {
            Err(LazyTablesError::InvalidConnectionString(
                "Cannot detect database type from connection string".to_string(),
            ))
        }
    }

    /// Create connection with automatic type detection from connection string
    /// Combines type detection with adapter selection for fully automatic workflow
    pub fn create_connection_from_string(
        connection_string: &str,
        name: String,
    ) -> Result<(DatabaseType, Box<dyn Connection>)> {
        let db_type = Self::detect_database_type(connection_string)?;

        // Parse connection string into config
        let config = Self::parse_connection_string(connection_string, name, db_type.clone())?;

        let connection = Self::create_connection(config)?;

        Ok((db_type, connection))
    }

    /// Parse connection string into ConnectionConfig
    /// Helper method for automatic configuration from connection strings
    fn parse_connection_string(
        connection_string: &str,
        name: String,
        db_type: DatabaseType,
    ) -> Result<ConnectionConfig> {
        // Basic URL parsing - could be enhanced with proper URL parsing library
        let url = connection_string.trim();

        // Extract components based on standard URL format
        // Format: protocol://[username[:password]@]host[:port][/database]

        if url.contains("://") {
            let parts: Vec<&str> = url.splitn(2, "://").collect();
            if parts.len() != 2 {
                return Err(LazyTablesError::InvalidConnectionString(
                    "Invalid connection string format".to_string(),
                ));
            }

            let authority_and_path = parts[1];

            // Split authority and path
            let (authority, database) = if authority_and_path.contains('/') {
                let parts: Vec<&str> = authority_and_path.splitn(2, '/').collect();
                (parts[0], Some(parts[1].to_string()))
            } else {
                (authority_and_path, None)
            };

            // Parse authority: [username[:password]@]host[:port]
            let (credentials, host_port) = if authority.contains('@') {
                let parts: Vec<&str> = authority.rsplitn(2, '@').collect();
                (Some(parts[1]), parts[0])
            } else {
                (None, authority)
            };

            // Parse host and port
            let (host, port) = if host_port.contains(':') {
                let parts: Vec<&str> = host_port.rsplitn(2, ':').collect();
                let port_str = parts[0];
                let host_str = parts[1];

                let port_num = port_str.parse::<u16>().map_err(|_| {
                    LazyTablesError::InvalidConnectionString("Invalid port number".to_string())
                })?;

                (host_str.to_string(), port_num)
            } else {
                // Use default ports
                let default_port = match db_type {
                    DatabaseType::PostgreSQL => 5432,
                    DatabaseType::MySQL | DatabaseType::MariaDB => 3306,
                    DatabaseType::Oracle => 1521,
                    DatabaseType::Redis => 6379,
                    DatabaseType::MongoDB => 27017,
                    DatabaseType::SQLite => 0, // SQLite doesn't use ports
                };
                (host_port.to_string(), default_port)
            };

            // Parse credentials
            let (username, password) = if let Some(creds) = credentials {
                if creds.contains(':') {
                    let parts: Vec<&str> = creds.splitn(2, ':').collect();
                    (parts[0].to_string(), Some(parts[1].to_string()))
                } else {
                    (creds.to_string(), None)
                }
            } else {
                ("".to_string(), None)
            };

            let mut config = ConnectionConfig::new(name, db_type, host, port, username);
            config.database = database;

            if let Some(pwd) = password {
                config.set_plain_password(pwd);
            }

            Ok(config)
        } else {
            Err(LazyTablesError::InvalidConnectionString(
                "Connection string must include protocol (e.g., postgresql://, mysql://)"
                    .to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_database_type() {
        assert_eq!(
            AdapterFactory::detect_database_type("postgresql://localhost/test").unwrap(),
            DatabaseType::PostgreSQL
        );

        assert_eq!(
            AdapterFactory::detect_database_type("mysql://localhost/test").unwrap(),
            DatabaseType::MySQL
        );

        assert_eq!(
            AdapterFactory::detect_database_type("sqlite:///path/to/db.sqlite").unwrap(),
            DatabaseType::SQLite
        );

        assert!(AdapterFactory::detect_database_type("invalid://localhost").is_err());
    }

    #[test]
    fn test_parse_connection_string() {
        let config = AdapterFactory::parse_connection_string(
            "postgresql://user:pass@localhost:5432/testdb",
            "test".to_string(),
            DatabaseType::PostgreSQL,
        )
        .unwrap();

        assert_eq!(config.database_type, DatabaseType::PostgreSQL);
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.username, "user");
        assert_eq!(config.database, Some("testdb".to_string()));
    }

    #[test]
    fn test_create_connection_mysql() {
        let config = ConnectionConfig::new(
            "test".to_string(),
            DatabaseType::MySQL,
            "localhost".to_string(),
            3306,
            "root".to_string(),
        );

        let connection = AdapterFactory::create_connection(config);
        assert!(connection.is_ok());
    }

    #[test]
    fn test_create_connection_postgresql() {
        let config = ConnectionConfig::new(
            "test".to_string(),
            DatabaseType::PostgreSQL,
            "localhost".to_string(),
            5432,
            "postgres".to_string(),
        );

        let connection = AdapterFactory::create_connection(config);
        assert!(connection.is_ok());
    }

    #[test]
    fn test_create_connection_sqlite() {
        let config = ConnectionConfig::new(
            "test".to_string(),
            DatabaseType::SQLite,
            "localhost".to_string(),
            0,
            "".to_string(),
        );

        let connection = AdapterFactory::create_connection(config);
        assert!(connection.is_ok());
    }

    #[test]
    fn test_create_connection_unsupported() {
        let config = ConnectionConfig::new(
            "test".to_string(),
            DatabaseType::Oracle,
            "localhost".to_string(),
            1521,
            "oracle".to_string(),
        );

        let connection = AdapterFactory::create_connection(config);
        assert!(connection.is_err());
    }

    #[test]
    fn test_create_connection_from_string() {
        let result = AdapterFactory::create_connection_from_string(
            "mysql://user:pass@localhost:3306/testdb",
            "test".to_string(),
        );

        assert!(result.is_ok());
        let (db_type, _connection) = result.unwrap();
        assert_eq!(db_type, DatabaseType::MySQL);
    }

    #[test]
    fn test_parse_mysql_connection_string() {
        let config = AdapterFactory::parse_connection_string(
            "mysql://user:pass@localhost:3306/testdb",
            "test".to_string(),
            DatabaseType::MySQL,
        )
        .unwrap();

        assert_eq!(config.database_type, DatabaseType::MySQL);
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 3306);
        assert_eq!(config.username, "user");
        assert_eq!(config.database, Some("testdb".to_string()));
    }

    #[test]
    fn test_parse_connection_string_with_default_port() {
        let config = AdapterFactory::parse_connection_string(
            "postgresql://user@localhost/testdb",
            "test".to_string(),
            DatabaseType::PostgreSQL,
        )
        .unwrap();

        assert_eq!(config.port, 5432); // Default PostgreSQL port
    }

    #[test]
    fn test_detect_mariadb_type() {
        assert_eq!(
            AdapterFactory::detect_database_type("mariadb://localhost/test").unwrap(),
            DatabaseType::MariaDB
        );
    }
}

