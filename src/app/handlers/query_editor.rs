// FilePath: src/app/handlers/query_editor.rs
//
// Event handlers for the Query Editor pane (VIM-style SQL editor)

#![forbid(unsafe_code)]

use crate::{app::App, core::error::Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle Query Editor pane keys - ONLY PANE WITH VIM INSERT MODE
pub(crate) async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    // Check if in command mode
    if app.state.query_editor.is_in_command_mode() {
        return handle_command_mode(app, key).await;
    }

    // Check if query editor is in insert mode
    if app.state.query_editor.is_insert_mode() {
        return handle_insert_mode(app, key).await;
    }

    // Normal mode - vim keybindings
    match key.code {
        // Shift+E - Execute query at cursor (PRIMARY binding, vim-style)
        KeyCode::Char('E') => {
            if let Err(e) = app.state.execute_query_at_cursor().await {
                app.state
                    .toast_manager
                    .error(format!("Query execution failed: {e}"));
            }
        }
        // Ctrl+Enter - Execute query at cursor (SECONDARY binding, familiar to SQL tool users)
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Err(e) = app.state.execute_query_at_cursor().await {
                app.state
                    .toast_manager
                    .error(format!("Query execution failed: {e}"));
            }
        }
        // 'i' - Enter insert mode at cursor
        KeyCode::Char('i') => {
            app.state.query_editor.set_insert_mode(true);
        }
        // 'a' - Enter insert mode after cursor
        KeyCode::Char('a') => {
            app.state.query_editor.move_cursor_right();
            app.state.query_editor.set_insert_mode(true);
        }
        // 'o' - New line below + insert mode
        KeyCode::Char('o') => {
            app.state.query_editor.insert_newline();
            app.state.query_editor.set_insert_mode(true);
        }
        // 'O' - New line above + insert mode
        KeyCode::Char('O') => {
            app.state.query_editor.move_cursor_up();
            app.state.query_editor.move_to_line_end();
            app.state.query_editor.insert_newline();
            app.state.query_editor.set_insert_mode(true);
        }
        // Vim motions
        KeyCode::Char('h') | KeyCode::Left => {
            app.state.query_editor.move_cursor_left();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.state.query_editor.move_cursor_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.state.query_editor.move_cursor_up();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.state.query_editor.move_cursor_right();
        }
        KeyCode::Char('w') => {
            app.state.query_editor.move_to_next_word();
        }
        KeyCode::Char('b') => {
            app.state.query_editor.move_to_prev_word();
        }
        KeyCode::Char('e') => {
            app.state.query_editor.move_to_end_of_word();
        }
        KeyCode::Char('0') => {
            app.state.query_editor.move_to_line_start();
        }
        KeyCode::Char('$') => {
            app.state.query_editor.move_to_line_end();
        }
        KeyCode::Char('g') => {
            if app.state.ui.pending_gg_command {
                app.state.query_editor.move_to_file_start();
                app.state.ui.pending_gg_command = false;
            } else {
                app.state.ui.pending_gg_command = true;
            }
        }
        KeyCode::Char('G') => {
            app.state.query_editor.move_to_file_end();
        }
        // ':' - Enter command mode
        KeyCode::Char(':') => {
            app.state.query_editor.enter_command_mode();
        }
        // Ctrl+d and Ctrl+u for page scrolling - TODO: implement scroll methods
        // KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
        //     app.state.query_editor.scroll_half_page_down();
        // }
        // KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
        //     app.state.query_editor.scroll_half_page_up();
        // }
        _ => {}
    }
    Ok(())
}

/// Handle query editor insert mode
async fn handle_insert_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        // Esc - Exit insert mode
        KeyCode::Esc => {
            if app.state.query_editor.are_suggestions_active() {
                app.state.query_editor.hide_suggestions();
            } else {
                app.state.query_editor.set_insert_mode(false);
            }
        }
        // Enter - Insert newline (Ctrl+Enter does NOT work in insert mode - use normal mode)
        KeyCode::Enter => {
            app.state.query_editor.insert_newline();
            app.state.query_content = app.state.query_editor.get_content().to_string();
            app.state.ui.query_modified = true;
        }
        // Ctrl+p - Navigate suggestions up (vim-style) - MUST come before Char(c) pattern
        KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
            if app.state.query_editor.are_suggestions_active() {
                app.state.query_editor.move_suggestion_up();
            }
        }
        // Ctrl+n - Navigate suggestions down (vim-style) - MUST come before Char(c) pattern
        KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
            if app.state.query_editor.are_suggestions_active() {
                app.state.query_editor.move_suggestion_down();
            }
        }
        // Regular typing
        KeyCode::Char(c) => {
            app.state.query_editor.insert_char(c);
            app.state.query_content = app.state.query_editor.get_content().to_string();
            app.state.ui.query_modified = true;
        }
        // Backspace
        KeyCode::Backspace => {
            app.state.query_editor.backspace();
            app.state.query_content = app.state.query_editor.get_content().to_string();
            app.state.ui.query_modified = true;
        }
        // Tab - Accept suggestion if active, otherwise insert tab character
        KeyCode::Tab => {
            if app.state.query_editor.are_suggestions_active() {
                app.state.query_editor.accept_suggestion();
                app.state.query_content = app.state.query_editor.get_content().to_string();
                app.state.ui.query_modified = true;
            } else {
                app.state.query_editor.insert_char('\t');
                app.state.query_content = app.state.query_editor.get_content().to_string();
                app.state.ui.query_modified = true;
            }
        }
        // Up arrow - Navigate suggestions or move cursor
        KeyCode::Up => {
            if app.state.query_editor.are_suggestions_active() {
                app.state.query_editor.move_suggestion_up();
            } else {
                app.state.query_editor.move_cursor_up();
            }
        }
        // Down arrow - Navigate suggestions or move cursor
        KeyCode::Down => {
            if app.state.query_editor.are_suggestions_active() {
                app.state.query_editor.move_suggestion_down();
            } else {
                app.state.query_editor.move_cursor_down();
            }
        }
        _ => {}
    }
    Ok(())
}

/// Handle query editor command mode (vim : commands)
async fn handle_command_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        // Esc - Exit command mode
        KeyCode::Esc => {
            app.state.query_editor.exit_command_mode();
        }
        // Backspace - Remove character from command buffer
        KeyCode::Backspace => {
            app.state.query_editor.backspace_command_buffer();
        }
        // Enter - Execute command
        KeyCode::Enter => {
            let command = app.state.query_editor.get_command_buffer().to_string();
            app.state.query_editor.exit_command_mode();

            // Parse and execute command
            match command.trim() {
                ":w" => {
                    // Save file
                    if let Err(e) = app.state.save_sql_file_with_connection().await {
                        app.state
                            .toast_manager
                            .error(format!("Failed to save file: {}", e));
                    } else {
                        app.state.query_editor.mark_saved();
                        app.state.toast_manager.success("File saved successfully");
                    }
                }
                ":q" => {
                    // Clear editor (with confirmation if modified)
                    if app.state.query_editor.is_modified() {
                        app.state
                            .toast_manager
                            .warning("No write since last change (use :q! to force)");
                    } else {
                        app.state.query_editor.reset();
                        app.state.toast_manager.info("Editor cleared");
                    }
                }
                ":q!" => {
                    // Force clear editor
                    app.state.query_editor.reset();
                    app.state.toast_manager.info("Editor cleared");
                }
                ":wq" => {
                    // Save and clear
                    if let Err(e) = app.state.save_sql_file_with_connection().await {
                        app.state
                            .toast_manager
                            .error(format!("Failed to save file: {}", e));
                    } else {
                        app.state.query_editor.mark_saved();
                        app.state.query_editor.reset();
                        app.state
                            .toast_manager
                            .success("File saved and editor cleared");
                    }
                }
                cmd if cmd.starts_with(":w ") => {
                    // Save with filename - future enhancement
                    app.state
                        .toast_manager
                        .warning("Save with filename not yet implemented");
                }
                _ => {
                    app.state
                        .toast_manager
                        .error(format!("Unknown command: {}", command));
                }
            }
        }
        // Regular typing - add to command buffer
        KeyCode::Char(c) => {
            app.state.query_editor.add_to_command_buffer(c);
        }
        _ => {}
    }
    Ok(())
}
