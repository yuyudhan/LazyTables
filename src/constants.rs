// FilePath: src/constants.rs

// Application-wide constants such as name and version.
// Version is pulled from Cargo.toml at compile time.

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "LazyTables";

/// Full version string
pub fn version_string() -> String {
    format!("{APP_NAME} v{VERSION}")
}
