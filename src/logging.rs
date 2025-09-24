// FilePath: src/logging.rs

use crate::{cli::LogLevel, config::Config, core::error::Result};
use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tracing_subscriber::{prelude::*, EnvFilter, Layer};

/// Debug message entry for the debug view
#[derive(Debug, Clone)]
pub struct DebugMessage {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub target: String,
    pub message: String,
    pub location: Option<String>,
}

/// In-memory log storage for debug view
#[derive(Debug)]
pub struct DebugLogStorage {
    messages: Arc<Mutex<VecDeque<DebugMessage>>>,
    max_messages: usize,
}

impl DebugLogStorage {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::new())),
            max_messages,
        }
    }

    pub fn add_message(&self, message: DebugMessage) {
        if let Ok(mut messages) = self.messages.lock() {
            messages.push_back(message);
            // Keep only the last max_messages
            while messages.len() > self.max_messages {
                messages.pop_front();
            }
        }
    }

    pub fn get_messages(&self) -> Vec<DebugMessage> {
        if let Ok(messages) = self.messages.lock() {
            messages.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn clear(&self) {
        if let Ok(mut messages) = self.messages.lock() {
            messages.clear();
        }
    }
}

lazy_static::lazy_static! {
    static ref DEBUG_LOG_STORAGE: DebugLogStorage = DebugLogStorage::new(1000);
}

/// Custom tracing layer to capture logs in memory
#[derive(Debug)]
struct MemoryLogLayer;

impl<S> Layer<S> for MemoryLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let mut visitor = LogVisitor::default();
        event.record(&mut visitor);

        let message = DebugMessage {
            timestamp: chrono::Utc::now(),
            level: metadata.level().to_string(),
            target: metadata.target().to_string(),
            message: visitor.message,
            location: metadata.file().map(|file| {
                if let Some(line) = metadata.line() {
                    format!("{}:{}", file, line)
                } else {
                    file.to_string()
                }
            }),
        };

        DEBUG_LOG_STORAGE.add_message(message);
    }
}

/// Visitor to extract the log message from the event
#[derive(Default)]
struct LogVisitor {
    message: String,
}

impl tracing::field::Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
            // Remove the quotes from the debug output
            if self.message.starts_with('"') && self.message.ends_with('"') {
                self.message = self.message[1..self.message.len()-1].to_string();
            }
        }
    }
}

/// Get debug messages for the debug view
pub fn get_debug_messages() -> Vec<DebugMessage> {
    DEBUG_LOG_STORAGE.get_messages()
}

/// Clear debug messages
pub fn clear_debug_messages() {
    DEBUG_LOG_STORAGE.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_storage_basic() {
        // Test that we can create and clear the storage
        clear_debug_messages();
        let messages = get_debug_messages();
        println!("After clear: {} messages", messages.len());

        // Add a test message directly
        let test_message = DebugMessage {
            timestamp: chrono::Utc::now(),
            level: "TEST".to_string(),
            target: "test_target".to_string(),
            message: "Test message".to_string(),
            location: Some("test.rs:123".to_string()),
        };

        DEBUG_LOG_STORAGE.add_message(test_message);
        let messages = get_debug_messages();
        assert!(messages.len() > 0, "Should have at least one message");

        // Find our test message
        let test_message = messages.iter().find(|m| m.target == "test_target");
        assert!(test_message.is_some(), "Should find our test message");

        let test_message = test_message.unwrap();
        assert_eq!(test_message.level, "TEST");
        assert_eq!(test_message.target, "test_target");
        assert_eq!(test_message.message, "Test message");
        assert_eq!(test_message.location, Some("test.rs:123".to_string()));

        println!("SUCCESS: Debug storage is working correctly");
    }

    #[test]
    fn test_debug_storage_limits() {
        // Clear messages
        clear_debug_messages();

        // Add more than the limit (1000)
        for i in 0..1500 {
            let message = DebugMessage {
                timestamp: chrono::Utc::now(),
                level: "INFO".to_string(),
                target: "test".to_string(),
                message: format!("Test message {}", i),
                location: None,
            };
            DEBUG_LOG_STORAGE.add_message(message);
        }

        let messages = get_debug_messages();
        assert!(messages.len() <= 1000, "Storage should be limited to 1000 messages, got {}", messages.len());
        assert!(messages.len() > 0, "Should have some messages");

        // Should have the most recent messages (higher numbers)
        if !messages.is_empty() {
            let last_message = &messages[messages.len() - 1];
            assert!(last_message.message.contains("1499"), "Should have the last message, got: {}", last_message.message);
        }

        println!("SUCCESS: Debug storage limits are working correctly");
    }
}

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
                .with_filter(filter.clone()),
        )
        .with(
            MemoryLogLayer
                .with_filter(filter)
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
