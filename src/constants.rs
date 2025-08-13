// FilePath: src/constants.rs

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "LazyTables";

/// Full version string
pub fn version_string() -> String {
    format!("{APP_NAME} v{VERSION}")
}
