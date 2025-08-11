// FilePath: src/app/state.rs

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
    /// Main content area
    MainContent,
}

impl FocusedPane {
    /// Get the next pane in clockwise order
    pub fn next(&self) -> Self {
        match self {
            Self::Connections => Self::Tables,
            Self::Tables => Self::Details,
            Self::Details => Self::MainContent,
            Self::MainContent => Self::Connections,
        }
    }

    /// Get the previous pane in counter-clockwise order
    pub fn previous(&self) -> Self {
        match self {
            Self::Connections => Self::MainContent,
            Self::Tables => Self::Connections,
            Self::Details => Self::Tables,
            Self::MainContent => Self::Details,
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
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
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
        }
    }

    /// Cycle focus to the next pane
    pub fn cycle_focus_forward(&mut self) {
        self.focused_pane = self.focused_pane.next();
    }

    /// Cycle focus to the previous pane
    pub fn cycle_focus_backward(&mut self) {
        self.focused_pane = self.focused_pane.previous();
    }

    /// Move selection up based on current focus
    pub fn move_up(&mut self) {
        match self.focused_pane {
            FocusedPane::Connections => {
                self.selected_connection = self.selected_connection.saturating_sub(1);
            }
            FocusedPane::Tables => {
                self.selected_table = self.selected_table.saturating_sub(1);
            }
            FocusedPane::MainContent => {
                self.current_row = self.current_row.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Move selection down based on current focus
    pub fn move_down(&mut self) {
        match self.focused_pane {
            FocusedPane::Connections => {
                self.selected_connection += 1;
            }
            FocusedPane::Tables => {
                self.selected_table += 1;
            }
            FocusedPane::MainContent => {
                self.current_row += 1;
            }
            _ => {}
        }
    }

    /// Move selection left based on current focus
    pub fn move_left(&mut self) {
        if self.focused_pane == FocusedPane::MainContent {
            self.current_column = self.current_column.saturating_sub(1);
        }
    }

    /// Move selection right based on current focus
    pub fn move_right(&mut self) {
        if self.focused_pane == FocusedPane::MainContent {
            self.current_column += 1;
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
