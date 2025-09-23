// FilePath: src/debug_log.rs

use std::fs::OpenOptions;
use std::io::Write;

/// Simple debug logging to file for TUI applications
pub fn debug_log(message: &str) {
    let log_path = std::env::current_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("lazytables_debug.log");

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        let _ = writeln!(file, "[{}] {}", timestamp, message);
    }
}

/// Macro for easier debug logging
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        crate::debug_log::debug_log(&format!($($arg)*));
    };
}
