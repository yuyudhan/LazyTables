// FilePath: src/io/mod.rs

// Async I/O operations module. Provides non-blocking file system wrappers
// using tokio::fs with timeouts to prevent UI freezes.

pub mod async_fs;

pub use async_fs::*;
