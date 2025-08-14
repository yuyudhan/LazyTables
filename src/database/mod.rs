// FilePath: src/database/mod.rs

// Database adapter modules will be added here
pub mod connection;
pub mod postgres;

pub use connection::{
    Connection, ConnectionConfig, ConnectionStatus, ConnectionStorage, DatabaseType, SslMode,
};
