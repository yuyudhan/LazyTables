// FilePath: src/commands/connection.rs

use super::{
    Command, CommandAction, CommandCategory, CommandContext, CommandId, CommandResult, ModalType,
};
use crate::core::error::Result;

/// Connect command - establishes database connection
pub struct ConnectCommand;

impl Command for ConnectCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        // Check if we're in the connections pane
        if context.state.ui.focused_pane != crate::app::FocusedPane::Connections {
            return Ok(CommandResult::Error(
                "Navigate to Connections pane to connect".to_string(),
            ));
        }

        // Check if a connection is selected
        if !context.state.db.connections.connections.is_empty() {
            let selected = context.state.ui.selected_connection;

            if let Some(connection) = context.state.db.connections.connections.get_mut(selected) {
                // Check if already connected
                if connection.is_connected() {
                    return Ok(CommandResult::Error(format!(
                        "Already connected to {}",
                        connection.name
                    )));
                }

                // Set status to connecting
                connection.status = crate::database::ConnectionStatus::Connecting;
                context
                    .state
                    .toast_manager
                    .info(format!("Connecting to {}...", connection.name));

                // Spawn async connection task using the connection manager
                let connection_config = connection.clone();
                let result = tokio::runtime::Handle::current().block_on(async {
                    // First establish the persistent connection
                    match context
                        .state
                        .connection_manager
                        .connect(&connection_config)
                        .await
                    {
                        Ok(_) => {
                            // Then load database objects to verify the connection works
                            context
                                .state
                                .connection_manager
                                .list_database_objects(&connection_config.id)
                                .await
                                .inspect(|objects| {
                                    // Update database state with loaded objects
                                    context.state.db.database_objects = Some(objects.clone());
                                    context.state.db.tables = objects
                                        .tables
                                        .iter()
                                        .map(|t| {
                                            if t.schema.as_deref() == Some("public")
                                                || t.schema.is_none()
                                            {
                                                t.name.clone()
                                            } else {
                                                t.qualified_name()
                                            }
                                        })
                                        .collect();
                                })
                        }
                        Err(e) => Err(crate::core::error::LazyTablesError::Connection(format!(
                            "Connection failed: {e}"
                        ))),
                    }
                });

                match result {
                    Ok(_database_objects) => {
                        // Update connection status to connected
                        if let Some(conn) =
                            context.state.db.connections.connections.get_mut(selected)
                        {
                            conn.status = crate::database::ConnectionStatus::Connected;
                        }
                        context
                            .state
                            .toast_manager
                            .success(format!("Connected to {}", connection_config.name));
                        Ok(CommandResult::SuccessWithMessage(format!(
                            "Connected to {}",
                            connection_config.name
                        )))
                    }
                    Err(error) => {
                        // Update connection status to failed
                        if let Some(conn) =
                            context.state.db.connections.connections.get_mut(selected)
                        {
                            conn.status =
                                crate::database::ConnectionStatus::Failed(error.to_string());
                        }
                        context
                            .state
                            .toast_manager
                            .error(format!("Connection failed: {}", error));
                        Ok(CommandResult::Error(format!(
                            "Connection failed: {}",
                            error
                        )))
                    }
                }
            } else {
                Ok(CommandResult::Error(
                    "Invalid connection selection".to_string(),
                ))
            }
        } else {
            Ok(CommandResult::Error("No connections available".to_string()))
        }
    }

    fn description(&self) -> &str {
        "Connect to selected database"
    }

    fn id(&self) -> CommandId {
        CommandId::Connect
    }

    fn shortcut(&self) -> Option<String> {
        Some("Enter".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Connection
    }

    fn can_execute(&self, context: &CommandContext) -> bool {
        context.state.ui.focused_pane == crate::app::FocusedPane::Connections
            && !context.state.db.connections.connections.is_empty()
    }
}

/// Add Connection command - opens connection creation modal
pub struct AddConnectionCommand;

impl Command for AddConnectionCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        // Initialize connection modal state
        use crate::ui::components::connection_modal::{ConnectionModalState, ModalStep};

        context.state.connection_modal_state = ConnectionModalState::new();
        context.state.connection_modal_state.current_step = ModalStep::DatabaseTypeSelection;
        context.state.ui.show_add_connection_modal = true;

        Ok(CommandResult::Action(CommandAction::OpenModal(
            ModalType::Connection,
        )))
    }

    fn description(&self) -> &str {
        "Add a new database connection"
    }

    fn id(&self) -> CommandId {
        CommandId::AddConnection
    }

    fn shortcut(&self) -> Option<String> {
        Some("a".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Connection
    }

    fn can_execute(&self, context: &CommandContext) -> bool {
        // Can add connection from connections pane
        context.state.ui.focused_pane == crate::app::FocusedPane::Connections
    }
}

/// Edit Connection command
pub struct EditConnectionCommand;

impl Command for EditConnectionCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        if !context.state.db.connections.connections.is_empty() {
            let selected = context.state.ui.selected_connection;

            if let Some(connection) = context.state.db.connections.connections.get(selected) {
                // Load existing connection data into modal
                use crate::ui::components::connection_modal::{ConnectionModalState, ModalStep};

                let mut modal_state = ConnectionModalState::new();

                // Pre-populate fields with existing connection data
                modal_state.name = connection.name.clone();
                modal_state.database_type = connection.database_type.clone();
                modal_state.host = connection.host.clone();
                modal_state.port_input = connection.port.to_string();
                modal_state.database = connection.database.clone().unwrap_or_default();
                modal_state.username = connection.username.clone();
                modal_state.ssl_mode = connection.ssl_mode.clone();

                // Set to connection details step since we already know the database type
                modal_state.current_step = ModalStep::ConnectionDetails;

                // Note: Password is not pre-populated for security reasons
                modal_state.password = String::new();

                context.state.connection_modal_state = modal_state;
                context.state.ui.show_edit_connection_modal = true;

                Ok(CommandResult::SuccessWithMessage(format!(
                    "Editing connection: {}",
                    connection.name
                )))
            } else {
                Ok(CommandResult::Error(
                    "Invalid connection selection".to_string(),
                ))
            }
        } else {
            Ok(CommandResult::Error("No connections available".to_string()))
        }
    }

    fn description(&self) -> &str {
        "Edit selected database connection"
    }

    fn id(&self) -> CommandId {
        CommandId::EditConnection
    }

    fn shortcut(&self) -> Option<String> {
        Some("e".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Connection
    }

    fn can_execute(&self, context: &CommandContext) -> bool {
        context.state.ui.focused_pane == crate::app::FocusedPane::Connections
            && !context.state.db.connections.connections.is_empty()
    }
}

/// Delete Connection command
pub struct DeleteConnectionCommand;

impl Command for DeleteConnectionCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        if !context.state.db.connections.connections.is_empty() {
            let selected = context.state.ui.selected_connection;
            Ok(CommandResult::RequiresConfirmation(format!(
                "Delete connection #{selected}? This cannot be undone."
            )))
        } else {
            Ok(CommandResult::Error("No connections available".to_string()))
        }
    }

    fn description(&self) -> &str {
        "Delete selected database connection"
    }

    fn id(&self) -> CommandId {
        CommandId::DeleteConnection
    }

    fn shortcut(&self) -> Option<String> {
        Some("d".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Connection
    }

    fn can_execute(&self, context: &CommandContext) -> bool {
        context.state.ui.focused_pane == crate::app::FocusedPane::Connections
            && !context.state.db.connections.connections.is_empty()
    }
}

/// Disconnect command
pub struct DisconnectCommand;

impl Command for DisconnectCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        // Get the currently selected connection
        let selected = context.state.ui.selected_connection;
        if let Some(connection) = context
            .state
            .db
            .connections
            .connections
            .get(selected)
            .cloned()
        {
            // Perform actual disconnection using the connection manager
            let connection_id = connection.id.clone();
            let connection_name = connection.name.clone();

            tokio::runtime::Handle::current().block_on(async {
                let _ = context
                    .state
                    .connection_manager
                    .disconnect(&connection_id)
                    .await;
            });

            // Update UI state synchronously
            context.state.disconnect_from_database_sync();

            context
                .state
                .toast_manager
                .info(format!("Disconnected from {}", connection_name));

            Ok(CommandResult::SuccessWithMessage(format!(
                "Disconnected from {}",
                connection_name
            )))
        } else {
            context
                .state
                .toast_manager
                .info("Disconnected from database");
            Ok(CommandResult::Success)
        }
    }

    fn description(&self) -> &str {
        "Disconnect from current database"
    }

    fn id(&self) -> CommandId {
        CommandId::Disconnect
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Connection
    }
}

/// Refresh Connections command
pub struct RefreshConnectionsCommand;

impl Command for RefreshConnectionsCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        // Reload connections from storage
        match crate::database::connection::ConnectionStorage::load() {
            Ok(storage) => {
                let old_count = context.state.db.connections.connections.len();
                context.state.db.connections = storage;
                let new_count = context.state.db.connections.connections.len();

                // Reset selection if it's out of bounds
                if context.state.ui.selected_connection >= new_count && new_count > 0 {
                    context.state.ui.selected_connection = new_count - 1;
                } else if new_count == 0 {
                    context.state.ui.selected_connection = 0;
                }

                let message = if new_count != old_count {
                    format!(
                        "Connections refreshed: {} connections (was {})",
                        new_count, old_count
                    )
                } else {
                    format!("Connections refreshed: {} connections", new_count)
                };

                context.state.toast_manager.success(&message);
                Ok(CommandResult::SuccessWithMessage(message))
            }
            Err(error) => {
                let error_msg = format!("Failed to refresh connections: {}", error);
                context.state.toast_manager.error(&error_msg);
                Ok(CommandResult::Error(error_msg))
            }
        }
    }

    fn description(&self) -> &str {
        "Refresh connection list"
    }

    fn id(&self) -> CommandId {
        CommandId::RefreshConnections
    }

    fn shortcut(&self) -> Option<String> {
        Some("r".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Connection
    }
}

/// Test Connection command
pub struct TestConnectionCommand;

impl Command for TestConnectionCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        if !context.state.db.connections.connections.is_empty() {
            let selected = context.state.ui.selected_connection;

            if let Some(connection) = context.state.db.connections.connections.get(selected) {
                context
                    .state
                    .toast_manager
                    .info(format!("Testing connection to {}...", connection.name));

                // Perform connection test using try_connect_to_database
                let connection_config = connection.clone();
                let start_time = std::time::Instant::now();

                let result = tokio::runtime::Handle::current().block_on(async {
                    context
                        .state
                        .db
                        .try_connect_to_database(
                            &connection_config,
                            &context.state.connection_manager,
                        )
                        .await
                });

                let test_duration = start_time.elapsed();

                match result {
                    Ok(database_objects) => {
                        let tables_count = database_objects.tables.len();
                        let views_count = database_objects.views.len();
                        let total_objects = database_objects.total_count;

                        let success_message = format!(
                            "✓ Connection test successful!\n\nDatabase: {}\nHost: {}:{}\nObjects found: {} (Tables: {}, Views: {})\nResponse time: {:.2}ms",
                            connection_config.database.as_deref().unwrap_or("default"),
                            connection_config.host,
                            connection_config.port,
                            total_objects,
                            tables_count,
                            views_count,
                            test_duration.as_millis()
                        );

                        context.state.toast_manager.success(format!(
                            "Connection test passed - {}ms response",
                            test_duration.as_millis()
                        ));

                        Ok(CommandResult::SuccessWithMessage(success_message))
                    }
                    Err(error) => {
                        let error_message = format!(
                            "✗ Connection test failed!\n\nHost: {}:{}\nDatabase: {}\nError: {}\nResponse time: {:.2}ms\n\nTroubleshooting:\n• Check if the database server is running\n• Verify host and port are correct\n• Ensure credentials are valid\n• Check network connectivity",
                            connection_config.host,
                            connection_config.port,
                            connection_config.database.as_deref().unwrap_or("default"),
                            error,
                            test_duration.as_millis()
                        );

                        context.state.toast_manager.error(format!(
                            "Connection test failed: {}",
                            error.split('\n').next().unwrap_or(&error)
                        ));

                        Ok(CommandResult::Error(error_message))
                    }
                }
            } else {
                Ok(CommandResult::Error(
                    "Invalid connection selection".to_string(),
                ))
            }
        } else {
            Ok(CommandResult::Error("No connections available".to_string()))
        }
    }

    fn description(&self) -> &str {
        "Test selected database connection"
    }

    fn id(&self) -> CommandId {
        CommandId::TestConnection
    }

    fn shortcut(&self) -> Option<String> {
        Some("t".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Connection
    }

    fn can_execute(&self, context: &CommandContext) -> bool {
        context.state.ui.focused_pane == crate::app::FocusedPane::Connections
            && !context.state.db.connections.connections.is_empty()
    }
}
