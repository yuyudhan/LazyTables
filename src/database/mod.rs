// FilePath: src/database/mod.rs

// Root module for the database layer. Re-exports shared types from lazytables-core
// and submodule implementations (adapters, connection manager, factory).

pub mod app_state;
pub mod connection;
pub mod connection_manager;
pub mod factory;
pub mod mysql;
pub mod objects;
pub mod postgres;
pub mod query_history;
pub mod sqlite;

pub use connection::{
    Connection, ConnectionConfig, ConnectionConfigExt, ConnectionStatus, ConnectionStorage,
    ConnectionStorageExt, DatabaseCapabilities, DatabaseType, FormattedError, HealthStatus,
    PoolStatus, ServerInfo, SslMode,
};

// Re-export adapter factory for AC3 compliance
pub use factory::AdapterFactory;

// Re-export connection manager
pub use connection_manager::ConnectionManager;

// Re-export database object types
pub use objects::{DatabaseObject, DatabaseObjectList, DatabaseObjectType};

// Re-export query history types
pub use query_history::{QueryHistoryEntry, QueryHistoryManager};

// Re-export app state types
pub use app_state::{ActiveConnectionState, AppStateDb, ConnectionSession, SqlFileActivity};

// Re-export metadata types from lazytables-core
pub use lazytables_core::metadata::{
    ColumnDefinition, ColumnSummary, ConstraintInfo, DataType, DatabaseSpecificMetadata,
    ForeignKeyInfo, IndexInfo, TableColumn, TableMetadata,
};
