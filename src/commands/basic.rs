// FilePath: src/commands/basic.rs

use super::{Command, CommandAction, CommandCategory, CommandContext, CommandId, CommandResult};
use crate::core::error::Result;

/// Quit command - exits the application
pub struct QuitCommand;

impl Command for QuitCommand {
    fn execute(&self, _context: &mut CommandContext) -> Result<CommandResult> {
        Ok(CommandResult::Action(CommandAction::Quit))
    }

    fn description(&self) -> &str {
        "Quit the application"
    }

    fn id(&self) -> CommandId {
        CommandId::Quit
    }

    fn shortcut(&self) -> Option<String> {
        Some("q".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::General
    }
}

/// Force quit command - exits without confirmation
pub struct ForceQuitCommand;

impl Command for ForceQuitCommand {
    fn execute(&self, _context: &mut CommandContext) -> Result<CommandResult> {
        Ok(CommandResult::Action(CommandAction::Quit))
    }

    fn description(&self) -> &str {
        "Force quit the application without confirmation"
    }

    fn id(&self) -> CommandId {
        CommandId::ForceQuit
    }

    fn shortcut(&self) -> Option<String> {
        Some("Q".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::General
    }
}

/// Help command - shows help overlay
pub struct HelpCommand;

impl Command for HelpCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        use crate::app::state::HelpMode;

        // Toggle help based on current pane
        context.state.ui.help_mode = match context.state.ui.focused_pane {
            crate::app::FocusedPane::Connections => HelpMode::Connections,
            crate::app::FocusedPane::Tables => HelpMode::Tables,
            crate::app::FocusedPane::Details => HelpMode::Details,
            crate::app::FocusedPane::TabularOutput => HelpMode::TabularOutput,
            crate::app::FocusedPane::QueryWindow => HelpMode::QueryWindow,
            crate::app::FocusedPane::SqlFiles => HelpMode::SqlFiles,
        };

        Ok(CommandResult::SuccessWithMessage(
            "Help overlay opened".to_string(),
        ))
    }

    fn description(&self) -> &str {
        "Show context-aware help"
    }

    fn id(&self) -> CommandId {
        CommandId::Help
    }

    fn shortcut(&self) -> Option<String> {
        Some("?".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::General
    }
}

/// Toggle help command
pub struct ToggleHelpCommand;

impl Command for ToggleHelpCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        use crate::app::state::HelpMode;

        if context.state.ui.help_mode != HelpMode::None {
            context.state.ui.help_mode = HelpMode::None;
            Ok(CommandResult::SuccessWithMessage(
                "Help overlay closed".to_string(),
            ))
        } else {
            // Delegate to HelpCommand
            HelpCommand.execute(context)
        }
    }

    fn description(&self) -> &str {
        "Toggle help overlay"
    }

    fn id(&self) -> CommandId {
        CommandId::ToggleHelp
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::General
    }
}

/// Save command - saves current content
pub struct SaveCommand;

impl Command for SaveCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        // Check what needs to be saved based on current focus
        match context.state.ui.focused_pane {
            crate::app::FocusedPane::QueryWindow => {
                // Save current query
                if !context.state.query_content.is_empty() {
                    let query = &context.state.query_content;
                    // Generate filename with timestamp
                    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                    let filename = format!("query_{timestamp}.sql");

                    // Save to sql_files directory
                    let sql_dir = dirs::home_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join(".lazytables")
                        .join("sql_files");

                    // Ensure directory exists
                    std::fs::create_dir_all(&sql_dir)?;

                    let filepath = sql_dir.join(&filename);
                    std::fs::write(&filepath, query)?;

                    // Add success toast
                    context
                        .state
                        .toast_manager
                        .success(format!("Query saved to {filename}"));

                    Ok(CommandResult::SuccessWithMessage(format!(
                        "Query saved to {}",
                        filepath.display()
                    )))
                } else {
                    Ok(CommandResult::Error("No query to save".to_string()))
                }
            }
            crate::app::FocusedPane::TabularOutput => {
                // Export results to CSV
                // TODO: Check if results exist once query results are implemented
                Ok(CommandResult::Error(
                    "Export not yet implemented".to_string(),
                ))
            }
            _ => Ok(CommandResult::Error(
                "Nothing to save in current pane".to_string(),
            )),
        }
    }

    fn description(&self) -> &str {
        "Save current content"
    }

    fn id(&self) -> CommandId {
        CommandId::Save
    }

    fn shortcut(&self) -> Option<String> {
        Some("Ctrl+s".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::File
    }

    fn can_execute(&self, context: &CommandContext) -> bool {
        // Can save if in query window with content or results pane with data
        match context.state.ui.focused_pane {
            crate::app::FocusedPane::QueryWindow => !context.state.query_content.is_empty(),
            crate::app::FocusedPane::TabularOutput => {
                // TODO: Check if results exist once query results are implemented
                false
            }
            _ => false,
        }
    }
}

/// Save As command - saves with a specific filename
pub struct SaveAsCommand;

impl Command for SaveAsCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        // This would typically open a file dialog or prompt
        // For now, delegate to SaveCommand
        SaveCommand.execute(context)
    }

    fn description(&self) -> &str {
        "Save current content with a specific name"
    }

    fn id(&self) -> CommandId {
        CommandId::SaveAs
    }

    fn shortcut(&self) -> Option<String> {
        Some("Ctrl+Shift+s".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::File
    }
}

/// Open command - opens a file
pub struct OpenCommand;

impl Command for OpenCommand {
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult> {
        match context.state.ui.focused_pane {
            crate::app::FocusedPane::SqlFiles => {
                // Load selected SQL file
                {
                    let selected = context.state.ui.selected_sql_file;
                    let sql_dir = dirs::home_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join(".lazytables")
                        .join("sql_files");

                    let files = std::fs::read_dir(&sql_dir)?
                        .filter_map(|entry| entry.ok())
                        .filter(|entry| {
                            entry
                                .path()
                                .extension()
                                .and_then(|ext| ext.to_str())
                                .map(|ext| ext == "sql")
                                .unwrap_or(false)
                        })
                        .collect::<Vec<_>>();

                    if selected < files.len() {
                        let content = std::fs::read_to_string(files[selected].path())?;
                        context.state.query_content = content;
                        context.state.ui.current_sql_file =
                            Some(files[selected].file_name().to_string_lossy().to_string());

                        Ok(CommandResult::SuccessWithMessage(format!(
                            "Loaded {}",
                            files[selected].file_name().to_string_lossy()
                        )))
                    } else {
                        Ok(CommandResult::Error("Invalid file selection".to_string()))
                    }
                }
            }
            _ => Ok(CommandResult::Error(
                "Navigate to SQL Files pane to open a file".to_string(),
            )),
        }
    }

    fn description(&self) -> &str {
        "Open a file"
    }

    fn id(&self) -> CommandId {
        CommandId::Open
    }

    fn shortcut(&self) -> Option<String> {
        Some("Ctrl+o".to_string())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::File
    }
}
