// FilePath: src/app/handlers/sql_files.rs
//
// Event handlers for the SQL Files pane (file browser and management)

#![forbid(unsafe_code)]

use crate::{app::App, core::error::Result};
use crossterm::event::{KeyCode, KeyEvent};

/// Handle SQL Files pane keys - DIRECT KEY BINDINGS
pub(crate) async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    // Handle special input modes
    if app.state.ui.sql_files_search_active {
        return handle_search_mode(app, key).await;
    }
    if app.state.ui.sql_files_rename_mode {
        return handle_rename_mode(app, key).await;
    }
    if app.state.ui.sql_files_create_mode {
        return handle_create_mode(app, key).await;
    }

    // Normal mode
    match key.code {
        // Enter - Load selected SQL file
        KeyCode::Enter => {
            if let Err(e) = app.state.load_selected_sql_file() {
                app.state
                    .toast_manager
                    .error(format!("Failed to load SQL file: {e}"));
            } else {
                app.state.toast_manager.success("SQL file loaded");
            }
        }
        // 'n' - Create new file
        KeyCode::Char('n') => {
            app.state.ui.enter_sql_files_create();
        }
        // 'r' - Rename file
        KeyCode::Char('r') => {
            if let Some(filename) = app.state.get_selected_sql_file() {
                app.state.ui.enter_sql_files_rename(&filename);
            }
        }
        // 'd' - Delete file
        KeyCode::Char('d') => {
            if !app.state.saved_sql_files.is_empty() {
                let index = app.state.get_filtered_sql_file_selection();
                app.state.ui.confirmation_modal = Some(crate::ui::ConfirmationModal {
                    title: "Delete SQL File".to_string(),
                    message: format!(
                        "Are you sure you want to delete '{}'?",
                        app.state
                            .saved_sql_files
                            .get(index)
                            .unwrap_or(&String::new())
                    ),
                    action: crate::ui::ConfirmationAction::DeleteSqlFile(index),
                });
            }
        }
        // '/' - Enter search mode
        KeyCode::Char('/') => {
            app.state.ui.enter_sql_files_search();
        }
        // j/k - Navigate files
        KeyCode::Char('j') | KeyCode::Down => {
            app.state.update_sql_file_selection_for_filtered(1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.state.update_sql_file_selection_for_filtered(-1);
        }
        _ => {}
    }
    Ok(())
}

/// Handle SQL files search mode
async fn handle_search_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.state.ui.exit_sql_files_search();
        }
        KeyCode::Backspace => {
            app.state.ui.backspace_sql_files_search();
        }
        KeyCode::Enter => {
            if let Err(e) = app.state.load_selected_sql_file() {
                app.state
                    .toast_manager
                    .error(format!("Failed to load SQL file: {e}"));
            } else {
                app.state.toast_manager.success("SQL file loaded");
            }
            app.state.ui.exit_sql_files_search();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.state.update_sql_file_selection_for_filtered(1);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.state.update_sql_file_selection_for_filtered(-1);
        }
        KeyCode::Char(c) if !matches!(c, 'j' | 'k') => {
            app.state.ui.add_to_sql_files_search(c);
        }
        _ => {}
    }
    Ok(())
}

/// Handle SQL files rename mode
async fn handle_rename_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.state.ui.exit_sql_files_rename();
        }
        KeyCode::Backspace => {
            app.state.ui.backspace_sql_files_rename();
        }
        KeyCode::Enter => {
            let new_name = app.state.ui.sql_files_rename_buffer.clone();
            if !new_name.is_empty() {
                let filtered_files = app.state.get_filtered_sql_files();
                let selected_index = app.state.get_filtered_sql_file_selection();
                if let Some(old_name) = filtered_files.get(selected_index) {
                    if let Some(original_index) = app
                        .state
                        .saved_sql_files
                        .iter()
                        .position(|f| f == old_name)
                    {
                        if let Err(e) =
                            app.state.rename_sql_file(original_index, &new_name).await
                        {
                            app.state
                                .toast_manager
                                .error(format!("Failed to rename file: {e}"));
                        } else {
                            app.state
                                .toast_manager
                                .success("File renamed successfully");
                        }
                    }
                }
            }
            app.state.ui.exit_sql_files_rename();
        }
        KeyCode::Char(c) => {
            app.state.ui.add_to_sql_files_rename(c);
        }
        _ => {}
    }
    Ok(())
}

/// Handle SQL files create mode
async fn handle_create_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.state.ui.exit_sql_files_create();
        }
        KeyCode::Backspace => {
            app.state.ui.backspace_sql_files_create();
        }
        KeyCode::Enter => {
            let filename = app.state.ui.sql_files_create_buffer.clone();
            if !filename.is_empty() {
                if let Err(e) = app.state.create_sql_file(&filename).await {
                    app.state
                        .toast_manager
                        .error(format!("Failed to create file: {e}"));
                } else {
                    app.state
                        .toast_manager
                        .success("File created successfully");
                    // Load the new file
                    let _ = app.state.load_query_file(&filename);
                }
            }
            app.state.ui.exit_sql_files_create();
        }
        KeyCode::Char(c) => {
            app.state.ui.add_to_sql_files_create(c);
        }
        _ => {}
    }
    Ok(())
}
