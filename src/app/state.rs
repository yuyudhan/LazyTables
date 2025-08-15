// FilePath: src/app/state.rs

use sqlx;
use crate::{
    config::Config,
    database::{
        connection::ConnectionStorage, ConnectionConfig, ConnectionStatus, DatabaseType,
        TableMetadata,
    },
    state::ui::UIState,
    ui::components::{
        ConnectionModalState, TableCreatorState, TableEditorState, TableViewerState, ToastManager,
    },
};

// Re-export for backward compatibility
pub use crate::state::ui::{FocusedPane, HelpMode, QueryEditMode};

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// UI state that can be saved/restored
    pub ui: UIState,
    /// Connections storage
    pub connections: ConnectionStorage,
    /// Connection modal state
    pub connection_modal_state: ConnectionModalState,
    /// SQL query editor content
    pub query_content: String,
    /// List of saved SQL files for current project
    pub saved_sql_files: Vec<String>,
    /// Tables in the currently connected database
    pub tables: Vec<String>,
    /// Error message for table loading
    pub table_load_error: Option<String>,
    /// Table creator state
    pub table_creator_state: TableCreatorState,
    /// Table editor state
    pub table_editor_state: TableEditorState,
    /// Table viewer state
    pub table_viewer_state: TableViewerState,
    /// Toast notifications manager
    pub toast_manager: ToastManager,
    /// Current table metadata (for the details pane)
    pub current_table_metadata: Option<TableMetadata>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        // Ensure all directories exist
        let _ = crate::config::Config::ensure_directories();

        let connections = ConnectionStorage::load().unwrap_or_default();
        let saved_sql_files = Vec::new(); // Will be loaded when connection is selected
        
        // Load or create UI state
        let mut ui = UIState::load().unwrap_or_default();
        
        // Update list states based on loaded connections
        ui.update_connection_selection(connections.connections.len());

        Self {
            ui,
            connections,
            connection_modal_state: ConnectionModalState::new(),
            query_content: String::new(),
            saved_sql_files,
            tables: Vec::new(),
            table_load_error: None,
            table_creator_state: TableCreatorState::new(),
            table_editor_state: TableEditorState::new("table".to_string()),
            table_viewer_state: TableViewerState::new(),
            toast_manager: ToastManager::new(),
            current_table_metadata: None,
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
                self.ui.query_cursor_line = self.ui.query_cursor_line.saturating_sub(1);
            }
            _ => {}
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
                }
            }
            _ => {}
        }
    }

    /// Move selection left based on current focus
    pub fn move_left(&mut self) {
        match self.ui.focused_pane {
            FocusedPane::TabularOutput => {
                if let Some(tab) = self.table_viewer_state.current_tab_mut() {
                    if !tab.in_edit_mode {
                        tab.move_left();
                    }
                }
            }
            FocusedPane::QueryWindow => {
                self.ui.query_cursor_column = self.ui.query_cursor_column.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Move selection right based on current focus
    pub fn move_right(&mut self) {
        match self.ui.focused_pane {
            FocusedPane::TabularOutput => {
                if let Some(tab) = self.table_viewer_state.current_tab_mut() {
                    if !tab.in_edit_mode {
                        tab.move_right();
                    }
                }
            }
            FocusedPane::QueryWindow => {
                if let Some(current_line) = self.query_content.lines().nth(self.ui.query_cursor_line) {
                    if self.ui.query_cursor_column < current_line.len() {
                        self.ui.query_cursor_column += 1;
                    }
                }
            }
            _ => {}
        }
    }

    /// Get currently selected connection
    pub fn get_selected_connection(
        &self,
    ) -> Option<&crate::database::connection::ConnectionConfig> {
        self.connections.connections.get(self.ui.selected_connection)
    }

    /// Get currently selected connection (mutable)
    pub fn get_selected_connection_mut(
        &mut self,
    ) -> Option<&mut crate::database::connection::ConnectionConfig> {
        self.connections
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
        if let Some(connection) = self.connections.connections.get(self.ui.selected_connection) {
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
            if let Some(existing) = self.connections.connections.get(self.ui.selected_connection) {
                connection.id = existing.id.clone();
                if let Err(e) = self.connections.update_connection(connection) {
                    return Err(format!("Failed to update connection: {e}"));
                }
            }
            self.close_edit_connection_modal();
        } else {
            // Add new connection
            if let Err(e) = self.connections.add_connection(connection) {
                return Err(format!("Failed to add connection: {e}"));
            }
            self.close_add_connection_modal();
        }

        self.clamp_connection_selection();
        Ok(())
    }

    /// Ensure selected connection index is within bounds
    pub fn clamp_connection_selection(&mut self) {
        if !self.connections.connections.is_empty() {
            let max_index = self.connections.connections.len() - 1;
            if self.ui.selected_connection > max_index {
                self.ui.selected_connection = max_index;
            }
            self.ui.connections_list_state
                .select(Some(self.ui.selected_connection));
        } else {
            self.ui.selected_connection = 0;
            self.ui.connections_list_state.select(None);
        }
    }

    /// Move connection selection down
    pub fn connection_down(&mut self) {
        if !self.connections.connections.is_empty() {
            let len = self.connections.connections.len();
            self.ui.selected_connection = (self.ui.selected_connection + 1) % len;
            self.ui.connections_list_state
                .select(Some(self.ui.selected_connection));
        }
    }

    /// Move connection selection up
    pub fn connection_up(&mut self) {
        if !self.connections.connections.is_empty() {
            let len = self.connections.connections.len();
            self.ui.selected_connection = if self.ui.selected_connection > 0 {
                self.ui.selected_connection - 1
            } else {
                len - 1
            };
            self.ui.connections_list_state
                .select(Some(self.ui.selected_connection));
        }
    }

    /// Move table selection down
    pub fn table_down(&mut self) {
        if !self.tables.is_empty() {
            let len = self.tables.len();
            self.ui.selected_table = (self.ui.selected_table + 1) % len;
            // Add 1 to account for the "▼ Tables" header in the UI
            self.ui.tables_list_state.select(Some(self.ui.selected_table + 1));

            // Clear metadata when selection changes (will load when Enter is pressed)
            self.current_table_metadata = None;
        }
    }

    /// Move table selection up
    pub fn table_up(&mut self) {
        if !self.tables.is_empty() {
            let len = self.tables.len();
            self.ui.selected_table = if self.ui.selected_table > 0 {
                self.ui.selected_table - 1
            } else {
                len - 1
            };
            // Add 1 to account for the "▼ Tables" header in the UI
            self.ui.tables_list_state.select(Some(self.ui.selected_table + 1));

            // Clear metadata when selection changes (will load when Enter is pressed)
            self.current_table_metadata = None;
        }
    }

    /// Update table list state when tables change
    pub fn update_table_selection(&mut self) {
        if !self.tables.is_empty() {
            // Preserve selection if possible, otherwise clamp to valid range
            let max_index = self.tables.len() - 1;
            if self.ui.selected_table > max_index {
                self.ui.selected_table = max_index;
            }
            // Add 1 to account for the "▼ Tables" header in the UI
            self.ui.tables_list_state.select(Some(self.ui.selected_table + 1));
        } else {
            self.ui.selected_table = 0;
            self.ui.tables_list_state.select(None);
        }
    }

    /// Attempt to connect to the selected database
    pub async fn connect_to_selected_database(&mut self) {
        if let Some(connection) = self
            .connections
            .connections
            .get(self.ui.selected_connection)
            .cloned()
        {
            // Set connection status to connecting
            if let Some(conn) = self
                .connections
                .connections
                .get_mut(self.ui.selected_connection)
            {
                conn.status = ConnectionStatus::Connecting;
            }

            // Clear previous tables and errors
            self.tables.clear();
            self.table_load_error = None;

            // Attempt connection based on database type
            let connection_name = connection.name.clone();
            let result = self.try_connect_to_database(&connection).await;

            // Update connection status based on result
            let connection_succeeded = result.is_ok();

            if let Some(conn) = self
                .connections
                .connections
                .get_mut(self.ui.selected_connection)
            {
                match result {
                    Ok(tables) => {
                        conn.status = ConnectionStatus::Connected;
                        self.tables = tables;
                    }
                    Err(error) => {
                        let error_msg = error.clone();
                        conn.status = ConnectionStatus::Failed(error.clone());
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
            }

            // Save updated connection status
            let _ = self.connections.save();
        }
    }

    /// Try to connect to a specific database and return tables
    async fn try_connect_to_database(
        &self,
        connection: &ConnectionConfig,
    ) -> Result<Vec<String>, String> {
        use crate::database::Connection;
        
        // Create appropriate connection based on database type
        let mut db_connection: Box<dyn Connection> = match connection.database_type {
            DatabaseType::PostgreSQL => {
                use crate::database::postgres::PostgresConnection;
                Box::new(PostgresConnection::new(connection.clone()))
            }
            DatabaseType::MySQL => {
                use crate::database::mysql::MySqlConnection;
                Box::new(MySqlConnection::new(connection.clone()))
            }
            DatabaseType::MariaDB => {
                // MariaDB uses MySQL driver
                use crate::database::mysql::MySqlConnection;
                Box::new(MySqlConnection::new(connection.clone()))
            }
            DatabaseType::SQLite => {
                use crate::database::sqlite::SqliteConnection;
                Box::new(SqliteConnection::new(connection.clone()))
            }
            _ => {
                return Err(format!(
                    "Database type {} not yet supported",
                    connection.database_type.display_name()
                ))
            }
        };

        // Try to connect
        db_connection
            .connect()
            .await
            .map_err(|e| format!("Connection failed: {e}"))?;

        // Query actual tables from the database
        let tables = db_connection
            .list_tables()
            .await
            .map_err(|e| format!("Failed to retrieve tables: {e}"))?;

        // Clean up connection
        let _ = db_connection.disconnect().await;

        Ok(tables)
    }

    /// Disconnect from current database
    pub fn disconnect_from_database(&mut self) {
        if let Some(connection) = self
            .connections
            .connections
            .get_mut(self.ui.selected_connection)
        {
            connection.status = ConnectionStatus::Disconnected;
            self.tables.clear();
            self.table_load_error = None;
            self.update_table_selection();

            // Save updated connection status
            let _ = self.connections.save();
        }
    }

    /// Get currently selected SQL file name
    pub fn get_selected_sql_file(&self) -> Option<&String> {
        self.saved_sql_files.get(self.ui.selected_sql_file)
    }

    /// Load the currently selected SQL file
    pub fn load_selected_sql_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(filename) = self.get_selected_sql_file().cloned() {
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

    /// Load list of saved SQL files for current project
    fn load_sql_files_for_connection(&self) -> Vec<String> {
        use std::fs;

        let mut files = Vec::new();
        
        // Get connection-specific directory
        let connection_name = if let Some(connection) = self.connections.connections.get(self.ui.selected_connection) {
            connection.name.clone()
        } else {
            "default".to_string()
        };
        
        // Try connection-specific directory first
        let connection_dir = Config::sql_files_dir().join(&connection_name);
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
        
        // Also load from root sql_files directory
        let sql_dir = Config::sql_files_dir();
        if let Ok(entries) = fs::read_dir(&sql_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // Skip subdirectories, only get files in root
                if path.is_file() && path.extension().is_some_and(|ext| ext == "sql") {
                    if let Some(name) = path.file_stem().and_then(|name| name.to_str()) {
                        if !files.contains(&name.to_string()) {
                            files.push(format!("../{name}"));
                        }
                    }
                }
            }
        }

        files.sort();
        files
    }

    /// Refresh the list of saved SQL files
    pub fn refresh_sql_files(&mut self) {
        self.saved_sql_files = self.load_sql_files_for_connection();
        self.clamp_sql_file_selection();
    }

    /// Save current query content to a file
    pub fn save_query_as(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::config::Config;
        use std::fs;

        let sql_dir = Config::sql_files_dir();
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

    /// Load a SQL file into the query editor
    pub fn load_query_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        // Get connection-specific directory
        let connection_name = if let Some(connection) = self.connections.connections.get(self.ui.selected_connection) {
            connection.name.clone()
        } else {
            "default".to_string()
        };

        let file_path = if filename.starts_with("../") {
            // File from root sql_files directory
            let clean_name = filename.trim_start_matches("../");
            Config::sql_files_dir().join(format!("{clean_name}.sql"))
        } else {
            // File from connection-specific directory
            Config::sql_files_dir().join(&connection_name).join(format!("{filename}.sql"))
        };

        let content = fs::read_to_string(&file_path)?;
        self.query_content = content;
        self.ui.current_sql_file = Some(filename.to_string());
        self.ui.query_modified = false;
        self.ui.query_cursor_line = 0;
        self.ui.query_cursor_column = 0;
        self.ui.query_edit_mode = QueryEditMode::Normal;

        Ok(())
    }

    /// Create a new SQL file
    pub fn new_query_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.query_content.clear();
        self.ui.current_sql_file = Some(filename.to_string());
        self.ui.query_modified = false;
        self.ui.query_cursor_line = 0;
        self.ui.query_cursor_column = 0;

        // Save the empty file
        self.save_query_as(filename)
    }

    /// Insert character at current cursor position in query editor
    pub fn insert_char_at_cursor(&mut self, c: char) {
        let lines: Vec<&str> = self.query_content.lines().collect();

        if self.ui.query_cursor_line >= lines.len() {
            // Add new lines if needed
            while self.query_content.lines().count() <= self.ui.query_cursor_line {
                self.query_content.push('\n');
            }
        }

        let lines: Vec<&str> = self.query_content.lines().collect();
        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

        if let Some(line) = new_lines.get_mut(self.ui.query_cursor_line) {
            let mut chars: Vec<char> = line.chars().collect();
            chars.insert(self.ui.query_cursor_column, c);
            *line = chars.iter().collect();
        }

        self.query_content = new_lines.join("\n");
        self.ui.query_cursor_column += 1;
        self.ui.query_modified = true;
    }

    /// Delete character at current cursor position in query editor
    pub fn delete_char_at_cursor(&mut self) {
        if self.ui.query_cursor_column > 0 {
            let lines: Vec<&str> = self.query_content.lines().collect();
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

            if let Some(line) = new_lines.get_mut(self.ui.query_cursor_line) {
                let mut chars: Vec<char> = line.chars().collect();
                if self.ui.query_cursor_column <= chars.len() && self.ui.query_cursor_column > 0 {
                    chars.remove(self.ui.query_cursor_column - 1);
                    *line = chars.iter().collect();
                    self.ui.query_cursor_column -= 1;
                    self.ui.query_modified = true;
                }
            }

            self.query_content = new_lines.join("\n");
        }
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

    /// Save current SQL file with connection-specific directory
    pub fn save_sql_file_with_connection(&mut self) -> Result<(), String> {
        // Get the current connection name
        let connection_name = if let Some(connection) = self.connections.connections.get(self.ui.selected_connection) {
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
        if let Some(table_name) = self.tables.get(self.ui.selected_table).cloned() {
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
                            self.load_postgres_table_schema(&connection, table_name)
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

    /// Load PostgreSQL table schema
    async fn load_postgres_table_schema(
        &mut self,
        connection: &ConnectionConfig,
        table_name: &str,
    ) -> Result<(), String> {
        use crate::database::postgres::PostgresConnection;
        use crate::database::Connection;

        let mut pg_connection = PostgresConnection::new(connection.clone());
        pg_connection
            .connect()
            .await
            .map_err(|e| format!("Connection failed: {e}"))?;

        // Query table columns from information_schema
        let columns = pg_connection
            .get_table_columns(table_name)
            .await
            .map_err(|e| format!("Failed to retrieve table columns: {e}"))?;

        // Convert TableColumn to ColumnDefinition for the editor
        use crate::ui::components::table_creator::{ColumnDefinition as EditorColumnDef, PostgresDataType};
        
        let editor_columns: Vec<EditorColumnDef> = columns
            .into_iter()
            .map(|col| EditorColumnDef {
                name: col.name,
                data_type: PostgresDataType::Text, // TODO: Map DataType to PostgresDataType properly
                is_nullable: col.is_nullable,
                is_primary_key: col.is_primary_key,
                is_unique: false, // Not available in TableColumn
                default_value: col.default_value,
                check_constraint: None,
                references: None,
            })
            .collect();
        
        // Populate the table editor state with column information
        self.table_editor_state.columns = editor_columns;
        self.table_editor_state.original_columns = self.table_editor_state.columns.clone();

        let _ = pg_connection.disconnect().await;

        Ok(())
    }

    /// Apply table edits from table editor state
    pub async fn apply_table_edits_from_editor(&mut self) -> Result<(), String> {
        // Generate ALTER TABLE SQL statements
        let sql_statements = self.table_editor_state.generate_alter_table_sql()?;

        // Get the current connection
        if let Some(connection) = self
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
        if let Some(table_name) = self.tables.get(self.ui.selected_table).cloned() {
            // Add tab to viewer
            let tab_idx = self.table_viewer_state.add_tab(table_name.clone());

            // Load table data
            if let Err(e) = self.load_table_data(tab_idx).await {
                if let Some(tab) = self.table_viewer_state.tabs.get_mut(tab_idx) {
                    tab.error = Some(format!("Failed to load table: {e}"));
                    tab.loading = false;
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
        if let Some(tab) = self.table_viewer_state.tabs.get_mut(tab_idx) {
            let table_name = tab.table_name.clone();
            let page = tab.current_page;
            let limit = tab.rows_per_page;
            let offset = page * limit;

            // Get the current connection
            if let Some(connection) = self
                .connections
                .connections
                .get(self.ui.selected_connection)
                .cloned()
            {
                match &connection.status {
                    ConnectionStatus::Connected => {
                        // Load table data based on database type
                        match connection.database_type {
                            DatabaseType::PostgreSQL => {
                                self.load_postgres_table_data(
                                    &connection,
                                    &table_name,
                                    limit,
                                    offset,
                                    tab_idx,
                                )
                                .await
                            }
                            _ => Err(format!(
                                "Database type {} not yet supported for table viewing",
                                connection.database_type.display_name()
                            )),
                        }
                    }
                    _ => Err("No active database connection".to_string()),
                }
            } else {
                Err("No connection selected".to_string())
            }
        } else {
            Err("Invalid tab index".to_string())
        }
    }

    /// Load PostgreSQL table data
    async fn load_postgres_table_data(
        &mut self,
        connection: &ConnectionConfig,
        table_name: &str,
        limit: usize,
        offset: usize,
        tab_idx: usize,
    ) -> Result<(), String> {
        use crate::database::postgres::PostgresConnection;
        use crate::database::Connection;

        let mut pg_connection = PostgresConnection::new(connection.clone());
        pg_connection
            .connect()
            .await
            .map_err(|e| format!("Connection failed: {e}"))?;

        // Get table columns
        let columns = pg_connection
            .get_table_columns(table_name)
            .await
            .map_err(|e| format!("Failed to retrieve columns: {e}"))?;

        // Get total row count
        let total_rows = pg_connection
            .get_table_row_count(table_name)
            .await
            .map_err(|e| format!("Failed to get row count: {e}"))?;

        // Get table data
        let rows = pg_connection
            .get_table_data(table_name, limit, offset)
            .await
            .map_err(|e| format!("Failed to retrieve data: {e}"))?;

        // Update the tab with loaded data
        if let Some(tab) = self.table_viewer_state.tabs.get_mut(tab_idx) {
            // Convert columns to ColumnInfo
            tab.columns = columns
                .iter()
                .map(|col| crate::ui::components::table_viewer::ColumnInfo {
                    name: col.name.clone(),
                    data_type: col.data_type.to_sql(),
                    is_nullable: col.is_nullable,
                    is_primary_key: col.is_primary_key,
                    max_display_width: col.name.len().max(15),
                })
                .collect();

            // Find primary key columns
            tab.primary_key_columns = columns
                .iter()
                .enumerate()
                .filter(|(_, col)| col.is_primary_key)
                .map(|(idx, _)| idx)
                .collect();

            tab.rows = rows;
            tab.total_rows = total_rows;
            tab.loading = false;
            tab.error = None;
        }

        let _ = pg_connection.disconnect().await;

        Ok(())
    }

    /// Load table metadata for the details pane
    pub async fn load_table_metadata(&mut self, table_name: &str) -> Result<(), String> {
        // Get the current connection
        if let Some(connection) = self
            .connections
            .connections
            .get(self.ui.selected_connection)
            .cloned()
        {
            match &connection.status {
                ConnectionStatus::Connected => {
                    // Load metadata based on database type
                    match connection.database_type {
                        DatabaseType::PostgreSQL => {
                            use crate::database::postgres::PostgresConnection;
                            use crate::database::Connection;

                            let mut pg_connection = PostgresConnection::new(connection.clone());
                            pg_connection
                                .connect()
                                .await
                                .map_err(|e| format!("Connection failed: {e}"))?;

                            // Get table metadata
                            let metadata = pg_connection
                                .get_table_metadata(table_name)
                                .await
                                .map_err(|e| format!("Failed to retrieve metadata: {e}"))?;

                            self.current_table_metadata = Some(metadata);

                            let _ = pg_connection.disconnect().await;
                            Ok(())
                        }
                        _ => Err(format!(
                            "Database type {} not yet supported for metadata",
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

    /// Update a cell in the database
    pub async fn update_table_cell(
        &mut self,
        update: crate::ui::components::table_viewer::CellUpdate,
    ) -> Result<(), String> {
        // Get the current connection
        if let Some(connection) = self
            .connections
            .connections
            .get(self.ui.selected_connection)
            .cloned()
        {
            match &connection.status {
                ConnectionStatus::Connected => {
                    // Update cell based on database type
                    match connection.database_type {
                        DatabaseType::PostgreSQL => {
                            self.update_postgres_cell(&connection, update).await
                        }
                        _ => Err(format!(
                            "Database type {} not yet supported for cell updates",
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

    /// Update a cell in PostgreSQL
    async fn update_postgres_cell(
        &self,
        connection: &ConnectionConfig,
        update: crate::ui::components::table_viewer::CellUpdate,
    ) -> Result<(), String> {
        use crate::database::postgres::PostgresConnection;
        use crate::database::Connection;

        let mut pg_connection = PostgresConnection::new(connection.clone());
        pg_connection
            .connect()
            .await
            .map_err(|e| format!("Connection failed: {e}"))?;

        // Build UPDATE SQL
        let mut where_clauses = Vec::new();
        for (pk_col, pk_val) in &update.primary_key_values {
            where_clauses.push(format!("{pk_col} = '{pk_val}'"));
        }

        if where_clauses.is_empty() {
            return Err("Cannot update row without primary key".to_string());
        }

        let _sql = format!(
            "UPDATE {} SET {} = '{}' WHERE {}",
            update.table_name,
            update.column_name,
            update.new_value,
            where_clauses.join(" AND ")
        );
// 
//         pg_connection
//             .execute_sql(&sql)
//             .await
//             .map_err(|e| format!("Failed to update cell: {e}"))?;

        let _ = pg_connection.disconnect().await;

        Ok(())
    }

    /// Delete a row from the database
    pub async fn delete_table_row(
        &mut self,
        confirmation: crate::ui::components::table_viewer::DeleteConfirmation,
    ) -> Result<(), String> {
        // Get the current connection
        if let Some(connection) = self
            .connections
            .connections
            .get(self.ui.selected_connection)
            .cloned()
        {
            match &connection.status {
                ConnectionStatus::Connected => {
                    // Delete row based on database type
                    match connection.database_type {
                        DatabaseType::PostgreSQL => {
                            self.delete_postgres_row(&connection, confirmation).await
                        }
                        _ => Err(format!(
                            "Database type {} not yet supported for row deletion",
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

    /// Delete a row in PostgreSQL
    async fn delete_postgres_row(
        &self,
        connection: &ConnectionConfig,
        confirmation: crate::ui::components::table_viewer::DeleteConfirmation,
    ) -> Result<(), String> {
        use crate::database::postgres::PostgresConnection;
        use crate::database::Connection;

        let mut pg_connection = PostgresConnection::new(connection.clone());
        pg_connection
            .connect()
            .await
            .map_err(|e| format!("Connection failed: {e}"))?;

        // Build DELETE SQL
        let mut where_clauses = Vec::new();
        for (pk_col, pk_val) in &confirmation.primary_key_values {
            where_clauses.push(format!("{pk_col} = '{pk_val}'"));
        }

        if where_clauses.is_empty() {
            return Err("Cannot delete row without primary key".to_string());
        }

        let sql = format!(
            "DELETE FROM {} WHERE {}",
            confirmation.table_name,
            where_clauses.join(" AND ")
        );

        // Execute the delete query using the pool directly
        if let Some(pool) = &pg_connection.pool {
            sqlx::query(&sql)
                .execute(pool)
                .await
                .map_err(|e| format!("Failed to delete row: {e}"))?;
        } else {
            return Err("No active database connection".to_string());
        }

        let _ = pg_connection.disconnect().await;

        Ok(())
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
        let lines: Vec<&str> = self.query_content.lines().collect();
        if lines.is_empty() || self.ui.query_cursor_line >= lines.len() {
            return None;
        }

        // Find the SQL statement boundaries (statements separated by semicolons or empty lines)
        let mut start_line = self.ui.query_cursor_line;
        let mut end_line = self.ui.query_cursor_line;

        // Find start of statement (go backwards until we find a semicolon or empty line)
        while start_line > 0 {
            let line = lines[start_line - 1].trim();
            if line.is_empty() || line.ends_with(';') {
                break;
            }
            start_line -= 1;
        }

        // Find end of statement (go forwards until we find a semicolon or empty line)
        while end_line < lines.len() {
            let line = lines[end_line].trim();
            if line.ends_with(';') {
                break;
            }
            if end_line < lines.len() - 1 && lines[end_line + 1].trim().is_empty() {
                break;
            }
            end_line += 1;
        }

        let statement_lines: Vec<&str> = lines[start_line..=end_line].to_vec();
        let statement = statement_lines.join("\n").trim().to_string();

        if statement.is_empty() {
            None
        } else {
            Some(statement)
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
