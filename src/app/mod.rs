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

        // Handle '?' key globally for context-aware help
        if key.code == KeyCode::Char('?') && key.modifiers == KeyModifiers::NONE {
            self.execute_command(CommandId::Help)?;
            return Ok(());
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
            {
                // Check if we're editing in table viewer
                if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                    if let Some(tab) = self.state.table_viewer_state.current_tab() {
                        if !tab.in_edit_mode && !tab.in_search_mode {
                            self.execute_command(CommandId::Quit)?;
                            return Ok(());
                        }
                    } else {
                        self.should_quit = true;
                        return Ok(());
                    }
                } else {
                    self.should_quit = true;
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
                            tab.search_query.push(c);
                            tab.update_search(&tab.search_query.clone());
                            return Ok(());
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
                        self.state.move_focus_left();
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('j')) => {
                        self.state.move_focus_down();
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                        self.state.move_focus_up();
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                        self.state.move_focus_right();
                    }
                    // Tab to cycle through panes
                    (KeyModifiers::NONE, KeyCode::Tab) => {
                        self.state.cycle_focus_forward();
                    }
                    (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                        self.state.cycle_focus_backward();
                    }
                    // Vim-style navigation within panes
                    (KeyModifiers::NONE, KeyCode::Char('j')) => {
                        self.state.move_down();
                    }
                    (KeyModifiers::NONE, KeyCode::Char('k')) => {
                        self.state.move_up();
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
                    // Connect/select action
                    (KeyModifiers::NONE, KeyCode::Enter) => {
                        if self.state.ui.focused_pane == crate::app::state::FocusedPane::Connections
                        {
                            // Handle database connection
                            if let Some(connection) = self
                                .state
                                .db
                                .connections
                                .connections
                                .get(self.state.ui.selected_connection)
                            {
                                match &connection.status {
                                    crate::database::ConnectionStatus::Connected => {
                                        // Disconnect if already connected
                                        self.state.disconnect_from_database();
                                    }
                                    _ => {
                                        // Connect if not connected or failed
                                        self.state.connect_to_selected_database().await;
                                    }
                                }
                            }
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
                                // TODO: Execute the SQL statement
                                println!("Executing SQL: {statement}");
                            }
                        }
                    }
                    // Table viewer specific commands
                    (KeyModifiers::NONE, KeyCode::Char('/')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Start search mode
                            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                if !tab.in_edit_mode {
                                    tab.start_search();
                                }
                            }
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Char('x')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Close current tab
                            self.state.table_viewer_state.close_current_tab();
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
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Reload current table
                            if let Err(e) = self.state.reload_current_table_tab().await {
                                self.state
                                    .toast_manager
                                    .error(format!("Failed to reload table: {e}"));
                            } else {
                                self.state.toast_manager.info("Table data refreshed");
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
                    // Jump navigation in table viewer
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
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Char('G')) => {
                        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
                            // Jump to last row
                            if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                                tab.jump_to_last();
                            }
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
                    // Leader key (Space) commands OR Connect in Connections pane
                    (KeyModifiers::NONE, KeyCode::Char(' ')) => {
                        if self.state.ui.focused_pane == crate::app::state::FocusedPane::Connections
                        {
                            // Handle database connection (same as Enter)
                            if let Some(connection) = self
                                .state
                                .db
                                .connections
                                .connections
                                .get(self.state.ui.selected_connection)
                            {
                                match &connection.status {
                                    crate::database::ConnectionStatus::Connected => {
                                        // Disconnect if already connected
                                        self.state.disconnect_from_database();
                                    }
                                    _ => {
                                        // Connect if not connected or failed
                                        self.state.connect_to_selected_database().await;
                                    }
                                }
                            }
                        } else {
                            // Track that space was pressed and wait for next key (leader key functionality)
                            self.leader_pressed = true;
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
                        KeyCode::Char('b') => {
                            self.state.move_to_prev_word();
                        }
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
        use crate::ui::components::ConnectionField;

        // Handle insert mode for text fields
        if self.state.connection_modal_state.in_insert_mode {
            match key.code {
                KeyCode::Esc => {
                    // Exit insert mode
                    self.state.connection_modal_state.exit_insert_mode();
                }
                KeyCode::Char(c) => {
                    self.state.connection_modal_state.handle_char_input(c);
                }
                KeyCode::Backspace => {
                    self.state.connection_modal_state.handle_backspace();
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Char('i') => {
                // Enter insert mode for text fields
                self.state.connection_modal_state.enter_insert_mode();
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
                self.state.connection_modal_state.next_field();
            }
            KeyCode::BackTab => {
                self.state.connection_modal_state.previous_field();
            }
            KeyCode::Down
                if matches!(
                    self.state.connection_modal_state.focused_field,
                    ConnectionField::DatabaseType | ConnectionField::SslMode
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
                    _ => {}
                }
            }
            KeyCode::Up
                if matches!(
                    self.state.connection_modal_state.focused_field,
                    ConnectionField::DatabaseType | ConnectionField::SslMode
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
}
