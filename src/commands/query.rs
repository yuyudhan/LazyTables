// FilePath: src/commands/query.rs

use super::{Command, CommandAction, CommandCategory, CommandContext, CommandId, CommandResult};
use crate::core::error::Result;
use crate::database::DatabaseType;

/// Execute query command
pub struct ExecuteQueryCommand;

impl Command for ExecuteQueryCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        // Get the query to execute from the current query editor
        let query = if !context.state.query_content.is_empty() {
            context.state.query_content.clone()
        } else {
            return Ok(CommandResult::Error("No query to execute".to_string()));
        };

        // Find the active connection
        let active_connection = context
            .state
            .db
            .connections
            .connections
            .iter()
            .find(|conn| conn.is_connected());

        let connection_config = match active_connection {
            Some(conn) => conn,
            None => {
                return Ok(CommandResult::Error(
                    "No active database connection".to_string(),
                ));
            }
        };

        // Get database type for context-aware execution
        let database_type = connection_config.database_type.clone();

        // Validate query syntax based on database type
        if let Err(validation_error) = validate_query_syntax(&query, &database_type) {
            return Ok(CommandResult::Error(format!(
                "Query validation failed for {}: {}",
                database_type.display_name(),
                validation_error
            )));
        }

        // Add info toast with database context
        context.state.toast_manager.info(format!(
            "Executing {} query: {}",
            database_type.display_name(),
            query.lines().next().unwrap_or("").chars().take(50).collect::<String>()
        ));

        // Create enhanced action with database context
        Ok(CommandResult::Action(CommandAction::ExecuteQueryWithContext {
            query,
            database_type,
            connection_name: connection_config.name.clone(),
        }))
    }

    fn description(&self) -> &str {
        "Execute current SQL query"
    }

    fn id(&self) -> CommandId {
        CommandId::ExecuteQuery
    }

    fn shortcut(&self) -> Option<String> {
        Some("Ctrl+Enter".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Query
    }

    fn can_execute(&self, context: &CommandContext) -> bool {
        // Can execute if there's a query and active connection
        let has_query = !context.state.query_content.is_empty();

        let has_connection = context
            .state
            .db
            .connections
            .connections
            .iter()
            .any(|conn| conn.is_connected());

        has_query && has_connection
    }
}

/// Save query command
pub struct SaveQueryCommand;

impl Command for SaveQueryCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        if context.state.query_content.is_empty() {
            return Ok(CommandResult::Error("No query to save".to_string()));
        }
        let query = &context.state.query_content;

        // Generate filename with timestamp
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = if let Some(ref current_file) = context.state.ui.current_sql_file {
            current_file.clone()
        } else {
            format!("query_{timestamp}.sql")
        };

        // Save to sql_files directory
        let sql_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".lazytables")
            .join("sql_files");

        // Ensure directory exists
        std::fs::create_dir_all(&sql_dir)?;

        let filepath = sql_dir.join(&filename);
        std::fs::write(&filepath, query)?;

        // Update state
        context.state.ui.current_sql_file = Some(filename.clone());
        context.state.ui.query_modified = false;

        // Add success toast
        context
            .state
            .toast_manager
            .success(format!("Query saved to {filename}"));

        Ok(CommandResult::SuccessWithMessage(format!(
            "Query saved to {}",
            filepath.display()
        )))
    }

    fn description(&self) -> &str {
        "Save current query"
    }

    fn id(&self) -> CommandId {
        CommandId::SaveQuery
    }

    fn shortcut(&self) -> Option<String> {
        Some("Ctrl+s".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Query
    }

    fn can_execute(&self, context: &CommandContext) -> bool {
        !context.state.query_content.is_empty()
    }
}

/// Validate SQL query syntax for specific database types
fn validate_query_syntax(query: &str, database_type: &DatabaseType) -> std::result::Result<(), String> {
    let query_upper = query.to_uppercase();

    match database_type {
        DatabaseType::MySQL | DatabaseType::MariaDB => {
            // MySQL-specific validation
            if query_upper.contains("RETURNING") && !query_upper.contains("INSERT") {
                return Err("RETURNING clause is not supported in MySQL for this statement type".to_string());
            }
            if query_upper.contains("SERIAL") {
                return Err("SERIAL type is not supported in MySQL. Use AUTO_INCREMENT instead".to_string());
            }
            if query_upper.contains("BOOLEAN") && query_upper.contains("CREATE TABLE") {
                return Ok(()); // MySQL supports BOOLEAN as alias for TINYINT(1)
            }
        }
        DatabaseType::PostgreSQL => {
            // PostgreSQL-specific validation
            if query_upper.contains("AUTO_INCREMENT") {
                return Err("AUTO_INCREMENT is not supported in PostgreSQL. Use SERIAL or IDENTITY instead".to_string());
            }
            if query_upper.contains("ENGINE=") {
                return Err("ENGINE clause is MySQL-specific, not supported in PostgreSQL".to_string());
            }
            if query_upper.contains("UNSIGNED") {
                return Err("UNSIGNED modifier is not supported in PostgreSQL".to_string());
            }
        }
        DatabaseType::SQLite => {
            // SQLite-specific validation
            if query_upper.contains("ENGINE=") {
                return Err("ENGINE clause is not supported in SQLite".to_string());
            }
            if query_upper.contains("UNSIGNED") {
                return Err("UNSIGNED modifier is not supported in SQLite".to_string());
            }
        }
        DatabaseType::Oracle | DatabaseType::Redis | DatabaseType::MongoDB => {
            // Basic validation for other database types
            // Can be expanded as support is added
        }
    }

    // Common SQL syntax validation
    if query.trim().is_empty() {
        return Err("Query cannot be empty".to_string());
    }

    // Check for balanced parentheses
    let mut paren_count = 0;
    for ch in query.chars() {
        match ch {
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            _ => {}
        }
        if paren_count < 0 {
            return Err("Unbalanced parentheses in query".to_string());
        }
    }
    if paren_count != 0 {
        return Err("Unbalanced parentheses in query".to_string());
    }

    Ok(())
}

/// New query command
pub struct NewQueryCommand;

impl Command for NewQueryCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        // Clear current query
        context.state.query_content.clear();
        context.state.ui.current_sql_file = None;
        context.state.ui.query_modified = false;
        context.state.ui.query_cursor_line = 0;
        context.state.ui.query_cursor_column = 0;

        // Switch to query window
        context.state.ui.focused_pane = crate::app::FocusedPane::QueryWindow;

        Ok(CommandResult::SuccessWithMessage(
            "New query created".to_string(),
        ))
    }

    fn description(&self) -> &str {
        "Create new query"
    }

    fn id(&self) -> CommandId {
        CommandId::NewQuery
    }

    fn shortcut(&self) -> Option<String> {
        Some("Ctrl+n".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Query
    }
}
