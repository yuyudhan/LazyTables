// FilePath: src/commands/navigation.rs

use super::{Command, CommandCategory, CommandContext, CommandId, CommandResult};
use crate::app::FocusedPane;
use crate::core::error::Result;

/// Navigate up command
pub struct NavigateUpCommand;

impl Command for NavigateUpCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        match context.state.focused_pane {
            FocusedPane::Connections => {
                if context.state.selected_connection > 0 {
                    context.state.selected_connection -= 1;
                    context.state.connections_list_state.select(Some(context.state.selected_connection));
                }
            }
            FocusedPane::Tables => {
                if context.state.selected_table > 0 {
                    context.state.selected_table -= 1;
                    context.state.tables_list_state.select(Some(context.state.selected_table));
                }
            }
            FocusedPane::SqlFiles => {
                if context.state.selected_sql_file > 0 {
                    context.state.selected_sql_file -= 1;
                }
            }
            FocusedPane::QueryWindow => {
                if context.state.query_cursor_line > 0 {
                    context.state.query_cursor_line -= 1;
                    // Adjust column if line is shorter
                    let line_len = context.state.query_content
                        .lines()
                        .nth(context.state.query_cursor_line)
                        .map(|l| l.len())
                        .unwrap_or(0);
                    context.state.query_cursor_column = 
                        context.state.query_cursor_column.min(line_len);
                }
            }
            _ => {}
        }
        Ok(CommandResult::Success)
    }
    
    fn description(&self) -> &str {
        "Navigate up"
    }
    
    fn id(&self) -> CommandId {
        CommandId::NavigateUp
    }
    
    fn shortcut(&self) -> Option<String> {
        Some("k".to_string())
    }
    
    fn category(&self) -> CommandCategory {
        CommandCategory::Navigation
    }
}

/// Navigate down command
pub struct NavigateDownCommand;

impl Command for NavigateDownCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        match context.state.focused_pane {
            FocusedPane::Connections => {
                let max = context.state.connections.connections.len();
                if context.state.selected_connection < max.saturating_sub(1) {
                    context.state.selected_connection += 1;
                    context.state.connections_list_state.select(Some(context.state.selected_connection));
                }
            }
            FocusedPane::Tables => {
                let max = context.state.tables.len();
                if context.state.selected_table < max.saturating_sub(1) {
                    context.state.selected_table += 1;
                    context.state.tables_list_state.select(Some(context.state.selected_table));
                }
            }
            FocusedPane::SqlFiles => {
                let max = context.state.saved_sql_files.len();
                if context.state.selected_sql_file < max.saturating_sub(1) {
                    context.state.selected_sql_file += 1;
                }
            }
            FocusedPane::QueryWindow => {
                let line_count = context.state.query_content.lines().count();
                if context.state.query_cursor_line < line_count.saturating_sub(1) {
                    context.state.query_cursor_line += 1;
                    // Adjust column if line is shorter
                    let line_len = context.state.query_content
                        .lines()
                        .nth(context.state.query_cursor_line)
                        .map(|l| l.len())
                        .unwrap_or(0);
                    context.state.query_cursor_column = 
                        context.state.query_cursor_column.min(line_len);
                }
            }
            _ => {}
        }
        Ok(CommandResult::Success)
    }
    
    fn description(&self) -> &str {
        "Navigate down"
    }
    
    fn id(&self) -> CommandId {
        CommandId::NavigateDown
    }
    
    fn shortcut(&self) -> Option<String> {
        Some("j".to_string())
    }
    
    fn category(&self) -> CommandCategory {
        CommandCategory::Navigation
    }
}

/// Navigate left command
pub struct NavigateLeftCommand;

impl Command for NavigateLeftCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        match context.state.focused_pane {
            FocusedPane::QueryWindow => {
                if context.state.query_cursor_column > 0 {
                    context.state.query_cursor_column -= 1;
                }
            }
            _ => {
                // Navigate to previous pane
                PreviousPaneCommand.execute(context)?;
            }
        }
        Ok(CommandResult::Success)
    }
    
    fn description(&self) -> &str {
        "Navigate left"
    }
    
    fn id(&self) -> CommandId {
        CommandId::NavigateLeft
    }
    
    fn shortcut(&self) -> Option<String> {
        Some("h".to_string())
    }
    
    fn category(&self) -> CommandCategory {
        CommandCategory::Navigation
    }
}

/// Navigate right command
pub struct NavigateRightCommand;

impl Command for NavigateRightCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        match context.state.focused_pane {
            FocusedPane::QueryWindow => {
                let line_len = context.state.query_content
                    .lines()
                    .nth(context.state.query_cursor_line)
                    .map(|l| l.len())
                    .unwrap_or(0);
                if context.state.query_cursor_column < line_len {
                    context.state.query_cursor_column += 1;
                }
            }
            _ => {
                // Navigate to next pane
                NextPaneCommand.execute(context)?;
            }
        }
        Ok(CommandResult::Success)
    }
    
    fn description(&self) -> &str {
        "Navigate right"
    }
    
    fn id(&self) -> CommandId {
        CommandId::NavigateRight
    }
    
    fn shortcut(&self) -> Option<String> {
        Some("l".to_string())
    }
    
    fn category(&self) -> CommandCategory {
        CommandCategory::Navigation
    }
}

/// Next pane command
pub struct NextPaneCommand;

impl Command for NextPaneCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        use FocusedPane::*;
        
        context.state.focused_pane = match context.state.focused_pane {
            Connections => Tables,
            Tables => Details,
            Details => TabularOutput,
            TabularOutput => SqlFiles,
            SqlFiles => QueryWindow,
            QueryWindow => Connections,
        };
        
        Ok(CommandResult::Success)
    }
    
    fn description(&self) -> &str {
        "Switch to next pane"
    }
    
    fn id(&self) -> CommandId {
        CommandId::NextPane
    }
    
    fn shortcut(&self) -> Option<String> {
        Some("Tab".to_string())
    }
    
    fn category(&self) -> CommandCategory {
        CommandCategory::Navigation
    }
}

/// Previous pane command
pub struct PreviousPaneCommand;

impl Command for PreviousPaneCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        use FocusedPane::*;
        
        context.state.focused_pane = match context.state.focused_pane {
            Connections => QueryWindow,
            Tables => Connections,
            Details => Tables,
            TabularOutput => Details,
            SqlFiles => TabularOutput,
            QueryWindow => SqlFiles,
        };
        
        Ok(CommandResult::Success)
    }
    
    fn description(&self) -> &str {
        "Switch to previous pane"
    }
    
    fn id(&self) -> CommandId {
        CommandId::PreviousPane
    }
    
    fn shortcut(&self) -> Option<String> {
        Some("Shift+Tab".to_string())
    }
    
    fn category(&self) -> CommandCategory {
        CommandCategory::Navigation
    }
}

/// Focus Connections pane command
pub struct FocusConnectionsPaneCommand;

impl Command for FocusConnectionsPaneCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        context.state.focused_pane = FocusedPane::Connections;
        Ok(CommandResult::Success)
    }
    
    fn description(&self) -> &str {
        "Focus Connections pane"
    }
    
    fn id(&self) -> CommandId {
        CommandId::FocusConnectionsPane
    }
    
    fn shortcut(&self) -> Option<String> {
        Some("c".to_string())
    }
    
    fn category(&self) -> CommandCategory {
        CommandCategory::Navigation
    }
}