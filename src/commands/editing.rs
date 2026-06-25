// FilePath: src/commands/editing.rs

#![forbid(unsafe_code)]

use super::{Command, CommandCategory, CommandContext, CommandId, CommandResult};
use crate::core::error::Result;

/// Start insert mode command
pub struct StartInsertModeCommand;

impl Command for StartInsertModeCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        match context.state.ui.focused_pane {
            crate::app::FocusedPane::QueryWindow => {
                // Query editor manages its own insert mode now
                context.state.query_editor.set_insert_mode(true);
                Ok(CommandResult::SuccessWithMessage("Insert mode".to_string()))
            }
            _ => Ok(CommandResult::Error(
                "Insert mode only available in Query Editor".to_string(),
            )),
        }
    }

    fn description(&self) -> &str {
        "Enter insert mode"
    }

    fn id(&self) -> CommandId {
        CommandId::StartInsertMode
    }

    fn shortcut(&self) -> Option<String> {
        Some("i".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Editing
    }
}

/// Exit insert mode command
pub struct ExitInsertModeCommand;

impl Command for ExitInsertModeCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        // Query editor manages its own insert mode now
        context.state.query_editor.set_insert_mode(false);
        Ok(CommandResult::SuccessWithMessage("Normal mode".to_string()))
    }

    fn description(&self) -> &str {
        "Exit insert mode"
    }

    fn id(&self) -> CommandId {
        CommandId::ExitInsertMode
    }

    fn shortcut(&self) -> Option<String> {
        Some("Esc".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Editing
    }
}
