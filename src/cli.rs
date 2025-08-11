// FilePath: src/cli.rs

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// LazyTables - Terminal-based SQL database viewer and editor
#[derive(Parser, Debug)]
#[command(name = "lazytables")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Set logging level
    #[arg(short, long, value_enum, default_value = "info")]
    pub log_level: LogLevel,

    /// Connection string to connect immediately
    #[arg(long)]
    pub connection: Option<String>,

    /// Database to select on startup
    #[arg(short = 'd', long)]
    pub database: Option<String>,

    /// Table to view on startup
    #[arg(short = 't', long)]
    pub table: Option<String>,

    /// Start in read-only mode
    #[arg(short = 'r', long)]
    pub read_only: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

