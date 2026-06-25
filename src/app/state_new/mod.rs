// FilePath: src/app/state/mod.rs
//
// Core application state structure and module exports

#![forbid(unsafe_code)]

use crate::{
    config::Config,
    database::{AppStateDb, ConnectionConfig, ConnectionManager, ConnectionStatus},
    state::{ui::UIState, DatabaseState},
    ui::components::{
        ConnectionModalState, ConnectionMode, DebugView, QueryEditor, TableViewerState,
        ToastManager,
    },
};

// Re-export sub-modules
pub mod connections;
pub mod modals;
pub mod navigation;
pub mod query;
pub mod sql_files;
pub mod tables;

// Re-export for backward compatibility
pub use crate::state::ui::{FocusedPane, HelpMode, HelpPaneFocus};
pub use crate::state::view::{AppView, ConnectionFormMode, OverlayView, TextInputMode};

/// Query editor movement directions
#[derive(Debug, Clone, Copy)]
pub enum QueryEditorMovement {
    Up,
    Down,
    Left,
    Right,
}

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// UI state that can be saved/restored
    pub ui: UIState,
    /// Database state separated from UI
    pub db: DatabaseState,
    /// Connection modal state
    pub connection_modal_state: ConnectionModalState,
    /// SQL query editor content
    pub query_content: String,
    /// List of saved SQL files for current project
    pub saved_sql_files: Vec<String>,
    /// Table viewer state
    pub table_viewer_state: TableViewerState,
    /// Toast notifications manager
    pub toast_manager: ToastManager,
    /// Query editor component
    pub query_editor: QueryEditor,
    /// Debug view component
    pub debug_view: DebugView,
    /// Connection mode component (for full-screen connection management)
    pub connection_mode: Option<ConnectionMode>,
    /// Application state database
    pub app_state_db: AppStateDb,
    /// Persistent connection manager
    pub connection_manager: ConnectionManager,
    /// Connection attempt in progress (stores connection index being attempted)
    pub connecting_in_progress: Option<usize>,
    /// Animation frame counter for loading dots (0-2)
    pub connecting_animation_frame: u8,
    /// Connection attempt start time for timeout tracking
    pub connection_start_time: Option<std::time::Instant>,
    /// Connection timeout in seconds
    pub connection_timeout_seconds: u64,
    /// Test connection in progress (modal test button)
    pub test_connection_in_progress: bool,
    /// Animation frame counter for test connection loading dots (0-2)
    pub test_animation_frame: u8,
    /// Test connection start time for timeout tracking
    pub test_start_time: Option<std::time::Instant>,
}

impl AppState {
    /// Create a new application state
    pub async fn new() -> Self {
        // Ensure all directories exist
        let _ = crate::config::Config::ensure_directories();

        let db = DatabaseState::new().await;
        let saved_sql_files = Vec::new(); // Will be loaded only when a connection is connected

        // Load or create UI state
        let mut ui = UIState::load().unwrap_or_default();

        // Update list states based on loaded connections
        ui.update_connection_selection(db.connections.connections.len());

        // Don't load SQL files during initialization to avoid block_on in async context
        // They will be loaded lazily when first needed or when a connection is established

        Self {
            ui,
            db,
            connection_modal_state: ConnectionModalState::new(),
            query_content: String::new(),
            saved_sql_files,
            table_viewer_state: TableViewerState::new(),
            toast_manager: ToastManager::new(),
            query_editor: QueryEditor::new(),
            debug_view: DebugView::new(),
            connection_mode: None,
            app_state_db: AppStateDb::new(),
            connection_manager: ConnectionManager::new(),
            connecting_in_progress: None,
            connecting_animation_frame: 0,
            connection_start_time: None,
            connection_timeout_seconds: 30, // 30 seconds timeout
            test_connection_in_progress: false,
            test_animation_frame: 0,
            test_start_time: None,
        }
    }

    /// Initialize the application state database asynchronously
    pub async fn initialize_app_db(&mut self) -> Result<(), String> {
        match AppStateDb::initialize().await {
            Ok(app_db) => {
                self.app_state_db = app_db;
                Ok(())
            }
            Err(e) => Err(format!("Failed to initialize application database: {}", e)),
        }
    }
}
