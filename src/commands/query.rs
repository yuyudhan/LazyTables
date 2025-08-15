// FilePath: src/commands/query.rs

use super::{Command, CommandAction, CommandCategory, CommandContext, CommandId, CommandResult};
use crate::core::error::Result;

/// Execute query command
pub struct ExecuteQueryCommand;

impl Command for ExecuteQueryCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        // Get the query to execute
        let query = if !context.state.query_content.is_empty() {
            context.state.query_content.clone()
        } else {
            return Ok(CommandResult::Error("No query to execute".to_string()));
        };

        // Check if connected
        let has_active_connection = context
            .state
            .db
            .connections
            .connections
            .iter()
            .any(|conn| conn.is_connected());

        if !has_active_connection {
            return Ok(CommandResult::Error(
                "No active database connection".to_string(),
            ));
        }

        // Add info toast
        context.state.toast_manager.info(format!(
            "Executing query: {}",
            query.lines().next().unwrap_or("")
        ));

        Ok(CommandResult::Action(CommandAction::ExecuteQuery(query)))
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
