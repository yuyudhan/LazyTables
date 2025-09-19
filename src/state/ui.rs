// FilePath: src/state/ui.rs

use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

/// Help display mode for context-aware help
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelpMode {
    /// No help displayed
    None,
    /// Connections pane help
    Connections,
    /// Tables pane help
    Tables,
    /// Details pane help
    Details,
    /// Tabular output help
    TabularOutput,
    /// SQL Files help
    SqlFiles,
    /// Query window help
    QueryWindow,
}

/// Internal editing state for query window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryEditMode {
    /// Normal navigation mode
    Normal,
    /// Insert/edit mode for typing
    Insert,
}

/// UI State - All UI-related state that can be saved/restored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIState {
    /// Currently focused pane
    pub focused_pane: FocusedPane,
    /// Last focused left column pane (for smarter navigation)
    pub last_left_pane: FocusedPane,
    /// Current help display mode
    pub help_mode: HelpMode,

    // Selection indices
    /// Selected connection index
    pub selected_connection: usize,
    /// Selected table index
    pub selected_table: usize,
    /// Selected SQL file index in the browser
    pub selected_sql_file: usize,

    // Cursor positions
    /// Current row in main content
    pub current_row: usize,
    /// Current column in main content
    pub current_column: usize,
    /// Current cursor position in query editor
    pub query_cursor_line: usize,
    pub query_cursor_column: usize,

    // Query editor state
    /// Query editor mode
    pub query_edit_mode: QueryEditMode,
    /// Whether query content has been modified
    pub query_modified: bool,
    /// Currently loaded SQL file path
    pub current_sql_file: Option<String>,
    /// Viewport offset - the first visible line in the query editor
    pub query_viewport_offset: usize,
    /// Number of visible lines in the query editor (updated on render)
    pub query_viewport_height: usize,

    // Vim command state
    /// Vim command buffer for :w, :q, etc
    pub vim_command_buffer: String,
    /// Whether we're in vim command mode (after pressing :)
    pub in_vim_command: bool,

    // Modal visibility states
    /// Show connection creation modal
    pub show_add_connection_modal: bool,
    /// Show connection edit modal
    pub show_edit_connection_modal: bool,
    /// Show table creator view
    pub show_table_creator: bool,
    /// Show table editor view
    pub show_table_editor: bool,

    /// Confirmation modal state
    #[serde(skip)]
    pub confirmation_modal: Option<crate::ui::ConfirmationModal>,

    // Hierarchical browsing state
    /// Expanded schemas/databases in tables pane
    pub expanded_schemas: std::collections::HashSet<String>,
    /// Expanded object type groups (Tables, Views, etc.)
    pub expanded_object_groups: std::collections::HashSet<String>,

    // List UI states (not serialized)
    #[serde(skip)]
    pub connections_list_state: ListState,
    #[serde(skip)]
    pub tables_list_state: ListState,
}

impl UIState {
    /// Create a new UI state with defaults
    pub fn new() -> Self {
        let mut connections_list_state = ListState::default();
        connections_list_state.select(Some(0));

        Self {
            focused_pane: FocusedPane::Connections,
            last_left_pane: FocusedPane::Connections,
            help_mode: HelpMode::None,
            selected_connection: 0,
            selected_table: 0,
            selected_sql_file: 0,
            current_row: 0,
            current_column: 0,
            query_cursor_line: 0,
            query_cursor_column: 0,
            query_edit_mode: QueryEditMode::Normal,
            query_modified: false,
            current_sql_file: None,
            query_viewport_offset: 0,
            query_viewport_height: 0,
            vim_command_buffer: String::new(),
            in_vim_command: false,
            show_add_connection_modal: false,
            show_edit_connection_modal: false,
            show_table_creator: false,
            show_table_editor: false,
            confirmation_modal: None,
            expanded_schemas: std::collections::HashSet::new(),
            expanded_object_groups: std::collections::HashSet::new(),
            connections_list_state,
            tables_list_state: ListState::default(),
        }
    }

    /// Save UI state to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state_file = Self::state_file_path()?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(state_file, json)?;
        Ok(())
    }

    /// Load UI state from disk
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let state_file = Self::state_file_path()?;

        if !state_file.exists() {
            return Ok(Self::new());
        }

        let json = fs::read_to_string(state_file)?;
        let mut state: Self = serde_json::from_str(&json)?;

        // Initialize non-serialized fields
        state.connections_list_state = ListState::default();
        if state.selected_connection > 0 {
            state
                .connections_list_state
                .select(Some(state.selected_connection));
        }

        state.tables_list_state = ListState::default();
        if state.selected_table > 0 {
            state
                .tables_list_state
                .select(Some(state.selected_table + 1));
        }

        Ok(state)
    }

    /// Get the path to the UI state file
    fn state_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not find config directory")?
            .join("lazytables");

        fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("ui_state.json"))
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
                // Smart selection: go to the last focused left pane
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

    /// Update connection list selection state
    pub fn update_connection_selection(&mut self, count: usize) {
        if count > 0 {
            // Clamp selection to valid range
            if self.selected_connection >= count {
                self.selected_connection = count - 1;
            }
            self.connections_list_state
                .select(Some(self.selected_connection));
        } else {
            self.selected_connection = 0;
            self.connections_list_state.select(None);
        }
    }

    /// Update table list selection state
    pub fn update_table_selection(&mut self, count: usize) {
        if count > 0 {
            // Clamp selection to valid range
            if self.selected_table >= count {
                self.selected_table = count - 1;
            }
            // Add 1 to account for the "▼ Tables" header in the UI
            self.tables_list_state.select(Some(self.selected_table + 1));
        } else {
            self.selected_table = 0;
            self.tables_list_state.select(None);
        }
    }

    /// Update SQL file list selection
    pub fn update_sql_file_selection(&mut self, count: usize) {
        if count > 0 && self.selected_sql_file >= count {
            self.selected_sql_file = count - 1;
        } else if count == 0 {
            self.selected_sql_file = 0;
        }
    }

    /// Move connection selection down
    pub fn connection_down(&mut self, max_count: usize) {
        if max_count > 0 {
            self.selected_connection = (self.selected_connection + 1) % max_count;
            self.connections_list_state
                .select(Some(self.selected_connection));
        }
    }

    /// Move connection selection up
    pub fn connection_up(&mut self, max_count: usize) {
        if max_count > 0 {
            self.selected_connection = if self.selected_connection > 0 {
                self.selected_connection - 1
            } else {
                max_count - 1
            };
            self.connections_list_state
                .select(Some(self.selected_connection));
        }
    }

    /// Move table selection down
    pub fn table_down(&mut self, max_count: usize) {
        if max_count > 0 {
            self.selected_table = (self.selected_table + 1) % max_count;
            // Add 1 to account for the "▼ Tables" header in the UI
            self.tables_list_state.select(Some(self.selected_table + 1));
        }
    }

    /// Move table selection up
    pub fn table_up(&mut self, max_count: usize) {
        if max_count > 0 {
            self.selected_table = if self.selected_table > 0 {
                self.selected_table - 1
            } else {
                max_count - 1
            };
            // Add 1 to account for the "▼ Tables" header in the UI
            self.tables_list_state.select(Some(self.selected_table + 1));
        }
    }

    /// Reset all UI state to defaults
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Clear modal states
    pub fn clear_modals(&mut self) {
        self.show_add_connection_modal = false;
        self.show_edit_connection_modal = false;
        self.show_table_creator = false;
        self.show_table_editor = false;
    }

    /// Enter vim command mode
    pub fn enter_vim_command(&mut self) {
        self.in_vim_command = true;
        self.vim_command_buffer.clear();
    }

    /// Exit vim command mode
    pub fn exit_vim_command(&mut self) {
        self.in_vim_command = false;
        self.vim_command_buffer.clear();
    }

    /// Toggle expansion state of a schema/database
    pub fn toggle_schema_expansion(&mut self, schema_name: &str) {
        if self.expanded_schemas.contains(schema_name) {
            self.expanded_schemas.remove(schema_name);
        } else {
            self.expanded_schemas.insert(schema_name.to_string());
        }
    }

    /// Check if a schema/database is expanded
    pub fn is_schema_expanded(&self, schema_name: &str) -> bool {
        self.expanded_schemas.contains(schema_name)
    }

    /// Toggle expansion state of an object group (Tables, Views, etc.)
    pub fn toggle_object_group_expansion(&mut self, group_name: &str) {
        if self.expanded_object_groups.contains(group_name) {
            self.expanded_object_groups.remove(group_name);
        } else {
            self.expanded_object_groups.insert(group_name.to_string());
        }
    }

    /// Check if an object group is expanded
    pub fn is_object_group_expanded(&self, group_name: &str) -> bool {
        self.expanded_object_groups.contains(group_name)
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}
