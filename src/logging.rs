// FilePath: src/logging.rs

use crate::{cli::LogLevel, config::Config, core::error::Result};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing_subscriber::{prelude::*, EnvFilter, Layer};

/// Initialize the logging system based on mode and level
pub fn init(level: LogLevel) -> Result<()> {
    let log_dir = get_log_dir()?;
    fs::create_dir_all(&log_dir)?;

    let is_dev_mode = is_development_mode();

    if is_dev_mode {
        init_development_logging(&log_dir, level)?;
        tracing::info!("Development logging initialized with level: {:?}", level);
    } else {
        init_production_logging(&log_dir)?;
        tracing::info!("Production logging initialized (warn/error only)");
    }

    log_startup_info(is_dev_mode);

    Ok(())
}

/// Check if we're running in development mode
fn is_development_mode() -> bool {
    // Check for debug build (most reliable indicator)
    if cfg!(debug_assertions) {
        return true;
    }

    // Check environment variables for explicit dev mode
    if std::env::var("LAZYTABLES_DEV").is_ok() {
        return true;
    }

    // For production builds, default to production mode
    false
}

/// Initialize logging for development mode
fn init_development_logging(log_dir: &Path, level: LogLevel) -> Result<()> {
    let debug_log_file = log_dir.join("debug.log");

    // Rotate log file if it gets too large (>10MB)
    rotate_log_file(&debug_log_file, 10 * 1024 * 1024)?;

    let debug_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&debug_log_file)?;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "lazytables={},sqlx=warn",
            tracing::Level::from(level)
        ))
    });

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(debug_file)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_level(true)
                .with_file(true)
                .with_line_number(true)
                .with_filter(filter),
        )
        .init();

    Ok(())
}

/// Initialize logging for production mode
fn init_production_logging(log_dir: &Path) -> Result<()> {
    let error_log_file = log_dir.join("error.log");

    // Rotate log file if it gets too large (>5MB)
    rotate_log_file(&error_log_file, 5 * 1024 * 1024)?;

    let error_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&error_log_file)?;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("lazytables=warn,sqlx=error"));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(error_file)
                .with_ansi(false)
                .with_target(false)
                .with_thread_ids(false)
                .with_level(true)
                .with_file(false)
                .with_line_number(false)
                .with_filter(filter),
        )
        .init();

    Ok(())
}

/// Rotate log file if it exceeds size limit
fn rotate_log_file(log_file: &PathBuf, max_size: u64) -> Result<()> {
    if let Ok(metadata) = std::fs::metadata(log_file) {
        if metadata.len() > max_size {
            let backup_file = log_file.with_extension("log.old");
            let _ = std::fs::rename(log_file, backup_file);
        }
    }
    Ok(())
}

/// Get the log directory path
fn get_log_dir() -> Result<PathBuf> {
    Ok(Config::data_dir().join("logs"))
}

/// Log startup information
fn log_startup_info(is_dev_mode: bool) {
    tracing::info!("LazyTables v{} starting up", env!("CARGO_PKG_VERSION"));
    tracing::info!(
        "Build mode: {}",
        if is_dev_mode {
            "development"
        } else {
            "production"
        }
    );
    if let Ok(log_dir) = get_log_dir() {
        tracing::info!("Log directory: {:?}", log_dir);
    }
}

/// Log shutdown information
pub fn log_shutdown() {
    tracing::info!("LazyTables shutting down gracefully");
}

/// Convenience macros for logging throughout the application
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*);
    };
}

/// Macro for creating spans for better tracing in dev mode
#[macro_export]
macro_rules! log_span {
    ($level:expr, $name:expr) => {
        tracing::span!($level, $name)
    };
    ($level:expr, $name:expr, $($fields:tt)*) => {
        tracing::span!($level, $name, $($fields)*)
    };
}
