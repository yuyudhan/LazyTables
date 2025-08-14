// FilePath: src/database/mod.rs

// Database adapter modules will be added here
pub mod connection;
pub mod postgres;

pub use connection::{
    Connection, ConnectionConfig, ConnectionStatus, ConnectionStorage, DatabaseType, SslMode,
};

/// Represents detailed metadata about a database table
#[derive(Debug, Clone)]
pub struct TableMetadata {
    pub table_name: String,
    pub row_count: usize,
    pub column_count: usize,
    pub total_size: String,
    pub table_size: String,
    pub indexes_size: String,
    pub primary_keys: Vec<String>,
    pub foreign_keys: Vec<String>,
    pub indexes: Vec<String>,
    pub comment: Option<String>,
}
