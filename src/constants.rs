// FilePath: src/constants.rs

#![forbid(unsafe_code)]

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "LazyTables";

/// Full version string
pub fn version_string() -> String {
    format!("{APP_NAME} v{VERSION}")
}
