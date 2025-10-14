// FilePath: src/app/state/navigation.rs
//
// Navigation and focus management functions

use crate::app::state::AppState;
use crate::state::ui::FocusedPane;

impl AppState {
    /// Cycle focus to the next pane
    pub fn cycle_focus_forward(&mut self) {
        let sql_panes_enabled = self.are_sql_panes_enabled();
        let query_editor_enabled = self.is_query_editor_enabled();
        let tables_enabled = self.is_tables_pane_enabled();
        let details_enabled = self.is_details_pane_enabled();
        let query_results_enabled = self.is_query_results_pane_enabled();
        self.ui.cycle_focus_forward(
            sql_panes_enabled,
            query_editor_enabled,
            tables_enabled,
            details_enabled,
            query_results_enabled,
        );
    }

    /// Cycle focus to the previous pane
    pub fn cycle_focus_backward(&mut self) {
        let sql_panes_enabled = self.are_sql_panes_enabled();
        let query_editor_enabled = self.is_query_editor_enabled();
        let tables_enabled = self.is_tables_pane_enabled();
        let details_enabled = self.is_details_pane_enabled();
        let query_results_enabled = self.is_query_results_pane_enabled();
        self.ui.cycle_focus_backward(
            sql_panes_enabled,
            query_editor_enabled,
            tables_enabled,
            details_enabled,
            query_results_enabled,
        );
    }

    /// Move focus left (Ctrl+h)
    pub fn move_focus_left(&mut self) {
        let sql_panes_enabled = self.are_sql_panes_enabled();
        let query_editor_enabled = self.is_query_editor_enabled();
        let details_enabled = self.is_details_pane_enabled();
        self.ui
            .move_focus_left(sql_panes_enabled, query_editor_enabled, details_enabled);
    }

    /// Move focus down (Ctrl+j)
    pub fn move_focus_down(&mut self) {
        let tables_enabled = self.is_tables_pane_enabled();
        let details_enabled = self.is_details_pane_enabled();
        let query_editor_enabled = self.is_query_editor_enabled();
        self.ui
            .move_focus_down(tables_enabled, details_enabled, query_editor_enabled);
    }

    /// Move focus up (Ctrl+k)
    pub fn move_focus_up(&mut self) {
        let tables_enabled = self.is_tables_pane_enabled();
        let details_enabled = self.is_details_pane_enabled();
        self.ui.move_focus_up(tables_enabled, details_enabled);
    }

    /// Move focus right (Ctrl+l)
    pub fn move_focus_right(&mut self) {
        let sql_panes_enabled = self.are_sql_panes_enabled();
        let query_editor_enabled = self.is_query_editor_enabled();
        let query_results_enabled = self.is_query_results_pane_enabled();
        self.ui.move_focus_right(
            sql_panes_enabled,
            query_editor_enabled,
            query_results_enabled,
        );
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

    /// Helper: Check if SQL panes (Query Editor and SQL Files) are enabled
    pub fn are_sql_panes_enabled(&self) -> bool {
        self.db
            .connections
            .connections
            .get(self.ui.selected_connection)
            .map(|conn| conn.is_connected())
            .unwrap_or(false)
    }

    /// Helper: Check if query editor is enabled
    pub fn is_query_editor_enabled(&self) -> bool {
        self.are_sql_panes_enabled() && self.ui.current_sql_file.is_some()
    }

    /// Helper: Check if tables pane is enabled
    pub fn is_tables_pane_enabled(&self) -> bool {
        self.db
            .connections
            .connections
            .get(self.ui.selected_connection)
            .map(|conn| conn.is_connected())
            .unwrap_or(false)
    }

    /// Helper: Check if details pane is enabled
    pub fn is_details_pane_enabled(&self) -> bool {
        let has_connection = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
            .map(|conn| conn.is_connected())
            .unwrap_or(false);

        let has_selected_table = self.ui.get_selected_table_name().is_some();

        has_connection && has_selected_table
    }

    /// Helper: Check if query results pane is enabled
    pub fn is_query_results_pane_enabled(&self) -> bool {
        let has_connection = self
            .db
            .connections
            .connections
            .get(self.ui.selected_connection)
            .map(|conn| conn.is_connected())
            .unwrap_or(false);

        let has_selected_table = self.ui.get_selected_table_name().is_some();

        has_connection && has_selected_table
    }
}
