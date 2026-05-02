// FilePath: crates/lazytables-core/src/lib.rs

// Core types and error definitions for LazyTables.
// This crate contains all shared data structures that are used across
// the workspace crates, enabling clean dependency boundaries.

pub mod constants;
pub mod database;
pub mod error;
pub mod metadata;
pub mod objects;
pub mod security;
pub mod table_viewer;
pub mod toast;
pub mod ui;
pub mod view;

// Re-export commonly used types at the crate root
pub use database::{
    ConnectionConfig, ConnectionStatus, ConnectionStorage, DatabaseCapabilities, DatabaseType,
    FormattedError, HealthStatus, PoolStatus, ServerInfo, SslMode,
};
pub use error::{ConnectionError, ConnectionErrorType, Error, LazyTablesError, Result};
pub use metadata::{
    ColumnDefinition, ColumnSummary, ConstraintInfo, DataType, DatabaseSpecificMetadata,
    ForeignKeyInfo, IndexInfo, TableColumn, TableMetadata,
};
pub use objects::{DatabaseObject, DatabaseObjectList, DatabaseObjectType};
pub use security::{EncryptedPassword, PasswordSource};
pub use table_viewer::{
    CellUpdate, ColumnInfo, DeleteConfirmation, SetNullConfirmation, TableTab, TableViewMode,
    TableViewerState,
};
pub use toast::{Toast, ToastManager, ToastType};
pub use ui::{
    ConfirmationAction, ConfirmationModal, FocusedPane, HelpMode, HelpPaneFocus,
    SelectableTableItem,
};
pub use view::{AppView, ConnectionFormMode, OverlayView, TextInputMode};
