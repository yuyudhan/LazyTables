// FilePath: src/app/handlers/connections.rs
//
// Event handlers for the Connections pane and connection modal

#![forbid(unsafe_code)]

use crate::{
    app::{App, ConnectionEvent, TestConnectionEvent},
    core::error::Result,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle Connections pane keys - DIRECT KEY BINDINGS (no insert mode)
pub(crate) async fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    // Search mode active - handle search input
    if app.state.ui.connections_search_active {
        match key.code {
            KeyCode::Esc => {
                app.state.ui.exit_connections_search();
            }
            KeyCode::Backspace => {
                app.state.ui.backspace_connections_search();
                app.state
                    .ui
                    .update_filtered_connections(&app.state.db.connections.connections);
            }
            KeyCode::Enter => {
                // Get selected connection index
                let selected_index = if let Some(index) = app
                    .state
                    .ui
                    .get_selected_connection_index(&app.state.db.connections.connections)
                {
                    index
                } else {
                    return Ok(()); // No connection selected
                };

                // Don't start new connection if one is already in progress
                if app.state.connecting_in_progress.is_some() {
                    app.state
                        .toast_manager
                        .warning("Connection attempt already in progress");
                    return Ok(());
                }

                // Mark connection as in progress
                app.state.connecting_in_progress = Some(selected_index);
                app.state.connecting_animation_frame = 0;
                app.state.connection_start_time = Some(std::time::Instant::now());

                // Set status to connecting immediately
                if let Some(conn) = app.state.db.connections.connections.get_mut(selected_index) {
                    conn.status = crate::database::ConnectionStatus::Connecting;
                    app.state
                        .toast_manager
                        .info(format!("Connecting to {}...", conn.name));
                }

                // Clone necessary data for background task
                let connection_config =
                    app.state.db.connections.connections[selected_index].clone();
                let connection_manager = app.state.connection_manager.clone();
                let tx = app.connection_events_tx.clone();

                // Spawn connection task in background
                tokio::spawn(async move {
                    // Attempt to establish connection
                    match connection_manager.connect(&connection_config).await {
                        Ok(_) => {
                            // Connection succeeded, now get database objects
                            match connection_manager
                                .list_database_objects(&connection_config.id)
                                .await
                            {
                                Ok(objects) => {
                                    // Send success event
                                    let _ = tx.send(ConnectionEvent::Success {
                                        connection_index: selected_index,
                                        objects,
                                    });
                                }
                                Err(e) => {
                                    // Connection succeeded but listing objects failed
                                    let _ = tx.send(ConnectionEvent::Failed {
                                        connection_index: selected_index,
                                        error: format!("Failed to load database objects: {}", e),
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            // Connection failed
                            let _ = tx.send(ConnectionEvent::Failed {
                                connection_index: selected_index,
                                error: e.to_string(),
                            });
                        }
                    }
                });

                app.state.ui.exit_connections_search();
            }
            KeyCode::Down => {
                app.state
                    .ui
                    .connections_selection_down(&app.state.db.connections.connections);
            }
            KeyCode::Up => {
                app.state
                    .ui
                    .connections_selection_up(&app.state.db.connections.connections);
            }
            KeyCode::Char(c) => {
                app.state.ui.add_to_connections_search(c);
                app.state
                    .ui
                    .update_filtered_connections(&app.state.db.connections.connections);
            }
            _ => {}
        }
        return Ok(());
    }

    // Normal mode - direct key bindings
    match key.code {
        // 'a' - Add new connection
        KeyCode::Char('a') => {
            app.state.open_add_connection_modal();
        }
        // 'e' - Edit selected connection
        KeyCode::Char('e') => {
            app.state.open_edit_connection_modal();
        }
        // 'd' - Delete selected connection
        KeyCode::Char('d') => {
            if !app.state.db.connections.connections.is_empty() {
                let index = app.state.ui.selected_connection;
                app.state.ui.confirmation_modal = Some(crate::ui::ConfirmationModal {
                    title: "Delete Connection".to_string(),
                    message: format!(
                        "Are you sure you want to delete the connection '{}'?",
                        app.state.db.connections.connections[index].name
                    ),
                    action: crate::ui::ConfirmationAction::DeleteConnection(index),
                });
            }
        }
        // Enter or Space - Connect to selected database
        KeyCode::Enter | KeyCode::Char(' ') => {
            // Get selected connection index
            let selected_index = if let Some(index) = app
                .state
                .ui
                .get_selected_connection_index(&app.state.db.connections.connections)
            {
                index
            } else {
                return Ok(()); // No connection selected
            };

            // Don't start new connection if one is already in progress
            if app.state.connecting_in_progress.is_some() {
                app.state
                    .toast_manager
                    .warning("Connection attempt already in progress");
                return Ok(());
            }

            // Mark connection as in progress
            app.state.connecting_in_progress = Some(selected_index);
            app.state.connecting_animation_frame = 0;
            app.state.connection_start_time = Some(std::time::Instant::now());

            // Set status to connecting immediately (for visual feedback)
            if let Some(conn) = app.state.db.connections.connections.get_mut(selected_index) {
                conn.status = crate::database::ConnectionStatus::Connecting;
                app.state
                    .toast_manager
                    .info(format!("Connecting to {}...", conn.name));
            }

            // Clone necessary data for background task
            let connection_config = app.state.db.connections.connections[selected_index].clone();
            let connection_manager = app.state.connection_manager.clone();
            let tx = app.connection_events_tx.clone();

            // Spawn connection task in background
            tokio::spawn(async move {
                // Attempt to establish connection
                match connection_manager.connect(&connection_config).await {
                    Ok(_) => {
                        // Connection succeeded, now get database objects
                        match connection_manager
                            .list_database_objects(&connection_config.id)
                            .await
                        {
                            Ok(objects) => {
                                // Send success event
                                let _ = tx.send(ConnectionEvent::Success {
                                    connection_index: selected_index,
                                    objects,
                                });
                            }
                            Err(e) => {
                                // Connection succeeded but listing objects failed
                                let _ = tx.send(ConnectionEvent::Failed {
                                    connection_index: selected_index,
                                    error: format!("Failed to load database objects: {}", e),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        // Connection failed
                        let _ = tx.send(ConnectionEvent::Failed {
                            connection_index: selected_index,
                            error: e.to_string(),
                        });
                    }
                }
            });
        }
        // 'r' - Refresh connections list
        KeyCode::Char('r') => {
            app.state.toast_manager.info("Connections refreshed");
        }
        // 'x' - Disconnect from current database
        KeyCode::Char('x') => {
            let selected = app.state.ui.selected_connection;
            if let Some(connection) = app.state.db.connections.connections.get(selected).cloned() {
                let connection_id = connection.id.clone();
                let connection_name = connection.name.clone();

                // Disconnect from the database
                let _ = app
                    .state
                    .connection_manager
                    .disconnect(&connection_id)
                    .await;
                app.state.disconnect_from_database().await;

                app.state
                    .toast_manager
                    .info(format!("Disconnected from {}", connection_name));
            }
        }
        // '/' - Enter search mode
        KeyCode::Char('/') => {
            app.state.ui.enter_connections_search();
        }
        // j/k or arrow keys - Navigate
        KeyCode::Char('j') | KeyCode::Down => {
            app.state
                .ui
                .connections_selection_down(&app.state.db.connections.connections);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.state
                .ui
                .connections_selection_up(&app.state.db.connections.connections);
        }
        _ => {}
    }
    Ok(())
}

/// Handle connection modal key event
pub(crate) async fn handle_connection_modal(app: &mut App, key: KeyEvent) -> Result<()> {
    use crate::ui::components::{ConnectionField, PasswordStorageType};

    match key.code {
        // PRIORITY 0: Abort test connection (Ctrl+C - highest priority)
        KeyCode::Char('c')
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && app.state.test_connection_in_progress =>
        {
            abort_test_connection(app);
            return Ok(());
        }

        // PRIORITY 1: Global shortcuts (work from any field EXCEPT text input fields)
        KeyCode::Char('t')
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !app.state.connection_modal_state.is_text_field() =>
        {
            // Plain 't': Test connection shortcut
            test_connection_from_modal(app).await;
        }
        KeyCode::Char('s') if !app.state.connection_modal_state.is_text_field() => {
            // Save shortcut - works from any field except text input fields
            if let Err(error) = app.state.save_connection_from_modal().await {
                app.state
                    .toast_manager
                    .error(format!("Failed to save connection: {}", &error));
                app.state.connection_modal_state.error_message = Some(error);
            } else {
                app.state
                    .toast_manager
                    .success("Connection saved successfully");
            }
        }
        KeyCode::Char('c') if !app.state.connection_modal_state.is_text_field() => {
            // Cancel shortcut - works from any field except text input fields
            if app.state.ui.current_view.is_connection_form() {
                app.state.close_add_connection_modal();
            } else {
                app.state.close_edit_connection_modal();
            }
        }

        // PRIORITY 2: Navigation and special keys
        KeyCode::Esc => {
            // Close the appropriate modal
            if app.state.ui.current_view.is_connection_form() {
                app.state.close_add_connection_modal();
            } else {
                app.state.close_edit_connection_modal();
            }
        }
        KeyCode::Tab => {
            // Tab for next field navigation
            app.state.connection_modal_state.focused_field =
                app.state.connection_modal_state.get_smart_next_field();
        }
        KeyCode::BackTab => {
            // Shift+Tab for previous field navigation
            app.state.connection_modal_state.focused_field =
                app.state.connection_modal_state.get_smart_previous_field();
        }
        // Arrow keys for navigation within sections
        KeyCode::Down => {
            // Handle database type and other dropdowns specially
            match app.state.connection_modal_state.focused_field {
                ConnectionField::DatabaseType => {
                    // Navigate database type dropdown
                    let current = app
                        .state
                        .connection_modal_state
                        .db_type_list_state
                        .selected()
                        .unwrap_or(0);
                    let max_types = 4; // PostgreSQL, MySQL, MariaDB, SQLite
                    let new_index = if current + 1 < max_types {
                        current + 1
                    } else {
                        0
                    };
                    app.state
                        .connection_modal_state
                        .select_database_type(new_index);
                }
                ConnectionField::SslMode => {
                    // Navigate SSL mode dropdown
                    let current = app
                        .state
                        .connection_modal_state
                        .ssl_list_state
                        .selected()
                        .unwrap_or(0);
                    let max_modes = 6; // All SSL modes
                    let new_index = if current + 1 < max_modes {
                        current + 1
                    } else {
                        0
                    };
                    app.state.connection_modal_state.select_ssl_mode(new_index);
                }
                ConnectionField::PasswordStorageType => {
                    // Cycle through password storage types
                    app.state
                        .connection_modal_state
                        .cycle_password_storage_type();
                }
                _ => {
                    // For other fields, move to next field
                    app.state.connection_modal_state.focused_field =
                        app.state.connection_modal_state.get_smart_next_field();
                }
            }
        }
        KeyCode::Up => {
            // Handle database type and other dropdowns specially
            match app.state.connection_modal_state.focused_field {
                ConnectionField::DatabaseType => {
                    // Navigate database type dropdown
                    let current = app
                        .state
                        .connection_modal_state
                        .db_type_list_state
                        .selected()
                        .unwrap_or(0);
                    let max_types = 4; // PostgreSQL, MySQL, MariaDB, SQLite
                    let new_index = if current > 0 {
                        current - 1
                    } else {
                        max_types - 1
                    };
                    app.state
                        .connection_modal_state
                        .select_database_type(new_index);
                }
                ConnectionField::SslMode => {
                    // Navigate SSL mode dropdown
                    let current = app
                        .state
                        .connection_modal_state
                        .ssl_list_state
                        .selected()
                        .unwrap_or(0);
                    let max_modes = 6; // All SSL modes
                    let new_index = if current > 0 {
                        current - 1
                    } else {
                        max_modes - 1
                    };
                    app.state.connection_modal_state.select_ssl_mode(new_index);
                }
                ConnectionField::PasswordStorageType => {
                    // Cycle backwards through password storage types
                    app.state.connection_modal_state.password_storage_type =
                        match app.state.connection_modal_state.password_storage_type {
                            PasswordStorageType::PlainText => PasswordStorageType::Encrypted,
                            PasswordStorageType::Environment => PasswordStorageType::PlainText,
                            PasswordStorageType::Encrypted => PasswordStorageType::Environment,
                        };
                }
                _ => {
                    // For other fields, move to previous field
                    app.state.connection_modal_state.focused_field =
                        app.state.connection_modal_state.get_smart_previous_field();
                }
            }
        }
        KeyCode::Enter => {
            // Handle Enter on button fields specially
            match app.state.connection_modal_state.focused_field {
                ConnectionField::Test => {
                    // Activate Test button
                    test_connection_from_modal(app).await;
                }
                ConnectionField::Save => {
                    // Activate Save button
                    if let Err(error) = app.state.save_connection_from_modal().await {
                        app.state
                            .toast_manager
                            .error(format!("Failed to save connection: {}", &error));
                        app.state.connection_modal_state.error_message = Some(error);
                    } else {
                        app.state
                            .toast_manager
                            .success("Connection saved successfully");
                    }
                }
                ConnectionField::Cancel => {
                    // Activate Cancel button
                    if app.state.ui.current_view.is_connection_form() {
                        app.state.close_add_connection_modal();
                    } else {
                        app.state.close_edit_connection_modal();
                    }
                }
                _ => {
                    // For all other fields, Enter moves to next field
                    app.state.connection_modal_state.next_field();
                }
            }
        }
        // Ctrl+T: Toggle between connection string and individual fields
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.state.connection_modal_state.using_connection_string =
                !app.state.connection_modal_state.using_connection_string;

            // Clear the opposite fields when switching
            if app.state.connection_modal_state.using_connection_string {
                // Clear individual fields when switching to connection string
                app.state.connection_modal_state.host = "localhost".to_string();
                app.state.connection_modal_state.port_input =
                    match app.state.connection_modal_state.database_type {
                        crate::database::DatabaseType::PostgreSQL => "5432".to_string(),
                        crate::database::DatabaseType::MySQL
                        | crate::database::DatabaseType::MariaDB => "3306".to_string(),
                        _ => "5432".to_string(),
                    };
                app.state.connection_modal_state.database.clear();
                app.state.connection_modal_state.username.clear();
                app.state.connection_modal_state.password.clear();
            } else {
                // Clear connection string when switching to individual fields
                app.state.connection_modal_state.connection_string.clear();
            }

            // Clear any test status when switching input methods
            app.state.connection_modal_state.test_status = None;
        }

        // PRIORITY 3: Text input for text fields (lowest priority, after shortcuts)
        KeyCode::Char(c) if app.state.connection_modal_state.is_text_field() => {
            app.state.connection_modal_state.handle_char_input(c);
        }
        KeyCode::Backspace if app.state.connection_modal_state.is_text_field() => {
            app.state.connection_modal_state.handle_backspace();
        }
        _ => {}
    }
    Ok(())
}

/// Test connection from modal
async fn test_connection_from_modal(app: &mut App) {
    use crate::ui::components::TestConnectionStatus;

    // Don't start new test if one is already in progress
    if app.state.test_connection_in_progress {
        app.state.toast_manager.warning("Test already in progress");
        return;
    }

    // Set status to testing and start timer
    app.state.connection_modal_state.test_status = Some(TestConnectionStatus::Testing);
    app.state.test_connection_in_progress = true;
    app.state.test_animation_frame = 0;
    app.state.test_start_time = Some(std::time::Instant::now());

    // Try to create a connection config (no uniqueness check needed for testing)
    let config = match app
        .state
        .connection_modal_state
        .try_create_connection(&[], None)
    {
        Ok(config) => config,
        Err(e) => {
            // Invalid config - send error immediately
            let _ = app
                .test_connection_events_tx
                .send(TestConnectionEvent::Failed(format!(
                    "Invalid configuration: {e}"
                )));
            return;
        }
    };

    // Clone sender for background task
    let tx = app.test_connection_events_tx.clone();

    // Spawn background task to test connection and store handle for abort capability
    let handle = tokio::spawn(async move {
        use crate::core::error::LazyTablesError;
        use crate::database::{Connection, DatabaseType};

        let result = match config.database_type {
            DatabaseType::PostgreSQL => {
                use crate::database::postgres::PostgresConnection;
                let mut conn = PostgresConnection::new(config);

                match conn.connect().await {
                    Ok(()) => {
                        // Connection succeeded, now test it
                        conn.test_connection()
                            .await
                            .map(|_| "Connection successful!".to_string())
                    }
                    Err(e) => {
                        // Parse error into structured ConnectionError
                        if let LazyTablesError::Database(ref sqlx_err) = e {
                            Err(LazyTablesError::ConnectionFailed(
                                conn.parse_connection_error(sqlx_err),
                            ))
                        } else {
                            Err(e)
                        }
                    }
                }
            }
            DatabaseType::MySQL | DatabaseType::MariaDB => {
                use crate::database::mysql::MySqlConnection;
                let mut conn = MySqlConnection::new(config);

                match conn.connect().await {
                    Ok(()) => {
                        // Connection succeeded, now test it
                        conn.test_connection()
                            .await
                            .map(|_| "Connection successful!".to_string())
                    }
                    Err(e) => {
                        // Parse error into structured ConnectionError
                        if let LazyTablesError::Database(ref sqlx_err) = e {
                            Err(LazyTablesError::ConnectionFailed(
                                conn.parse_connection_error(sqlx_err),
                            ))
                        } else {
                            Err(e)
                        }
                    }
                }
            }
            DatabaseType::SQLite => {
                use crate::database::sqlite::SqliteConnection;
                let mut conn = SqliteConnection::new(config);

                match conn.connect().await {
                    Ok(()) => {
                        // Connection succeeded, now test it
                        conn.test_connection()
                            .await
                            .map(|_| "Connection successful!".to_string())
                    }
                    Err(e) => {
                        // Parse error into structured ConnectionError
                        if let LazyTablesError::Database(ref sqlx_err) = e {
                            Err(LazyTablesError::ConnectionFailed(
                                conn.parse_connection_error(sqlx_err),
                            ))
                        } else {
                            Err(e)
                        }
                    }
                }
            }
            _ => Err(LazyTablesError::Connection(
                "Database type not yet supported".to_string(),
            )),
        };

        // Send result back to main loop with properly formatted errors
        let event = match result {
            Ok(msg) => TestConnectionEvent::Success(msg),
            Err(e) => {
                // Format error for display
                let error_message = match &e {
                    LazyTablesError::ConnectionFailed(conn_err) => {
                        // Use our structured error's formatted display
                        conn_err.format_for_display()
                    }
                    _ => e.to_string(),
                };
                TestConnectionEvent::Failed(error_message)
            }
        };

        let _ = tx.send(event);
    });

    // Store task handle for abort capability
    app.test_connection_task_handle = Some(handle);
}

/// Abort ongoing test connection
fn abort_test_connection(app: &mut App) {
    use crate::ui::components::TestConnectionStatus;

    // Only abort if test is actually in progress
    if !app.state.test_connection_in_progress {
        return;
    }

    // Abort the background task if it exists
    if let Some(handle) = app.test_connection_task_handle.take() {
        handle.abort();
    }

    // Update status to show test was aborted
    app.state.connection_modal_state.test_status = Some(TestConnectionStatus::Failed(
        "Test aborted by user".to_string(),
    ));

    // Clear all test-related state
    app.state.test_connection_in_progress = false;
    app.state.test_start_time = None;
    app.state.test_animation_frame = 0;

    // Notify user
    app.state.toast_manager.warning("Connection test aborted");
}
