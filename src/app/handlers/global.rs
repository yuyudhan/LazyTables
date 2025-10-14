// FilePath: src/app/handlers/global.rs
//
// Global event handlers that work across all panes and overlays

use crate::{
    app::{App, FocusedPane},
    commands::CommandId,
    core::error::Result,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle global keys that work everywhere
pub(crate) fn handle(app: &mut App, key: KeyEvent) -> Result<Option<()>> {
    match (key.modifiers, key.code) {
        // Help - toggle with '?'
        (KeyModifiers::NONE, KeyCode::Char('?')) => {
            app.execute_command(CommandId::ToggleHelp)?;
            Ok(Some(()))
        }
        // Debug view - toggle with Ctrl+B
        (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
            app.state.ui.toggle_debug_view();
            Ok(Some(()))
        }
        // Quit application - 'q' (only if not in edit modes)
        (KeyModifiers::NONE, KeyCode::Char('q')) if can_quit(app) => {
            app.state.ui.confirmation_modal = Some(crate::ui::ConfirmationModal {
                title: "Exit LazyTables".to_string(),
                message: "Are you sure you want to exit?\n\nAll active database connections will be closed.".to_string(),
                action: crate::ui::ConfirmationAction::ExitApplication,
            });
            Ok(Some(()))
        }
        // Number keys 1-6 for direct pane navigation (only in main view)
        // BUT NOT when editing a table cell - numbers should go into the edit buffer
        (KeyModifiers::NONE, KeyCode::Char(c @ '1'..='6')) if app.state.ui.is_in_main() => {
            // Check if we're in table viewer edit mode
            let in_table_edit_mode = app.state.ui.focused_pane == FocusedPane::TabularOutput
                && app.state.table_viewer_state.current_tab()
                    .map(|tab| tab.in_edit_mode)
                    .unwrap_or(false);

            // Don't intercept number keys if in edit mode - let them pass through
            if in_table_edit_mode {
                return Ok(None); // Not handled, will be passed to table viewer edit handler
            }

            if let Some(pane) = FocusedPane::from_number(c.to_digit(10).unwrap() as u8) {
                // Check if the target pane is enabled before navigating to it
                let is_enabled = match pane {
                    FocusedPane::Connections => true, // Always enabled
                    FocusedPane::Tables => app.state.is_tables_pane_enabled(),
                    FocusedPane::Details => app.state.is_details_pane_enabled(),
                    FocusedPane::TabularOutput => app.state.is_query_results_pane_enabled(),
                    FocusedPane::QueryWindow => app.state.is_query_editor_enabled(),
                    FocusedPane::SqlFiles => app.state.are_sql_panes_enabled(),
                };

                if is_enabled {
                    app.state.ui.focused_pane = pane;
                    app.state.ui.cancel_pending_gg();
                }
                // If disabled, do nothing (stay in current pane)
            }
            Ok(Some(()))
        }
        // Tab/Shift+Tab for pane cycling
        // Skip Tab in query editor insert mode (Tab inserts tab character there)
        (KeyModifiers::NONE, KeyCode::Tab)
            if app.state.ui.is_in_main()
                && !(app.state.ui.focused_pane == FocusedPane::QueryWindow
                    && app.state.query_editor.is_insert_mode()) =>
        {
            app.state.cycle_focus_forward();
            app.state.ui.cancel_pending_gg();
            Ok(Some(()))
        }
        (KeyModifiers::SHIFT, KeyCode::BackTab)
            if app.state.ui.is_in_main()
                && !(app.state.ui.focused_pane == FocusedPane::QueryWindow
                    && app.state.query_editor.is_insert_mode()) =>
        {
            app.state.cycle_focus_backward();
            app.state.ui.cancel_pending_gg();
            Ok(Some(()))
        }
        // Ctrl+h/j/k/l for pane navigation
        (KeyModifiers::CONTROL, KeyCode::Char('h')) if app.state.ui.is_in_main() => {
            app.state.move_focus_left();
            Ok(Some(()))
        }
        (KeyModifiers::CONTROL, KeyCode::Char('j')) if app.state.ui.is_in_main() => {
            app.state.move_focus_down();
            Ok(Some(()))
        }
        (KeyModifiers::CONTROL, KeyCode::Char('k')) if app.state.ui.is_in_main() => {
            app.state.move_focus_up();
            Ok(Some(()))
        }
        (KeyModifiers::CONTROL, KeyCode::Char('l')) if app.state.ui.is_in_main() => {
            app.state.move_focus_right();
            Ok(Some(()))
        }
        _ => Ok(None), // Key not handled globally
    }
}

/// Check if quit action is allowed (not in edit/insert modes)
pub(crate) fn can_quit(app: &App) -> bool {
    if !app.state.ui.is_in_main() {
        return false;
    }
    // Check for active edit/search modes
    if app.state.ui.connections_search_active
        || app.state.ui.tables_search_active
        || app.state.ui.sql_files_search_active
        || app.state.ui.sql_files_rename_mode
        || app.state.ui.sql_files_create_mode
    {
        return false;
    }
    // Check table viewer edit mode
    if app.state.ui.focused_pane == FocusedPane::TabularOutput {
        if let Some(tab) = app.state.table_viewer_state.current_tab() {
            if tab.in_edit_mode || tab.in_search_mode {
                return false;
            }
        }
    }
    // Check query editor insert mode
    if app.state.ui.focused_pane == FocusedPane::QueryWindow
        && app.state.query_editor.is_insert_mode()
    {
        return false;
    }
    true
}
