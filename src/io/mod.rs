// FilePath: src/io/mod.rs

//! Async I/O operations module
//!
//! This module provides non-blocking async wrappers for file system operations
//! to prevent UI freezes in the TUI application.

pub mod async_fs;

pub use async_fs::*;
