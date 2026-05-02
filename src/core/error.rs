// FilePath: src/core/error.rs

// Re-exports core error types from lazytables-core and provides conversion
// helpers for crate-specific dependencies (sqlx, toml).
// Orphan rules prevent direct From impls since both types are from external crates.

pub use lazytables_core::error::*;

/// Convert sqlx::Error to LazyTablesError
pub fn from_sqlx_error(err: sqlx::Error) -> LazyTablesError {
    LazyTablesError::Database(err.to_string())
}

/// Convert toml deserialization error to LazyTablesError
pub fn from_toml_de_error(err: toml::de::Error) -> LazyTablesError {
    LazyTablesError::Serialization(err.to_string())
}

/// Convert toml serialization error to LazyTablesError
pub fn from_toml_ser_error(err: toml::ser::Error) -> LazyTablesError {
    LazyTablesError::Config(err.to_string())
}

/// Extension trait to convert Result<T, sqlx::Error> to Result<T, LazyTablesError>
/// This replaces the From<sqlx::Error> impl that we can no longer have due to orphan rules.
pub trait SqlxResultExt<T> {
    /// Convert sqlx error result to LazyTablesError result
    fn map_sqlx_err(self) -> Result<T>;
}

impl<T> SqlxResultExt<T> for std::result::Result<T, sqlx::Error> {
    fn map_sqlx_err(self) -> Result<T> {
        self.map_err(from_sqlx_error)
    }
}

/// Extension trait to convert Result<T, toml::de::Error> to Result<T, LazyTablesError>
pub trait TomlDeResultExt<T> {
    /// Convert toml deserialization error result to LazyTablesError result
    fn map_toml_de_err(self) -> Result<T>;
}

impl<T> TomlDeResultExt<T> for std::result::Result<T, toml::de::Error> {
    fn map_toml_de_err(self) -> Result<T> {
        self.map_err(from_toml_de_error)
    }
}

/// Extension trait to convert Result<T, toml::ser::Error> to Result<T, LazyTablesError>
pub trait TomlSerResultExt<T> {
    /// Convert toml serialization error result to LazyTablesError result
    fn map_toml_ser_err(self) -> Result<T>;
}

impl<T> TomlSerResultExt<T> for std::result::Result<T, toml::ser::Error> {
    fn map_toml_ser_err(self) -> Result<T> {
        self.map_err(from_toml_ser_error)
    }
}
