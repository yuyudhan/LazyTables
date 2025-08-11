// FilePath: src/core/error.rs

use thiserror::Error;

/// Result type alias for LazyTables
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for LazyTables
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::de::Error),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Event handling error: {0}")]
    Event(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not connected to database")]
    NotConnected,

    #[error("Operation not supported: {0}")]
    NotSupported(String),

    #[error("{0}")]
    Other(String),
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::Config(err.to_string())
    }
}

