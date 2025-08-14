// FilePath: src/app/state.rs

use crate::{database::{connection::ConnectionStorage, ConnectionConfig, ConnectionStatus, DatabaseType}, ui::components::ConnectionModalState};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

/// Application modes (vim-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    /// Normal mode - navigation and commands
    Normal,
    /// Insert mode - editing data
    Insert,
    /// Visual mode - selection
    Visual,
    /// Command mode - executing commands
    Command,
    /// Query mode - SQL editor
    Query,
}

/// Which pane currently has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusedPane {
    /// Connections list pane
    Connections,
    /// Tables/Views list pane
    Tables,
    /// Table details pane
    Details,
    /// Tabular output area
    TabularOutput,
    /// SQL file browser
    SqlFiles,
    /// Query window (SQL editor)
    QueryWindow,
}

impl FocusedPane {
    /// Get the next pane in clockwise order
    pub fn next(&self) -> Self {
        match self {
            Self::Connections => Self::Tables,
            Self::Tables => Self::Details,
            Self::Details => Self::TabularOutput,
            Self::TabularOutput => Self::SqlFiles,
            Self::SqlFiles => Self::QueryWindow,
            Self::QueryWindow => Self::Connections,
        }
    }

    /// Get the previous pane in counter-clockwise order
    pub fn previous(&self) -> Self {
        match self {
            Self::Connections => Self::QueryWindow,
            Self::Tables => Self::Connections,
            Self::Details => Self::Tables,
            Self::TabularOutput => Self::Details,
            Self::SqlFiles => Self::TabularOutput,
            Self::QueryWindow => Self::SqlFiles,
        }
    }
}

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current mode
    pub mode: Mode,
    /// Currently focused pane
    pub focused_pane: FocusedPane,
    /// Selected connection index
    pub selected_connection: usize,
    /// Selected table index
    pub selected_table: usize,
    /// Current row in main content
    pub current_row: usize,
    /// Current column in main content
    pub current_column: usize,
    /// Command buffer for command mode
    pub command_buffer: String,
    /// Search query
    pub search_query: String,
    /// Is search active
    pub search_active: bool,
    /// Show help overlay
    pub show_help: bool,
    /// Leader key (Space) was pressed
    pub leader_pressed: bool,
    /// Connections storage
    pub connections: ConnectionStorage,
    /// Connections list state for UI selection
    pub connections_list_state: ListState,
    /// Tables list state for UI selection
    pub tables_list_state: ListState,
    /// Show connection creation modal
    pub show_add_connection_modal: bool,
    /// Show connection edit modal
    pub show_edit_connection_modal: bool,
    /// Connection modal state
    pub connection_modal_state: ConnectionModalState,
    /// SQL query editor content
    pub query_content: String,
    /// Current cursor position in query editor
    pub query_cursor_line: usize,
    pub query_cursor_column: usize,
    /// List of saved SQL files for current project
    pub saved_sql_files: Vec<String>,
    /// Selected SQL file index in the browser
    pub selected_sql_file: usize,
    /// Currently loaded SQL file path
    pub current_sql_file: Option<String>,
    /// Whether query content has been modified
    pub query_modified: bool,
    /// Last focused left column pane (for smarter navigation)
    pub last_left_pane: FocusedPane,
    /// Tables in the currently connected database
    pub tables: Vec<String>,
    /// Error message for table loading
    pub table_load_error: Option<String>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        // Ensure all directories exist
        let _ = crate::config::Config::ensure_directories();

        let connections = ConnectionStorage::load().unwrap_or_default();
        let saved_sql_files = Self::load_sql_files();
        
        // Initialize connections list state
        let mut connections_list_state = ListState::default();
        if !connections.connections.is_empty() {
            connections_list_state.select(Some(0));
        }
        
        // Initialize tables list state
        let tables_list_state = ListState::default();

        Self {
            mode: Mode::Normal,
            focused_pane: FocusedPane::Connections,
            selected_connection: 0,
            selected_table: 0,
            current_row: 0,
            current_column: 0,
            command_buffer: String::new(),
            search_query: String::new(),
            search_active: false,
            show_help: false,
            leader_pressed: false,
            connections,
            connections_list_state,
            tables_list_state,
            show_add_connection_modal: false,
            show_edit_connection_modal: false,
            connection_modal_state: ConnectionModalState::new(),
            query_content: String::new(),
            query_cursor_line: 0,
            query_cursor_column: 0,
            saved_sql_files,
            selected_sql_file: 0,
            current_sql_file: None,
            query_modified: false,
            last_left_pane: FocusedPane::Connections,
            tables: Vec::new(),
            table_load_error: None,
        }
    }

    /// Cycle focus to the next pane
    pub fn cycle_focus_forward(&mut self) {
        let new_pane = self.focused_pane.next();
        self.update_focus(new_pane);
    }

    /// Cycle focus to the previous pane
    pub fn cycle_focus_backward(&mut self) {
        let new_pane = self.focused_pane.previous();
        self.update_focus(new_pane);
    }

    /// Move focus left (Ctrl+h)
    pub fn move_focus_left(&mut self) {
        let new_pane = match self.focused_pane {
            FocusedPane::TabularOutput => {
                // Smart selection: go to the last focused left pane, defaulting to middle (Tables)
                match self.last_left_pane {
                    FocusedPane::Connections | FocusedPane::Tables | FocusedPane::Details => {
                        self.last_left_pane
                    }
                    _ => FocusedPane::Tables, // Default to middle pane
                }
            }
            FocusedPane::QueryWindow => FocusedPane::Details,
            FocusedPane::SqlFiles => FocusedPane::QueryWindow,
            // Left column panes don't have anything to the left
            _ => self.focused_pane,
        };

        self.update_focus(new_pane);
    }

    /// Move focus down (Ctrl+j)
    pub fn move_focus_down(&mut self) {
        let new_pane = match self.focused_pane {
            FocusedPane::Connections => FocusedPane::Tables,
            FocusedPane::Tables => FocusedPane::Details,
            FocusedPane::TabularOutput => FocusedPane::QueryWindow,
            // Bottom panes don't have anything below
            _ => self.focused_pane,
        };

        self.update_focus(new_pane);
    }

    /// Move focus up (Ctrl+k)
    pub fn move_focus_up(&mut self) {
        let new_pane = match self.focused_pane {
            FocusedPane::Tables => FocusedPane::Connections,
            FocusedPane::Details => FocusedPane::Tables,
            FocusedPane::QueryWindow => FocusedPane::TabularOutput,
            FocusedPane::SqlFiles => FocusedPane::TabularOutput,
            // Top panes don't have anything above
            _ => self.focused_pane,
        };

        self.update_focus(new_pane);
    }

    /// Move focus right (Ctrl+l)
    pub fn move_focus_right(&mut self) {
        let new_pane = match self.focused_pane {
            FocusedPane::Connections => FocusedPane::TabularOutput,
            FocusedPane::Tables => FocusedPane::TabularOutput,
            FocusedPane::Details => FocusedPane::QueryWindow,
            FocusedPane::QueryWindow => FocusedPane::SqlFiles,
            // Right column panes don't have anything to the right
            _ => self.focused_pane,
        };

        self.update_focus(new_pane);
    }

    /// Update focus and track left pane usage for smart navigation
    fn update_focus(&mut self, new_pane: FocusedPane) {
        // Track the last focused left column pane for smart navigation
        if matches!(
            self.focused_pane,
            FocusedPane::Connections | FocusedPane::Tables | FocusedPane::Details
        ) {
            self.last_left_pane = self.focused_pane;
        }

        self.focused_pane = new_pane;
    }

    /// Move selection up based on current focus
    pub fn move_up(&mut self) {
        match self.focused_pane {
            FocusedPane::Connections => {
                self.connection_up();
            }
            FocusedPane::Tables => {
                self.table_up();
            }
            FocusedPane::TabularOutput => {
                self.current_row = self.current_row.saturating_sub(1);
            }
            FocusedPane::SqlFiles => {
                self.selected_sql_file = self.selected_sql_file.saturating_sub(1);
            }
            FocusedPane::QueryWindow => {
                self.query_cursor_line = self.query_cursor_line.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Move selection down based on current focus
    pub fn move_down(&mut self) {
        match self.focused_pane {
            FocusedPane::Connections => {
                self.connection_down();
            }
            FocusedPane::Tables => {
                self.table_down();
            }
            FocusedPane::TabularOutput => {
                self.current_row += 1;
            }
            FocusedPane::SqlFiles => {
                let max_files = self.saved_sql_files.len().saturating_sub(1);
                if self.selected_sql_file < max_files {
                    self.selected_sql_file += 1;
                }
            }
            FocusedPane::QueryWindow => {
                let lines = self.query_content.lines().count();
                if self.query_cursor_line < lines.saturating_sub(1) {
                    self.query_cursor_line += 1;
                }
            }
            _ => {}
        }
    }

    /// Move selection left based on current focus
    pub fn move_left(&mut self) {
        match self.focused_pane {
            FocusedPane::TabularOutput => {
                self.current_column = self.current_column.saturating_sub(1);
            }
            FocusedPane::QueryWindow => {
                self.query_cursor_column = self.query_cursor_column.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Move selection right based on current focus
    pub fn move_right(&mut self) {
        match self.focused_pane {
            FocusedPane::TabularOutput => {
                self.current_column += 1;
            }
            FocusedPane::QueryWindow => {
                if let Some(current_line) = self.query_content.lines().nth(self.query_cursor_line) {
                    if self.query_cursor_column < current_line.len() {
                        self.query_cursor_column += 1;
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
        self.connections.connections.get(self.selected_connection)
    }

    /// Get currently selected connection (mutable)
    pub fn get_selected_connection_mut(
        &mut self,
    ) -> Option<&mut crate::database::connection::ConnectionConfig> {
        self.connections
            .connections
            .get_mut(self.selected_connection)
    }

    /// Open the add connection modal
    pub fn open_add_connection_modal(&mut self) {
        self.show_add_connection_modal = true;
        self.connection_modal_state = ConnectionModalState::new(); // Reset state
    }

    /// Close the add connection modal
    pub fn close_add_connection_modal(&mut self) {
        self.show_add_connection_modal = false;
        self.connection_modal_state.clear(); // Clear any input
    }

    /// Open the edit connection modal for the currently selected connection
    pub fn open_edit_connection_modal(&mut self) {
        if let Some(connection) = self.connections.connections.get(self.selected_connection) {
            self.connection_modal_state.populate_from_connection(connection);
            self.show_edit_connection_modal = true;
        }
    }

    /// Close the edit connection modal
    pub fn close_edit_connection_modal(&mut self) {
        self.show_edit_connection_modal = false;
        self.connection_modal_state.clear(); // Clear any input
    }

    /// Save connection from modal
    pub fn save_connection_from_modal(&mut self) -> Result<(), String> {
        let mut connection = self.connection_modal_state.try_create_connection()?;
        
        if self.show_edit_connection_modal {
            // Update existing connection - preserve ID
            if let Some(existing) = self.connections.connections.get(self.selected_connection) {
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
            if self.selected_connection > max_index {
                self.selected_connection = max_index;
            }
            self.connections_list_state.select(Some(self.selected_connection));
        } else {
            self.selected_connection = 0;
            self.connections_list_state.select(None);
        }
    }

    /// Move connection selection down
    pub fn connection_down(&mut self) {
        if !self.connections.connections.is_empty() {
            let len = self.connections.connections.len();
            self.selected_connection = (self.selected_connection + 1) % len;
            self.connections_list_state.select(Some(self.selected_connection));
        }
    }

    /// Move connection selection up
    pub fn connection_up(&mut self) {
        if !self.connections.connections.is_empty() {
            let len = self.connections.connections.len();
            self.selected_connection = if self.selected_connection > 0 {
                self.selected_connection - 1
            } else {
                len - 1
            };
            self.connections_list_state.select(Some(self.selected_connection));
        }
    }

    /// Move table selection down
    pub fn table_down(&mut self) {
        if !self.tables.is_empty() {
            let len = self.tables.len();
            self.selected_table = (self.selected_table + 1) % len;
            self.tables_list_state.select(Some(self.selected_table));
        }
    }

    /// Move table selection up
    pub fn table_up(&mut self) {
        if !self.tables.is_empty() {
            let len = self.tables.len();
            self.selected_table = if self.selected_table > 0 {
                self.selected_table - 1
            } else {
                len - 1
            };
            self.tables_list_state.select(Some(self.selected_table));
        }
    }

    /// Update table list state when tables change
    pub fn update_table_selection(&mut self) {
        if !self.tables.is_empty() {
            self.selected_table = 0;
            self.tables_list_state.select(Some(0));
        } else {
            self.selected_table = 0;
            self.tables_list_state.select(None);
        }
    }

    /// Attempt to connect to the selected database
    pub async fn connect_to_selected_database(&mut self) {
        if let Some(connection) = self.connections.connections.get(self.selected_connection).cloned() {
            // Set connection status to connecting
            if let Some(conn) = self.connections.connections.get_mut(self.selected_connection) {
                conn.status = ConnectionStatus::Connecting;
            }
            
            // Clear previous tables and errors
            self.tables.clear();
            self.table_load_error = None;
            
            // Attempt connection based on database type
            let result = self.try_connect_to_database(&connection).await;
            
            // Update connection status based on result
            if let Some(conn) = self.connections.connections.get_mut(self.selected_connection) {
                match result {
                    Ok(tables) => {
                        conn.status = ConnectionStatus::Connected;
                        self.tables = tables;
                        self.update_table_selection();
                    }
                    Err(error) => {
                        conn.status = ConnectionStatus::Failed(error);
                    }
                }
            }
            
            // Save updated connection status
            let _ = self.connections.save();
        }
    }

    /// Try to connect to a specific database and return tables
    async fn try_connect_to_database(&self, connection: &ConnectionConfig) -> Result<Vec<String>, String> {
        match connection.database_type {
            DatabaseType::PostgreSQL => {
                self.connect_postgresql(connection).await
            }
            _ => {
                Err(format!("Database type {} not yet supported", connection.database_type.display_name()))
            }
        }
    }

    /// Connect to PostgreSQL and retrieve table list
    async fn connect_postgresql(&self, connection: &ConnectionConfig) -> Result<Vec<String>, String> {
        use crate::database::postgres::PostgresConnection;
        use crate::database::Connection;
        
        // Create connection config
        let mut pg_connection = PostgresConnection::new(connection.clone());
        
        // Try to connect
        pg_connection.connect().await.map_err(|e| format!("Connection failed: {e}"))?;
        
        // Query actual tables from the database
        let tables = pg_connection.get_tables().await
            .map_err(|e| format!("Failed to retrieve tables: {e}"))?;
        
        // Clean up connection
        let _ = pg_connection.disconnect().await;
        
        Ok(tables)
    }

    /// Disconnect from current database
    pub fn disconnect_from_database(&mut self) {
        if let Some(connection) = self.connections.connections.get_mut(self.selected_connection) {
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
        self.saved_sql_files.get(self.selected_sql_file)
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
            if self.selected_sql_file > max_index {
                self.selected_sql_file = max_index;
            }
        } else {
            self.selected_sql_file = 0;
        }
    }

    /// Load list of saved SQL files for current project
    fn load_sql_files() -> Vec<String> {
        use crate::config::Config;
        use std::fs;

        let sql_dir = Config::sql_files_dir();
        if let Ok(entries) = fs::read_dir(sql_dir) {
            entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "sql") {
                        path.file_stem()
                            .and_then(|name| name.to_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Refresh the list of saved SQL files
    pub fn refresh_sql_files(&mut self) {
        self.saved_sql_files = Self::load_sql_files();
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

        self.current_sql_file = Some(filename.to_string());
        self.query_modified = false;
        self.refresh_sql_files();

        Ok(())
    }

    /// Save current query content to the currently loaded file
    pub fn save_query(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(filename) = &self.current_sql_file.clone() {
            self.save_query_as(filename)
        } else {
            Err("No file is currently loaded".into())
        }
    }

    /// Load a SQL file into the query editor
    pub fn load_query_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        use crate::config::Config;
        use std::fs;

        let sql_dir = Config::sql_files_dir();
        let file_path = sql_dir.join(format!("{filename}.sql"));

        let content = fs::read_to_string(&file_path)?;
        self.query_content = content;
        self.current_sql_file = Some(filename.to_string());
        self.query_modified = false;
        self.query_cursor_line = 0;
        self.query_cursor_column = 0;

        Ok(())
    }

    /// Create a new SQL file
    pub fn new_query_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.query_content.clear();
        self.current_sql_file = Some(filename.to_string());
        self.query_modified = false;
        self.query_cursor_line = 0;
        self.query_cursor_column = 0;

        // Save the empty file
        self.save_query_as(filename)
    }

    /// Insert character at current cursor position in query editor
    pub fn insert_char_at_cursor(&mut self, c: char) {
        let lines: Vec<&str> = self.query_content.lines().collect();

        if self.query_cursor_line >= lines.len() {
            // Add new lines if needed
            while self.query_content.lines().count() <= self.query_cursor_line {
                self.query_content.push('\n');
            }
        }

        let lines: Vec<&str> = self.query_content.lines().collect();
        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

        if let Some(line) = new_lines.get_mut(self.query_cursor_line) {
            let mut chars: Vec<char> = line.chars().collect();
            chars.insert(self.query_cursor_column, c);
            *line = chars.iter().collect();
        }

        self.query_content = new_lines.join("\n");
        self.query_cursor_column += 1;
        self.query_modified = true;
    }

    /// Delete character at current cursor position in query editor
    pub fn delete_char_at_cursor(&mut self) {
        if self.query_cursor_column > 0 {
            let lines: Vec<&str> = self.query_content.lines().collect();
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

            if let Some(line) = new_lines.get_mut(self.query_cursor_line) {
                let mut chars: Vec<char> = line.chars().collect();
                if self.query_cursor_column <= chars.len() && self.query_cursor_column > 0 {
                    chars.remove(self.query_cursor_column - 1);
                    *line = chars.iter().collect();
                    self.query_cursor_column -= 1;
                    self.query_modified = true;
                }
            }

            self.query_content = new_lines.join("\n");
        }
    }

    /// Get the SQL statement under the cursor
    pub fn get_statement_under_cursor(&self) -> Option<String> {
        let lines: Vec<&str> = self.query_content.lines().collect();
        if lines.is_empty() || self.query_cursor_line >= lines.len() {
            return None;
        }

        // Find the SQL statement boundaries (statements separated by semicolons or empty lines)
        let mut start_line = self.query_cursor_line;
        let mut end_line = self.query_cursor_line;

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
