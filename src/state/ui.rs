// FilePath: src/state/ui.rs

use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Check if a string contains all characters from query in sequence
fn matches_sequence(text: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let mut query_chars = query.chars();
    let mut current_char = query_chars.next();

    for text_char in text.chars() {
        if let Some(q_char) = current_char {
            if text_char == q_char {
                current_char = query_chars.next();
                if current_char.is_none() {
                    return true; // All query characters found in sequence
                }
            }
        }
    }

    false
}

/// Represents a selectable item in the tables pane
#[derive(Debug, Clone)]
pub struct SelectableTableItem {
    /// The display name of the item
    pub display_name: String,
    /// The actual table/view name for database operations
    pub object_name: String,
    /// The schema this object belongs to (if any)
    pub schema: Option<String>,
    /// Type of database object
    pub object_type: crate::database::objects::DatabaseObjectType,
    /// Whether this item is selectable (false for headers)
    pub is_selectable: bool,
    /// The index of this item in the display list
    pub display_index: usize,
}

impl SelectableTableItem {
    /// Create a new selectable table item
    pub fn new_selectable(
        display_name: String,
        object_name: String,
        schema: Option<String>,
        object_type: crate::database::objects::DatabaseObjectType,
        display_index: usize,
    ) -> Self {
        Self {
            display_name,
            object_name,
            schema,
            object_type,
            is_selectable: true,
            display_index,
        }
    }

    /// Create a non-selectable header item
    pub fn new_header(display_name: String, display_index: usize) -> Self {
        Self {
            display_name,
            object_name: String::new(),
            schema: None,
            object_type: crate::database::objects::DatabaseObjectType::Table,
            is_selectable: false,
            display_index,
        }
    }

    /// Get the qualified name for database operations
    pub fn qualified_name(&self) -> String {
        if let Some(ref schema) = self.schema {
            format!("{}.{}", schema, self.object_name)
        } else {
            self.object_name.clone()
        }
    }
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

/// Which pane is focused in the help modal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelpPaneFocus {
    /// Left pane (pane-specific help)
    Left,
    /// Right pane (global commands)
    Right,
}

/// Internal editing state for query window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryEditMode {
    /// Normal navigation mode
    Normal,
    /// Insert/edit mode for typing
    Insert,
}

/// Connection mode type (Add new or Edit existing)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionModeType {
    /// Adding a new connection
    Add,
    /// Editing an existing connection
    Edit,
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
    /// Which pane is focused in the help modal (left or right)
    pub help_pane_focus: HelpPaneFocus,
    /// Vertical scroll offset for left help pane
    pub help_left_scroll_offset: usize,
    /// Vertical scroll offset for right help pane
    pub help_right_scroll_offset: usize,

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
    /// Viewport offset for the details pane scrolling
    pub details_viewport_offset: usize,
    /// Height of the details pane viewport
    pub details_viewport_height: usize,
    /// Content height of details pane (updated during rendering)
    pub details_content_height: usize,
    /// Maximum scroll offset for details pane (updated during rendering)
    pub details_max_scroll_offset: usize,

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

    /// Show debug view (full screen debug logs and diagnostics)
    pub show_debug_view: bool,
    /// Debug view scroll offset
    pub debug_view_scroll_offset: usize,

    /// Show connection mode (full screen connection management)
    pub show_connection_mode: bool,
    /// Connection mode type (Add or Edit)
    pub connection_mode_type: ConnectionModeType,
    /// Connection mode scroll offset
    pub connection_mode_scroll_offset: usize,

    /// Confirmation modal state
    #[serde(skip)]
    pub confirmation_modal: Option<crate::ui::ConfirmationModal>,

    // Hierarchical browsing state
    /// Expanded schemas/databases in tables pane
    pub expanded_schemas: std::collections::HashSet<String>,
    /// Expanded object type groups (Tables, Views, etc.)
    pub expanded_object_groups: std::collections::HashSet<String>,

    // Table selection system
    /// Flat list of selectable table items for navigation
    #[serde(skip)]
    pub selectable_table_items: Vec<SelectableTableItem>,
    /// Index of currently selected table in the selectable items list
    pub selected_table_item_index: usize,

    // Table search state
    /// Whether search mode is active in tables pane
    pub tables_search_active: bool,
    /// Current search query for tables
    pub tables_search_query: String,
    /// Filtered table items based on search
    #[serde(skip)]
    pub filtered_table_items: Vec<SelectableTableItem>,

    // Vim navigation state
    /// Whether 'g' key was pressed and we're waiting for the second 'g' for gg command
    #[serde(skip)]
    pub pending_gg_command: bool,

    // Connections pane search state
    /// Whether search mode is active in connections pane
    pub connections_search_active: bool,
    /// Current search query for connections
    pub connections_search_query: String,
    /// Filtered connections based on search
    #[serde(skip)]
    pub filtered_connections: Vec<usize>,

    // SQL Files pane state
    /// Whether search mode is active in SQL files pane
    pub sql_files_search_active: bool,
    /// Current search query for SQL files
    pub sql_files_search_query: String,
    /// Whether rename mode is active
    pub sql_files_rename_mode: bool,
    /// New name buffer during rename
    pub sql_files_rename_buffer: String,
    /// Whether create new file mode is active
    pub sql_files_create_mode: bool,
    /// New file name buffer during creation
    pub sql_files_create_buffer: String,

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
            help_pane_focus: HelpPaneFocus::Left,
            help_left_scroll_offset: 0,
            help_right_scroll_offset: 0,
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
            details_viewport_offset: 0,
            details_viewport_height: 0,
            details_content_height: 0,
            details_max_scroll_offset: 0,
            vim_command_buffer: String::new(),
            in_vim_command: false,
            show_add_connection_modal: false,
            show_edit_connection_modal: false,
            show_table_creator: false,
            show_table_editor: false,
            show_debug_view: false,
            debug_view_scroll_offset: 0,
            show_connection_mode: false,
            connection_mode_type: ConnectionModeType::Add,
            connection_mode_scroll_offset: 0,
            confirmation_modal: None,
            expanded_schemas: std::collections::HashSet::new(),
            expanded_object_groups: {
                let mut groups = std::collections::HashSet::new();
                groups.insert("Tables".to_string());
                groups.insert("Views".to_string());
                groups
            },
            selectable_table_items: Vec::new(),
            selected_table_item_index: 0,
            tables_search_active: false,
            tables_search_query: String::new(),
            filtered_table_items: Vec::new(),
            pending_gg_command: false,
            connections_search_active: false,
            connections_search_query: String::new(),
            filtered_connections: Vec::new(),
            sql_files_search_active: false,
            sql_files_search_query: String::new(),
            sql_files_rename_mode: false,
            sql_files_rename_buffer: String::new(),
            sql_files_create_mode: false,
            sql_files_create_buffer: String::new(),
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

    /// Cycle focus to the next pane (connection-aware)
    pub fn cycle_focus_forward(&mut self, sql_panes_enabled: bool) {
        let mut new_pane = self.focused_pane.next();

        // Skip disabled SQL panes
        if !sql_panes_enabled {
            while matches!(new_pane, FocusedPane::QueryWindow | FocusedPane::SqlFiles) {
                new_pane = new_pane.next();
                // Prevent infinite loop if we end up back where we started
                if new_pane == self.focused_pane {
                    break;
                }
            }
        }

        self.update_focus(new_pane);
    }

    /// Cycle focus to the previous pane (connection-aware)
    pub fn cycle_focus_backward(&mut self, sql_panes_enabled: bool) {
        let mut new_pane = self.focused_pane.previous();

        // Skip disabled SQL panes
        if !sql_panes_enabled {
            while matches!(new_pane, FocusedPane::QueryWindow | FocusedPane::SqlFiles) {
                new_pane = new_pane.previous();
                // Prevent infinite loop if we end up back where we started
                if new_pane == self.focused_pane {
                    break;
                }
            }
        }

        self.update_focus(new_pane);
    }

    /// Move focus left (Ctrl+h) (connection-aware)
    pub fn move_focus_left(&mut self, sql_panes_enabled: bool) {
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
            FocusedPane::QueryWindow => {
                if sql_panes_enabled {
                    FocusedPane::Details
                } else {
                    // SQL panes disabled, go to tabular output instead
                    FocusedPane::TabularOutput
                }
            }
            FocusedPane::SqlFiles => {
                if sql_panes_enabled {
                    FocusedPane::QueryWindow
                } else {
                    // SQL panes disabled, go to tabular output instead
                    FocusedPane::TabularOutput
                }
            }
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

    /// Move focus right (Ctrl+l) (connection-aware)
    pub fn move_focus_right(&mut self, sql_panes_enabled: bool) {
        let new_pane = match self.focused_pane {
            FocusedPane::Connections => FocusedPane::TabularOutput,
            FocusedPane::Tables => FocusedPane::TabularOutput,
            FocusedPane::Details => {
                if sql_panes_enabled {
                    FocusedPane::QueryWindow
                } else {
                    // SQL panes disabled, stay in place or go to tabular output
                    self.focused_pane
                }
            }
            FocusedPane::QueryWindow => {
                if sql_panes_enabled {
                    FocusedPane::SqlFiles
                } else {
                    // This shouldn't happen if SQL panes are disabled, but handle gracefully
                    FocusedPane::TabularOutput
                }
            }
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
        self.show_debug_view = false;
        self.show_connection_mode = false;
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

    /// Build the selectable table items list from database objects
    pub fn build_selectable_table_items(
        &mut self,
        db_objects: &Option<crate::database::objects::DatabaseObjectList>,
    ) {
        self.selectable_table_items.clear();

        if let Some(ref objects) = db_objects {
            let mut display_index = 0;

            // Add tables section
            if !objects.tables.is_empty() {
                let is_expanded = self.is_object_group_expanded("Tables");
                self.selectable_table_items
                    .push(SelectableTableItem::new_header(
                        "▼ Tables".to_string(),
                        display_index,
                    ));
                display_index += 1;

                if is_expanded {
                    for table in &objects.tables {
                        self.selectable_table_items
                            .push(SelectableTableItem::new_selectable(
                                format!("  📋 {}", table.name),
                                table.name.clone(),
                                table.schema.clone(),
                                table.object_type.clone(),
                                display_index,
                            ));
                        display_index += 1;
                    }
                }
            }

            // Add views section
            if !objects.views.is_empty() {
                if !self.selectable_table_items.is_empty() {
                    self.selectable_table_items
                        .push(SelectableTableItem::new_header(
                            "".to_string(),
                            display_index,
                        ));
                    display_index += 1;
                }

                let is_expanded = self.is_object_group_expanded("Views");
                self.selectable_table_items
                    .push(SelectableTableItem::new_header(
                        "▼ Views".to_string(),
                        display_index,
                    ));
                display_index += 1;

                if is_expanded {
                    for view in &objects.views {
                        self.selectable_table_items
                            .push(SelectableTableItem::new_selectable(
                                format!("  👁️ {}", view.name),
                                view.name.clone(),
                                view.schema.clone(),
                                view.object_type.clone(),
                                display_index,
                            ));
                        display_index += 1;
                    }
                }
            }

            // Add materialized views section
            if !objects.materialized_views.is_empty() {
                if !self.selectable_table_items.is_empty() {
                    self.selectable_table_items
                        .push(SelectableTableItem::new_header(
                            "".to_string(),
                            display_index,
                        ));
                    display_index += 1;
                }

                let is_expanded = self.is_object_group_expanded("Materialized Views");
                self.selectable_table_items
                    .push(SelectableTableItem::new_header(
                        "▼ Materialized Views".to_string(),
                        display_index,
                    ));
                display_index += 1;

                if is_expanded {
                    for mv in &objects.materialized_views {
                        self.selectable_table_items
                            .push(SelectableTableItem::new_selectable(
                                format!("  🔄 {}", mv.name),
                                mv.name.clone(),
                                mv.schema.clone(),
                                mv.object_type.clone(),
                                display_index,
                            ));
                        display_index += 1;
                    }
                }
            }

            // Add foreign tables section
            if !objects.foreign_tables.is_empty() {
                if !self.selectable_table_items.is_empty() {
                    self.selectable_table_items
                        .push(SelectableTableItem::new_header(
                            "".to_string(),
                            display_index,
                        ));
                    display_index += 1;
                }

                let is_expanded = self.is_object_group_expanded("Foreign Tables");
                self.selectable_table_items
                    .push(SelectableTableItem::new_header(
                        "▼ Foreign Tables".to_string(),
                        display_index,
                    ));
                display_index += 1;

                if is_expanded {
                    for ft in &objects.foreign_tables {
                        self.selectable_table_items
                            .push(SelectableTableItem::new_selectable(
                                format!("  🔗 {}", ft.name),
                                ft.name.clone(),
                                ft.schema.clone(),
                                ft.object_type.clone(),
                                display_index,
                            ));
                        display_index += 1;
                    }
                }
            }
        }

        // Reset selection to first selectable item
        self.selected_table_item_index = self.find_first_selectable_index();
        self.update_tables_list_state_selection();
    }

    /// Find the index of the first selectable item
    fn find_first_selectable_index(&self) -> usize {
        for (idx, item) in self.selectable_table_items.iter().enumerate() {
            if item.is_selectable {
                return idx;
            }
        }
        0
    }

    /// Move table selection down (j key)
    pub fn table_selection_down(&mut self) {
        if self.selectable_table_items.is_empty() {
            return;
        }

        let mut next_index = self.selected_table_item_index + 1;

        // Find next selectable item
        while next_index < self.selectable_table_items.len() {
            if self.selectable_table_items[next_index].is_selectable {
                self.selected_table_item_index = next_index;
                self.update_tables_list_state_selection();
                return;
            }
            next_index += 1;
        }

        // If we reached the end, wrap to first selectable item
        self.selected_table_item_index = self.find_first_selectable_index();
        self.update_tables_list_state_selection();
    }

    /// Move table selection up (k key)
    pub fn table_selection_up(&mut self) {
        if self.selectable_table_items.is_empty() {
            return;
        }

        // Find previous selectable item, wrapping around if needed
        if self.selected_table_item_index > 0 {
            // Look backwards from current position
            let mut prev_index = self.selected_table_item_index - 1;
            loop {
                if self.selectable_table_items[prev_index].is_selectable {
                    self.selected_table_item_index = prev_index;
                    self.update_tables_list_state_selection();
                    return;
                }
                if prev_index == 0 {
                    break;
                }
                prev_index -= 1;
            }
        }

        // If we didn't find anything above, or we're at index 0, wrap to last selectable item
        for i in (0..self.selectable_table_items.len()).rev() {
            if self.selectable_table_items[i].is_selectable {
                self.selected_table_item_index = i;
                self.update_tables_list_state_selection();
                return;
            }
        }
    }

    /// Update the tables list state selection to match our internal selection
    fn update_tables_list_state_selection(&mut self) {
        if !self.selectable_table_items.is_empty()
            && self.selected_table_item_index < self.selectable_table_items.len()
        {
            self.tables_list_state
                .select(Some(self.selected_table_item_index));
        } else {
            self.tables_list_state.select(None);
        }
    }

    /// Get the currently selected table item
    pub fn get_selected_table_item(&self) -> Option<&SelectableTableItem> {
        let items = self.get_display_table_items();
        if self.selected_table_item_index < items.len() {
            let item = &items[self.selected_table_item_index];
            if item.is_selectable {
                return Some(item);
            }
        }
        None
    }

    /// Get the currently selected table name for database operations
    pub fn get_selected_table_name(&self) -> Option<String> {
        self.get_selected_table_item()
            .map(|item| item.qualified_name())
    }

    /// Enter search mode for tables pane
    pub fn enter_tables_search(&mut self) {
        self.tables_search_active = true;
        self.tables_search_query.clear();
        self.update_filtered_table_items();
    }

    /// Exit search mode for tables pane
    pub fn exit_tables_search(&mut self) {
        self.tables_search_active = false;
        self.tables_search_query.clear();
        self.filtered_table_items.clear();
        // Reset selection to the main list
        self.selected_table_item_index = self.find_first_selectable_index();
        self.update_tables_list_state_selection();
    }

    /// Add character to search query
    pub fn add_to_tables_search(&mut self, ch: char) {
        if self.tables_search_active {
            self.tables_search_query.push(ch);
            self.update_filtered_table_items();
        }
    }

    /// Remove character from search query
    pub fn backspace_tables_search(&mut self) {
        if self.tables_search_active && !self.tables_search_query.is_empty() {
            self.tables_search_query.pop();
            self.update_filtered_table_items();
        }
    }

    /// Update filtered table items based on search query
    fn update_filtered_table_items(&mut self) {
        if !self.tables_search_active || self.tables_search_query.is_empty() {
            self.filtered_table_items.clear();
            return;
        }

        let query = self.tables_search_query.to_lowercase();
        self.filtered_table_items.clear();

        for item in &self.selectable_table_items {
            if item.is_selectable {
                // Check if the table name contains the search query characters in sequence
                let table_name = item.object_name.to_lowercase();
                if matches_sequence(&table_name, &query) {
                    self.filtered_table_items.push(item.clone());
                }
            }
        }

        // Reset selection to first filtered item
        self.selected_table_item_index = 0;
        self.update_tables_list_state_selection();
    }

    /// Get the items to display (either filtered or all)
    pub fn get_display_table_items(&self) -> &[SelectableTableItem] {
        if self.tables_search_active && !self.filtered_table_items.is_empty() {
            &self.filtered_table_items
        } else {
            &self.selectable_table_items
        }
    }

    /// Navigate down in search results or main list
    pub fn table_search_selection_down(&mut self) {
        let items = if self.tables_search_active && !self.filtered_table_items.is_empty() {
            &self.filtered_table_items
        } else {
            &self.selectable_table_items
        };

        if items.is_empty() {
            return;
        }

        if self.tables_search_active && !self.filtered_table_items.is_empty() {
            // Navigate through filtered results
            self.selected_table_item_index = (self.selected_table_item_index + 1) % items.len();
        } else {
            // Use existing navigation logic for main list
            self.table_selection_down();
            return;
        }

        self.update_tables_list_state_selection();
    }

    /// Navigate up in search results or main list
    pub fn table_search_selection_up(&mut self) {
        let items = if self.tables_search_active && !self.filtered_table_items.is_empty() {
            &self.filtered_table_items
        } else {
            &self.selectable_table_items
        };

        if items.is_empty() {
            return;
        }

        if self.tables_search_active && !self.filtered_table_items.is_empty() {
            // Navigate through filtered results
            self.selected_table_item_index = if self.selected_table_item_index > 0 {
                self.selected_table_item_index - 1
            } else {
                items.len() - 1
            };
        } else {
            // Use existing navigation logic for main list
            self.table_selection_up();
            return;
        }

        self.update_tables_list_state_selection();
    }

    /// Go to first selectable table (vim gg command)
    pub fn table_go_to_first(&mut self) {
        self.selected_table_item_index = self.find_first_selectable_index();
        self.update_tables_list_state_selection();
        self.pending_gg_command = false;
    }

    /// Go to last selectable table (vim G command)
    pub fn table_go_to_last(&mut self) {
        let items = self.get_display_table_items();
        if !items.is_empty() {
            // Find the last selectable item
            for i in (0..items.len()).rev() {
                if items[i].is_selectable {
                    self.selected_table_item_index = i;
                    break;
                }
            }
        }
        self.update_tables_list_state_selection();
    }

    /// Handle 'g' key press for vim navigation
    pub fn handle_g_key_press(&mut self) {
        if self.pending_gg_command {
            // Second 'g' pressed - execute gg command (go to top)
            self.table_go_to_first();
        } else {
            // First 'g' pressed - wait for second 'g'
            self.pending_gg_command = true;
        }
    }

    /// Cancel pending gg command
    pub fn cancel_pending_gg(&mut self) {
        self.pending_gg_command = false;
    }

    // === SQL FILES FUNCTIONALITY ===

    /// Enter search mode for SQL files pane
    pub fn enter_sql_files_search(&mut self) {
        self.sql_files_search_active = true;
        self.sql_files_search_query.clear();
    }

    /// Exit search mode for SQL files pane
    pub fn exit_sql_files_search(&mut self) {
        self.sql_files_search_active = false;
        self.sql_files_search_query.clear();
    }

    /// Add character to SQL files search query
    pub fn add_to_sql_files_search(&mut self, ch: char) {
        if self.sql_files_search_active {
            self.sql_files_search_query.push(ch);
        }
    }

    /// Remove character from SQL files search query
    pub fn backspace_sql_files_search(&mut self) {
        if self.sql_files_search_active && !self.sql_files_search_query.is_empty() {
            self.sql_files_search_query.pop();
        }
    }

    /// Filter SQL files based on search query
    pub fn filter_sql_files(&self, files: &[String]) -> Vec<String> {
        if !self.sql_files_search_active || self.sql_files_search_query.is_empty() {
            return files.to_vec();
        }

        let query = self.sql_files_search_query.to_lowercase();
        files
            .iter()
            .filter(|file| {
                let filename = file.to_lowercase();
                matches_sequence(&filename, &query)
            })
            .cloned()
            .collect()
    }

    /// Enter rename mode for SQL files pane
    pub fn enter_sql_files_rename(&mut self, current_name: &str) {
        self.sql_files_rename_mode = true;
        self.sql_files_rename_buffer = current_name.to_string();
    }

    /// Exit rename mode for SQL files pane
    pub fn exit_sql_files_rename(&mut self) {
        self.sql_files_rename_mode = false;
        self.sql_files_rename_buffer.clear();
    }

    /// Add character to rename buffer
    pub fn add_to_sql_files_rename(&mut self, ch: char) {
        if self.sql_files_rename_mode {
            self.sql_files_rename_buffer.push(ch);
        }
    }

    /// Remove character from rename buffer
    pub fn backspace_sql_files_rename(&mut self) {
        if self.sql_files_rename_mode && !self.sql_files_rename_buffer.is_empty() {
            self.sql_files_rename_buffer.pop();
        }
    }

    /// Enter create new file mode for SQL files pane
    pub fn enter_sql_files_create(&mut self) {
        self.sql_files_create_mode = true;
        self.sql_files_create_buffer.clear();
    }

    /// Exit create new file mode for SQL files pane
    pub fn exit_sql_files_create(&mut self) {
        self.sql_files_create_mode = false;
        self.sql_files_create_buffer.clear();
        // Also clear search state to ensure files are visible
        self.sql_files_search_active = false;
        self.sql_files_search_query.clear();
    }

    /// Add character to create buffer
    pub fn add_to_sql_files_create(&mut self, ch: char) {
        if self.sql_files_create_mode {
            self.sql_files_create_buffer.push(ch);
        }
    }

    /// Remove character from create buffer
    pub fn backspace_sql_files_create(&mut self) {
        if self.sql_files_create_mode && !self.sql_files_create_buffer.is_empty() {
            self.sql_files_create_buffer.pop();
        }
    }

    /// Clear all SQL files input modes
    pub fn clear_sql_files_input_modes(&mut self) {
        self.exit_sql_files_search();
        self.exit_sql_files_rename();
        self.exit_sql_files_create();
    }

    // === DEBUG VIEW FUNCTIONALITY ===

    /// Toggle debug view visibility
    pub fn toggle_debug_view(&mut self) {
        self.show_debug_view = !self.show_debug_view;
        if self.show_debug_view {
            // Reset scroll when opening
            self.debug_view_scroll_offset = 0;
        }
    }

    /// Scroll debug view down
    pub fn debug_view_scroll_down(&mut self, max_lines: usize) {
        if max_lines > 0 && self.debug_view_scroll_offset < max_lines.saturating_sub(1) {
            self.debug_view_scroll_offset += 1;
        }
    }

    /// Scroll debug view up
    pub fn debug_view_scroll_up(&mut self) {
        if self.debug_view_scroll_offset > 0 {
            self.debug_view_scroll_offset -= 1;
        }
    }

    /// Page down in debug view
    pub fn debug_view_page_down(&mut self, max_lines: usize, page_size: usize) {
        self.debug_view_scroll_offset =
            (self.debug_view_scroll_offset + page_size).min(max_lines.saturating_sub(1));
    }

    /// Page up in debug view
    pub fn debug_view_page_up(&mut self, page_size: usize) {
        self.debug_view_scroll_offset = self.debug_view_scroll_offset.saturating_sub(page_size);
    }

    /// Go to top of debug view
    pub fn debug_view_go_to_top(&mut self) {
        self.debug_view_scroll_offset = 0;
    }

    /// Go to bottom of debug view
    pub fn debug_view_go_to_bottom(&mut self, max_lines: usize) {
        self.debug_view_scroll_offset = max_lines.saturating_sub(1);
    }

    // === CONNECTION MODE FUNCTIONALITY ===

    /// Enter connection mode for adding a new connection
    pub fn enter_add_connection_mode(&mut self) {
        self.show_connection_mode = true;
        self.connection_mode_type = ConnectionModeType::Add;
        self.connection_mode_scroll_offset = 0;
    }

    /// Enter connection mode for editing an existing connection
    pub fn enter_edit_connection_mode(&mut self) {
        self.show_connection_mode = true;
        self.connection_mode_type = ConnectionModeType::Edit;
        self.connection_mode_scroll_offset = 0;
    }

    /// Exit connection mode
    pub fn exit_connection_mode(&mut self) {
        self.show_connection_mode = false;
        self.connection_mode_scroll_offset = 0;
    }

    /// Scroll connection mode down
    pub fn connection_mode_scroll_down(&mut self, max_lines: usize) {
        if max_lines > 0 && self.connection_mode_scroll_offset < max_lines.saturating_sub(1) {
            self.connection_mode_scroll_offset += 1;
        }
    }

    /// Scroll connection mode up
    pub fn connection_mode_scroll_up(&mut self) {
        if self.connection_mode_scroll_offset > 0 {
            self.connection_mode_scroll_offset -= 1;
        }
    }

    // === CONNECTIONS SEARCH FUNCTIONALITY ===

    /// Enter search mode for connections pane
    pub fn enter_connections_search(&mut self) {
        self.connections_search_active = true;
        self.connections_search_query.clear();
        self.filtered_connections.clear();
        // Reset selection to first connection when entering search
        self.selected_connection = 0;
        self.connections_list_state.select(Some(0));
    }

    /// Exit search mode for connections pane
    pub fn exit_connections_search(&mut self) {
        self.connections_search_active = false;
        self.connections_search_query.clear();
        self.filtered_connections.clear();
    }

    /// Add character to connections search query
    pub fn add_to_connections_search(&mut self, ch: char) {
        if self.connections_search_active {
            self.connections_search_query.push(ch);
        }
    }

    /// Remove character from connections search query
    pub fn backspace_connections_search(&mut self) {
        if self.connections_search_active && !self.connections_search_query.is_empty() {
            self.connections_search_query.pop();
        }
    }

    /// Update filtered connections based on search query
    pub fn update_filtered_connections(
        &mut self,
        connections: &[crate::database::ConnectionConfig],
    ) {
        if !self.connections_search_active || self.connections_search_query.is_empty() {
            self.filtered_connections.clear();
            return;
        }

        let query = self.connections_search_query.to_lowercase();
        self.filtered_connections.clear();

        for (index, connection) in connections.iter().enumerate() {
            // Check if the connection name matches the search query using sequence matching
            let connection_name = connection.name.to_lowercase();
            if matches_sequence(&connection_name, &query) {
                self.filtered_connections.push(index);
            }
        }

        // Reset selection to first filtered connection if we have results
        if !self.filtered_connections.is_empty() {
            self.selected_connection = 0;
            self.connections_list_state.select(Some(0));
        } else {
            // No results, clear selection
            self.connections_list_state.select(None);
        }
    }

    /// Get the display connections list (either filtered or all)
    pub fn get_display_connections(
        &self,
        connections: &[crate::database::ConnectionConfig],
    ) -> Vec<usize> {
        if self.connections_search_active && !self.connections_search_query.is_empty() {
            // Search is active with a query - return filtered results
            self.filtered_connections.clone()
        } else if self.connections_search_active {
            // Search is active but no query yet - show all connections
            (0..connections.len()).collect()
        } else {
            // Normal mode, show all connections by index
            (0..connections.len()).collect()
        }
    }

    /// Navigate down in connections (handles both search and normal mode)
    pub fn connections_selection_down(
        &mut self,
        connections: &[crate::database::ConnectionConfig],
    ) {
        let display_connections = self.get_display_connections(connections);

        if display_connections.is_empty() {
            return;
        }

        if self.connections_search_active && !self.filtered_connections.is_empty() {
            // Navigate through filtered results
            self.selected_connection = (self.selected_connection + 1) % display_connections.len();
        } else if !self.connections_search_active {
            // Use existing navigation logic for normal mode
            self.connection_down(connections.len());
            return;
        }

        self.connections_list_state
            .select(Some(self.selected_connection));
    }

    /// Navigate up in connections (handles both search and normal mode)
    pub fn connections_selection_up(&mut self, connections: &[crate::database::ConnectionConfig]) {
        let display_connections = self.get_display_connections(connections);

        if display_connections.is_empty() {
            return;
        }

        if self.connections_search_active && !self.filtered_connections.is_empty() {
            // Navigate through filtered results
            self.selected_connection = if self.selected_connection > 0 {
                self.selected_connection - 1
            } else {
                display_connections.len() - 1
            };
        } else if !self.connections_search_active {
            // Use existing navigation logic for normal mode
            self.connection_up(connections.len());
            return;
        }

        self.connections_list_state
            .select(Some(self.selected_connection));
    }

    /// Get the currently selected connection index (accounting for search filter)
    pub fn get_selected_connection_index(
        &self,
        connections: &[crate::database::ConnectionConfig],
    ) -> Option<usize> {
        let display_connections = self.get_display_connections(connections);

        if display_connections.is_empty() {
            return None;
        }

        if self.connections_search_active && !self.filtered_connections.is_empty() {
            // Return the actual connection index from filtered results
            display_connections.get(self.selected_connection).cloned()
        } else {
            // Return the selected connection index directly
            Some(self.selected_connection)
        }
    }

    // === HELP MODAL NAVIGATION FUNCTIONALITY ===

    /// Switch focus between left and right help panes
    pub fn toggle_help_pane_focus(&mut self) {
        self.help_pane_focus = match self.help_pane_focus {
            HelpPaneFocus::Left => HelpPaneFocus::Right,
            HelpPaneFocus::Right => HelpPaneFocus::Left,
        };
    }

    /// Reset help modal state when help is opened
    pub fn reset_help_modal_state(&mut self) {
        self.help_pane_focus = HelpPaneFocus::Left;
        self.help_left_scroll_offset = 0;
        self.help_right_scroll_offset = 0;
    }

    /// Scroll the currently focused help pane down
    pub fn help_scroll_down(&mut self, max_lines: usize) {
        match self.help_pane_focus {
            HelpPaneFocus::Left => {
                if max_lines > 0 && self.help_left_scroll_offset < max_lines.saturating_sub(1) {
                    self.help_left_scroll_offset += 1;
                }
            }
            HelpPaneFocus::Right => {
                if max_lines > 0 && self.help_right_scroll_offset < max_lines.saturating_sub(1) {
                    self.help_right_scroll_offset += 1;
                }
            }
        }
    }

    /// Scroll the currently focused help pane up
    pub fn help_scroll_up(&mut self) {
        match self.help_pane_focus {
            HelpPaneFocus::Left => {
                if self.help_left_scroll_offset > 0 {
                    self.help_left_scroll_offset -= 1;
                }
            }
            HelpPaneFocus::Right => {
                if self.help_right_scroll_offset > 0 {
                    self.help_right_scroll_offset -= 1;
                }
            }
        }
    }

    /// Page down in the currently focused help pane
    pub fn help_page_down(&mut self, max_lines: usize, page_size: usize) {
        match self.help_pane_focus {
            HelpPaneFocus::Left => {
                self.help_left_scroll_offset =
                    (self.help_left_scroll_offset + page_size).min(max_lines.saturating_sub(1));
            }
            HelpPaneFocus::Right => {
                self.help_right_scroll_offset =
                    (self.help_right_scroll_offset + page_size).min(max_lines.saturating_sub(1));
            }
        }
    }

    /// Page up in the currently focused help pane
    pub fn help_page_up(&mut self, page_size: usize) {
        match self.help_pane_focus {
            HelpPaneFocus::Left => {
                self.help_left_scroll_offset =
                    self.help_left_scroll_offset.saturating_sub(page_size);
            }
            HelpPaneFocus::Right => {
                self.help_right_scroll_offset =
                    self.help_right_scroll_offset.saturating_sub(page_size);
            }
        }
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_sequence() {
        assert!(matches_sequence("users", "usr"));
        assert!(matches_sequence("user_profiles", "upr"));
        assert!(matches_sequence("customer_orders", "co"));
        assert!(matches_sequence("employees", "emp"));
        assert!(!matches_sequence("users", "xyz"));
        assert!(!matches_sequence("short", "longquery"));
        assert!(matches_sequence("anything", ""));

        // Debug specific cases
        assert!(matches_sequence("users", "users"));
        assert!(matches_sequence("accounts", "u")); // accounts DOES match 'u' at position 5
        assert!(matches_sequence("orders", "o")); // orders should match 'o'

        // Test that we can search for 'j' and 'k' characters
        assert!(matches_sequence("projects", "j")); // should match 'j' in "projects"
        assert!(matches_sequence("tasks", "k")); // should match 'k' in "tasks"
    }

    #[test]
    fn test_tables_search_functionality() {
        let mut ui_state = UIState::new();

        // Add some mock table items
        ui_state.selectable_table_items = vec![
            SelectableTableItem::new_selectable(
                "users".to_string(),
                "users".to_string(),
                None,
                crate::database::objects::DatabaseObjectType::Table,
                0,
            ),
            SelectableTableItem::new_selectable(
                "accounts".to_string(),
                "accounts".to_string(),
                None,
                crate::database::objects::DatabaseObjectType::Table,
                1,
            ),
            SelectableTableItem::new_selectable(
                "orders".to_string(),
                "orders".to_string(),
                None,
                crate::database::objects::DatabaseObjectType::Table,
                2,
            ),
        ];

        // Test entering search mode
        ui_state.enter_tables_search();
        assert!(ui_state.tables_search_active);
        assert!(ui_state.tables_search_query.is_empty());

        // Test adding search query
        ui_state.add_to_tables_search('u');
        assert_eq!(ui_state.tables_search_query, "u");
        assert_eq!(ui_state.filtered_table_items.len(), 2); // "users" and "accounts" both match "u"

        ui_state.add_to_tables_search('s');
        assert_eq!(ui_state.tables_search_query, "us");
        assert_eq!(ui_state.filtered_table_items.len(), 2); // both "users" and "accounts" match "us" sequence (u, s)

        ui_state.add_to_tables_search('e');
        assert_eq!(ui_state.tables_search_query, "use");
        assert_eq!(ui_state.filtered_table_items.len(), 1); // only "users" matches "use" sequence

        ui_state.add_to_tables_search('r');
        assert_eq!(ui_state.tables_search_query, "user");
        assert_eq!(ui_state.filtered_table_items.len(), 1); // only "users" matches "user" sequence

        ui_state.add_to_tables_search('s');
        assert_eq!(ui_state.tables_search_query, "users");
        assert_eq!(ui_state.filtered_table_items.len(), 1); // only "users" matches "users" completely

        // Test backspace
        ui_state.backspace_tables_search(); // remove 's'
        assert_eq!(ui_state.tables_search_query, "user");
        assert_eq!(ui_state.filtered_table_items.len(), 1);

        // Test exiting search
        ui_state.exit_tables_search();
        assert!(!ui_state.tables_search_active);
        assert!(ui_state.tables_search_query.is_empty());
        assert!(ui_state.filtered_table_items.is_empty());
    }

    #[test]
    fn test_tables_search_with_j_k_characters() {
        let mut ui_state = UIState::new();

        // Add mock table items that contain 'j' and 'k' characters
        ui_state.selectable_table_items = vec![
            SelectableTableItem::new_selectable(
                "projects".to_string(),
                "projects".to_string(),
                None,
                crate::database::objects::DatabaseObjectType::Table,
                0,
            ),
            SelectableTableItem::new_selectable(
                "tasks".to_string(),
                "tasks".to_string(),
                None,
                crate::database::objects::DatabaseObjectType::Table,
                1,
            ),
            SelectableTableItem::new_selectable(
                "events".to_string(),
                "events".to_string(),
                None,
                crate::database::objects::DatabaseObjectType::Table,
                2,
            ),
        ];

        // Test entering search mode and searching for 'j'
        ui_state.enter_tables_search();
        ui_state.add_to_tables_search('j');
        assert_eq!(ui_state.tables_search_query, "j");
        assert_eq!(ui_state.filtered_table_items.len(), 1); // only "projects" contains 'j'
        assert_eq!(ui_state.filtered_table_items[0].object_name, "projects");

        // Clear and test searching for 'k'
        ui_state.exit_tables_search();
        ui_state.enter_tables_search();
        ui_state.add_to_tables_search('k');
        assert_eq!(ui_state.tables_search_query, "k");
        assert_eq!(ui_state.filtered_table_items.len(), 1); // only "tasks" contains 'k'
        assert_eq!(ui_state.filtered_table_items[0].object_name, "tasks");
    }

    #[test]
    fn test_connections_search_functionality() {
        let mut ui_state = UIState::new();

        // Mock some connections
        use crate::database::{ConnectionConfig, ConnectionStatus, DatabaseType};
        let mock_connections = vec![
            ConnectionConfig {
                id: "1".to_string(),
                name: "Production DB".to_string(),
                database_type: DatabaseType::PostgreSQL,
                host: "localhost".to_string(),
                port: 5432,
                database: Some("prod".to_string()),
                username: "user".to_string(),
                password_source: None,
                password: None,
                ssl_mode: crate::database::SslMode::Prefer,
                timeout: None,
                status: ConnectionStatus::Disconnected,
            },
            ConnectionConfig {
                id: "2".to_string(),
                name: "Development DB".to_string(),
                database_type: DatabaseType::MySQL,
                host: "localhost".to_string(),
                port: 3306,
                database: Some("dev".to_string()),
                username: "user".to_string(),
                password_source: None,
                password: None,
                ssl_mode: crate::database::SslMode::Prefer,
                timeout: None,
                status: ConnectionStatus::Disconnected,
            },
            ConnectionConfig {
                id: "3".to_string(),
                name: "Test Database".to_string(),
                database_type: DatabaseType::SQLite,
                host: "test.db".to_string(),
                port: 0,
                database: Some("test.db".to_string()),
                username: "".to_string(),
                password_source: None,
                password: None,
                ssl_mode: crate::database::SslMode::Disable,
                timeout: None,
                status: ConnectionStatus::Disconnected,
            },
        ];

        // Test entering search mode
        ui_state.enter_connections_search();
        assert!(ui_state.connections_search_active);
        assert!(ui_state.connections_search_query.is_empty());

        // Test adding search query
        ui_state.add_to_connections_search('p');
        ui_state.update_filtered_connections(&mock_connections);
        assert_eq!(ui_state.connections_search_query, "p");
        // Both "Production DB" and "Development DB" contain 'p'
        assert_eq!(ui_state.filtered_connections.len(), 2);
        assert!(ui_state.filtered_connections.contains(&0)); // "Production DB"
        assert!(ui_state.filtered_connections.contains(&1)); // "Development DB"

        ui_state.add_to_connections_search('r');
        ui_state.update_filtered_connections(&mock_connections);
        assert_eq!(ui_state.connections_search_query, "pr");
        // Only "Production DB" has 'p' followed by 'r' in sequence
        assert_eq!(ui_state.filtered_connections.len(), 1); // Should only be "Production DB"
        assert_eq!(ui_state.filtered_connections[0], 0); // First connection

        ui_state.add_to_connections_search('o');
        ui_state.update_filtered_connections(&mock_connections);
        assert_eq!(ui_state.connections_search_query, "pro");
        assert_eq!(ui_state.filtered_connections.len(), 1); // "Production DB" matches "pro"

        // Test that adding 'x' filters out results
        ui_state.add_to_connections_search('x');
        ui_state.update_filtered_connections(&mock_connections);
        assert_eq!(ui_state.connections_search_query, "prox");
        assert_eq!(ui_state.filtered_connections.len(), 0); // No connections match "prox"

        // Test backspace
        ui_state.backspace_connections_search();
        ui_state.update_filtered_connections(&mock_connections);
        assert_eq!(ui_state.connections_search_query, "pro");
        assert_eq!(ui_state.filtered_connections.len(), 1); // Back to "Production DB"

        // Test different search query
        ui_state.connections_search_query.clear();
        ui_state.add_to_connections_search('d');
        ui_state.update_filtered_connections(&mock_connections);
        assert_eq!(ui_state.connections_search_query, "d");
        assert_eq!(ui_state.filtered_connections.len(), 3); // All connections contain 'd': "Production DB", "Development DB", "Test Database"

        // Test exiting search
        ui_state.exit_connections_search();
        assert!(!ui_state.connections_search_active);
        assert!(ui_state.connections_search_query.is_empty());
        assert!(ui_state.filtered_connections.is_empty());

        // Test get_selected_connection_index
        ui_state.selected_connection = 1;
        assert_eq!(
            ui_state.get_selected_connection_index(&mock_connections),
            Some(1)
        );

        // Test with search active - use a query that uniquely identifies one connection
        ui_state.enter_connections_search();
        ui_state.add_to_connections_search('e');
        ui_state.add_to_connections_search('v');
        ui_state.update_filtered_connections(&mock_connections);
        // "ev" should match only "Development DB"
        ui_state.selected_connection = 0; // First (and only) in filtered results
        assert_eq!(
            ui_state.get_selected_connection_index(&mock_connections),
            Some(1)
        ); // Should return actual index of "Development DB"
    }

    #[test]
    fn test_vim_navigation_commands() {
        let mut ui_state = UIState::new();

        // Add mock table items
        ui_state.selectable_table_items = vec![
            SelectableTableItem::new_header("▼ Tables".to_string(), 0),
            SelectableTableItem::new_selectable(
                "first_table".to_string(),
                "first_table".to_string(),
                None,
                crate::database::objects::DatabaseObjectType::Table,
                1,
            ),
            SelectableTableItem::new_selectable(
                "middle_table".to_string(),
                "middle_table".to_string(),
                None,
                crate::database::objects::DatabaseObjectType::Table,
                2,
            ),
            SelectableTableItem::new_selectable(
                "last_table".to_string(),
                "last_table".to_string(),
                None,
                crate::database::objects::DatabaseObjectType::Table,
                3,
            ),
        ];

        // Start at middle
        ui_state.selected_table_item_index = 2;

        // Test G command (go to last)
        ui_state.table_go_to_last();
        assert_eq!(ui_state.selected_table_item_index, 3);
        if let Some(item) = ui_state.get_selected_table_item() {
            assert_eq!(item.object_name, "last_table");
        }

        // Test gg command (go to first)
        ui_state.table_go_to_first();
        assert_eq!(ui_state.selected_table_item_index, 1); // First selectable item (skipping header)
        if let Some(item) = ui_state.get_selected_table_item() {
            assert_eq!(item.object_name, "first_table");
        }

        // Test gg command sequence
        ui_state.selected_table_item_index = 2; // Start at middle
        assert!(!ui_state.pending_gg_command);

        // First 'g' press
        ui_state.handle_g_key_press();
        assert!(ui_state.pending_gg_command);
        assert_eq!(ui_state.selected_table_item_index, 2); // Should not move yet

        // Second 'g' press
        ui_state.handle_g_key_press();
        assert!(!ui_state.pending_gg_command); // Should be reset
        assert_eq!(ui_state.selected_table_item_index, 1); // Should move to first

        // Test canceling pending gg
        ui_state.selected_table_item_index = 2;
        ui_state.handle_g_key_press(); // First 'g'
        assert!(ui_state.pending_gg_command);
        ui_state.cancel_pending_gg();
        assert!(!ui_state.pending_gg_command);
        assert_eq!(ui_state.selected_table_item_index, 2); // Should stay in place
    }
}
