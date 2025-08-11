// FilePath: src/logging.rs

use crate::{cli::LogLevel, core::error::Result};
use directories::ProjectDirs;
use std::{fs, path::PathBuf};
use tracing_subscriber::{prelude::*, EnvFilter};

/// Initialize the logging system
pub fn init(level: LogLevel) -> Result<()> {
    let log_dir = get_log_dir()?;
    fs::create_dir_all(&log_dir)?;

    let log_file = log_dir.join("lazytables.log");
    let file_appender = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(std::fs::File::create(log_file)?);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "lazytables={},sqlx={}",
            tracing::Level::from(level),
            tracing::Level::WARN
        ))
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(file_appender)
        .init();

    Ok(())
}

/// Get the log directory path
fn get_log_dir() -> Result<PathBuf> {
    Ok(ProjectDirs::from("com", "lazytables", "LazyTables")
        .map(|dirs| dirs.cache_dir().join("logs"))
        .unwrap_or_else(|| PathBuf::from(".cache/lazytables/logs")))
}

