// FilePath: src/app/handlers/tables.rs

// Event handler for the Tables pane (table/view navigation and selection).
// Handles browsing database objects, search filtering, and opening tables for viewing.

use crate::{app::App, core::error::Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle Tables pane keys - DIRECT KEY BINDINGS
pub(crate) async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    // Search mode active
    if app.state.ui.tables_search_active {
        match key.code {
            KeyCode::Esc => {
                app.state.ui.exit_tables_search();
            }
            KeyCode::Backspace => {
                app.state.ui.backspace_tables_search();
            }
            KeyCode::Enter => {
                app.state.open_table_for_viewing().await;
                app.state.ui.exit_tables_search();
            }
            KeyCode::Down => {
                app.state.ui.table_search_selection_down();
            }
            KeyCode::Up => {
                app.state.ui.table_search_selection_up();
            }
            KeyCode::Char(c) => {
                app.state.ui.add_to_tables_search(c);
            }
            _ => {}
        }
        return Ok(());
    }

    // Normal mode
    match key.code {
        // Enter or Space - Open table for viewing
        KeyCode::Enter | KeyCode::Char(' ') => {
            app.state.open_table_for_viewing().await;
        }
        // 'r' - Refresh tables list (preserve schema filter)
        KeyCode::Char('r') => {
            let saved_schema = app.state.db.selected_schema.clone();
            app.state.connect_to_selected_database().await;
            app.state.db.selected_schema = saved_schema;
            app.state.rebuild_table_items();
            app.state.toast_manager.info("Tables refreshed");
        }
        // 's' - Cycle schema filter
        KeyCode::Char('s') => {
            app.state.cycle_schema_filter();
        }
        // 'S' - Reset schema filter to All
        KeyCode::Char('S') => {
            app.state.reset_schema_filter();
        }
        // '/' - Enter search mode
        KeyCode::Char('/') => {
            app.state.ui.enter_tables_search();
        }
        // j/k - Navigate
        KeyCode::Char('j') | KeyCode::Down => {
            app.state.ui.table_search_selection_down();
            app.state.ui.cancel_pending_gg();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.state.ui.table_search_selection_up();
            app.state.ui.cancel_pending_gg();
        }
        // 'g' - First press of gg
        KeyCode::Char('g') => {
            app.state.ui.handle_g_key_press();
        }
        // 'G' - Jump to last table
        KeyCode::Char('G') => {
            app.state.ui.table_go_to_last();
        }
        // Ctrl+d - Page down (half page)
        KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
            for _ in 0..10 {
                app.state.ui.table_search_selection_down();
            }
        }
        // Ctrl+u - Page up (half page)
        KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
            for _ in 0..10 {
                app.state.ui.table_search_selection_up();
            }
        }
        // Tab - Toggle group expansion (when on a header)
        KeyCode::Tab if key.modifiers == KeyModifiers::NONE => {
            // Extract header info first to avoid borrow issues
            let header_info = app.state.ui.get_selected_item_raw().and_then(|item| {
                if item.is_selectable {
                    None
                } else if let Some(ref schema_name) = item.schema_header_name {
                    Some((true, schema_name.clone()))
                } else {
                    let group_name = item
                        .display_name
                        .trim_start_matches("▼ ")
                        .trim_start_matches("▶ ")
                        .trim()
                        .to_string();
                    if group_name.is_empty() {
                        None
                    } else {
                        Some((false, group_name))
                    }
                }
            });

            if let Some((is_schema, name)) = header_info {
                if is_schema {
                    // Schema header
                    let is_expanded_before = app.state.ui.is_schema_expanded(&name);
                    app.state.ui.toggle_schema_expansion(&name);
                    app.state.rebuild_table_items();
                    app.state.toast_manager.info(format!(
                        "{} schema {}",
                        if !is_expanded_before {
                            "Expanded"
                        } else {
                            "Collapsed"
                        },
                        name
                    ));
                } else {
                    // Regular object group header
                    let is_expanded_before = app.state.ui.is_object_group_expanded(&name);
                    app.state.ui.toggle_object_group_expansion(&name);
                    app.state.rebuild_table_items();
                    app.state.toast_manager.info(format!(
                        "{} {}",
                        if !is_expanded_before {
                            "Expanded"
                        } else {
                            "Collapsed"
                        },
                        name
                    ));
                }
            }
            // If it's not a header, Tab is handled by global keys for pane cycling
        }
        _ => {}
    }
    Ok(())
}
