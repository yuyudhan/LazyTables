// FilePath: src/database/mod.rs

// Database adapter modules
pub mod connection;
pub mod factory;
pub mod mysql;
pub mod objects;
pub mod postgres;
pub mod query_history;
pub mod sqlite;

pub use connection::{
    ConnectionConfig, ConnectionStatus, ConnectionStorage, DatabaseCapabilities, DatabaseType,
    FormattedError, HealthStatus, PoolStatus, ServerInfo, SslMode,
};

// Re-export the Connection trait from connection module
pub use connection::Connection;

// Re-export adapter factory for AC3 compliance
pub use factory::AdapterFactory;

// Re-export database object types
pub use objects::{DatabaseObject, DatabaseObjectList, DatabaseObjectType};

// Re-export query history types
pub use query_history::{QueryHistoryEntry, QueryHistoryManager};

// Note: Table metadata types are defined below in this module

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
    // Basic information
    pub table_name: String,
    pub schema_name: Option<String>,
    pub table_type: String, // TABLE, VIEW, MATERIALIZED VIEW, etc.
    pub row_count: usize,
    pub column_count: usize,
    pub comment: Option<String>,

    // Storage information
    pub total_size: i64,
    pub table_size: i64,
    pub indexes_size: i64,
    pub toast_size: Option<i64>, // PostgreSQL TOAST storage

    // Schema relationships
    pub primary_keys: Vec<String>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
    pub indexes: Vec<IndexInfo>,
    pub constraints: Vec<ConstraintInfo>,

    // Performance & maintenance
    pub last_vacuum: Option<String>,
    pub last_analyze: Option<String>,
    pub auto_vacuum_enabled: Option<bool>,
    pub table_owner: Option<String>,

    // Database-specific information
    pub database_specific: DatabaseSpecificMetadata,

    // Timestamps
    pub created_at: Option<String>,
    pub modified_at: Option<String>,

    // Column summary for quick reference
    pub columns_summary: Vec<ColumnSummary>,
}

/// Foreign key relationship information
#[derive(Debug, Clone)]
pub struct ForeignKeyInfo {
    pub constraint_name: String,
    pub column_names: Vec<String>,
    pub referenced_table: String,
    pub referenced_columns: Vec<String>,
    pub on_delete: Option<String>,
    pub on_update: Option<String>,
}

/// Index information
#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
    pub index_type: Option<String>, // BTREE, HASH, GIN, etc.
    pub size: Option<i64>,
}

/// Constraint information
#[derive(Debug, Clone)]
pub struct ConstraintInfo {
    pub name: String,
    pub constraint_type: String, // CHECK, UNIQUE, NOT NULL, etc.
    pub definition: Option<String>,
    pub columns: Vec<String>,
}

/// Column summary for quick reference in details pane
#[derive(Debug, Clone)]
pub struct ColumnSummary {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub has_default: bool,
    pub max_length: Option<i32>,
}

/// Database-specific metadata
#[derive(Debug, Clone)]
pub enum DatabaseSpecificMetadata {
    PostgreSQL {
        oid: Option<u32>,
        tablespace: Option<String>,
        has_indexes: bool,
        has_rules: bool,
        has_triggers: bool,
        row_security: bool,
        inheritance_count: Option<i32>,
        partition_key: Option<String>,
        is_partitioned: bool,
    },
    MySQL {
        engine: Option<String>,
        charset: Option<String>,
        collation: Option<String>,
        auto_increment: Option<u64>,
        checksum: Option<u64>,
        create_options: Option<String>,
    },
    SQLite {
        root_page: Option<i32>,
        sql_definition: Option<String>,
        strict_mode: bool,
    },
    None,
}

impl TableMetadata {
    /// Create a basic TableMetadata with minimal information for backward compatibility
    pub fn basic(
        table_name: String,
        row_count: usize,
        column_count: usize,
        total_size: i64,
        table_size: i64,
        indexes_size: i64,
        primary_keys: Vec<String>,
        foreign_keys: Vec<String>,
        indexes: Vec<String>,
        comment: Option<String>,
    ) -> Self {
        Self {
            table_name,
            schema_name: None,
            table_type: "TABLE".to_string(),
            row_count,
            column_count,
            comment,
            total_size,
            table_size,
            indexes_size,
            toast_size: None,
            primary_keys,
            foreign_keys: foreign_keys
                .into_iter()
                .map(|fk| ForeignKeyInfo {
                    constraint_name: fk.clone(),
                    column_names: vec![],
                    referenced_table: "".to_string(),
                    referenced_columns: vec![],
                    on_delete: None,
                    on_update: None,
                })
                .collect(),
            indexes: indexes
                .into_iter()
                .map(|idx| IndexInfo {
                    name: idx,
                    columns: vec![],
                    is_unique: false,
                    is_primary: false,
                    index_type: None,
                    size: None,
                })
                .collect(),
            constraints: vec![],
            last_vacuum: None,
            last_analyze: None,
            auto_vacuum_enabled: None,
            table_owner: None,
            database_specific: DatabaseSpecificMetadata::None,
            created_at: None,
            modified_at: None,
            columns_summary: vec![],
        }
    }

    /// Get display name including schema if available
    pub fn display_name(&self) -> String {
        if let Some(ref schema) = self.schema_name {
            format!("{}.{}", schema, self.table_name)
        } else {
            self.table_name.clone()
        }
    }

    /// Format size for display
    pub fn format_size(bytes: i64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Get a summary of relationships for display
    pub fn relationships_summary(&self) -> String {
        let mut parts = vec![];

        if !self.primary_keys.is_empty() {
            parts.push(format!("PK: {}", self.primary_keys.join(", ")));
        }

        if !self.foreign_keys.is_empty() {
            parts.push(format!("{} FK(s)", self.foreign_keys.len()));
        }

        if !self.indexes.is_empty() {
            let unique_count = self.indexes.iter().filter(|idx| idx.is_unique).count();
            if unique_count > 0 {
                parts.push(format!("{} idx ({} unique)", self.indexes.len(), unique_count));
            } else {
                parts.push(format!("{} idx", self.indexes.len()));
            }
        }

        if parts.is_empty() {
            "No constraints".to_string()
        } else {
            parts.join(" • ")
        }
    }
}
