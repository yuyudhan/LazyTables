// FilePath: src/commands/mod.rs

use crate::app::AppState;
use crate::core::error::Result;
use std::collections::HashMap;
use std::fmt;

pub mod basic;
pub mod connection;
pub mod editing;
pub mod navigation;
pub mod query;

pub use basic::*;
pub use connection::*;
pub use editing::*;
pub use navigation::*;
pub use query::*;

/// Unique identifier for each command
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandId {
    // Application commands
    Quit,
    ForceQuit,
    Help,
    ToggleHelp,

    // Navigation commands
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    NextPane,
    PreviousPane,
    FocusConnectionsPane,
    FocusTablesPane,
    FocusDetailsPane,
    FocusQueryPane,
    FocusResultsPane,
    FocusSqlFilesPane,

    // Connection commands
    Connect,
    Disconnect,
    AddConnection,
    EditConnection,
    DeleteConnection,
    RefreshConnections,
    TestConnection,

    // Query commands
    ExecuteQuery,
    ExecuteCurrentStatement,
    SaveQuery,
    LoadQuery,
    NewQuery,
    ClearQuery,
    FormatQuery,
    ExplainQuery,

    // Table commands
    CreateTable,
    DropTable,
    TruncateTable,
    RefreshTables,
    ShowTableStructure,
    ShowTableData,
    ExportTable,
    ImportTable,

    // Editing commands
    StartInsertMode,
    ExitInsertMode,
    StartVisualMode,
    ExitVisualMode,
    StartCommandMode,
    ExitCommandMode,
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    Delete,
    SelectAll,

    // File commands
    Save,
    SaveAs,
    Open,
    Close,
    NewFile,

    // View commands
    ToggleFullscreen,
    ZoomIn,
    ZoomOut,
    ResetZoom,
    ToggleLineNumbers,
    ToggleWordWrap,

    // Search commands
    Search,
    SearchNext,
    SearchPrevious,
    Replace,
    ReplaceAll,

    // Settings commands
    OpenSettings,
    ReloadConfig,
    ToggleTheme,

    // Custom command for extensions
    Custom(String),
}

impl fmt::Display for CommandId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandId::Quit => write!(f, "Quit"),
            CommandId::ForceQuit => write!(f, "Force Quit"),
            CommandId::Help => write!(f, "Help"),
            CommandId::ToggleHelp => write!(f, "Toggle Help"),
            CommandId::Connect => write!(f, "Connect"),
            CommandId::Save => write!(f, "Save"),
            CommandId::ExecuteQuery => write!(f, "Execute Query"),
            CommandId::Custom(name) => write!(f, "Custom: {name}"),
            _ => write!(f, "{self:?}"),
        }
    }
}

/// Context passed to commands for execution
pub struct CommandContext<'a> {
    pub state: &'a mut AppState,
    pub config: &'a crate::config::Config,
}

/// Result of command execution
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// Command executed successfully
    Success,
    /// Command executed with a message
    SuccessWithMessage(String),
    /// Command failed with an error message
    Error(String),
    /// Command requires confirmation
    RequiresConfirmation(String),
    /// Command was cancelled
    Cancelled,
    /// Command triggered an action that should be handled elsewhere
    Action(CommandAction),
}

/// Actions that can be triggered by commands
#[derive(Debug, Clone)]
pub enum CommandAction {
    Quit,
    OpenModal(ModalType),
    CloseModal,
    ExecuteQuery(String),
    LoadFile(String),
    SaveFile(String),
    Navigate(NavigationTarget),
}

#[derive(Debug, Clone)]
pub enum ModalType {
    Help,
    Connection,
    TableCreator,
    Settings,
    Confirmation(String),
}

#[derive(Debug, Clone)]
pub enum NavigationTarget {
    Pane(crate::app::FocusedPane),
    Line(usize),
    Column(usize),
    Cell(usize, usize),
}

/// Main trait for all commands
pub trait Command: Send + Sync {
    /// Execute the command
    fn execute(&self, context: &mut CommandContext) -> Result<CommandResult>;

    /// Undo the command (if supported)
    fn undo(&self, _context: &mut CommandContext) -> Result<CommandResult> {
        Ok(CommandResult::Error(
            "Undo not supported for this command".to_string(),
        ))
    }

    /// Get command description
    fn description(&self) -> &str;

    /// Get command ID
    fn id(&self) -> CommandId;

    /// Check if command can be executed in current context
    fn can_execute(&self, _context: &CommandContext) -> bool {
        true
    }

    /// Check if command supports undo
    fn supports_undo(&self) -> bool {
        false
    }

    /// Get keyboard shortcut for this command (if any)
    fn shortcut(&self) -> Option<String> {
        None
    }

    /// Get category for grouping commands
    fn category(&self) -> CommandCategory {
        CommandCategory::General
    }
}

/// Categories for grouping commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    General,
    Navigation,
    Editing,
    Connection,
    Query,
    Table,
    File,
    View,
    Search,
    Settings,
}

impl fmt::Display for CommandCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandCategory::General => write!(f, "General"),
            CommandCategory::Navigation => write!(f, "Navigation"),
            CommandCategory::Editing => write!(f, "Editing"),
            CommandCategory::Connection => write!(f, "Connection"),
            CommandCategory::Query => write!(f, "Query"),
            CommandCategory::Table => write!(f, "Table"),
            CommandCategory::File => write!(f, "File"),
            CommandCategory::View => write!(f, "View"),
            CommandCategory::Search => write!(f, "Search"),
            CommandCategory::Settings => write!(f, "Settings"),
        }
    }
}

/// Registry for managing commands
pub struct CommandRegistry {
    commands: HashMap<CommandId, Box<dyn Command>>,
    shortcuts: HashMap<String, CommandId>,
    history: Vec<CommandId>,
    undo_stack: Vec<CommandId>,
    redo_stack: Vec<CommandId>,
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
            shortcuts: HashMap::new(),
            history: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };

        // Register default commands
        registry.register_default_commands();

        registry
    }

    /// Register a command
    pub fn register(&mut self, command: Box<dyn Command>) {
        let id = command.id();
        if let Some(shortcut) = command.shortcut() {
            self.shortcuts.insert(shortcut, id.clone());
        }
        self.commands.insert(id, command);
    }

    /// Execute a command by ID
    pub fn execute(
        &mut self,
        id: CommandId,
        context: &mut CommandContext,
    ) -> Result<CommandResult> {
        if let Some(command) = self.commands.get(&id) {
            // Check if command can be executed
            if !command.can_execute(context) {
                return Ok(CommandResult::Error(
                    "Command cannot be executed in current context".to_string(),
                ));
            }

            // Execute the command
            let result = command.execute(context)?;

            // Add to history
            self.history.push(id.clone());

            // Add to undo stack if command supports undo
            if command.supports_undo() {
                self.undo_stack.push(id.clone());
                self.redo_stack.clear(); // Clear redo stack on new action
            }

            Ok(result)
        } else {
            Ok(CommandResult::Error(format!("Command not found: {id}")))
        }
    }

    /// Undo the last command
    pub fn undo(&mut self, context: &mut CommandContext) -> Result<CommandResult> {
        if let Some(id) = self.undo_stack.pop() {
            if let Some(command) = self.commands.get(&id) {
                let result = command.undo(context)?;
                self.redo_stack.push(id);
                Ok(result)
            } else {
                Ok(CommandResult::Error(
                    "Command not found for undo".to_string(),
                ))
            }
        } else {
            Ok(CommandResult::Error("Nothing to undo".to_string()))
        }
    }

    /// Redo the last undone command
    pub fn redo(&mut self, context: &mut CommandContext) -> Result<CommandResult> {
        if let Some(id) = self.redo_stack.pop() {
            if let Some(command) = self.commands.get(&id) {
                let result = command.execute(context)?;
                self.undo_stack.push(id);
                Ok(result)
            } else {
                Ok(CommandResult::Error(
                    "Command not found for redo".to_string(),
                ))
            }
        } else {
            Ok(CommandResult::Error("Nothing to redo".to_string()))
        }
    }

    /// Get command by ID
    pub fn get(&self, id: CommandId) -> Option<&dyn Command> {
        self.commands.get(&id).map(|c| c.as_ref())
    }

    /// Get command by shortcut
    pub fn get_by_shortcut(&self, shortcut: &str) -> Option<&dyn Command> {
        self.shortcuts
            .get(shortcut)
            .and_then(|id| self.commands.get(id))
            .map(|c| c.as_ref())
    }

    /// Get all commands in a category
    pub fn get_by_category(&self, category: CommandCategory) -> Vec<&dyn Command> {
        self.commands
            .values()
            .filter(|c| c.category() == category)
            .map(|c| c.as_ref())
            .collect()
    }

    /// Get command history
    pub fn history(&self) -> &[CommandId] {
        &self.history
    }

    /// Clear command history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Register default commands
    fn register_default_commands(&mut self) {
        // Register basic commands
        self.register(Box::new(basic::QuitCommand));
        self.register(Box::new(basic::HelpCommand));
        self.register(Box::new(basic::SaveCommand));

        // Register connection commands
        self.register(Box::new(connection::ConnectCommand));
        self.register(Box::new(connection::AddConnectionCommand));

        // Register navigation commands
        self.register(Box::new(navigation::NavigateUpCommand));
        self.register(Box::new(navigation::NavigateDownCommand));
        self.register(Box::new(navigation::NavigateLeftCommand));
        self.register(Box::new(navigation::NavigateRightCommand));
        self.register(Box::new(navigation::NextPaneCommand));

        // Register query commands
        self.register(Box::new(query::ExecuteQueryCommand));
        self.register(Box::new(query::SaveQueryCommand));
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
