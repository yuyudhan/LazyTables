// FilePath: src/app/mod.rs

use crate::{
    commands::{CommandAction, CommandContext, CommandId, CommandRegistry, CommandResult},
    config::Config,
    core::error::Result,
    event::{Event, EventHandler},
    ui::UI,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
use std::time::Duration;

pub mod state;

pub use state::{AppState, FocusedPane};

// Simplified internal mode for compatibility - not shown to user
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
    Query,
}

/// Main application structure
pub struct App {
    /// Application state
    pub state: AppState,
    /// Event handler
    event_handler: EventHandler,
    /// User interface
    ui: UI,
    /// Configuration
    config: Config,
    /// Command registry
    command_registry: CommandRegistry,
    /// Flag to quit the application
    should_quit: bool,
    /// Internal mode for key handling (not shown to user)
    mode: Mode,
    /// Command buffer for : commands
    command_buffer: String,
    /// Leader key state for compound commands
    leader_pressed: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(config: Config) -> Result<Self> {
        let state = AppState::new();
        let event_handler = EventHandler::new(Duration::from_millis(250));
        let ui = UI::new(&config)?;
        let command_registry = CommandRegistry::new();

        Ok(Self {
            state,
            event_handler,
            ui,
            config,
            command_registry,
            should_quit: false,
            mode: Mode::Normal,
            command_buffer: String::new(),
            leader_pressed: false,
        })
    }

    /// Run the application main loop
    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        // Initialize the application state database
        if let Err(e) = self.state.initialize_app_db().await {
            eprintln!("Warning: Failed to initialize application database: {}", e);
            eprintln!("Some features may not work correctly.");
        }

        self.event_handler.start()?;

        while !self.should_quit {
            // Draw UI
            terminal.draw(|frame| self.draw(frame))?;

            // Handle events
            if let Some(event) = self.event_handler.next()? {
                self.handle_event(event).await?;
            }
        }

        Ok(())
    }

    /// Draw the user interface
    fn draw(&mut self, frame: &mut Frame) {
        self.ui.draw(frame, &mut self.state);
    }

    /// Handle application events
    async fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event).await?,
            Event::Mouse(_) => {
                // Mouse events will be handled in future
            }
            Event::Resize(_, _) => {
                // Terminal resize is handled automatically by ratatui
            }
            Event::Tick => {
                // Handle periodic updates
                self.tick().await?;
            }
        }
        Ok(())
    }

    /// Execute a command by ID
    fn execute_command(&mut self, command_id: CommandId) -> Result<()> {
        let mut context = CommandContext {
            state: &mut self.state,
            config: &self.config,
        };

        match self.command_registry.execute(command_id, &mut context)? {
            CommandResult::Success => {}
            CommandResult::SuccessWithMessage(msg) => {
                self.state.toast_manager.success(&msg);
            }
            CommandResult::Error(msg) => {
                self.state.toast_manager.error(&msg);
            }
            CommandResult::RequiresConfirmation(msg) => {
                self.state.toast_manager.warning(format!("Confirm: {msg}"));
            }
            CommandResult::Cancelled => {}
            CommandResult::Action(action) => {
                self.handle_command_action(action)?;
            }
        }

        Ok(())
    }

    /// Handle command actions
    fn handle_command_action(&mut self, action: CommandAction) -> Result<()> {
        match action {
            CommandAction::Quit => {
                self.should_quit = true;
            }
            CommandAction::OpenModal(modal_type) => {
                use crate::commands::ModalType;
                match modal_type {
                    ModalType::Help => {
                        self.execute_command(CommandId::Help)?;
                    }
                    ModalType::Connection => {
                        self.state.ui.show_add_connection_modal = true;
                    }
                    _ => {}
                }
            }
            CommandAction::CloseModal => {
                self.state.ui.show_add_connection_modal = false;
                self.state.ui.show_edit_connection_modal = false;
                self.state.ui.show_table_creator = false;
                self.state.ui.show_table_editor = false;
            }
            CommandAction::ExecuteQuery(query) => {
                // TODO: Execute query through database connection
                self.state.toast_manager.info(format!("Executing: {query}"));
            }
            CommandAction::ExecuteQueryWithContext {
                query,
                database_type,
                connection_name,
            } => {
                // Enhanced query execution with database context
                self.state.toast_manager.info(format!(
                    "Executing {} query on {}: {}",
                    database_type.display_name(),
                    connection_name,
                    query
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(50)
                        .collect::<String>()
                ));
                // TODO: Execute query through specific database adapter
            }
            CommandAction::LoadFile(path) => {
                // TODO: Load file
                self.state.toast_manager.info(format!("Loading: {path}"));
            }
            CommandAction::SaveFile(path) => {
                // TODO: Save file
                self.state.toast_manager.info(format!("Saving: {path}"));
            }
            CommandAction::Navigate(target) => {
                use crate::commands::NavigationTarget;
                if let NavigationTarget::Pane(pane) = target {
                    self.state.ui.focused_pane = pane;
                }
            }
        }
        Ok(())
    }

    /// Handle keyboard events
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // Handle ESC key globally to close help overlay
        if key.code == KeyCode::Esc && self.state.ui.help_mode != crate::app::state::HelpMode::None
        {
            self.state.ui.help_mode = crate::app::state::HelpMode::None;
            return Ok(());
        }

        // Handle help modal navigation keys when help is active
        if self.state.ui.help_mode != crate::app::state::HelpMode::None {
            match (key.modifiers, key.code) {
                // Left/Right arrow keys or h/l to switch between help panes
                (KeyModifiers::NONE, KeyCode::Left)
                | (KeyModifiers::NONE, KeyCode::Right)
                | (KeyModifiers::NONE, KeyCode::Char('h'))
                | (KeyModifiers::NONE, KeyCode::Char('l')) => {
                    self.state.ui.toggle_help_pane_focus();
                    return Ok(());
                }
                // Up/Down arrow keys or j/k for scrolling
                (KeyModifiers::NONE, KeyCode::Up) | (KeyModifiers::NONE, KeyCode::Char('k')) => {
                    self.state.ui.help_scroll_up();
                    return Ok(());
                }
                (KeyModifiers::NONE, KeyCode::Down) | (KeyModifiers::NONE, KeyCode::Char('j')) => {
                    // We need to pass the max_lines, but for now we'll use a reasonable default
                    self.state.ui.help_scroll_down(100);
                    return Ok(());
                }
                // Page Up/Down for faster scrolling
                (KeyModifiers::NONE, KeyCode::PageUp) | (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                    self.state.ui.help_page_up(10);
                    return Ok(());
                }
                (KeyModifiers::NONE, KeyCode::PageDown) | (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                    self.state.ui.help_page_down(100, 10);
                    return Ok(());
                }
                // Tab to switch between panes
                (KeyModifiers::NONE, KeyCode::Tab) => {
                    self.state.ui.toggle_help_pane_focus();
                    return Ok(());
                }
                // Other keys fall through to be handled normally (like '?' for closing)
                _ => {}
            }
        }

        // Handle ESC to exit search modes or cancel pending gg command
        if key.code == KeyCode::Esc {
            if self.state.ui.connections_search_active {
                self.state.ui.exit_connections_search();
                return Ok(());
            } else if self.state.ui.tables_search_active {
                self.state.ui.exit_tables_search();
                return Ok(());
            } else if self.state.ui.pending_gg_command {
                self.state.ui.cancel_pending_gg();
                return Ok(());
            }
        }

        // Handle '?' key globally for context-aware help (toggle functionality)
        if key.code == KeyCode::Char('?') && key.modifiers == KeyModifiers::NONE {
            self.execute_command(CommandId::ToggleHelp)?;
            return Ok(());
        }

        // Handle Ctrl+B globally for debug view toggle
        if key.code == KeyCode::Char('b') && key.modifiers == KeyModifiers::CONTROL {
            self.state.ui.toggle_debug_view();
            return Ok(());
        }

        // Handle confirmation modal
        if let Some(modal) = &self.state.ui.confirmation_modal {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    // Execute the confirmed action
                    match &modal.action {
                        crate::ui::ConfirmationAction::DeleteConnection(index) => {
                            let index = *index;
                            // Delete the connection
                            if let Some(connection) =
                                self.state.db.connections.connections.get(index)
                            {
                                let conn_id = connection.id.clone();
                                if let Err(e) =
                                    self.state.db.connections.remove_connection(&conn_id)
                                {
                                    self.state
                                        .toast_manager
                                        .error(format!("Failed to delete connection: {e}"));
                                } else {
                                    self.state
                                        .toast_manager
                                        .success("Connection deleted successfully");
                                    // Adjust selection if needed
                                    if self.state.ui.selected_connection
                                        >= self.state.db.connections.connections.len()
                                        && self.state.ui.selected_connection > 0
                                    {
                                        self.state.ui.selected_connection -= 1;
                                    }
                                }
                            }
                        }
                        crate::ui::ConfirmationAction::DeleteTable(_) => {
                            // Handle table deletion if needed in future
                        }
                        crate::ui::ConfirmationAction::DeleteSqlFile(index) => {
                            let index = *index;
                            // Delete the SQL file
                            if let Err(e) = self.state.delete_sql_file(index) {
                                self.state
                                    .toast_manager
                                    .error(format!("Failed to delete SQL file: {e}"));
                            } else {
                                self.state.toast_manager.success("SQL file deleted");
                            }
                            // Update UI selection after deletion
                            self.state
                                .ui
                                .update_sql_file_selection(self.state.saved_sql_files.len());
                        }
                        crate::ui::ConfirmationAction::ExitApplication => {
                            // Exit the application
                            self.should_quit = true;
                        }
                    }
                    self.state.ui.confirmation_modal = None;
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    // Cancel the action
                    self.state.ui.confirmation_modal = None;
                }
                _ => {}
            }
            return Ok(());
        }

        // Handle debug view navigation when debug view is open
        if self.state.ui.show_debug_view {
            let debug_messages = crate::logging::get_debug_messages();
            let max_lines = debug_messages.len();

            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.state.ui.debug_view_scroll_down(max_lines);
                    return Ok(());
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.state.ui.debug_view_scroll_up();
                    return Ok(());
                }
                KeyCode::PageDown => {
                    self.state.ui.debug_view_page_down(max_lines, 10);
                    return Ok(());
                }
                KeyCode::PageUp => {
                    self.state.ui.debug_view_page_up(10);
                    return Ok(());
                }
                KeyCode::Char('g') => {
                    // Handle gg for go to top
                    if self.state.ui.pending_gg_command {
                        self.state.ui.debug_view_go_to_top();
                        self.state.ui.pending_gg_command = false;
                    } else {
                        self.state.ui.pending_gg_command = true;
                    }
                    return Ok(());
                }
                KeyCode::Char('G') => {
                    self.state.ui.debug_view_go_to_bottom(max_lines);
                    return Ok(());
                }
                KeyCode::Char('c') => {
                    // Clear debug messages
                    crate::logging::clear_debug_messages();
                    self.state.toast_manager.info("Debug messages cleared");
                    return Ok(());
                }
                _ => {}
            }
        }

        // Handle delete confirmation dialog in table viewer
        if let Some(confirmation) = &self.state.table_viewer_state.delete_confirmation {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    // Confirm delete
                    let confirmation = confirmation.clone();
                    if let Err(e) = self.state.delete_table_row(confirmation).await {
                        self.state
                            .toast_manager
                            .error(format!("Failed to delete row: {e}"));
                    } else {
                        self.state.toast_manager.success("Row deleted successfully");
                        // Reload table data
                        let tab_idx = self.state.table_viewer_state.active_tab;
                        let _ = self.state.load_table_data(tab_idx).await;
                    }
                    self.state.table_viewer_state.delete_confirmation = None;
                    return Ok(());
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    // Cancel delete
                    self.state.table_viewer_state.delete_confirmation = None;
                    self.state.toast_manager.info("Delete cancelled");
                    return Ok(());
                }
                _ => {
                    // Ignore other keys while confirmation is shown
                    return Ok(());
                }
            }
        }

        // Handle 'q' key globally to quit (except when in modals or editing)
        if key.code == KeyCode::Char('q') && key.modifiers == KeyModifiers::NONE {
            // Don't quit if we're in a modal or editing
            if !self.state.ui.show_add_connection_modal
                && !self.state.ui.show_edit_connection_modal
                && !self.state.ui.show_table_creator
                && !self.state.ui.show_table_editor
                && self.state.ui.confirmation_modal.is_none()
                && !self.state.ui.connections_search_active
                && !self.state.ui.sql_files_search_active
                && !self.state.ui.sql_files_rename_mode
                && !self.state.ui.sql_files_create_mode
            {
                // Check if we're editing in table viewer
                if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                    if let Some(tab) = self.state.table_viewer_state.current_tab() {
                        if !tab.in_edit_mode && !tab.in_search_mode {
                            // Show exit confirmation modal
                            self.state.ui.confirmation_modal = Some(crate::ui::ConfirmationModal {
                                title: "Exit LazyTables".to_string(),
                                message: "Are you sure you want to exit?\n\nAll active database connections will be closed.".to_string(),
                                action: crate::ui::ConfirmationAction::ExitApplication,
                            });
                            return Ok(());
                        }
                    } else {
                        // Show exit confirmation modal
                        self.state.ui.confirmation_modal = Some(crate::ui::ConfirmationModal {
                            title: "Exit LazyTables".to_string(),
                            message: "Are you sure you want to exit?\n\nAll active database connections will be closed.".to_string(),
                            action: crate::ui::ConfirmationAction::ExitApplication,
                        });
                        return Ok(());
                    }
                } else {
                    // Show exit confirmation modal
                    self.state.ui.confirmation_modal = Some(crate::ui::ConfirmationModal {
                        title: "Exit LazyTables".to_string(),
                        message: "Are you sure you want to exit?\n\nAll active database connections will be closed.".to_string(),
                        action: crate::ui::ConfirmationAction::ExitApplication,
                    });
                    return Ok(());
                }
            }
        }

        // Handle ESC or Enter in table viewer edit mode to save
        if (key.code == KeyCode::Esc || key.code == KeyCode::Enter)
            && self.state.ui.focused_pane == FocusedPane::TabularOutput
        {
            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                if tab.in_edit_mode {
                    // Save the edit and update database
                    if let Some(update) = tab.save_edit() {
                        if let Err(e) = self.state.update_table_cell(update).await {
                            self.state
                                .toast_manager
                                .error(format!("Failed to update cell: {e}"));
                        } else {
                            self.state
                                .toast_manager
                                .success("Cell updated successfully");
                        }
                    }
                    return Ok(());
                }
            }
        }

        // Handle Ctrl+C in table viewer edit mode
        if key.code == KeyCode::Char('c')
            && key.modifiers == KeyModifiers::CONTROL
            && self.state.ui.focused_pane == FocusedPane::TabularOutput
        {
            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                if tab.in_edit_mode {
                    // Cancel edit without saving
                    tab.cancel_edit();
                    return Ok(());
                }
            }
        }

        // Handle typing in table viewer edit mode or search mode
        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                if tab.in_edit_mode {
                    match key.code {
                        KeyCode::Char(c) => {
                            tab.edit_buffer.push(c);
                            return Ok(());
                        }
                        KeyCode::Backspace => {
                            tab.edit_buffer.pop();
                            return Ok(());
                        }
                        _ => {}
                    }
                } else if tab.in_search_mode {
                    match key.code {
                        KeyCode::Esc => {
                            tab.cancel_search();
                            return Ok(());
                        }
                        KeyCode::Enter => {
                            tab.in_search_mode = false;
                            return Ok(());
                        }
                        KeyCode::Char('n') => {
                            tab.next_search_result();
                            return Ok(());
                        }
                        KeyCode::Char('N') => {
                            tab.prev_search_result();
                            return Ok(());
                        }
                        KeyCode::Char(c) => {
                            // Don't treat navigation keys as search input
                            if c == 'h' || c == 'l' || c == 'j' || c == 'k' {
                                // Exit search mode and let the key be handled as navigation
                                tab.in_search_mode = false;
                                // Don't return early - let the key be processed normally
                            } else {
                                tab.search_query.push(c);
                                tab.update_search(&tab.search_query.clone());
                                return Ok(());
                            }
                        }
                        KeyCode::Backspace => {
                            tab.search_query.pop();
                            tab.update_search(&tab.search_query.clone());
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }

        // Handle connection modal if active
        if self.state.ui.show_add_connection_modal || self.state.ui.show_edit_connection_modal {
            return self.handle_connection_modal_key_event(key).await;
        }

        // Handle table creator if active
        if self.state.ui.show_table_creator {
            return self.handle_table_creator_key_event(key).await;
        }

        // Handle table editor if active
        if self.state.ui.show_table_editor {
            return self.handle_table_editor_key_event(key).await;
        }

        // Handle connections search input
        if self.state.ui.connections_search_active
            && self.state.ui.focused_pane == FocusedPane::Connections
        {
            match key.code {
                KeyCode::Backspace => {
                    self.state.ui.backspace_connections_search();
                    // Update filtered connections after character removal
                    self.state
                        .ui
                        .update_filtered_connections(&self.state.db.connections.connections);
                    return Ok(());
                }
                KeyCode::Enter => {
                    // Connect to the highlighted connection
                    self.state.connect_to_selected_database().await;
                    // Exit search mode
                    self.state.ui.exit_connections_search();
                    return Ok(());
                }
                KeyCode::Down => {
                    self.state
                        .ui
                        .connections_selection_down(&self.state.db.connections.connections);
                    return Ok(());
                }
                KeyCode::Up => {
                    self.state
                        .ui
                        .connections_selection_up(&self.state.db.connections.connections);
                    return Ok(());
                }
                KeyCode::Char(c) => {
                    self.state.ui.add_to_connections_search(c);
                    // Update filtered connections after character addition
                    self.state
                        .ui
                        .update_filtered_connections(&self.state.db.connections.connections);
                    return Ok(());
                }
                _ => {}
            }
        }

        // Handle tables search input
        if self.state.ui.tables_search_active && self.state.ui.focused_pane == FocusedPane::Tables {
            match key.code {
                KeyCode::Backspace => {
                    self.state.ui.backspace_tables_search();
                    return Ok(());
                }
                KeyCode::Enter => {
                    // Select the highlighted table and open it for viewing
                    self.state.open_table_for_viewing().await;
                    // Exit search mode
                    self.state.ui.exit_tables_search();
                    return Ok(());
                }
                KeyCode::Down => {
                    self.state.ui.table_search_selection_down();
                    return Ok(());
                }
                KeyCode::Up => {
                    self.state.ui.table_search_selection_up();
                    return Ok(());
                }
                KeyCode::Char(c) => {
                    self.state.ui.add_to_tables_search(c);
                    return Ok(());
                }
                _ => {}
            }
        }

        // Handle input modes in SQL files pane
        if self.state.ui.focused_pane == FocusedPane::SqlFiles {
            if self.state.ui.sql_files_search_active {
                match key.code {
                    KeyCode::Esc => {
                        self.state.ui.exit_sql_files_search();
                        return Ok(());
                    }
                    KeyCode::Backspace => {
                        self.state.ui.backspace_sql_files_search();
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        // Load the selected SQL file before exiting search mode
                        if let Err(e) = self.state.load_selected_sql_file() {
                            self.state
                                .toast_manager
                                .error(format!("Failed to load SQL file: {e}"));
                        } else {
                            self.state.toast_manager.success("SQL file loaded");
                        }
                        self.state.ui.exit_sql_files_search();
                        return Ok(());
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.state.update_sql_file_selection_for_filtered(1);
                        return Ok(());
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.state.update_sql_file_selection_for_filtered(-1);
                        return Ok(());
                    }
                    KeyCode::Char(c) => {
                        // Don't handle navigation keys as input
                        if c != 'j' && c != 'k' {
                            self.state.ui.add_to_sql_files_search(c);
                        }
                        return Ok(());
                    }
                    _ => {}
                }
            } else if self.state.ui.sql_files_rename_mode {
                match key.code {
                    KeyCode::Esc => {
                        self.state.ui.exit_sql_files_rename();
                        return Ok(());
                    }
                    KeyCode::Backspace => {
                        self.state.ui.backspace_sql_files_rename();
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        let new_name = self.state.ui.sql_files_rename_buffer.clone();
                        if !new_name.is_empty() {
                            let filtered_files = self.state.get_filtered_sql_files();
                            let selected_index = self.state.get_filtered_sql_file_selection();
                            if let Some(old_name) = filtered_files.get(selected_index) {
                                if let Some(original_index) = self
                                    .state
                                    .saved_sql_files
                                    .iter()
                                    .position(|f| f == old_name)
                                {
                                    if let Err(e) =
                                        self.state.rename_sql_file(original_index, &new_name)
                                    {
                                        self.state
                                            .toast_manager
                                            .error(format!("Failed to rename file: {e}"));
                                    } else {
                                        self.state
                                            .toast_manager
                                            .success("File renamed successfully");
                                    }
                                }
                            }
                        }
                        self.state.ui.exit_sql_files_rename();
                        return Ok(());
                    }
                    KeyCode::Char(c) => {
                        self.state.ui.add_to_sql_files_rename(c);
                        return Ok(());
                    }
                    _ => {}
                }
            } else if self.state.ui.sql_files_create_mode {
                match key.code {
                    KeyCode::Esc => {
                        self.state.ui.exit_sql_files_create();
                        return Ok(());
                    }
                    KeyCode::Backspace => {
                        self.state.ui.backspace_sql_files_create();
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        let filename = self.state.ui.sql_files_create_buffer.clone();
                        if !filename.is_empty() {
                            if let Err(e) = self.state.create_sql_file(&filename) {
                                self.state
                                    .toast_manager
                                    .error(format!("Failed to create file: {e}"));
                            } else {
                                self.state
                                    .toast_manager
                                    .success("File created successfully");
                            }
                        }
                        self.state.ui.exit_sql_files_create();
                        return Ok(());
                    }
                    KeyCode::Char(c) => {
                        self.state.ui.add_to_sql_files_create(c);
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        match self.mode {
            Mode::Normal => {
                match (key.modifiers, key.code) {
                    // Enter command mode with ':'
                    (KeyModifiers::NONE, KeyCode::Char(':')) => {
                        self.mode = Mode::Command;
                        self.command_buffer.clear();
                    }
                    // Pane navigation with Ctrl+h/j/k/l (directional movement)
                    (KeyModifiers::CONTROL, KeyCode::Char('h')) => {
                        self.state.ui.cancel_pending_gg();
                        self.state.move_focus_left();
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('j')) => {
                        self.state.ui.cancel_pending_gg();
                        self.state.move_focus_down();
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                        self.state.ui.cancel_pending_gg();
                        self.state.move_focus_up();
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                        self.state.ui.cancel_pending_gg();
                        self.state.move_focus_right();
                    }
                    // Tab to cycle through panes
                    (KeyModifiers::NONE, KeyCode::Tab) => {
                        self.state.ui.cancel_pending_gg();
                        self.state.cycle_focus_forward();
                    }
                    (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                        self.state.ui.cancel_pending_gg();
                        self.state.cycle_focus_backward();
                    }
                    // Vim-style navigation within panes
                    (KeyModifiers::NONE, KeyCode::Char('j')) => {
                        if self.state.ui.focused_pane == FocusedPane::Connections
                            && !self.state.ui.connections_search_active
                        {
                            self.state
                                .ui
                                .connections_selection_down(&self.state.db.connections.connections);
                        } else if self.state.ui.focused_pane == FocusedPane::Tables
                            && !self.state.ui.tables_search_active
                        {
                            self.state.ui.table_search_selection_down();
                            // Cancel any pending gg command
                            self.state.ui.cancel_pending_gg();
                        } else if self.state.ui.focused_pane == FocusedPane::SqlFiles
                            && !self.state.ui.sql_files_search_active
                            && !self.state.ui.sql_files_rename_mode
                            && !self.state.ui.sql_files_create_mode
                        {
                            self.state.update_sql_file_selection_for_filtered(1);
                        } else {
                            self.state.move_down();
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Char('k')) => {
                        if self.state.ui.focused_pane == FocusedPane::Connections
                            && !self.state.ui.connections_search_active
                        {
                            self.state
                                .ui
                                .connections_selection_up(&self.state.db.connections.connections);
                        } else if self.state.ui.focused_pane == FocusedPane::Tables
                            && !self.state.ui.tables_search_active
                        {
                            self.state.ui.table_search_selection_up();
                            // Cancel any pending gg command
                            self.state.ui.cancel_pending_gg();
                        } else if self.state.ui.focused_pane == FocusedPane::SqlFiles
                            && !self.state.ui.sql_files_search_active
                            && !self.state.ui.sql_files_rename_mode
                            && !self.state.ui.sql_files_create_mode
                        {
                            self.state.update_sql_file_selection_for_filtered(-1);
                        } else {
                            self.state.move_up();
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Char('h')) => {
                        self.state.move_left();
                    }
                    (KeyModifiers::NONE, KeyCode::Char('l')) => {
                        self.state.move_right();
                    }
                    // Enter insert mode (or Query mode for query window, or edit cell in table viewer)
                    (KeyModifiers::NONE, KeyCode::Char('i')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Start editing cell in table viewer
                            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                tab.start_edit();
                            }
                        } else if self.state.ui.focused_pane == FocusedPane::QueryWindow
                            || self.state.ui.focused_pane == FocusedPane::SqlFiles
                        {
                            self.mode = Mode::Query;
                        } else {
                            self.mode = Mode::Insert;
                        }
                    }
                    // Enter visual mode
                    (KeyModifiers::NONE, KeyCode::Char('v')) => {
                        self.mode = Mode::Visual;
                    }
                    // Show help overlay (already handled globally)
                    // This branch is kept for backwards compatibility
                    (KeyModifiers::NONE, KeyCode::Char('?')) => {
                        // Already handled globally
                    }
                    // Add connection (only in connections pane)
                    (KeyModifiers::NONE, KeyCode::Char('a')) => {
                        if self.state.ui.focused_pane == crate::app::state::FocusedPane::Connections
                        {
                            self.state.open_add_connection_modal();
                        }
                    }
                    // Edit table/connection based on focused pane
                    (KeyModifiers::NONE, KeyCode::Char('e')) => {
                        if self.state.ui.focused_pane == crate::app::state::FocusedPane::Tables
                            && !self.state.db.tables.is_empty()
                        {
                            // Check if we have an active connection
                            if let Some(connection) = self
                                .state
                                .db
                                .connections
                                .connections
                                .get(self.state.ui.selected_connection)
                            {
                                if matches!(
                                    connection.status,
                                    crate::database::ConnectionStatus::Connected
                                ) {
                                    self.state.open_table_editor().await;
                                }
                            }
                        } else if self.state.ui.focused_pane
                            == crate::app::state::FocusedPane::Connections
                            && !self.state.db.connections.connections.is_empty()
                        {
                            self.state.open_edit_connection_modal();
                        }
                    }
                    // Create new table (only in tables pane when connected) or next search result
                    (KeyModifiers::NONE, KeyCode::Char('n')) => {
                        if self.state.ui.focused_pane == crate::app::state::FocusedPane::Tables {
                            // Check if we have an active connection
                            if let Some(connection) = self
                                .state
                                .db
                                .connections
                                .connections
                                .get(self.state.ui.selected_connection)
                            {
                                if matches!(
                                    connection.status,
                                    crate::database::ConnectionStatus::Connected
                                ) {
                                    self.state.open_table_creator();
                                }
                            }
                        } else if self.state.ui.focused_pane == FocusedPane::SqlFiles {
                            // Create new SQL file
                            if !self.state.ui.sql_files_search_active
                                && !self.state.ui.sql_files_rename_mode
                                && !self.state.ui.sql_files_create_mode
                            {
                                self.state.ui.enter_sql_files_create();
                            }
                        } else if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Next search result (when not in search mode)
                            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                if !tab.in_edit_mode
                                    && !tab.in_search_mode
                                    && !tab.search_results.is_empty()
                                {
                                    tab.next_search_result();
                                }
                            }
                        }
                    }
                    // Previous search result
                    (KeyModifiers::NONE, KeyCode::Char('N')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Previous search result (when not in search mode)
                            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                if !tab.in_edit_mode
                                    && !tab.in_search_mode
                                    && !tab.search_results.is_empty()
                                {
                                    tab.prev_search_result();
                                }
                            }
                        }
                    }
                    // Handle 'x' key for different panes
                    (KeyModifiers::NONE, KeyCode::Char('x')) => {
                        match self.state.ui.focused_pane {
                            FocusedPane::Connections => {
                                // Disconnect current connection
                                self.state.disconnect_from_database().await;
                                self.state.toast_manager.info("Connection disconnected");
                            }
                            FocusedPane::TabularOutput => {
                                // Close current tab
                                self.state.table_viewer_state.close_current_tab();
                            }
                            _ => {}
                        }
                    }
                    // Copy SQL file (only in SQL Files pane)
                    (KeyModifiers::NONE, KeyCode::Char('c'))
                        if self.state.ui.focused_pane == FocusedPane::SqlFiles =>
                    {
                        if !self.state.ui.sql_files_search_active
                            && !self.state.ui.sql_files_rename_mode
                            && !self.state.ui.sql_files_create_mode
                            && !self.state.saved_sql_files.is_empty()
                        {
                            let filtered_files = self.state.get_filtered_sql_files();
                            let selected_index = self.state.get_filtered_sql_file_selection();
                            if let Some(filename) = filtered_files.get(selected_index) {
                                // Find the original index in the full list
                                if let Some(original_index) = self
                                    .state
                                    .saved_sql_files
                                    .iter()
                                    .position(|f| f == filename)
                                {
                                    let new_name = format!("{}_copy", filename);
                                    if let Err(e) =
                                        self.state.duplicate_sql_file(original_index, &new_name)
                                    {
                                        self.state
                                            .toast_manager
                                            .error(format!("Failed to copy file: {e}"));
                                    } else {
                                        self.state
                                            .toast_manager
                                            .success(format!("File copied as {new_name}"));
                                    }
                                }
                            }
                        }
                    }
                    // Delete connection with confirmation (only in Connections pane)
                    (KeyModifiers::NONE, KeyCode::Char('d'))
                        if self.state.ui.focused_pane
                            == crate::app::state::FocusedPane::Connections =>
                    {
                        if let Some(connection) = self
                            .state
                            .db
                            .connections
                            .connections
                            .get(self.state.ui.selected_connection)
                        {
                            let conn_name = connection.name.clone();
                            self.state.ui.confirmation_modal = Some(crate::ui::ConfirmationModal {
                                title: "Delete Connection".to_string(),
                                message: format!(
                                    "Are you sure you want to delete the connection '{conn_name}'?"
                                ),
                                action: crate::ui::ConfirmationAction::DeleteConnection(
                                    self.state.ui.selected_connection,
                                ),
                            });
                        }
                    }
                    // Delete SQL file with confirmation (only in SQL Files pane)
                    (KeyModifiers::NONE, KeyCode::Char('d'))
                        if self.state.ui.focused_pane == FocusedPane::SqlFiles =>
                    {
                        if !self.state.ui.sql_files_search_active
                            && !self.state.ui.sql_files_rename_mode
                            && !self.state.ui.sql_files_create_mode
                            && !self.state.saved_sql_files.is_empty()
                        {
                            let filtered_files = self.state.get_filtered_sql_files();
                            let selected_index = self.state.get_filtered_sql_file_selection();
                            if let Some(filename) = filtered_files.get(selected_index) {
                                self.state.ui.confirmation_modal =
                                    Some(crate::ui::ConfirmationModal {
                                        title: "Delete SQL File".to_string(),
                                        message: format!(
                                        "Are you sure you want to delete the SQL file '{filename}'?"
                                    ),
                                        action: crate::ui::ConfirmationAction::DeleteSqlFile(
                                            // Find the original index in the full list
                                            self.state
                                                .saved_sql_files
                                                .iter()
                                                .position(|f| f == filename)
                                                .unwrap_or(0),
                                        ),
                                    });
                            }
                        }
                    }
                    // Connect/select action
                    (KeyModifiers::NONE, KeyCode::Enter)
                    | (KeyModifiers::NONE, KeyCode::Char(' ')) => {
                        if self.state.ui.focused_pane == crate::app::state::FocusedPane::Connections
                        {
                            // Always connect to selected database (disconnecting others)
                            // This will work correctly with search mode through get_selected_connection_index
                            self.state.connect_to_selected_database().await;
                        } else if self.state.ui.focused_pane
                            == crate::app::state::FocusedPane::Tables
                        {
                            // Open table for viewing
                            self.state.open_table_for_viewing().await;
                        } else if self.state.ui.focused_pane == FocusedPane::Details {
                            // Load metadata for current table if not already loaded
                            if self.state.db.current_table_metadata.is_none() {
                                if let Some(table_name) = self
                                    .state
                                    .db
                                    .tables
                                    .get(self.state.ui.selected_table)
                                    .cloned()
                                {
                                    if let Err(e) =
                                        self.state.load_table_metadata(&table_name).await
                                    {
                                        self.state
                                            .toast_manager
                                            .error(format!("Failed to load table metadata: {e}"));
                                    }
                                }
                            }
                        } else if self.state.ui.focused_pane == FocusedPane::SqlFiles {
                            // Load selected SQL file
                            if let Err(e) = self.state.load_selected_sql_file() {
                                self.state
                                    .toast_manager
                                    .error(format!("Failed to load SQL file: {e}"));
                            } else {
                                self.state.toast_manager.success("SQL file loaded");
                            }
                        }
                    }
                    // SQL Query operations - when query window or SQL files pane is focused
                    (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                        if self.state.ui.focused_pane == FocusedPane::QueryWindow
                            || self.state.ui.focused_pane == FocusedPane::SqlFiles
                        {
                            // Save current query
                            if let Err(e) = self.state.save_query() {
                                self.state
                                    .toast_manager
                                    .error(format!("Failed to save query: {e}"));
                            } else {
                                self.state.toast_manager.success("Query saved successfully");
                            }
                        }
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('o')) => {
                        if self.state.ui.focused_pane == FocusedPane::QueryWindow
                            || self.state.ui.focused_pane == FocusedPane::SqlFiles
                        {
                            // Refresh SQL file list
                            self.state.refresh_sql_files();
                            self.state.clamp_sql_file_selection();
                        }
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                        if self.state.ui.focused_pane == FocusedPane::QueryWindow
                            || self.state.ui.focused_pane == FocusedPane::SqlFiles
                        {
                            // Create new query file
                            let filename = format!(
                                "query_{}",
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs()
                            );
                            if let Err(e) = self.state.new_query_file(&filename) {
                                self.state
                                    .toast_manager
                                    .error(format!("Failed to create new query: {e}"));
                            } else {
                                self.state.toast_manager.success("New query file created");
                            }
                        }
                    }
                    (KeyModifiers::CONTROL, KeyCode::Enter) => {
                        if self.state.ui.focused_pane == FocusedPane::QueryWindow {
                            // Execute SQL query under cursor
                            if let Some(statement) = self.state.get_statement_under_cursor() {
                                // Execute the SQL statement
                                match self
                                    .state
                                    .db
                                    .execute_query(&statement, self.state.ui.selected_connection)
                                    .await
                                {
                                    Ok((columns, rows)) => {
                                        // Create a new tab for query results
                                        let query_preview = if statement.len() > 30 {
                                            format!("{}...", &statement[..30])
                                        } else {
                                            statement.clone()
                                        };
                                        let tab_name = format!("Query: {query_preview}");

                                        // Add tab and populate with results
                                        let tab_idx =
                                            self.state.table_viewer_state.add_tab(tab_name);
                                        if let Some(tab) =
                                            self.state.table_viewer_state.tabs.get_mut(tab_idx)
                                        {
                                            // Convert column names to ColumnInfo
                                            tab.columns = columns.iter().map(|name| {
                                                crate::ui::components::table_viewer::ColumnInfo {
                                                    name: name.clone(),
                                                    data_type: "TEXT".to_string(),
                                                    is_nullable: true,
                                                    is_primary_key: false,
                                                    max_display_width: 20,
                                                }
                                            }).collect();
                                            tab.rows = rows;
                                            tab.total_rows = tab.rows.len();
                                            tab.loading = false;
                                            tab.error = None;
                                        }

                                        self.state.toast_manager.success(format!(
                                            "Query executed: {} rows returned",
                                            self.state.table_viewer_state.tabs[tab_idx].rows.len()
                                        ));
                                        // Focus on tabular output to see results
                                        self.state.ui.focused_pane = FocusedPane::TabularOutput;
                                    }
                                    Err(e) => {
                                        self.state
                                            .toast_manager
                                            .error(format!("Query failed: {e}"));
                                    }
                                }
                            } else {
                                self.state
                                    .toast_manager
                                    .warning("No SQL statement under cursor");
                            }
                        }
                    }
                    // Search commands
                    (KeyModifiers::NONE, KeyCode::Char('/')) => {
                        if self.state.ui.focused_pane == FocusedPane::Connections {
                            // Start search mode in connections pane
                            self.state.ui.enter_connections_search();
                        } else if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Start search mode in table viewer
                            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                if !tab.in_edit_mode {
                                    tab.start_search();
                                }
                            }
                        } else if self.state.ui.focused_pane == FocusedPane::Tables {
                            // Start search mode in tables pane
                            self.state.ui.enter_tables_search();
                        } else if self.state.ui.focused_pane == FocusedPane::SqlFiles {
                            // Start search mode in SQL files pane
                            if !self.state.ui.sql_files_rename_mode
                                && !self.state.ui.sql_files_create_mode
                            {
                                self.state.ui.enter_sql_files_search();
                            }
                        }
                    }
                    // Handle uppercase S for previous tab (both with and without SHIFT modifier)
                    (KeyModifiers::SHIFT, KeyCode::Char('S'))
                    | (KeyModifiers::SHIFT, KeyCode::Char('s'))
                    | (KeyModifiers::NONE, KeyCode::Char('S')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Previous tab
                            self.state.table_viewer_state.prev_tab();
                        }
                    }
                    // Handle uppercase D for next tab (with SHIFT modifier)
                    (KeyModifiers::SHIFT, KeyCode::Char('D'))
                    | (KeyModifiers::SHIFT, KeyCode::Char('d'))
                    | (KeyModifiers::NONE, KeyCode::Char('D')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Next tab
                            self.state.table_viewer_state.next_tab();
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Char('r')) => {
                        match self.state.ui.focused_pane {
                            FocusedPane::TabularOutput => {
                                // Reload current table
                                if let Err(e) = self.state.reload_current_table_tab().await {
                                    self.state
                                        .toast_manager
                                        .error(format!("Failed to reload table: {e}"));
                                } else {
                                    self.state.toast_manager.info("Table data refreshed");
                                }
                            }
                            FocusedPane::Details => {
                                // Refresh table metadata
                                if let Some(table_name) = self.state.ui.get_selected_table_name() {
                                    if let Err(e) =
                                        self.state.load_table_metadata(&table_name).await
                                    {
                                        self.state
                                            .toast_manager
                                            .error(format!("Failed to refresh metadata: {e}"));
                                    } else {
                                        self.state.toast_manager.info("Table metadata refreshed");
                                        // Reset scroll position to show the new data
                                        self.state.ui.details_viewport_offset = 0;
                                    }
                                } else {
                                    self.state.toast_manager.warning("No table selected");
                                }
                            }
                            FocusedPane::SqlFiles => {
                                // Rename SQL file
                                if !self.state.ui.sql_files_search_active
                                    && !self.state.ui.sql_files_rename_mode
                                    && !self.state.ui.sql_files_create_mode
                                    && !self.state.saved_sql_files.is_empty()
                                {
                                    let filtered_files = self.state.get_filtered_sql_files();
                                    let selected_index =
                                        self.state.get_filtered_sql_file_selection();
                                    if let Some(filename) = filtered_files.get(selected_index) {
                                        self.state.ui.enter_sql_files_rename(filename);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Char('t')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Toggle between data and schema view
                            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                tab.toggle_view_mode();
                                let mode_name = match tab.view_mode {
                                    crate::ui::components::table_viewer::TableViewMode::Data => {
                                        "Data"
                                    }
                                    crate::ui::components::table_viewer::TableViewMode::Schema => {
                                        "Schema"
                                    }
                                };
                                self.state
                                    .toast_manager
                                    .info(format!("Switched to {} view", mode_name));
                            }
                        }
                    }
                    // Pagination
                    (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                if tab.page_down() {
                                    // Need to reload data
                                    let tab_idx = self.state.table_viewer_state.active_tab;
                                    if let Err(e) = self.state.load_table_data(tab_idx).await {
                                        self.state
                                            .toast_manager
                                            .error(format!("Failed to load next page: {e}"));
                                    }
                                }
                            }
                        }
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                if tab.page_up() {
                                    // Need to reload data
                                    let tab_idx = self.state.table_viewer_state.active_tab;
                                    if let Err(e) = self.state.load_table_data(tab_idx).await {
                                        self.state
                                            .toast_manager
                                            .error(format!("Failed to load previous page: {e}"));
                                    }
                                }
                            }
                        }
                    }
                    // Jump navigation in table viewer and tables pane
                    (KeyModifiers::NONE, KeyCode::Char('g')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            if self.leader_pressed {
                                // gg - jump to first row
                                if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                    tab.jump_to_first();
                                }
                                self.leader_pressed = false;
                            } else {
                                self.leader_pressed = true;
                            }
                        } else if self.state.ui.focused_pane == FocusedPane::Tables
                            && !self.state.ui.tables_search_active
                        {
                            self.state.ui.handle_g_key_press();
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Char('G')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Jump to last row
                            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                tab.jump_to_last();
                            }
                        } else if self.state.ui.focused_pane == FocusedPane::Tables
                            && !self.state.ui.tables_search_active
                        {
                            self.state.ui.table_go_to_last();
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Char('0')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Jump to first column
                            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                tab.jump_to_first_col();
                            }
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Char('$')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Jump to last column
                            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                tab.jump_to_last_col();
                            }
                        }
                    }
                    // Handle 'd' for delete row (dd vim command)
                    (KeyModifiers::NONE, KeyCode::Char('d')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Check if there's a delete confirmation dialog
                            if self.state.table_viewer_state.delete_confirmation.is_some() {
                                // This is handled elsewhere (Enter/Esc for confirm/cancel)
                            } else {
                                // Check for double 'd' press (dd command)
                                let now = std::time::Instant::now();
                                if let Some(last_press) = self.state.table_viewer_state.last_d_press
                                {
                                    if now.duration_since(last_press).as_millis() < 500 {
                                        // Double 'd' detected - prepare delete confirmation
                                        if let Some(confirmation) = self
                                            .state
                                            .table_viewer_state
                                            .prepare_delete_confirmation()
                                        {
                                            self.state.table_viewer_state.delete_confirmation =
                                                Some(confirmation);
                                        } else {
                                            self.state
                                                .toast_manager
                                                .error("Cannot delete row without primary key");
                                        }
                                        self.state.table_viewer_state.last_d_press = None;
                                    } else {
                                        self.state.table_viewer_state.last_d_press = Some(now);
                                    }
                                } else {
                                    self.state.table_viewer_state.last_d_press = Some(now);
                                }
                            }
                        }
                    }
                    // Handle 'y' for yank/copy row (yy vim command)
                    (KeyModifiers::NONE, KeyCode::Char('y')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Check for double 'y' press (yy command)
                            let now = std::time::Instant::now();
                            if let Some(last_press) = self.state.table_viewer_state.last_y_press {
                                if now.duration_since(last_press).as_millis() < 500 {
                                    // Double 'y' detected - copy row to clipboard
                                    match self.state.table_viewer_state.copy_row_csv() {
                                        Ok(()) => {
                                            self.state
                                                .toast_manager
                                                .info("Row copied to clipboard");
                                        }
                                        Err(e) => {
                                            self.state
                                                .toast_manager
                                                .error(format!("Failed to copy row: {e}"));
                                        }
                                    }
                                    self.state.table_viewer_state.last_y_press = None;
                                } else {
                                    self.state.table_viewer_state.last_y_press = Some(now);
                                }
                            } else {
                                self.state.table_viewer_state.last_y_press = Some(now);
                            }
                        }
                    }
                    _ => {
                        // Handle leader key combinations
                        if self.leader_pressed {
                            self.leader_pressed = false;
                            // Leader key combinations can be added here for future features
                            // For now, just reset the leader state
                        }
                    }
                }
            }
            Mode::Insert => {
                match key.code {
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                    }
                    _ => {
                        // Handle insert mode input
                    }
                }
            }
            Mode::Visual => {
                match key.code {
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                    }
                    _ => {
                        // Handle visual mode selection
                    }
                }
            }
            Mode::Command => {
                match key.code {
                    KeyCode::Esc => {
                        self.command_buffer.clear();
                        self.mode = Mode::Normal;
                    }
                    KeyCode::Enter => {
                        // Execute command
                        let command = self.command_buffer.trim();
                        if command == "q" || command == "quit" {
                            self.execute_command(CommandId::Quit)?;
                        }
                        self.command_buffer.clear();
                        self.mode = Mode::Normal;
                    }
                    KeyCode::Char(c) => {
                        self.command_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        self.command_buffer.pop();
                    }
                    _ => {}
                }
            }
            Mode::Query => {
                use crate::app::state::QueryEditMode;

                // Handle vim command mode
                if self.state.ui.in_vim_command {
                    match key.code {
                        KeyCode::Esc => {
                            self.state.ui.in_vim_command = false;
                            self.state.ui.vim_command_buffer.clear();
                        }
                        KeyCode::Enter => {
                            let command = self.state.ui.vim_command_buffer.trim();
                            if command == "w" {
                                // Save file
                                if let Err(e) = self.state.save_sql_file_with_connection() {
                                    self.state
                                        .toast_manager
                                        .error(format!("Failed to save: {e}"));
                                } else {
                                    self.state.toast_manager.success("File saved");
                                }
                            } else if command == "q" {
                                // Quit query mode
                                if self.state.ui.query_modified {
                                    self.state.toast_manager.warning(
                                        "Unsaved changes! Use :w to save or :q! to force quit",
                                    );
                                } else {
                                    self.mode = Mode::Normal;
                                }
                            } else if command == "q!" {
                                // Force quit
                                self.mode = Mode::Normal;
                            } else if command == "wq" {
                                // Save and quit
                                if let Err(e) = self.state.save_sql_file_with_connection() {
                                    self.state
                                        .toast_manager
                                        .error(format!("Failed to save: {e}"));
                                } else {
                                    self.state.toast_manager.success("File saved");
                                    self.mode = Mode::Normal;
                                }
                            }
                            self.state.ui.in_vim_command = false;
                            self.state.ui.vim_command_buffer.clear();
                        }
                        KeyCode::Char(c) => {
                            self.state.ui.vim_command_buffer.push(c);
                        }
                        KeyCode::Backspace => {
                            self.state.ui.vim_command_buffer.pop();
                        }
                        _ => {}
                    }
                } else if self.state.ui.query_edit_mode == QueryEditMode::Insert {
                    // Insert mode - handle text input
                    match key.code {
                        KeyCode::Esc => {
                            self.state.ui.query_edit_mode = QueryEditMode::Normal;
                        }
                        KeyCode::Enter => {
                            self.state.insert_char_at_cursor('\n');
                        }
                        KeyCode::Char(c) => {
                            self.state.insert_char_at_cursor(c);
                        }
                        KeyCode::Backspace => {
                            self.state.delete_char_at_cursor();
                        }
                        KeyCode::Left => {
                            self.state.move_left();
                        }
                        KeyCode::Right => {
                            self.state.move_right();
                        }
                        KeyCode::Up => {
                            self.state.move_up();
                        }
                        KeyCode::Down => {
                            self.state.move_down();
                        }
                        _ => {}
                    }
                } else {
                    // Normal mode - vim navigation
                    match key.code {
                        KeyCode::Esc => {
                            self.mode = Mode::Normal;
                        }
                        KeyCode::Char('i') => {
                            self.state.ui.query_edit_mode = QueryEditMode::Insert;
                        }
                        KeyCode::Char(':') => {
                            self.state.ui.in_vim_command = true;
                            self.state.ui.vim_command_buffer.clear();
                        }
                        // Vim navigation
                        KeyCode::Char('h') => {
                            self.state.move_left();
                        }
                        KeyCode::Char('j') => {
                            self.state.move_down();
                        }
                        KeyCode::Char('k') => {
                            self.state.move_up();
                        }
                        KeyCode::Char('l') => {
                            self.state.move_right();
                        }
                        // Word navigation
                        KeyCode::Char('w') => {
                            self.state.move_to_next_word();
                        }
                        // Note: 'b' key navigation removed to avoid confusion with Ctrl+B debug toggle
                        KeyCode::Char('e') => {
                            self.state.move_to_end_of_word();
                        }
                        // Line navigation
                        KeyCode::Char('0') => {
                            self.state.move_to_line_start();
                        }
                        KeyCode::Char('$') => {
                            self.state.move_to_line_end();
                        }
                        // File navigation
                        KeyCode::Char('g') => {
                            // Check for double 'g' (gg command)
                            // Note: For simplicity, using single 'g' for now
                            // TODO: Implement proper double-key detection
                            self.state.move_to_file_start();
                        }
                        KeyCode::Char('G') => {
                            self.state.move_to_file_end();
                        }
                        // Page scrolling
                        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.state.scroll_half_page_down();
                        }
                        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.state.scroll_half_page_up();
                        }
                        // Execute query
                        KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if let Some(statement) = self.state.get_statement_under_cursor() {
                                // TODO: Execute the SQL statement
                                println!("Executing SQL: {statement}");
                            }
                        }
                        // Legacy shortcuts still work in normal mode
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if let Err(e) = self.state.save_sql_file_with_connection() {
                                self.state
                                    .toast_manager
                                    .error(format!("Failed to save: {e}"));
                            } else {
                                self.state.toast_manager.success("File saved");
                            }
                        }
                        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Create new query file
                            let filename = format!(
                                "query_{}.sql",
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs()
                            );
                            self.state.ui.current_sql_file = Some(filename);
                            self.state.query_content = String::new();
                            self.state.ui.query_modified = false;
                            self.state.ui.query_cursor_line = 0;
                            self.state.ui.query_cursor_column = 0;
                            self.state.toast_manager.success("New query file created");
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle connection modal key events
    async fn handle_connection_modal_key_event(&mut self, key: KeyEvent) -> Result<()> {
        use crate::ui::components::{ConnectionField, PasswordStorageType};

        match key.code {
            // Direct text input for text fields
            KeyCode::Char(c) if self.state.connection_modal_state.is_text_field() => {
                self.state.connection_modal_state.handle_char_input(c);
            }
            KeyCode::Backspace if self.state.connection_modal_state.is_text_field() => {
                self.state.connection_modal_state.handle_backspace();
            }
            KeyCode::Esc => {
                // In connection details step, Esc goes back to database type selection
                if self.state.connection_modal_state.current_step
                    == crate::ui::components::ModalStep::ConnectionDetails
                {
                    self.state.connection_modal_state.go_back();
                } else {
                    // Close the appropriate modal
                    if self.state.ui.show_add_connection_modal {
                        self.state.close_add_connection_modal();
                    } else {
                        self.state.close_edit_connection_modal();
                    }
                }
            }
            KeyCode::Tab => {
                // Use smart navigation that skips irrelevant fields
                self.state.connection_modal_state.focused_field =
                    self.state.connection_modal_state.get_smart_next_field();
            }
            KeyCode::BackTab => {
                // Use smart navigation that skips irrelevant fields
                self.state.connection_modal_state.focused_field =
                    self.state.connection_modal_state.get_smart_previous_field();
            }
            KeyCode::Down
                if matches!(
                    self.state.connection_modal_state.focused_field,
                    ConnectionField::DatabaseType
                        | ConnectionField::SslMode
                        | ConnectionField::PasswordStorageType
                ) =>
            {
                // Handle dropdown navigation
                match self.state.connection_modal_state.focused_field {
                    ConnectionField::DatabaseType => {
                        let current = self
                            .state
                            .connection_modal_state
                            .db_type_list_state
                            .selected()
                            .unwrap_or(0);
                        let new_index = (current + 1).min(3); // 4 database types (0-3)
                        self.state
                            .connection_modal_state
                            .select_database_type(new_index);
                    }
                    ConnectionField::SslMode => {
                        let current = self
                            .state
                            .connection_modal_state
                            .ssl_list_state
                            .selected()
                            .unwrap_or(0);
                        let new_index = (current + 1).min(5); // 6 SSL modes (0-5)
                        self.state.connection_modal_state.select_ssl_mode(new_index);
                    }
                    ConnectionField::PasswordStorageType => {
                        self.state
                            .connection_modal_state
                            .cycle_password_storage_type();
                    }
                    _ => {}
                }
            }
            KeyCode::Up
                if matches!(
                    self.state.connection_modal_state.focused_field,
                    ConnectionField::DatabaseType
                        | ConnectionField::SslMode
                        | ConnectionField::PasswordStorageType
                ) =>
            {
                // Handle dropdown navigation
                match self.state.connection_modal_state.focused_field {
                    ConnectionField::DatabaseType => {
                        let current = self
                            .state
                            .connection_modal_state
                            .db_type_list_state
                            .selected()
                            .unwrap_or(0);
                        let new_index = current.saturating_sub(1);
                        self.state
                            .connection_modal_state
                            .select_database_type(new_index);
                    }
                    ConnectionField::SslMode => {
                        let current = self
                            .state
                            .connection_modal_state
                            .ssl_list_state
                            .selected()
                            .unwrap_or(0);
                        let new_index = current.saturating_sub(1);
                        self.state.connection_modal_state.select_ssl_mode(new_index);
                    }
                    ConnectionField::PasswordStorageType => {
                        // Cycle backwards through password storage types
                        self.state.connection_modal_state.password_storage_type =
                            match self.state.connection_modal_state.password_storage_type {
                                PasswordStorageType::PlainText => PasswordStorageType::Encrypted,
                                PasswordStorageType::Environment => PasswordStorageType::PlainText,
                                PasswordStorageType::Encrypted => PasswordStorageType::Environment,
                            };
                    }
                    _ => {}
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.connection_modal_state.next_field();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.connection_modal_state.previous_field();
            }
            KeyCode::Enter => {
                match self.state.connection_modal_state.focused_field {
                    ConnectionField::Save => {
                        // Try to save the connection
                        if let Err(error) = self.state.save_connection_from_modal() {
                            self.state
                                .toast_manager
                                .error(format!("Failed to save connection: {}", &error));
                            self.state.connection_modal_state.error_message = Some(error);
                        } else {
                            self.state
                                .toast_manager
                                .success("Connection saved successfully");
                        }
                    }
                    ConnectionField::Cancel => {
                        // Close the appropriate modal
                        if self.state.ui.show_add_connection_modal {
                            self.state.close_add_connection_modal();
                        } else {
                            self.state.close_edit_connection_modal();
                        }
                    }
                    ConnectionField::Test => {
                        // Test the connection
                        self.test_connection_from_modal().await;
                    }
                    ConnectionField::DatabaseType => {
                        // In database type selection step, Enter advances to next step
                        if self.state.connection_modal_state.current_step
                            == crate::ui::components::ModalStep::DatabaseTypeSelection
                        {
                            self.state.connection_modal_state.advance_step();
                        } else {
                            self.state.connection_modal_state.next_field();
                        }
                    }
                    _ => {
                        // For regular fields, Enter moves to next field
                        self.state.connection_modal_state.next_field();
                    }
                }
            }
            KeyCode::Char('s') => {
                // Save shortcut - works from any field
                if let Err(error) = self.state.save_connection_from_modal() {
                    self.state
                        .toast_manager
                        .error(format!("Failed to save connection: {}", &error));
                    self.state.connection_modal_state.error_message = Some(error);
                } else {
                    self.state
                        .toast_manager
                        .success("Connection saved successfully");
                }
            }
            KeyCode::Char('c') => {
                // Cancel shortcut - works from any field
                if self.state.ui.show_add_connection_modal {
                    self.state.close_add_connection_modal();
                } else {
                    self.state.close_edit_connection_modal();
                }
            }
            KeyCode::Char('b') => {
                // Back shortcut (only in connection details step)
                if self.state.connection_modal_state.current_step
                    == crate::ui::components::ModalStep::ConnectionDetails
                {
                    self.state.connection_modal_state.go_back();
                }
            }
            KeyCode::Char('t') => {
                // Test connection shortcut
                if self.state.connection_modal_state.current_step
                    == crate::ui::components::ModalStep::ConnectionDetails
                {
                    // Test the connection
                    self.test_connection_from_modal().await;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle table creator key events
    async fn handle_table_creator_key_event(&mut self, key: KeyEvent) -> Result<()> {
        use crate::ui::components::{ColumnField, TableCreatorField};

        // Handle insert mode for text fields
        if self.state.table_creator_state.in_insert_mode {
            match key.code {
                KeyCode::Esc => {
                    // Exit insert mode
                    self.state.table_creator_state.exit_insert_mode();
                }
                KeyCode::Char(c) => {
                    self.state.table_creator_state.handle_char_input(c);
                }
                KeyCode::Backspace => {
                    self.state.table_creator_state.handle_backspace();
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Char('i') => {
                // Enter insert mode for text fields
                self.state.table_creator_state.enter_insert_mode();
            }
            KeyCode::Esc => {
                // Close table creator
                self.state.close_table_creator();
            }
            KeyCode::Tab => {
                self.state.table_creator_state.next_field();
            }
            KeyCode::BackTab => {
                self.state.table_creator_state.previous_field();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // Special handling for data type dropdown
                if let TableCreatorField::Column(idx, ColumnField::DataType) =
                    self.state.table_creator_state.focused_field
                {
                    // Navigate data type dropdown
                    let current = self
                        .state
                        .table_creator_state
                        .data_type_list_state
                        .selected()
                        .unwrap_or(0);
                    let types = crate::ui::components::PostgresDataType::common_types();
                    let new_index = (current + 1).min(types.len() - 1);
                    self.state
                        .table_creator_state
                        .data_type_list_state
                        .select(Some(new_index));

                    // Update the column's data type
                    if let Some(column) = self.state.table_creator_state.columns.get_mut(idx) {
                        if let Some(new_type) = types.get(new_index) {
                            column.data_type = new_type.clone();
                        }
                    }
                } else {
                    self.state.table_creator_state.next_field();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                // Special handling for data type dropdown
                if let TableCreatorField::Column(idx, ColumnField::DataType) =
                    self.state.table_creator_state.focused_field
                {
                    // Navigate data type dropdown
                    let current = self
                        .state
                        .table_creator_state
                        .data_type_list_state
                        .selected()
                        .unwrap_or(0);
                    let new_index = current.saturating_sub(1);
                    self.state
                        .table_creator_state
                        .data_type_list_state
                        .select(Some(new_index));

                    // Update the column's data type
                    let types = crate::ui::components::PostgresDataType::common_types();
                    if let Some(column) = self.state.table_creator_state.columns.get_mut(idx) {
                        if let Some(new_type) = types.get(new_index) {
                            column.data_type = new_type.clone();
                        }
                    }
                } else {
                    self.state.table_creator_state.previous_field();
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Handle different field actions
                match self.state.table_creator_state.focused_field {
                    TableCreatorField::Column(_, field) => {
                        match field {
                            ColumnField::Nullable
                            | ColumnField::PrimaryKey
                            | ColumnField::Unique => {
                                // Toggle boolean fields
                                self.state.table_creator_state.toggle_boolean_field();
                            }
                            ColumnField::Delete => {
                                // Delete the current column
                                self.state.table_creator_state.delete_current_column();
                            }
                            _ => {
                                // Move to next field for other columns
                                self.state.table_creator_state.next_field();
                            }
                        }
                    }
                    TableCreatorField::AddColumn => {
                        self.state.table_creator_state.add_column();
                    }
                    TableCreatorField::Save => {
                        // Save the table
                        if let Err(error) = self.state.create_table_from_creator().await {
                            self.state
                                .toast_manager
                                .error(format!("Failed to create table: {}", &error));
                            self.state.table_creator_state.error_message = Some(error);
                        } else {
                            self.state
                                .toast_manager
                                .success("Table created successfully");
                        }
                    }
                    TableCreatorField::Cancel => {
                        self.state.close_table_creator();
                    }
                    _ => {
                        self.state.table_creator_state.next_field();
                    }
                }
            }
            KeyCode::Char('a') => {
                // Quick add column
                self.state.table_creator_state.add_column();
            }
            KeyCode::Char('d') => {
                // Quick delete column when on a column row
                if let TableCreatorField::Column(_, _) =
                    self.state.table_creator_state.focused_field
                {
                    self.state.table_creator_state.delete_current_column();
                }
            }
            KeyCode::Char('s') => {
                // Quick save
                if let Err(error) = self.state.create_table_from_creator().await {
                    self.state
                        .toast_manager
                        .error(format!("Failed to create table: {}", &error));
                    self.state.table_creator_state.error_message = Some(error);
                } else {
                    self.state
                        .toast_manager
                        .success("Table created successfully");
                }
            }
            KeyCode::Char('c') => {
                // Quick cancel
                self.state.close_table_creator();
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle table editor key events
    async fn handle_table_editor_key_event(&mut self, key: KeyEvent) -> Result<()> {
        use crate::ui::components::{ColumnField, TableCreatorField};

        // Handle insert mode for text fields
        if self.state.table_editor_state.in_insert_mode {
            match key.code {
                KeyCode::Esc => {
                    // Exit insert mode
                    self.state.table_editor_state.exit_insert_mode();
                }
                KeyCode::Char(c) => {
                    self.state.table_editor_state.handle_char_input(c);
                }
                KeyCode::Backspace => {
                    self.state.table_editor_state.handle_backspace();
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Char('i') => {
                // Enter insert mode for text fields
                self.state.table_editor_state.enter_insert_mode();
            }
            KeyCode::Esc => {
                // Close table editor
                self.state.close_table_editor();
            }
            KeyCode::Tab => {
                self.state.table_editor_state.next_field();
            }
            KeyCode::BackTab => {
                self.state.table_editor_state.previous_field();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // Special handling for data type dropdown
                if let TableCreatorField::Column(idx, ColumnField::DataType) =
                    self.state.table_editor_state.focused_field
                {
                    // Navigate data type dropdown
                    let current = self
                        .state
                        .table_editor_state
                        .data_type_list_state
                        .selected()
                        .unwrap_or(0);
                    let types = crate::ui::components::PostgresDataType::common_types();
                    let new_index = (current + 1).min(types.len() - 1);
                    self.state
                        .table_editor_state
                        .data_type_list_state
                        .select(Some(new_index));

                    // Update the column's data type
                    if let Some(column) = self.state.table_editor_state.columns.get_mut(idx) {
                        if let Some(new_type) = types.get(new_index) {
                            column.data_type = new_type.clone();
                        }
                    }
                } else {
                    self.state.table_editor_state.next_field();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                // Special handling for data type dropdown
                if let TableCreatorField::Column(idx, ColumnField::DataType) =
                    self.state.table_editor_state.focused_field
                {
                    // Navigate data type dropdown
                    let current = self
                        .state
                        .table_editor_state
                        .data_type_list_state
                        .selected()
                        .unwrap_or(0);
                    let new_index = current.saturating_sub(1);
                    self.state
                        .table_editor_state
                        .data_type_list_state
                        .select(Some(new_index));

                    // Update the column's data type
                    let types = crate::ui::components::PostgresDataType::common_types();
                    if let Some(column) = self.state.table_editor_state.columns.get_mut(idx) {
                        if let Some(new_type) = types.get(new_index) {
                            column.data_type = new_type.clone();
                        }
                    }
                } else {
                    self.state.table_editor_state.previous_field();
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Handle different field actions
                match self.state.table_editor_state.focused_field {
                    TableCreatorField::Column(_, field) => {
                        match field {
                            ColumnField::Nullable
                            | ColumnField::PrimaryKey
                            | ColumnField::Unique => {
                                // Toggle boolean fields
                                self.state.table_editor_state.toggle_boolean_field();
                            }
                            ColumnField::Delete => {
                                // Delete the current column
                                self.state.table_editor_state.delete_current_column();
                            }
                            _ => {
                                // Move to next field for other columns
                                self.state.table_editor_state.next_field();
                            }
                        }
                    }
                    TableCreatorField::AddColumn => {
                        self.state.table_editor_state.add_column();
                    }
                    TableCreatorField::Save => {
                        // Save the table changes
                        if let Err(error) = self.state.apply_table_edits_from_editor().await {
                            self.state
                                .toast_manager
                                .error(format!("Failed to apply table changes: {}", &error));
                            self.state.table_editor_state.error_message = Some(error);
                        } else {
                            self.state
                                .toast_manager
                                .success("Table updated successfully");
                        }
                    }
                    TableCreatorField::Cancel => {
                        self.state.close_table_editor();
                    }
                    _ => {
                        self.state.table_editor_state.next_field();
                    }
                }
            }
            KeyCode::Char('a') => {
                // Quick add column
                self.state.table_editor_state.add_column();
            }
            KeyCode::Char('d') => {
                // Quick delete column when on a column row
                if let TableCreatorField::Column(_, _) = self.state.table_editor_state.focused_field
                {
                    self.state.table_editor_state.delete_current_column();
                }
            }
            KeyCode::Char('s') => {
                // Quick save
                if let Err(error) = self.state.apply_table_edits_from_editor().await {
                    self.state
                        .toast_manager
                        .error(format!("Failed to apply table changes: {}", &error));
                    self.state.table_editor_state.error_message = Some(error);
                } else {
                    self.state
                        .toast_manager
                        .success("Table updated successfully");
                }
            }
            KeyCode::Char('c') => {
                // Quick cancel
                self.state.close_table_editor();
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle periodic updates
    async fn tick(&mut self) -> Result<()> {
        // Update any time-based state here
        Ok(())
    }

    /// Test connection from modal
    async fn test_connection_from_modal(&mut self) {
        use crate::database::Connection;
        use crate::ui::components::TestConnectionStatus;

        // Set status to testing
        self.state.connection_modal_state.test_status = Some(TestConnectionStatus::Testing);

        // Try to create a connection config
        match self.state.connection_modal_state.try_create_connection() {
            Ok(config) => {
                // Create a connection instance based on database type
                use crate::database::DatabaseType;

                match config.database_type {
                    DatabaseType::PostgreSQL => {
                        use crate::database::postgres::PostgresConnection;
                        let mut conn = PostgresConnection::new(config);

                        // Try to connect
                        match conn.connect().await {
                            Ok(()) => {
                                // Try to test the connection
                                match conn.test_connection().await {
                                    Ok(()) => {
                                        self.state.connection_modal_state.test_status =
                                            Some(TestConnectionStatus::Success(
                                                "Connection successful!".to_string(),
                                            ));
                                    }
                                    Err(e) => {
                                        self.state.connection_modal_state.test_status =
                                            Some(TestConnectionStatus::Failed(format!(
                                                "Test failed: {e}"
                                            )));
                                    }
                                }
                            }
                            Err(e) => {
                                self.state.connection_modal_state.test_status = Some(
                                    TestConnectionStatus::Failed(format!("Connection failed: {e}")),
                                );
                            }
                        }
                    }
                    DatabaseType::MySQL | DatabaseType::MariaDB => {
                        use crate::database::mysql::MySqlConnection;
                        let mut conn = MySqlConnection::new(config);

                        match conn.connect().await {
                            Ok(()) => match conn.test_connection().await {
                                Ok(()) => {
                                    self.state.connection_modal_state.test_status =
                                        Some(TestConnectionStatus::Success(
                                            "Connection successful!".to_string(),
                                        ));
                                }
                                Err(e) => {
                                    self.state.connection_modal_state.test_status = Some(
                                        TestConnectionStatus::Failed(format!("Test failed: {e}")),
                                    );
                                }
                            },
                            Err(e) => {
                                self.state.connection_modal_state.test_status = Some(
                                    TestConnectionStatus::Failed(format!("Connection failed: {e}")),
                                );
                            }
                        }
                    }
                    DatabaseType::SQLite => {
                        use crate::database::sqlite::SqliteConnection;
                        let mut conn = SqliteConnection::new(config);

                        match conn.connect().await {
                            Ok(()) => match conn.test_connection().await {
                                Ok(()) => {
                                    self.state.connection_modal_state.test_status =
                                        Some(TestConnectionStatus::Success(
                                            "Connection successful!".to_string(),
                                        ));
                                }
                                Err(e) => {
                                    self.state.connection_modal_state.test_status = Some(
                                        TestConnectionStatus::Failed(format!("Test failed: {e}")),
                                    );
                                }
                            },
                            Err(e) => {
                                self.state.connection_modal_state.test_status = Some(
                                    TestConnectionStatus::Failed(format!("Connection failed: {e}")),
                                );
                            }
                        }
                    }
                    _ => {
                        self.state.connection_modal_state.test_status =
                            Some(TestConnectionStatus::Failed(
                                "Database type not yet supported".to_string(),
                            ));
                    }
                }
            }
            Err(e) => {
                self.state.connection_modal_state.test_status = Some(TestConnectionStatus::Failed(
                    format!("Invalid configuration: {e}"),
                ));
            }
        }
    }
}
