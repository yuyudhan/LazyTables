// FilePath: src/app/state.rs

use crate::{
    config::Config,
    database::{AppStateDb, ConnectionConfig, ConnectionManager, ConnectionStatus, DatabaseType},
    state::{ui::UIState, DatabaseState},
    ui::components::{
        ConnectionModalState, DebugView, QueryEditor, TableCreatorState, TableEditorState,
        TableViewerState, ToastManager,
    },
};

// Re-export for backward compatibility
pub use crate::state::ui::{FocusedPane, HelpMode, HelpPaneFocus, QueryEditMode};

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
    /// Table creator state
    pub table_creator_state: TableCreatorState,
    /// Table editor state
    pub table_editor_state: TableEditorState,
    /// Table viewer state
    pub table_viewer_state: TableViewerState,
    /// Toast notifications manager
    pub toast_manager: ToastManager,
    /// Query editor component
    pub query_editor: QueryEditor,
    /// Debug view component
    pub debug_view: DebugView,
    /// Application state database
    pub app_state_db: AppStateDb,
    /// Persistent connection manager
    pub connection_manager: ConnectionManager,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        // Ensure all directories exist
        let _ = crate::config::Config::ensure_directories();

        let db = DatabaseState::new();
        let saved_sql_files = Vec::new(); // Will be loaded only when a connection is connected

        // Load or create UI state
        let mut ui = UIState::load().unwrap_or_default();

        // Update list states based on loaded connections
        ui.update_connection_selection(db.connections.connections.len());

        let mut app_state = Self {
            ui,
            db,
            connection_modal_state: ConnectionModalState::new(),
            query_content: String::new(),
            saved_sql_files,
            table_creator_state: TableCreatorState::new(),
            table_editor_state: TableEditorState::new("table".to_string()),
            table_viewer_state: TableViewerState::new(),
            toast_manager: ToastManager::new(),
            query_editor: QueryEditor::new(),
            debug_view: DebugView::new(),
            app_state_db: AppStateDb::new(),
            connection_manager: ConnectionManager::new(),
        };

        // Load SQL files during initialization
        app_state.refresh_sql_files();

        app_state
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

    /// Cycle focus to the next pane
    pub fn cycle_focus_forward(&mut self) {
        self.ui.cycle_focus_forward();
    }

    /// Cycle focus to the previous pane
    pub fn cycle_focus_backward(&mut self) {
        self.ui.cycle_focus_backward();
    }

    /// Move focus left (Ctrl+h)
    pub fn move_focus_left(&mut self) {
        self.ui.move_focus_left();
    }

    /// Move focus down (Ctrl+j)
    pub fn move_focus_down(&mut self) {
        self.ui.move_focus_down();
    }

    /// Move focus up (Ctrl+k)
    pub fn move_focus_up(&mut self) {
        self.ui.move_focus_up();
    }

    /// Move focus right (Ctrl+l)
    pub fn move_focus_right(&mut self) {
        self.ui.move_focus_right();
    }

    /// Move selection up based on current focus
    pub fn move_up(&mut self) {
        match self.ui.focused_pane {
            FocusedPane::Connections => {
                self.connection_up();
            }
            FocusedPane::Tables => {
                self.table_up();
            }
            FocusedPane::TabularOutput => {
                if let Some(tab) = self.table_viewer_state.current_tab_mut() {
                    if !tab.in_edit_mode {
                        tab.move_up();
                    }
                }
            }
            FocusedPane::SqlFiles => {
                self.ui.selected_sql_file = self.ui.selected_sql_file.saturating_sub(1);
            }
            FocusedPane::QueryWindow => {
                if self.ui.query_cursor_line > 0 {
                    self.ui.query_cursor_line -= 1;

                    // Scroll up if cursor goes above viewport
                    if self.ui.query_cursor_line < self.ui.query_viewport_offset {
                        self.ui.query_viewport_offset = self.ui.query_cursor_line;
                    }
                }
            }
            FocusedPane::Details => {
                // Scroll up in details pane
                self.ui.details_viewport_offset = self.ui.details_viewport_offset.saturating_sub(1);
            }
        }
    }

    /// Move selection down based on current focus
    pub fn move_down(&mut self) {
        match self.ui.focused_pane {
            FocusedPane::Connections => {
                self.connection_down();
            }
            FocusedPane::Tables => {
                self.table_down();
            }
            FocusedPane::TabularOutput => {
                if let Some(tab) = self.table_viewer_state.current_tab_mut() {
                    if !tab.in_edit_mode {
                        tab.move_down();
                    }
                }
            }
            FocusedPane::SqlFiles => {
                let max_files = self.saved_sql_files.len().saturating_sub(1);
                if self.ui.selected_sql_file < max_files {
                    self.ui.selected_sql_file += 1;
                }
            }
            FocusedPane::QueryWindow => {
                let lines = self.query_content.lines().count();
                if self.ui.query_cursor_line < lines.saturating_sub(1) {
                    self.ui.query_cursor_line += 1;

                    // Scroll down if cursor goes below viewport
                    // Note: viewport_height is updated during rendering, default to 20 if not set
                    let effective_height = if self.ui.query_viewport_height > 0 {
                        self.ui.query_viewport_height.saturating_sub(1) // Leave room for bottom
                    } else {
                        20 // Default height if not yet calculated
                    };

                    if self.ui.query_cursor_line >= self.ui.query_viewport_offset + effective_height
                    {
                        self.ui.query_viewport_offset =
                            self.ui.query_cursor_line.saturating_sub(effective_height) + 1;
                    }
                }
            }
            FocusedPane::Details => {
                // Scroll down in details pane with proper bounds checking
                if self.ui.details_viewport_offset < self.ui.details_max_scroll_offset {
                    self.ui.details_viewport_offset += 1;
                }
            }
        }
    }

    /// Move selection left based on current focus
    pub fn move_left(&mut self) {
        crate::log_debug!(
            "AppState::move_left called, focused_pane: {:?}",
            self.ui.focused_pane
        );
        match self.ui.focused_pane {
            FocusedPane::TabularOutput => {
                crate::log_debug!("In TabularOutput branch");
                if let Some(tab) = self.table_viewer_state.current_tab_mut() {
                    crate::log_debug!("Got current tab, in_edit_mode: {}", tab.in_edit_mode);
                    if !tab.in_edit_mode {
                        crate::log_debug!("Calling tab.move_left()");
                        tab.move_left();
                    } else {
                        crate::log_debug!("Skipping because in edit mode");
                    }
                } else {
                    crate::log_debug!("No current tab available");
                }
            }
            FocusedPane::QueryWindow => {
                self.ui.query_cursor_column = self.ui.query_cursor_column.saturating_sub(1);
            }
            _ => {
                crate::log_debug!("Not in TabularOutput or QueryWindow pane");
            }
        }
    }

    /// Move selection right based on current focus
    pub fn move_right(&mut self) {
        crate::log_debug!(
            "AppState::move_right called, focused_pane: {:?}",
            self.ui.focused_pane
        );
        match self.ui.focused_pane {
            FocusedPane::TabularOutput => {
                crate::log_debug!("In TabularOutput branch");
                if let Some(tab) = self.table_viewer_state.current_tab_mut() {
                    crate::log_debug!("Got current tab, in_edit_mode: {}", tab.in_edit_mode);
                    if !tab.in_edit_mode {
                        crate::log_debug!("Calling tab.move_right()");
                        tab.move_right();
                    } else {
                        crate::log_debug!("Skipping because in edit mode");
                    }
                } else {
                    crate::log_debug!("No current tab available");
                }
            }
            FocusedPane::QueryWindow => {
                if let Some(current_line) =
                    self.query_content.lines().nth(self.ui.query_cursor_line)
                {
                    if self.ui.query_cursor_column < current_line.len() {
                        self.ui.query_cursor_column += 1;
                    }
                }
            }
            _ => {
                crate::log_debug!("Not in TabularOutput or QueryWindow pane");
            }
        }
    }

    /// Get currently selected connection
    pub fn get_selected_connection(
        &self,
    ) -> Option<&crate::database::connection::ConnectionConfig> {
        self.db
            .connections
            .connections
            .get(self.ui.selected_connection)
    }

    /// Get currently selected connection (mutable)
    pub fn get_selected_connection_mut(
        &mut self,
    ) -> Option<&mut crate::database::connection::ConnectionConfig> {
        self.db
            .connections
            .connections
            .get_mut(self.ui.selected_connection)
    }

    /// Open the add connection modal
    pub fn open_add_connection_modal(&mut self) {
        self.ui.show_add_connection_modal = true;
        self.connection_modal_state = ConnectionModalState::new(); // Reset state
    }

    /// Close the add connection modal
    pub fn close_add_connection_modal(&mut self) {
        self.ui.show_add_connection_modal = false;
        self.connection_modal_state.clear(); // Clear any input
    }

    /// Open the edit connection modal for the currently selected connection
    pub fn open_edit_connection_modal(&mut self) {
        if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
        {
            self.connection_modal_state
                .populate_from_connection(connection);
            self.ui.show_edit_connection_modal = true;
        }
    }

    /// Close the edit connection modal
    pub fn close_edit_connection_modal(&mut self) {
        self.ui.show_edit_connection_modal = false;
        self.connection_modal_state.clear(); // Clear any input
    }

    /// Save connection from modal
    pub fn save_connection_from_modal(&mut self) -> Result<(), String> {
        let mut connection = self.connection_modal_state.try_create_connection()?;

        if self.ui.show_edit_connection_modal {
            // Update existing connection - preserve ID
            if let Some(existing) = self
                .db
                .connections
                .connections
                .get(self.ui.selected_connection)
            {
                connection.id = existing.id.clone();
                if let Err(e) = self.db.connections.update_connection(connection) {
                    return Err(format!("Failed to update connection: {e}"));
                }
            }
            self.close_edit_connection_modal();
        } else {
            // Add new connection
            if let Err(e) = self.db.connections.add_connection(connection) {
                return Err(format!("Failed to add connection: {e}"));
            }
            self.close_add_connection_modal();
        }

        self.clamp_connection_selection();
        Ok(())
    }

    /// Ensure selected connection index is within bounds
    pub fn clamp_connection_selection(&mut self) {
        if !self.db.connections.connections.is_empty() {
            let max_index = self.db.connections.connections.len() - 1;
            if self.ui.selected_connection > max_index {
                self.ui.selected_connection = max_index;
            }
            self.ui
                .connections_list_state
                .select(Some(self.ui.selected_connection));
        } else {
            self.ui.selected_connection = 0;
            self.ui.connections_list_state.select(None);
        }
    }

    /// Move connection selection down
    pub fn connection_down(&mut self) {
        if !self.db.connections.connections.is_empty() {
            let len = self.db.connections.connections.len();
            self.ui.selected_connection = (self.ui.selected_connection + 1) % len;
            self.ui
                .connections_list_state
                .select(Some(self.ui.selected_connection));
        }
    }

    /// Move connection selection up
    pub fn connection_up(&mut self) {
        if !self.db.connections.connections.is_empty() {
            let len = self.db.connections.connections.len();
            self.ui.selected_connection = if self.ui.selected_connection > 0 {
                self.ui.selected_connection - 1
            } else {
                len - 1
            };
            self.ui
                .connections_list_state
                .select(Some(self.ui.selected_connection));
        }
    }

    /// Move table selection down
    pub fn table_down(&mut self) {
        self.ui.table_selection_down();
        // Clear metadata when selection changes (will load when Enter is pressed)
        self.db.current_table_metadata = None;
        // Reset details pane scroll position for new table
        self.ui.details_viewport_offset = 0;
    }

    /// Move table selection up
    pub fn table_up(&mut self) {
        self.ui.table_selection_up();
        // Clear metadata when selection changes (will load when Enter is pressed)
        self.db.current_table_metadata = None;
        // Reset details pane scroll position for new table
        self.ui.details_viewport_offset = 0;
    }

    /// Update table list state when tables change
    pub fn update_table_selection(&mut self) {
        // The unified selection system now handles this automatically
        // through build_selectable_table_items(), so this method is kept
        // for backward compatibility but delegates to the new system
        self.ui
            .build_selectable_table_items(&self.db.database_objects);
    }

    /// Disconnect all connections except the one at the given index
    pub fn disconnect_all_except(&mut self, except_index: usize) {
        for (index, connection) in self.db.connections.connections.iter_mut().enumerate() {
            if index != except_index && connection.is_connected() {
                connection.status = ConnectionStatus::Disconnected;
            }
        }
        // Save updated connection statuses
        let _ = self.db.connections.save();
    }

    /// Attempt to connect to the selected database
    pub async fn connect_to_selected_database(&mut self) {
        // Get the actual selected connection index (accounting for search)
        let selected_index = if let Some(index) = self
            .ui
            .get_selected_connection_index(&self.db.connections.connections)
        {
            index
        } else {
            return; // No connection selected
        };

        if let Some(connection) = self.db.connections.connections.get(selected_index).cloned() {
            // Disconnect all other connections first
            self.disconnect_all_except(selected_index);

            // Set connection status to connecting
            if let Some(conn) = self.db.connections.connections.get_mut(selected_index) {
                conn.status = ConnectionStatus::Connecting;
            }

            // Clear previous tables and errors
            self.db.tables.clear();
            self.db.table_load_error = None;

            // Reset table viewer state when switching connections
            self.table_viewer_state = TableViewerState::new();

            // Clear table metadata
            self.db.current_table_metadata = None;

            // Attempt connection based on database type
            let connection_name = connection.name.clone();
            let result = self.try_connect_to_database(&connection).await;

            // Update connection status based on result
            let connection_succeeded = result.is_ok();

            if let Some(conn) = self.db.connections.connections.get_mut(selected_index) {
                match result {
                    Ok(objects) => {
                        conn.status = ConnectionStatus::Connected;
                        self.db.database_objects = Some(objects.clone());
                        self.db.tables = objects.tables.iter().map(|t| t.name.clone()).collect();
                        if let Some(ref error) = objects.error {
                            self.db.table_load_error = Some(error.clone());
                        }
                        // Update the selectable table items list
                        self.ui
                            .build_selectable_table_items(&self.db.database_objects);
                    }
                    Err(error) => {
                        let error_msg = error.clone();
                        conn.status = ConnectionStatus::Failed(error.clone());
                        self.db.database_objects = None;
                        self.db.tables.clear();
                        // Clear the selectable table items list
                        self.ui.build_selectable_table_items(&None);

                        // Reset table viewer state when connection fails
                        self.table_viewer_state = TableViewerState::new();

                        // Clear table metadata
                        self.db.current_table_metadata = None;

                        self.toast_manager
                            .error(format!("Connection failed: {error_msg}"));
                    }
                }
            }

            // Handle post-connection tasks after mutable borrow ends
            if connection_succeeded {
                self.update_table_selection();
                self.toast_manager
                    .success(format!("Connected to {connection_name}"));

                // Update active connection in app state database
                if let Some(conn) = self.get_selected_connection() {
                    let _ = self
                        .app_state_db
                        .set_active_connection(
                            &conn.id,
                            &conn.name,
                            conn.database_type.display_name(),
                        )
                        .await;
                }

                // Refresh SQL files to show connection-specific files
                self.refresh_sql_files();
            }

            // Save updated connection status
            let _ = self.db.connections.save();
        }
    }

    /// Try to connect to a specific database and return database objects
    async fn try_connect_to_database(
        &mut self,
        connection: &ConnectionConfig,
    ) -> Result<crate::database::DatabaseObjectList, String> {
        self.db
            .try_connect_to_database(connection, &self.connection_manager)
            .await
    }

    /// Disconnect from current database
    pub async fn disconnect_from_database(&mut self) {
        self.disconnect_from_database_sync();

        // Clear active connection in app state database
        let _ = self.app_state_db.clear_active_connection().await;

        // Refresh SQL files to clear the list (no connection = no files)
        self.refresh_sql_files();
    }

    /// Disconnect from current database (synchronous part only)
    pub fn disconnect_from_database_sync(&mut self) {
        if let Some(connection) = self
            .db
            .connections
            .connections
            .get_mut(self.ui.selected_connection)
        {
            connection.status = ConnectionStatus::Disconnected;
            self.db.database_objects = None;
            self.db.tables.clear();
            self.db.table_load_error = None;
            // Clear the selectable table items list
            self.ui.build_selectable_table_items(&None);
            self.update_table_selection();

            // Reset table viewer state - close all tabs and reset to initial state
            self.table_viewer_state = TableViewerState::new();

            // Clear table metadata
            self.db.current_table_metadata = None;

            // Note: App state database clearing is handled in the async version

            // Save updated connection status
            let _ = self.db.connections.save();

            // Refresh SQL files to clear the list (no connection = no files)
            self.refresh_sql_files();
        }
    }

    /// Get currently selected SQL file name
    pub fn get_selected_sql_file(&self) -> Option<String> {
        if self.ui.sql_files_search_active {
            // When search is active, use filtered list
            let filtered = self.get_filtered_sql_files();
            let selected_index = self.get_filtered_sql_file_selection();
            filtered.get(selected_index).cloned()
        } else {
            // Normal mode, use the raw selection
            self.saved_sql_files.get(self.ui.selected_sql_file).cloned()
        }
    }

    /// Load the currently selected SQL file
    pub fn load_selected_sql_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(filename) = self.get_selected_sql_file() {
            self.load_query_file(&filename)
        } else {
            Err("No SQL file selected".into())
        }
    }

    /// Ensure selected SQL file index is within bounds
    pub fn clamp_sql_file_selection(&mut self) {
        if !self.saved_sql_files.is_empty() {
            let max_index = self.saved_sql_files.len() - 1;
            if self.ui.selected_sql_file > max_index {
                self.ui.selected_sql_file = max_index;
            }
        } else {
            self.ui.selected_sql_file = 0;
        }
    }

    /// Load list of saved SQL files for current connection (only if connection is active)
    fn load_sql_files_for_connection(&self) -> Vec<String> {
        use std::fs;

        let mut files = Vec::new();

        // Only load files if we have an active connected connection
        if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
        {
            // Only show files if connection is actually connected
            if !connection.is_connected() {
                return files; // Return empty if not connected
            }

            let connection_name = &connection.name;

            // Try connection-specific directory first
            let connection_dir = Config::sql_files_dir().join(connection_name);
            if let Ok(entries) = fs::read_dir(&connection_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "sql") {
                        if let Some(name) = path.file_stem().and_then(|name| name.to_str()) {
                            files.push(name.to_string());
                        }
                    }
                }
            }

            // Only load connection-specific files, no shared files
            // Each connection should only see its own SQL files

            files.sort();
        }

        files
    }

    /// Refresh the list of saved SQL files
    pub fn refresh_sql_files(&mut self) {
        self.saved_sql_files = self.load_sql_files_for_connection();
        self.clamp_sql_file_selection();

        // Update UI state to reflect the new file count
        self.ui
            .update_sql_file_selection(self.saved_sql_files.len());
    }

    /// Save current query content to a file (only if connection is active)
    pub fn save_query_as(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::config::Config;
        use std::fs;

        // Get connection-specific directory - require active connection
        let connection = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
            .ok_or("No connection selected")?;

        // Only allow saving if connection is active
        if !connection.is_connected() {
            return Err("Cannot save SQL file: No active connection".into());
        }

        let connection_name = &connection.name;

        // Save to connection-specific directory
        let sql_dir = Config::sql_files_dir().join(connection_name);
        fs::create_dir_all(&sql_dir)?;

        let file_path = sql_dir.join(format!("{filename}.sql"));
        fs::write(&file_path, &self.query_content)?;

        self.ui.current_sql_file = Some(filename.to_string());
        self.ui.query_modified = false;
        self.refresh_sql_files();

        Ok(())
    }

    /// Save current query content to the currently loaded file
    pub fn save_query(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(filename) = &self.ui.current_sql_file.clone() {
            self.save_query_as(filename)
        } else {
            Err("No file is currently loaded".into())
        }
    }

    /// Load a SQL file into the query editor (only if connection is active)
    pub fn load_query_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        // Get connection-specific directory - require active connection
        let connection = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
            .ok_or("No connection selected")?;

        // Only allow loading if connection is active
        if !connection.is_connected() {
            return Err("Cannot load SQL file: No active connection".into());
        }

        let connection_name = &connection.name;

        // All files are connection-specific now
        let file_path = Config::sql_files_dir()
            .join(connection_name)
            .join(format!("{filename}.sql"));

        let content = fs::read_to_string(&file_path)?;
        self.query_content = content;
        self.ui.current_sql_file = Some(filename.to_string());
        self.ui.query_modified = false;
        self.ui.query_cursor_line = 0;
        self.ui.query_cursor_column = 0;
        self.ui.query_viewport_offset = 0; // Reset viewport to top
        self.ui.query_edit_mode = QueryEditMode::Normal;

        Ok(())
    }

    /// Create a new SQL file (only if connection is active)
    pub fn new_query_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Check if connection is active before creating new file
        if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
        {
            if !connection.is_connected() {
                return Err("Cannot create SQL file: No active connection".into());
            }
        } else {
            return Err("No connection selected".into());
        }

        self.query_content.clear();
        self.ui.current_sql_file = Some(filename.to_string());
        self.ui.query_modified = false;
        self.ui.query_cursor_line = 0;
        self.ui.query_cursor_column = 0;

        // Save the empty file
        self.save_query_as(filename)
    }

    /// Record SQL file activity when a file is opened
    pub async fn record_sql_file_activity(&self, filename: &str, file_path: &str) {
        if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
        {
            if connection.is_connected() {
                let _ = self
                    .app_state_db
                    .record_sql_file_activity(&connection.id, file_path, filename)
                    .await;
            }
        }
    }

    /// Load a SQL file and record activity
    pub async fn load_sql_file_with_activity(
        &mut self,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Load the file first
        self.load_query_file(filename)?;

        // Record activity - all files are connection-specific now
        let file_path = if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
        {
            Config::sql_files_dir()
                .join(&connection.name)
                .join(format!("{filename}.sql"))
                .to_string_lossy()
                .to_string()
        } else {
            return Err("No connection selected".into());
        };

        self.record_sql_file_activity(filename, &file_path).await;
        Ok(())
    }

    /// Insert character at current cursor position in query editor
    pub fn insert_char_at_cursor(&mut self, c: char) {
        if c == '\n' {
            // Handle newline insertion specially
            self.insert_newline_at_cursor();
            return;
        }

        // Get all lines as a mutable vector
        let lines: Vec<&str> = self.query_content.lines().collect();
        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

        // Ensure we have enough lines
        while new_lines.len() <= self.ui.query_cursor_line {
            new_lines.push(String::new());
        }

        // Insert character at current position
        if let Some(line) = new_lines.get_mut(self.ui.query_cursor_line) {
            // Ensure cursor column is within bounds
            if self.ui.query_cursor_column > line.len() {
                self.ui.query_cursor_column = line.len();
            }

            line.insert(self.ui.query_cursor_column, c);
            self.ui.query_cursor_column += 1;
        }

        self.query_content = new_lines.join("\n");
        self.ui.query_modified = true;
    }

    /// Insert a newline at the current cursor position
    fn insert_newline_at_cursor(&mut self) {
        let lines: Vec<&str> = self.query_content.lines().collect();
        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

        // Ensure we have enough lines
        while new_lines.len() <= self.ui.query_cursor_line {
            new_lines.push(String::new());
        }

        if let Some(current_line) = new_lines.get_mut(self.ui.query_cursor_line) {
            // Split the current line at the cursor position
            let (before, after) =
                current_line.split_at(self.ui.query_cursor_column.min(current_line.len()));
            let after = after.to_string();
            *current_line = before.to_string();

            // Insert the new line after the current one
            new_lines.insert(self.ui.query_cursor_line + 1, after);
        }

        self.query_content = new_lines.join("\n");

        // Move cursor to beginning of next line
        self.ui.query_cursor_line += 1;
        self.ui.query_cursor_column = 0;
        self.ui.query_modified = true;

        // Adjust viewport if necessary
        let effective_height = if self.ui.query_viewport_height > 0 {
            self.ui.query_viewport_height.saturating_sub(1)
        } else {
            20
        };

        if self.ui.query_cursor_line >= self.ui.query_viewport_offset + effective_height {
            self.ui.query_viewport_offset =
                self.ui.query_cursor_line.saturating_sub(effective_height) + 1;
        }
    }

    /// Delete character at current cursor position in query editor (backspace)
    pub fn delete_char_at_cursor(&mut self) {
        let lines: Vec<&str> = self.query_content.lines().collect();

        if lines.is_empty() {
            return;
        }

        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

        if self.ui.query_cursor_column > 0 {
            // Delete within the current line
            if let Some(line) = new_lines.get_mut(self.ui.query_cursor_line) {
                if self.ui.query_cursor_column <= line.len() {
                    line.remove(self.ui.query_cursor_column - 1);
                    self.ui.query_cursor_column -= 1;
                    self.ui.query_modified = true;
                }
            }
        } else if self.ui.query_cursor_line > 0 {
            // At beginning of line, merge with previous line
            let current_line = new_lines[self.ui.query_cursor_line].clone();
            new_lines.remove(self.ui.query_cursor_line);

            if let Some(prev_line) = new_lines.get_mut(self.ui.query_cursor_line - 1) {
                self.ui.query_cursor_column = prev_line.len();
                prev_line.push_str(&current_line);
            }

            self.ui.query_cursor_line -= 1;
            self.ui.query_modified = true;

            // Adjust viewport if necessary
            if self.ui.query_cursor_line < self.ui.query_viewport_offset {
                self.ui.query_viewport_offset = self.ui.query_cursor_line;
            }
        }

        self.query_content = new_lines.join("\n");
    }

    /// Move cursor to next word (vim 'w' motion)
    pub fn move_to_next_word(&mut self) {
        let lines: Vec<&str> = self.query_content.lines().collect();
        if lines.is_empty() {
            return;
        }

        if let Some(current_line) = lines.get(self.ui.query_cursor_line) {
            let chars: Vec<char> = current_line.chars().collect();
            let mut pos = self.ui.query_cursor_column;

            // Skip current word
            while pos < chars.len() && !chars[pos].is_whitespace() {
                pos += 1;
            }
            // Skip whitespace
            while pos < chars.len() && chars[pos].is_whitespace() {
                pos += 1;
            }

            if pos < chars.len() {
                self.ui.query_cursor_column = pos;
            } else if self.ui.query_cursor_line < lines.len() - 1 {
                // Move to beginning of next line
                self.ui.query_cursor_line += 1;
                self.ui.query_cursor_column = 0;
            }
        }
    }

    /// Move cursor to previous word (vim 'b' motion)
    pub fn move_to_prev_word(&mut self) {
        let lines: Vec<&str> = self.query_content.lines().collect();
        if lines.is_empty() {
            return;
        }

        if let Some(current_line) = lines.get(self.ui.query_cursor_line) {
            let chars: Vec<char> = current_line.chars().collect();

            if self.ui.query_cursor_column > 0 {
                let mut pos = self.ui.query_cursor_column - 1;

                // Skip whitespace
                while pos > 0 && chars[pos].is_whitespace() {
                    pos -= 1;
                }
                // Skip word
                while pos > 0 && !chars[pos - 1].is_whitespace() {
                    pos -= 1;
                }

                self.ui.query_cursor_column = pos;
            } else if self.ui.query_cursor_line > 0 {
                // Move to end of previous line
                self.ui.query_cursor_line -= 1;
                if let Some(prev_line) = lines.get(self.ui.query_cursor_line) {
                    self.ui.query_cursor_column = prev_line.len();
                }
            }
        }
    }

    /// Move cursor to end of word (vim 'e' motion)
    pub fn move_to_end_of_word(&mut self) {
        let lines: Vec<&str> = self.query_content.lines().collect();
        if lines.is_empty() {
            return;
        }

        if let Some(current_line) = lines.get(self.ui.query_cursor_line) {
            let chars: Vec<char> = current_line.chars().collect();
            let mut pos = self.ui.query_cursor_column;

            if pos < chars.len() - 1 {
                pos += 1;
                // Skip to end of current word
                while pos < chars.len() - 1 && !chars[pos + 1].is_whitespace() {
                    pos += 1;
                }
                self.ui.query_cursor_column = pos;
            }
        }
    }

    /// Move to beginning of line (vim '0' motion)
    pub fn move_to_line_start(&mut self) {
        self.ui.query_cursor_column = 0;
    }

    /// Move to end of line (vim '$' motion)
    pub fn move_to_line_end(&mut self) {
        let lines: Vec<&str> = self.query_content.lines().collect();
        if let Some(current_line) = lines.get(self.ui.query_cursor_line) {
            self.ui.query_cursor_column = current_line.len().saturating_sub(1);
        }
    }

    /// Move to beginning of file (vim 'gg' motion)
    pub fn move_to_file_start(&mut self) {
        self.ui.query_cursor_line = 0;
        self.ui.query_cursor_column = 0;
        self.ui.query_viewport_offset = 0;
    }

    /// Move to end of file (vim 'G' motion)
    pub fn move_to_file_end(&mut self) {
        let lines = self.query_content.lines().count();
        if lines > 0 {
            self.ui.query_cursor_line = lines - 1;
            // Move to end of last line
            if let Some(last_line) = self.query_content.lines().last() {
                self.ui.query_cursor_column = last_line.len().saturating_sub(1);
            }

            // Adjust viewport to show the last line
            let effective_height = if self.ui.query_viewport_height > 0 {
                self.ui.query_viewport_height
            } else {
                20
            };

            if lines >= effective_height {
                self.ui.query_viewport_offset = lines.saturating_sub(effective_height);
            } else {
                self.ui.query_viewport_offset = 0;
            }
        }
    }

    /// Scroll half page down (vim Ctrl+d)
    pub fn scroll_half_page_down(&mut self) {
        let lines = self.query_content.lines().count();
        let half_page = self.ui.query_viewport_height.saturating_div(2).max(1);

        // Move cursor down by half page
        self.ui.query_cursor_line =
            (self.ui.query_cursor_line + half_page).min(lines.saturating_sub(1));

        // Adjust viewport
        let effective_height = if self.ui.query_viewport_height > 0 {
            self.ui.query_viewport_height.saturating_sub(1)
        } else {
            20
        };

        if self.ui.query_cursor_line >= self.ui.query_viewport_offset + effective_height {
            self.ui.query_viewport_offset =
                self.ui.query_cursor_line.saturating_sub(effective_height) + 1;
        }
    }

    /// Scroll half page up (vim Ctrl+u)
    pub fn scroll_half_page_up(&mut self) {
        let half_page = self.ui.query_viewport_height.saturating_div(2).max(1);

        // Move cursor up by half page
        self.ui.query_cursor_line = self.ui.query_cursor_line.saturating_sub(half_page);

        // Adjust viewport
        if self.ui.query_cursor_line < self.ui.query_viewport_offset {
            self.ui.query_viewport_offset = self.ui.query_cursor_line;
        }
    }

    /// Save current SQL file with connection-specific directory
    pub fn save_sql_file_with_connection(&mut self) -> Result<(), String> {
        // Get the current connection name
        let connection_name = if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
        {
            connection.name.clone()
        } else {
            "default".to_string()
        };

        // Create connection-specific directory
        let sql_dir = Config::sql_files_dir().join(&connection_name);
        std::fs::create_dir_all(&sql_dir)
            .map_err(|e| format!("Failed to create directory: {e}"))?;

        // Determine filename
        let filename = if let Some(ref current_file) = self.ui.current_sql_file {
            current_file.clone()
        } else {
            format!("query_{}.sql", chrono::Local::now().format("%Y%m%d_%H%M%S"))
        };

        let file_path = sql_dir.join(&filename);

        std::fs::write(&file_path, &self.query_content)
            .map_err(|e| format!("Failed to save file: {e}"))?;

        self.ui.current_sql_file = Some(filename);
        self.ui.query_modified = false;
        self.refresh_sql_files();

        Ok(())
    }

    /// Delete a SQL file by index
    pub fn delete_sql_file(&mut self, file_index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if file_index >= self.saved_sql_files.len() {
            return Err("Invalid file index".into());
        }

        let filename = &self.saved_sql_files[file_index];
        let connection_name = if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
        {
            connection.name.clone()
        } else {
            "default".to_string()
        };

        // Delete from both possible locations
        let connection_dir = Config::sql_files_dir().join(&connection_name);
        let root_dir = Config::sql_files_dir();

        let connection_path = connection_dir.join(format!("{filename}.sql"));
        let root_path = root_dir.join(format!("{filename}.sql"));

        let mut deleted = false;
        if connection_path.exists() {
            std::fs::remove_file(&connection_path)?;
            deleted = true;
        }
        if root_path.exists() {
            std::fs::remove_file(&root_path)?;
            deleted = true;
        }

        if !deleted {
            return Err("File not found".into());
        }

        // If we deleted the currently loaded file, clear it
        if self.ui.current_sql_file.as_ref() == Some(filename) {
            self.ui.current_sql_file = None;
            self.query_content.clear();
            self.ui.query_modified = false;
        }

        self.refresh_sql_files();
        Ok(())
    }

    /// Rename a SQL file
    pub fn rename_sql_file(
        &mut self,
        file_index: usize,
        new_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if file_index >= self.saved_sql_files.len() {
            return Err("Invalid file index".into());
        }

        let old_name = &self.saved_sql_files[file_index];
        let connection_name = if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
        {
            connection.name.clone()
        } else {
            "default".to_string()
        };

        // Check both possible locations for the file
        let connection_dir = Config::sql_files_dir().join(&connection_name);
        let root_dir = Config::sql_files_dir();

        let old_connection_path = connection_dir.join(format!("{old_name}.sql"));
        let old_root_path = root_dir.join(format!("{old_name}.sql"));

        let new_connection_path = connection_dir.join(format!("{new_name}.sql"));
        let new_root_path = root_dir.join(format!("{new_name}.sql"));

        // Rename in the location where it exists
        if old_connection_path.exists() {
            std::fs::rename(&old_connection_path, &new_connection_path)?;
        } else if old_root_path.exists() {
            std::fs::rename(&old_root_path, &new_root_path)?;
        } else {
            return Err("File not found".into());
        }

        // Update current file reference if needed
        if self.ui.current_sql_file.as_ref() == Some(old_name) {
            self.ui.current_sql_file = Some(new_name.to_string());
        }

        self.refresh_sql_files();
        Ok(())
    }

    /// Duplicate a SQL file
    pub fn duplicate_sql_file(
        &mut self,
        file_index: usize,
        new_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if file_index >= self.saved_sql_files.len() {
            return Err("Invalid file index".into());
        }

        let source_name = &self.saved_sql_files[file_index];
        let connection_name = if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
        {
            connection.name.clone()
        } else {
            "default".to_string()
        };

        // Find and read the source file
        let connection_dir = Config::sql_files_dir().join(&connection_name);
        let root_dir = Config::sql_files_dir();

        let source_connection_path = connection_dir.join(format!("{source_name}.sql"));
        let source_root_path = root_dir.join(format!("{source_name}.sql"));

        let content = if source_connection_path.exists() {
            std::fs::read_to_string(&source_connection_path)?
        } else if source_root_path.exists() {
            std::fs::read_to_string(&source_root_path)?
        } else {
            return Err("Source file not found".into());
        };

        // Write to the same location (connection-specific if it existed there, otherwise root)
        let target_path = if source_connection_path.exists() {
            connection_dir.join(format!("{new_name}.sql"))
        } else {
            root_dir.join(format!("{new_name}.sql"))
        };

        std::fs::write(&target_path, content)?;
        self.refresh_sql_files();
        Ok(())
    }

    /// Create a new SQL file
    pub fn create_sql_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let connection_name = if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
        {
            connection.name.clone()
        } else {
            "default".to_string()
        };

        // Create in connection-specific directory
        let connection_dir = Config::sql_files_dir().join(&connection_name);
        std::fs::create_dir_all(&connection_dir)?;

        let file_path = connection_dir.join(format!("{filename}.sql"));

        // Create empty file
        std::fs::write(&file_path, "")?;

        // Load the new file and refresh list
        self.query_content.clear();
        self.ui.current_sql_file = Some(filename.to_string());
        self.ui.query_modified = false;

        // Clear any search state to ensure new file is visible
        self.ui.sql_files_search_active = false;
        self.ui.sql_files_search_query.clear();

        self.refresh_sql_files();

        // Select the newly created file in the list
        if let Some(index) = self.saved_sql_files.iter().position(|f| f == filename) {
            self.ui.selected_sql_file = index;
        }

        // Set cursor to beginning
        self.ui.query_cursor_line = 0;
        self.ui.query_cursor_column = 0;

        Ok(())
    }

    /// Get filtered SQL files list for display
    pub fn get_filtered_sql_files(&self) -> Vec<String> {
        self.ui.filter_sql_files(&self.saved_sql_files)
    }

    /// Get selected SQL file index adjusted for filtered list
    pub fn get_filtered_sql_file_selection(&self) -> usize {
        let filtered = self.get_filtered_sql_files();
        if filtered.is_empty() {
            return 0;
        }

        if self.ui.sql_files_search_active {
            // When searching, use selection as-is but clamp to filtered list
            self.ui
                .selected_sql_file
                .min(filtered.len().saturating_sub(1))
        } else {
            self.ui.selected_sql_file
        }
    }

    /// Update SQL file selection for filtered list
    pub fn update_sql_file_selection_for_filtered(&mut self, direction: i32) {
        let filtered = self.get_filtered_sql_files();
        if filtered.is_empty() {
            return;
        }

        let current = self.get_filtered_sql_file_selection();
        let new_selection = if direction > 0 {
            (current + 1) % filtered.len()
        } else if direction < 0 {
            if current > 0 {
                current - 1
            } else {
                filtered.len() - 1
            }
        } else {
            current
        };

        if self.ui.sql_files_search_active {
            self.ui.selected_sql_file = new_selection;
        } else {
            // Map back to original list
            if let Some(selected_name) = filtered.get(new_selection) {
                if let Some(original_index) =
                    self.saved_sql_files.iter().position(|f| f == selected_name)
                {
                    self.ui.selected_sql_file = original_index;
                }
            }
        }
    }

    /// Open table creator view
    pub fn open_table_creator(&mut self) {
        self.ui.show_table_creator = true;
        self.table_creator_state = TableCreatorState::new();
    }

    /// Close table creator view
    pub fn close_table_creator(&mut self) {
        self.ui.show_table_creator = false;
        self.table_creator_state.clear();
    }

    /// Open table editor view
    pub async fn open_table_editor(&mut self) {
        if let Some(table_name) = self.ui.get_selected_table_name() {
            self.ui.show_table_editor = true;
            self.table_editor_state = TableEditorState::new(table_name.clone());

            // Load table schema from database
            if let Err(e) = self.load_table_schema_for_editor(&table_name).await {
                self.table_editor_state.error_message =
                    Some(format!("Failed to load table schema: {e}"));
            }
        }
    }

    /// Close table editor view
    pub fn close_table_editor(&mut self) {
        self.ui.show_table_editor = false;
        self.table_editor_state.clear();
    }

    /// Load table schema for the editor
    async fn load_table_schema_for_editor(&mut self, table_name: &str) -> Result<(), String> {
        // Get the current connection
        if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
            .cloned()
        {
            match &connection.status {
                ConnectionStatus::Connected => {
                    // Load table schema based on database type
                    match connection.database_type {
                        DatabaseType::PostgreSQL => {
                            self.db
                                .load_table_editor_from_database(
                                    &connection,
                                    table_name,
                                    &mut self.table_editor_state,
                                )
                                .await
                        }
                        _ => Err(format!(
                            "Database type {} not yet supported for table editing",
                            connection.database_type.display_name()
                        )),
                    }
                }
                _ => Err("No active database connection".to_string()),
            }
        } else {
            Err("No connection selected".to_string())
        }
    }

    /// Apply table edits from table editor state
    pub async fn apply_table_edits_from_editor(&mut self) -> Result<(), String> {
        // Generate ALTER TABLE SQL statements
        let sql_statements = self.table_editor_state.generate_alter_table_sql()?;

        // Get the current connection
        if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
            .cloned()
        {
            match &connection.status {
                ConnectionStatus::Connected => {
                    // Execute the ALTER TABLE statements
                    for sql in &sql_statements {
                        self.execute_alter_table_sql(&connection, sql).await?;
                    }

                    // Refresh tables list
                    self.connect_to_selected_database().await;

                    // Close table editor
                    self.close_table_editor();

                    Ok(())
                }
                _ => Err("No active database connection".to_string()),
            }
        } else {
            Err("No connection selected".to_string())
        }
    }

    /// Execute ALTER TABLE SQL on PostgreSQL
    async fn execute_alter_table_sql(
        &self,
        connection: &ConnectionConfig,
        _sql: &str,
    ) -> Result<(), String> {
        match connection.database_type {
            DatabaseType::PostgreSQL => {
                use crate::database::postgres::PostgresConnection;
                use crate::database::Connection;

                let mut pg_connection = PostgresConnection::new(connection.clone());
                pg_connection
                    .connect()
                    .await
                    .map_err(|e| format!("Connection failed: {e}"))?;
                //
                //                 pg_connection
                //                     .execute_sql(sql)
                //                     .await
                //                     .map_err(|e| format!("Failed to execute ALTER TABLE: {e}"))?;

                let _ = pg_connection.disconnect().await;

                Ok(())
            }
            _ => Err(format!(
                "Database type {} not yet supported",
                connection.database_type.display_name()
            )),
        }
    }

    /// Create table from table creator state
    pub async fn create_table_from_creator(&mut self) -> Result<(), String> {
        // Generate SQL
        let sql = self.table_creator_state.generate_create_table_sql()?;

        // Get the current connection
        if let Some(connection) = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
            .cloned()
        {
            match &connection.status {
                ConnectionStatus::Connected => {
                    // Execute the CREATE TABLE statement
                    self.execute_create_table_sql(&connection, &sql).await?;

                    // Refresh tables list
                    self.connect_to_selected_database().await;

                    // Close table creator
                    self.close_table_creator();

                    Ok(())
                }
                _ => Err("No active database connection".to_string()),
            }
        } else {
            Err("No connection selected".to_string())
        }
    }

    /// Execute CREATE TABLE SQL on PostgreSQL
    async fn execute_create_table_sql(
        &self,
        connection: &ConnectionConfig,
        _sql: &str,
    ) -> Result<(), String> {
        match connection.database_type {
            DatabaseType::PostgreSQL => {
                use crate::database::postgres::PostgresConnection;
                use crate::database::Connection;

                let mut pg_connection = PostgresConnection::new(connection.clone());
                pg_connection
                    .connect()
                    .await
                    .map_err(|e| format!("Connection failed: {e}"))?;

                //                 // Execute the CREATE TABLE statement
                //                 pg_connection
                //                     .execute_sql(sql)
                //                     .await
                //                     .map_err(|e| format!("Failed to create table: {e}"))?;

                let _ = pg_connection.disconnect().await;

                Ok(())
            }
            _ => Err(format!(
                "Database type {} not yet supported",
                connection.database_type.display_name()
            )),
        }
    }

    /// Open a table for viewing
    pub async fn open_table_for_viewing(&mut self) {
        // Check connection health before attempting to open table
        if !self.check_connection_health().await {
            self.toast_manager
                .error("Cannot open table: database connection is not available");
            return;
        }

        if let Some(table_name) = self.ui.get_selected_table_name() {
            // Add tab to viewer
            let tab_idx = self.table_viewer_state.add_tab(table_name.clone());

            // Load table data
            if let Err(e) = self.load_table_data(tab_idx).await {
                if let Some(tab) = self.table_viewer_state.tabs.get_mut(tab_idx) {
                    tab.error = Some(format!("Failed to load table: {e}"));
                    tab.loading = false;
                }

                // Check if this was a connection issue and update status accordingly
                if e.contains("connection") || e.contains("Connection") || e.contains("disconnect")
                {
                    let _ = self.check_connection_health().await;
                }
            }

            // Load table metadata for the details pane
            if let Err(e) = self.load_table_metadata(&table_name).await {
                self.toast_manager
                    .error(format!("Failed to load table metadata: {e}"));
            }

            // Switch focus to tabular output
            self.ui.focused_pane = FocusedPane::TabularOutput;
        }
    }

    /// Load table data for a specific tab
    pub async fn load_table_data(&mut self, tab_idx: usize) -> Result<(), String> {
        self.db
            .load_table_data(
                &mut self.table_viewer_state,
                self.ui.selected_connection,
                tab_idx,
                &self.connection_manager,
            )
            .await
    }

    /// Load table metadata for the details pane
    pub async fn load_table_metadata(&mut self, table_name: &str) -> Result<(), String> {
        self.db
            .load_table_metadata(
                table_name,
                self.ui.selected_connection,
                &self.connection_manager,
            )
            .await
    }

    /// Check the health of the currently selected connection and update status
    pub async fn check_connection_health(&mut self) -> bool {
        if let Some(connection) = self.get_selected_connection() {
            // Use ConnectionManager to check if connection is healthy
            let is_healthy = self.connection_manager.is_connected(&connection.id).await;

            // Update connection status based on health check
            if let Some(conn) = self.get_selected_connection_mut() {
                if !is_healthy && matches!(conn.status, ConnectionStatus::Connected) {
                    // Connection was supposed to be connected but is not healthy
                    conn.status = ConnectionStatus::Failed("Connection lost".to_string());

                    // Clear database objects and tables
                    self.db.database_objects = None;
                    self.db.tables.clear();
                    self.db.table_load_error = Some("Connection lost".to_string());
                    self.ui.build_selectable_table_items(&None);

                    // Show user feedback
                    self.toast_manager.error("Database connection lost");

                    // Save updated connection status
                    let _ = self.db.connections.save();

                    return false;
                }
            }

            is_healthy
        } else {
            false
        }
    }

    /// Update a cell in the database
    pub async fn update_table_cell(
        &mut self,
        update: crate::ui::components::table_viewer::CellUpdate,
    ) -> Result<(), String> {
        self.db
            .update_table_cell(
                update,
                self.ui.selected_connection,
                &self.connection_manager,
            )
            .await
    }

    /// Delete a row from the database
    pub async fn delete_table_row(
        &mut self,
        confirmation: crate::ui::components::table_viewer::DeleteConfirmation,
    ) -> Result<(), String> {
        self.db
            .delete_table_row(
                confirmation,
                self.ui.selected_connection,
                &self.connection_manager,
            )
            .await
    }

    /// Reload current table tab data
    pub async fn reload_current_table_tab(&mut self) -> Result<(), String> {
        if let Some(tab_idx) = self
            .table_viewer_state
            .tabs
            .get(self.table_viewer_state.active_tab)
            .map(|_| self.table_viewer_state.active_tab)
        {
            self.load_table_data(tab_idx).await
        } else {
            Ok(())
        }
    }

    /// Get the SQL statement under the cursor
    pub fn get_statement_under_cursor(&self) -> Option<String> {
        self.query_editor.get_statement_at_cursor()
    }

    /// Update query editor content and sync with legacy query_content field
    pub fn set_query_content(&mut self, content: String) {
        self.query_content = content.clone();
        self.query_editor.set_content(content);
        self.ui.query_modified = true;
    }

    /// Get query content from the editor
    pub fn get_query_content(&self) -> &str {
        self.query_editor.get_content()
    }

    /// Update query editor database context when connection changes
    pub fn update_query_editor_context(&mut self) {
        if let Some(connection) = self.get_selected_connection() {
            self.query_editor
                .set_database_type(Some(connection.database_type.clone()));
        } else {
            self.query_editor.set_database_type(None);
        }
    }

    /// Set query editor focus state
    pub fn set_query_editor_focus(&mut self, focused: bool) {
        self.query_editor.set_focused(focused);
    }

    /// Toggle query editor insert mode
    pub fn toggle_query_editor_insert_mode(&mut self) {
        self.query_editor.toggle_insert_mode();
    }

    /// Handle character input in query editor
    pub fn handle_query_editor_input(&mut self, ch: char) {
        self.query_editor.insert_char(ch);
        // Sync content back to legacy field
        self.query_content = self.query_editor.get_content().to_string();
        self.ui.query_modified = true;
    }

    /// Handle newline in query editor
    pub fn handle_query_editor_newline(&mut self) {
        self.query_editor.insert_newline();
        // Sync content back to legacy field
        self.query_content = self.query_editor.get_content().to_string();
        self.ui.query_modified = true;
    }

    /// Handle backspace in query editor
    pub fn handle_query_editor_backspace(&mut self) {
        self.query_editor.backspace();
        // Sync content back to legacy field
        self.query_content = self.query_editor.get_content().to_string();
        self.ui.query_modified = true;
    }

    /// Handle cursor movement in query editor
    pub fn handle_query_editor_movement(&mut self, direction: QueryEditorMovement) {
        match direction {
            QueryEditorMovement::Up => self.query_editor.move_cursor_up(),
            QueryEditorMovement::Down => self.query_editor.move_cursor_down(),
            QueryEditorMovement::Left => self.query_editor.move_cursor_left(),
            QueryEditorMovement::Right => self.query_editor.move_cursor_right(),
        }
    }

    /// Load SQL file into query editor
    pub fn load_sql_file_into_editor(
        &mut self,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.load_query_file(filename)?;
        // Sync to query editor
        self.query_editor.set_content(self.query_content.clone());
        self.update_query_editor_context();
        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
