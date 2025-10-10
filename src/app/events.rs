// FilePath: src/app/events.rs

//! Background task events system
//!
//! This module provides event types for communicating between background async tasks
//! and the main UI thread, preventing UI freezes during long-running operations.

use crate::database::{DatabaseObjectList, TableMetadata};

/// Events sent from background tasks to the main UI thread
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Connection attempt completed
    ConnectionComplete {
        connection_index: usize,
        connection_id: String,
        result: Result<DatabaseObjectList, String>,
    },

    /// Connection test completed
    ConnectionTestComplete {
        result: Result<TestConnectionResult, String>,
    },

    /// Disconnect completed
    DisconnectComplete {
        connection_id: String,
    },

    /// File save operation completed
    FileSaveComplete {
        filename: String,
        result: Result<(), String>,
    },

    /// File load operation completed
    FileLoadComplete {
        filename: String,
        result: Result<String, String>,
    },

    /// SQL files list loaded
    SqlFilesLoaded {
        files: Vec<String>,
    },

    /// Connection refresh completed
    ConnectionsRefreshComplete {
        result: Result<usize, String>, // Returns new connection count
    },

    /// Query execution completed
    QueryExecutionComplete {
        result: Result<(Vec<String>, Vec<Vec<String>>), String>,
    },

    /// Table data load completed
    TableDataLoadComplete {
        tab_index: usize,
        result: Result<(), String>,
    },

    /// Table metadata load completed
    TableMetadataLoadComplete {
        result: Result<TableMetadata, String>,
    },
}

/// Result of a connection test
#[derive(Debug, Clone)]
pub struct TestConnectionResult {
    pub success: bool,
    pub message: String,
    pub tables_count: usize,
    pub views_count: usize,
    pub total_objects: usize,
    pub response_time_ms: u128,
}

impl TestConnectionResult {
    /// Create a successful test result
    pub fn success(
        message: String,
        tables_count: usize,
        views_count: usize,
        total_objects: usize,
        response_time_ms: u128,
    ) -> Self {
        Self {
            success: true,
            message,
            tables_count,
            views_count,
            total_objects,
            response_time_ms,
        }
    }

    /// Create a failed test result
    pub fn failure(message: String, response_time_ms: u128) -> Self {
        Self {
            success: false,
            message,
            tables_count: 0,
            views_count: 0,
            total_objects: 0,
            response_time_ms,
        }
    }
}
