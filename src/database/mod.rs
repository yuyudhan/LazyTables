// FilePath: src/database/mod.rs

// Database adapter modules
pub mod connection;
pub mod postgres;
pub mod mysql;
pub mod sqlite;

pub use connection::{
    ConnectionConfig, ConnectionStatus, ConnectionStorage, DatabaseType, SslMode,
};

use async_trait::async_trait;
use crate::core::error::Result;

/// Database connection trait that all database implementations must implement
#[async_trait]
pub trait Connection: Send + Sync {
    /// Connect to the database
    async fn connect(&mut self) -> Result<()>;
    
    /// Disconnect from the database
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Check if connected
    async fn is_connected(&self) -> bool;
    
    /// Test the connection
    async fn test_connection(&self) -> Result<()>;
    
    /// List available databases
    async fn list_databases(&self) -> Result<Vec<String>>;
    
    /// List tables in the current database
    async fn list_tables(&self) -> Result<Vec<String>>;
    
    /// Get metadata for a specific table
    async fn get_table_metadata(&self, table_name: &str) -> Result<TableMetadata>;
    
    /// Get columns for a specific table
    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>>;
    
    /// Get row count for a table
    async fn get_table_row_count(&self, table_name: &str) -> Result<usize>;
    
    /// Get table data with pagination
    async fn get_table_data(
        &self,
        table_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Vec<String>>>;
}

/// Represents a table column
#[derive(Debug, Clone)]
pub struct TableColumn {
    pub name: String,
    pub data_type: DataType,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
}

/// Column definition for table creation
#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub default_value: Option<String>,
}

/// Supported data types
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Integer,
    BigInt,
    SmallInt,
    Decimal,
    Float,
    Double,
    Boolean,
    Text,
    Varchar(Option<usize>),
    Char(Option<usize>),
    Date,
    Time,
    Timestamp,
    Json,
    Uuid,
    Bytea,
    Array(Box<DataType>),
}

impl DataType {
    /// Convert to SQL type string for the given database
    pub fn to_sql(&self) -> String {
        match self {
            DataType::Integer => "INTEGER".to_string(),
            DataType::BigInt => "BIGINT".to_string(),
            DataType::SmallInt => "SMALLINT".to_string(),
            DataType::Decimal => "DECIMAL".to_string(),
            DataType::Float => "FLOAT".to_string(),
            DataType::Double => "DOUBLE".to_string(),
            DataType::Boolean => "BOOLEAN".to_string(),
            DataType::Text => "TEXT".to_string(),
            DataType::Varchar(len) => {
                if let Some(l) = len {
                    format!("VARCHAR({l})")
                } else {
                    "VARCHAR".to_string()
                }
            }
            DataType::Char(len) => {
                if let Some(l) = len {
                    format!("CHAR({l})")
                } else {
                    "CHAR".to_string()
                }
            }
            DataType::Date => "DATE".to_string(),
            DataType::Time => "TIME".to_string(),
            DataType::Timestamp => "TIMESTAMP".to_string(),
            DataType::Json => "JSON".to_string(),
            DataType::Uuid => "UUID".to_string(),
            DataType::Bytea => "BYTEA".to_string(),
            DataType::Array(inner) => format!("{}[]", inner.to_sql()),
        }
    }
}

/// Represents detailed metadata about a database table
#[derive(Debug, Clone)]
pub struct TableMetadata {
    pub table_name: String,
    pub row_count: usize,
    pub column_count: usize,
    pub total_size: i64,
    pub table_size: i64,
    pub indexes_size: i64,
    pub primary_keys: Vec<String>,
    pub foreign_keys: Vec<String>,
    pub indexes: Vec<String>,
    pub comment: Option<String>,
}
