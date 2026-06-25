// FilePath: src/app/handlers/overlays.rs
//
// Event handlers for overlay views (modals, debug view, help)

#![forbid(unsafe_code)]

use crate::{
    app::{App, AppView, HelpMode, OverlayView},
    core::error::Result,
};
use crossterm::event::{KeyCode, KeyEvent};

/// Handle overlay keys (connection form, table creator/editor, debug view)
pub(crate) async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    // ESC closes overlay and returns to main, EXCEPT for Help overlay
    // Help overlay only closes with '?' key, not ESC
    if key.code == KeyCode::Esc
        && !matches!(
            app.state.ui.current_view,
            AppView::Overlay(OverlayView::Help)
        )
    {
        app.state.ui.return_to_main();
        return Ok(());
    }

    // Route to specific overlay handler
    match &app.state.ui.current_view {
        AppView::Overlay(OverlayView::ConnectionForm(_)) => {
            super::connections::handle_connection_modal(app, key).await
        }
        AppView::Overlay(OverlayView::DebugView) => handle_debug_view(app, key),
        AppView::Overlay(OverlayView::Help) => handle_help(app, key),
        _ => Ok(()),
    }
}

/// Handle debug view keys
pub(crate) fn handle_debug_view(app: &mut App, key: KeyEvent) -> Result<()> {
    let debug_messages = crate::logging::get_debug_messages();
    let max_lines = debug_messages.len();

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.state.ui.debug_view_scroll_down(max_lines);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.state.ui.debug_view_scroll_up();
        }
        KeyCode::PageDown => {
            app.state.ui.debug_view_page_down(max_lines, 10);
        }
        KeyCode::PageUp => {
            app.state.ui.debug_view_page_up(10);
        }
        KeyCode::Char('g') => {
            if app.state.ui.pending_gg_command {
                app.state.ui.debug_view_go_to_top();
                app.state.ui.pending_gg_command = false;
            } else {
                app.state.ui.pending_gg_command = true;
            }
        }
        KeyCode::Char('G') => {
            app.state.ui.debug_view_go_to_bottom(max_lines);
        }
        KeyCode::Char('c') => {
            crate::logging::clear_debug_messages();
            app.state.toast_manager.info("Debug messages cleared");
        }
        _ => {}
    }
    Ok(())
}

/// Handle help overlay keys
pub(crate) fn handle_help(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        // Close help modal with '?' key only (ESC is disabled for help modal)
        KeyCode::Char('?') => {
            app.state.ui.help_mode = HelpMode::None;
            app.state.ui.return_to_main();
        }
        // Switch between left and right help panes
        KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
            app.state.ui.toggle_help_pane_focus();
        }
        KeyCode::Tab => {
            app.state.ui.toggle_help_pane_focus();
        }
        // Scroll up in focused help pane
        KeyCode::Up | KeyCode::Char('k') => {
            app.state.ui.help_scroll_up();
        }
        // Scroll down in focused help pane
        KeyCode::Down | KeyCode::Char('j') => {
            app.state.ui.help_scroll_down(100);
        }
        // Page navigation
        KeyCode::PageUp => {
            app.state.ui.help_page_up(10);
        }
        KeyCode::PageDown => {
            app.state.ui.help_page_down(100, 10);
        }
        _ => {}
    }
    Ok(())
}

/// Handle confirmation modal keys
pub(crate) async fn handle_confirmation_modal(app: &mut App, key: KeyEvent) -> Result<()> {
    if let Some(modal) = &app.state.ui.confirmation_modal {
        match key.code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Execute the confirmed action
                match &modal.action {
                    crate::ui::ConfirmationAction::DeleteConnection(index) => {
                        let index = *index;
                        if let Some(connection) =
                            app.state.db.connections.connections.get(index)
                        {
                            let conn_id = connection.id.clone();
                            if let Err(e) =
                                app.state.db.connections.remove_connection(&conn_id).await
                            {
                                app.state
                                    .toast_manager
                                    .error(format!("Failed to delete connection: {e}"));
                            } else {
                                app.state
                                    .toast_manager
                                    .success("Connection deleted successfully");
                                if app.state.ui.selected_connection
                                    >= app.state.db.connections.connections.len()
                                    && app.state.ui.selected_connection > 0
                                {
                                    app.state.ui.selected_connection -= 1;
                                }
                            }
                        }
                    }
                    crate::ui::ConfirmationAction::DeleteSqlFile(index) => {
                        let index = *index;
                        if let Err(e) = app.state.delete_sql_file(index).await {
                            app.state
                                .toast_manager
                                .error(format!("Failed to delete SQL file: {e}"));
                        } else {
                            app.state.toast_manager.success("SQL file deleted");
                        }
                        app.state
                            .ui
                            .update_sql_file_selection(app.state.saved_sql_files.len());
                    }
                    crate::ui::ConfirmationAction::ExitApplication => {
                        app.should_quit = true;
                    }
                    crate::ui::ConfirmationAction::QuitQueryEditor => {
                        // Just close the confirmation, stay in main view
                    }
                    _ => {}
                }
                app.state.ui.confirmation_modal = None;
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                app.state.ui.confirmation_modal = None;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle table delete confirmation keys
pub(crate) async fn handle_table_delete_confirmation(app: &mut App, key: KeyEvent) -> Result<()> {
    if let Some(confirmation) = &app.state.table_viewer_state.delete_confirmation {
        match key.code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                let confirmation = confirmation.clone();
                if let Err(e) = app.state.delete_table_row(confirmation).await {
                    app.state
                        .toast_manager
                        .error(format!("Failed to delete row: {e}"));
                } else {
                    app.state.toast_manager.success("Row deleted successfully");
                    let tab_idx = app.state.table_viewer_state.active_tab;
                    let _ = app.state.load_table_data(tab_idx).await;
                }
                app.state.table_viewer_state.delete_confirmation = None;
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                app.state.table_viewer_state.delete_confirmation = None;
                app.state.toast_manager.info("Delete cancelled");
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle set NULL confirmation keys
pub(crate) async fn handle_set_null_confirmation(app: &mut App, key: KeyEvent) -> Result<()> {
    if let Some(confirmation) = &app.state.table_viewer_state.set_null_confirmation {
        match key.code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                let confirmation = confirmation.clone();
                if let Err(e) = app.state.set_cell_to_null(confirmation).await {
                    app.state
                        .toast_manager
                        .error(format!("Failed to set NULL: {e}"));
                } else {
                    app.state.toast_manager.success("Cell set to NULL successfully");
                    let tab_idx = app.state.table_viewer_state.active_tab;
                    let _ = app.state.load_table_data(tab_idx).await;
                }
                app.state.table_viewer_state.set_null_confirmation = None;
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                app.state.table_viewer_state.set_null_confirmation = None;
                app.state.toast_manager.info("Set NULL cancelled");
            }
            _ => {}
        }
    }
    Ok(())
}
