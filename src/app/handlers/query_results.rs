// FilePath: src/app/handlers/query_results.rs
//
// Event handlers for the Query Results / Table Viewer pane

#![forbid(unsafe_code)]

use crate::{app::App, core::error::Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle Query Results pane keys - has its own edit mode
pub(crate) async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    // Check if in edit mode
    if let Some(tab) = app.state.table_viewer_state.current_tab_mut() {
        if tab.in_edit_mode {
            return handle_edit_mode(app, key).await;
        }
        if tab.in_search_mode {
            return handle_search_mode(app, key).await;
        }
    }

    // Normal navigation mode
    match key.code {
        // 'i' or Enter - Start editing current cell
        KeyCode::Char('i') | KeyCode::Enter => {
            if let Some(tab) = app.state.table_viewer_state.current_tab_mut() {
                tab.start_edit();
            }
        }
        // Ctrl+d - Page down (must come before plain 'd')
        KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
            if let Some(tab) = app.state.table_viewer_state.current_tab_mut() {
                if tab.view_mode == crate::ui::components::table_viewer::TableViewMode::Schema {
                    tab.page_down_schema();
                } else {
                    // In data view, page down through data pages
                    if tab.page_down() {
                        let tab_idx = app.state.table_viewer_state.active_tab;
                        if let Err(e) = app.state.load_table_data(tab_idx).await {
                            app.state
                                .toast_manager
                                .error(format!("Failed to load page: {e}"));
                        }
                    }
                }
            }
        }
        // 'd' - Delete current row (double-tap within 500ms)
        KeyCode::Char('d') => {
            let now = std::time::Instant::now();
            let should_delete = if let Some(last_press) = app.state.table_viewer_state.last_d_press {
                // Check if within 500ms window
                now.duration_since(last_press).as_millis() < 500
            } else {
                false
            };

            if should_delete {
                // Double-tap detected - prepare delete confirmation
                if let Some(confirmation) = app.state.table_viewer_state.prepare_delete_confirmation() {
                    app.state.table_viewer_state.delete_confirmation = Some(confirmation);
                } else {
                    app.state.toast_manager.error("Cannot delete row: no primary key found");
                }
                // Reset the last press
                app.state.table_viewer_state.last_d_press = None;
            } else {
                // First 'd' press - record timestamp
                app.state.table_viewer_state.last_d_press = Some(now);
                app.state.toast_manager.info("Press 'd' again to delete row, or 'c' to set NULL");
            }
        }
        // 'c' - Set cell to NULL (after 'd' press) or Copy cell (after 'y' press)
        KeyCode::Char('c') => {
            let now = std::time::Instant::now();

            // Check for 'yc' sequence - copy cell
            let should_copy_cell = if let Some(last_press) = app.state.table_viewer_state.last_y_press {
                // Check if within 500ms window after 'y' press
                now.duration_since(last_press).as_millis() < 500
            } else {
                false
            };

            // Check for 'dc' sequence - set NULL
            let should_set_null = if let Some(last_press) = app.state.table_viewer_state.last_d_press {
                // Check if within 500ms window after 'd' press
                now.duration_since(last_press).as_millis() < 500
            } else {
                false
            };

            if should_copy_cell {
                // 'yc' sequence detected - copy cell to clipboard
                match app.state.table_viewer_state.copy_cell() {
                    Ok(()) => {
                        app.state.toast_manager.success("Cell copied to clipboard");
                    }
                    Err(e) => {
                        app.state.toast_manager.error(format!("Failed to copy cell: {e}"));
                    }
                }
                // Reset the last press
                app.state.table_viewer_state.last_y_press = None;
            } else if should_set_null {
                // 'dc' sequence detected - prepare set NULL confirmation
                if let Some(confirmation) = app.state.table_viewer_state.prepare_set_null_confirmation() {
                    app.state.table_viewer_state.set_null_confirmation = Some(confirmation);
                } else {
                    // Check why we can't set NULL
                    if let Some(tab) = app.state.table_viewer_state.current_tab() {
                        if tab.selected_col < tab.columns.len() {
                            let column = &tab.columns[tab.selected_col];
                            if !column.is_nullable {
                                app.state.toast_manager.error(format!(
                                    "Cannot set NULL: column '{}' is NOT NULL",
                                    column.name
                                ));
                            } else if tab.primary_key_columns.is_empty() {
                                app.state.toast_manager.error("Cannot set NULL: no primary key found");
                            } else {
                                app.state.toast_manager.error("Cannot set NULL on current cell");
                            }
                        }
                    }
                }
                // Reset the last press
                app.state.table_viewer_state.last_d_press = None;
            }
            // If 'c' pressed without prior 'd' or 'y', do nothing (ignore)
        }
        // 'y' - Copy current row (double-tap within 500ms)
        KeyCode::Char('y') => {
            let now = std::time::Instant::now();
            let should_copy = if let Some(last_press) = app.state.table_viewer_state.last_y_press {
                // Check if within 500ms window
                now.duration_since(last_press).as_millis() < 500
            } else {
                false
            };

            if should_copy {
                // Double-tap detected - copy row to clipboard
                match app.state.table_viewer_state.copy_row_csv() {
                    Ok(()) => {
                        app.state.toast_manager.success("Row copied to clipboard (CSV format)");
                    }
                    Err(e) => {
                        app.state.toast_manager.error(format!("Failed to copy row: {e}"));
                    }
                }
                // Reset the last press
                app.state.table_viewer_state.last_y_press = None;
            } else {
                // First 'y' press - record timestamp
                app.state.table_viewer_state.last_y_press = Some(now);
                app.state.toast_manager.info("Press 'y' again to copy row, or 'c' to copy cell");
            }
        }
        // '/' - Enter search mode
        KeyCode::Char('/') => {
            if let Some(tab) = app.state.table_viewer_state.current_tab_mut() {
                tab.start_search();
            }
        }
        // 't' - Toggle between Data and Schema view
        KeyCode::Char('t') => {
            if let Some(tab) = app.state.table_viewer_state.current_tab_mut() {
                tab.toggle_view_mode();
                let mode = match tab.view_mode {
                    crate::ui::components::table_viewer::TableViewMode::Data => "Data",
                    crate::ui::components::table_viewer::TableViewMode::Schema => "Schema",
                };
                app.state
                    .toast_manager
                    .info(format!("Switched to {} view", mode));
            }
        }
        // 'r' - Refresh table data (works with or without Ctrl)
        KeyCode::Char('r') => {
            let tab_idx = app.state.table_viewer_state.active_tab;
            if let Err(e) = app.state.load_table_data(tab_idx).await {
                app.state
                    .toast_manager
                    .error(format!("Failed to refresh: {e}"));
            } else {
                app.state.toast_manager.success("Table data refreshed");
            }
        }
        // Ctrl+u - Page up
        KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
            if let Some(tab) = app.state.table_viewer_state.current_tab_mut() {
                if tab.view_mode == crate::ui::components::table_viewer::TableViewMode::Schema {
                    tab.page_up_schema();
                } else {
                    // In data view, page up through data pages
                    if tab.page_up() {
                        let tab_idx = app.state.table_viewer_state.active_tab;
                        if let Err(e) = app.state.load_table_data(tab_idx).await {
                            app.state
                                .toast_manager
                                .error(format!("Failed to load page: {e}"));
                        }
                    }
                }
            }
        }
        // h/j/k/l - Navigate cells
        KeyCode::Char('h') | KeyCode::Left => {
            app.state.move_left();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.state.move_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.state.move_up();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.state.move_right();
        }
        // 'H' - Switch to previous tab
        KeyCode::Char('H') => {
            app.state.table_viewer_state.prev_tab();
        }
        // 'L' - Switch to next tab
        KeyCode::Char('L') => {
            app.state.table_viewer_state.next_tab();
        }
        // 'x' - Close current tab
        KeyCode::Char('x') => {
            let table_name = app.state.table_viewer_state.current_tab()
                .map(|tab| tab.table_name.clone());

            app.state.table_viewer_state.close_current_tab();

            if let Some(name) = table_name {
                app.state.toast_manager.info(format!("Closed tab: {}", name));
            }
        }
        // 'g' - First press of gg (jump to top)
        KeyCode::Char('g') => {
            if app.state.ui.pending_gg_command {
                // Second 'g' press - jump to top
                if let Some(tab) = app.state.table_viewer_state.current_tab_mut() {
                    if tab.view_mode
                        == crate::ui::components::table_viewer::TableViewMode::Schema
                    {
                        tab.jump_to_top_schema();
                    } else {
                        tab.jump_to_first();
                    }
                }
                app.state.ui.pending_gg_command = false;
            } else {
                // First 'g' press - set pending
                app.state.ui.pending_gg_command = true;
            }
        }
        // 'G' - Jump to bottom
        KeyCode::Char('G') => {
            if let Some(tab) = app.state.table_viewer_state.current_tab_mut() {
                if tab.view_mode == crate::ui::components::table_viewer::TableViewMode::Schema
                {
                    tab.jump_to_bottom_schema();
                } else {
                    tab.jump_to_last();
                }
            }
            app.state.ui.cancel_pending_gg();
        }
        // '0' - Jump to first column (only in data view)
        KeyCode::Char('0') => {
            if let Some(tab) = app.state.table_viewer_state.current_tab_mut() {
                if tab.view_mode == crate::ui::components::table_viewer::TableViewMode::Data {
                    tab.jump_to_first_col();
                }
            }
        }
        // '$' - Jump to last column (only in data view)
        KeyCode::Char('$') => {
            if let Some(tab) = app.state.table_viewer_state.current_tab_mut() {
                if tab.view_mode == crate::ui::components::table_viewer::TableViewMode::Data {
                    tab.jump_to_last_col();
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Handle table viewer edit mode keys
async fn handle_edit_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    if let Some(tab) = app.state.table_viewer_state.current_tab_mut() {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                // Save edit
                if let Some(update) = tab.save_edit() {
                    if let Err(e) = app.state.update_table_cell(update).await {
                        app.state
                            .toast_manager
                            .error(format!("Failed to update cell: {e}"));
                    } else {
                        app.state
                            .toast_manager
                            .success("Cell updated successfully");
                    }
                }
            }
            KeyCode::Char(c) if key.modifiers == KeyModifiers::CONTROL && c == 'c' => {
                // Cancel edit
                tab.cancel_edit();
            }
            KeyCode::Char(c) => {
                tab.edit_buffer.push(c);
            }
            KeyCode::Backspace => {
                tab.edit_buffer.pop();
            }
            _ => {}
        }
    }
    Ok(())
}

/// Handle table viewer search mode keys
async fn handle_search_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    if let Some(tab) = app.state.table_viewer_state.current_tab_mut() {
        match key.code {
            KeyCode::Esc => {
                tab.cancel_search();
            }
            KeyCode::Enter => {
                tab.in_search_mode = false;
            }
            KeyCode::Char('n') => {
                tab.next_search_result();
            }
            KeyCode::Char('N') => {
                tab.prev_search_result();
            }
            KeyCode::Char(c) if !matches!(c, 'h' | 'j' | 'k' | 'l') => {
                tab.search_query.push(c);
                tab.update_search(&tab.search_query.clone());
            }
            KeyCode::Backspace => {
                tab.search_query.pop();
                tab.update_search(&tab.search_query.clone());
            }
            _ => {}
        }
    }
    Ok(())
}
