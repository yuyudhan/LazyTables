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

pub use state::{AppState, AppView, ConnectionFormMode, FocusedPane, OverlayView, TextInputMode};

/// Connection event sent from background tasks to main event loop
#[derive(Debug)]
enum ConnectionEvent {
    Success {
        connection_index: usize,
        objects: crate::database::DatabaseObjectList,
    },
    Failed {
        connection_index: usize,
        error: String,
    },
}

/// Test connection event sent from background tasks to main event loop
#[derive(Debug)]
enum TestConnectionEvent {
    Success(String),
    Failed(String),
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
    /// Tick counter for periodic connection health checks
    tick_counter: u32,
    /// Channel receiver for connection completion events
    connection_events_rx: tokio::sync::mpsc::UnboundedReceiver<ConnectionEvent>,
    /// Channel sender for connection events (cloned for background tasks)
    connection_events_tx: tokio::sync::mpsc::UnboundedSender<ConnectionEvent>,
    /// Channel receiver for test connection completion events
    test_connection_events_rx: tokio::sync::mpsc::UnboundedReceiver<TestConnectionEvent>,
    /// Channel sender for test connection events (cloned for background tasks)
    test_connection_events_tx: tokio::sync::mpsc::UnboundedSender<TestConnectionEvent>,
}

impl App {
    /// Create a new application instance
    pub async fn new(config: Config) -> Result<Self> {
        let state = AppState::new().await;
        let event_handler = EventHandler::new(Duration::from_millis(250));
        let ui = UI::new(&config)?;
        let command_registry = CommandRegistry::new();

        // Create channel for connection events
        let (connection_events_tx, connection_events_rx) = tokio::sync::mpsc::unbounded_channel();

        // Create channel for test connection events
        let (test_connection_events_tx, test_connection_events_rx) =
            tokio::sync::mpsc::unbounded_channel();

        Ok(Self {
            state,
            event_handler,
            ui,
            config,
            command_registry,
            should_quit: false,
            tick_counter: 0,
            connection_events_rx,
            connection_events_tx,
            test_connection_events_rx,
            test_connection_events_tx,
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
                        // Handled by overlay system;
                    }
                    _ => {}
                }
            }
            CommandAction::CloseModal => {
                // Handled by overlay system;
                // Handled by overlay system;
                // Handled by overlay system;
                // Handled by overlay system;
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

    /// Handle application keyboard events
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // 1. Handle global keys first (work everywhere)
        if self.handle_global_keys(key)?.is_some() {
            return Ok(());
        }

        // 2. Handle overlays (forms, modals, debug view)
        if self.state.ui.is_in_overlay() {
            return self.handle_overlay_keys(key).await;
        }

        // 3. Handle confirmation modals
        if self.state.ui.confirmation_modal.is_some() {
            return self.handle_confirmation_modal_keys(key).await;
        }

        // 4. Handle table viewer delete confirmation
        if self.state.table_viewer_state.delete_confirmation.is_some() {
            return self.handle_table_delete_confirmation_keys(key).await;
        }

        // 5. Route to focused pane handler (main view)
        match self.state.ui.focused_pane {
            FocusedPane::Connections => self.handle_connections_keys(key).await,
            FocusedPane::Tables => self.handle_tables_keys(key).await,
            FocusedPane::Details => self.handle_details_keys(key),
            FocusedPane::TabularOutput => self.handle_query_results_keys(key).await,
            FocusedPane::SqlFiles => self.handle_sql_files_keys(key).await,
            FocusedPane::QueryWindow => self.handle_query_editor_keys(key).await,
        }
    }

    /// Handle global keys that work everywhere
    fn handle_global_keys(&mut self, key: KeyEvent) -> Result<Option<()>> {
        match (key.modifiers, key.code) {
            // Help - toggle with '?'
            (KeyModifiers::NONE, KeyCode::Char('?')) => {
                self.execute_command(CommandId::ToggleHelp)?;
                Ok(Some(()))
            }
            // Debug view - toggle with Ctrl+B
            (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
                self.state.ui.toggle_debug_view();
                Ok(Some(()))
            }
            // Quit application - 'q' (only if not in edit modes)
            (KeyModifiers::NONE, KeyCode::Char('q')) if self.can_quit() => {
                self.state.ui.confirmation_modal = Some(crate::ui::ConfirmationModal {
                    title: "Exit LazyTables".to_string(),
                    message: "Are you sure you want to exit?\n\nAll active database connections will be closed.".to_string(),
                    action: crate::ui::ConfirmationAction::ExitApplication,
                });
                Ok(Some(()))
            }
            // Number keys 1-6 for direct pane navigation (only in main view)
            (KeyModifiers::NONE, KeyCode::Char(c @ '1'..='6')) if self.state.ui.is_in_main() => {
                if let Some(pane) = FocusedPane::from_number(c.to_digit(10).unwrap() as u8) {
                    self.state.ui.focused_pane = pane;
                    self.state.ui.cancel_pending_gg();
                }
                Ok(Some(()))
            }
            // Tab/Shift+Tab for pane cycling
            // Skip Tab in query editor insert mode (Tab inserts tab character there)
            (KeyModifiers::NONE, KeyCode::Tab)
                if self.state.ui.is_in_main()
                    && !(self.state.ui.focused_pane == FocusedPane::QueryWindow
                        && self.state.query_editor.is_insert_mode()) =>
            {
                self.state.cycle_focus_forward();
                self.state.ui.cancel_pending_gg();
                Ok(Some(()))
            }
            (KeyModifiers::SHIFT, KeyCode::BackTab)
                if self.state.ui.is_in_main()
                    && !(self.state.ui.focused_pane == FocusedPane::QueryWindow
                        && self.state.query_editor.is_insert_mode()) =>
            {
                self.state.cycle_focus_backward();
                self.state.ui.cancel_pending_gg();
                Ok(Some(()))
            }
            // Ctrl+h/j/k/l for pane navigation
            (KeyModifiers::CONTROL, KeyCode::Char('h')) if self.state.ui.is_in_main() => {
                self.state.move_focus_left();
                Ok(Some(()))
            }
            (KeyModifiers::CONTROL, KeyCode::Char('j')) if self.state.ui.is_in_main() => {
                self.state.move_focus_down();
                Ok(Some(()))
            }
            (KeyModifiers::CONTROL, KeyCode::Char('k')) if self.state.ui.is_in_main() => {
                self.state.move_focus_up();
                Ok(Some(()))
            }
            (KeyModifiers::CONTROL, KeyCode::Char('l')) if self.state.ui.is_in_main() => {
                self.state.move_focus_right();
                Ok(Some(()))
            }
            _ => Ok(None), // Key not handled globally
        }
    }

    /// Check if quit action is allowed (not in edit/insert modes)
    fn can_quit(&self) -> bool {
        if !self.state.ui.is_in_main() {
            return false;
        }
        // Check for active edit/search modes
        if self.state.ui.connections_search_active
            || self.state.ui.tables_search_active
            || self.state.ui.sql_files_search_active
            || self.state.ui.sql_files_rename_mode
            || self.state.ui.sql_files_create_mode
        {
            return false;
        }
        // Check table viewer edit mode
        if self.state.ui.focused_pane == FocusedPane::TabularOutput {
            if let Some(tab) = self.state.table_viewer_state.current_tab() {
                if tab.in_edit_mode || tab.in_search_mode {
                    return false;
                }
            }
        }
        // Check query editor insert mode
        if self.state.ui.focused_pane == FocusedPane::QueryWindow
            && self.state.query_editor.is_insert_mode()
        {
            return false;
        }
        true
    }

    /// Handle overlay keys (connection form, table creator/editor, debug view)
    async fn handle_overlay_keys(&mut self, key: KeyEvent) -> Result<()> {
        // ESC always closes overlay and returns to main
        if key.code == KeyCode::Esc {
            self.state.ui.return_to_main();
            return Ok(());
        }

        // Route to specific overlay handler
        match &self.state.ui.current_view {
            AppView::Overlay(OverlayView::ConnectionForm(_)) => {
                self.handle_connection_modal_key_event(key).await
            }
            AppView::Overlay(OverlayView::TableCreator) => {
                self.handle_table_creator_key_event(key).await
            }
            AppView::Overlay(OverlayView::TableEditor) => {
                self.handle_table_editor_key_event(key).await
            }
            AppView::Overlay(OverlayView::DebugView) => self.handle_debug_view_keys(key),
            AppView::Overlay(OverlayView::Help) => self.handle_help_keys(key),
            _ => Ok(()),
        }
    }

    /// Handle debug view keys
    fn handle_debug_view_keys(&mut self, key: KeyEvent) -> Result<()> {
        let debug_messages = crate::logging::get_debug_messages();
        let max_lines = debug_messages.len();

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.ui.debug_view_scroll_down(max_lines);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.ui.debug_view_scroll_up();
            }
            KeyCode::PageDown => {
                self.state.ui.debug_view_page_down(max_lines, 10);
            }
            KeyCode::PageUp => {
                self.state.ui.debug_view_page_up(10);
            }
            KeyCode::Char('g') => {
                if self.state.ui.pending_gg_command {
                    self.state.ui.debug_view_go_to_top();
                    self.state.ui.pending_gg_command = false;
                } else {
                    self.state.ui.pending_gg_command = true;
                }
            }
            KeyCode::Char('G') => {
                self.state.ui.debug_view_go_to_bottom(max_lines);
            }
            KeyCode::Char('c') => {
                crate::logging::clear_debug_messages();
                self.state.toast_manager.info("Debug messages cleared");
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle help overlay keys
    fn handle_help_keys(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                self.state.ui.toggle_help_pane_focus();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.ui.help_scroll_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.ui.help_scroll_down(100);
            }
            KeyCode::PageUp => {
                self.state.ui.help_page_up(10);
            }
            KeyCode::PageDown => {
                self.state.ui.help_page_down(100, 10);
            }
            KeyCode::Tab => {
                self.state.ui.toggle_help_pane_focus();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle confirmation modal keys
    async fn handle_confirmation_modal_keys(&mut self, key: KeyEvent) -> Result<()> {
        if let Some(modal) = &self.state.ui.confirmation_modal {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    // Execute the confirmed action
                    match &modal.action {
                        crate::ui::ConfirmationAction::DeleteConnection(index) => {
                            let index = *index;
                            if let Some(connection) =
                                self.state.db.connections.connections.get(index)
                            {
                                let conn_id = connection.id.clone();
                                if let Err(e) =
                                    self.state.db.connections.remove_connection(&conn_id).await
                                {
                                    self.state
                                        .toast_manager
                                        .error(format!("Failed to delete connection: {e}"));
                                } else {
                                    self.state
                                        .toast_manager
                                        .success("Connection deleted successfully");
                                    if self.state.ui.selected_connection
                                        >= self.state.db.connections.connections.len()
                                        && self.state.ui.selected_connection > 0
                                    {
                                        self.state.ui.selected_connection -= 1;
                                    }
                                }
                            }
                        }
                        crate::ui::ConfirmationAction::DeleteSqlFile(index) => {
                            let index = *index;
                            if let Err(e) = self.state.delete_sql_file(index).await {
                                self.state
                                    .toast_manager
                                    .error(format!("Failed to delete SQL file: {e}"));
                            } else {
                                self.state.toast_manager.success("SQL file deleted");
                            }
                            self.state
                                .ui
                                .update_sql_file_selection(self.state.saved_sql_files.len());
                        }
                        crate::ui::ConfirmationAction::ExitApplication => {
                            self.should_quit = true;
                        }
                        crate::ui::ConfirmationAction::QuitQueryEditor => {
                            // Just close the confirmation, stay in main view
                        }
                        _ => {}
                    }
                    self.state.ui.confirmation_modal = None;
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.state.ui.confirmation_modal = None;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handle table delete confirmation keys
    async fn handle_table_delete_confirmation_keys(&mut self, key: KeyEvent) -> Result<()> {
        if let Some(confirmation) = &self.state.table_viewer_state.delete_confirmation {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    let confirmation = confirmation.clone();
                    if let Err(e) = self.state.delete_table_row(confirmation).await {
                        self.state
                            .toast_manager
                            .error(format!("Failed to delete row: {e}"));
                    } else {
                        self.state.toast_manager.success("Row deleted successfully");
                        let tab_idx = self.state.table_viewer_state.active_tab;
                        let _ = self.state.load_table_data(tab_idx).await;
                    }
                    self.state.table_viewer_state.delete_confirmation = None;
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.state.table_viewer_state.delete_confirmation = None;
                    self.state.toast_manager.info("Delete cancelled");
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handle Connections pane keys - DIRECT KEY BINDINGS (no insert mode)
    async fn handle_connections_keys(&mut self, key: KeyEvent) -> Result<()> {
        // Search mode active - handle search input
        if self.state.ui.connections_search_active {
            match key.code {
                KeyCode::Esc => {
                    self.state.ui.exit_connections_search();
                }
                KeyCode::Backspace => {
                    self.state.ui.backspace_connections_search();
                    self.state
                        .ui
                        .update_filtered_connections(&self.state.db.connections.connections);
                }
                KeyCode::Enter => {
                    // Get selected connection index
                    let selected_index = if let Some(index) = self
                        .state
                        .ui
                        .get_selected_connection_index(&self.state.db.connections.connections)
                    {
                        index
                    } else {
                        return Ok(()); // No connection selected
                    };

                    // Don't start new connection if one is already in progress
                    if self.state.connecting_in_progress.is_some() {
                        self.state
                            .toast_manager
                            .warning("Connection attempt already in progress");
                        return Ok(());
                    }

                    // Mark connection as in progress
                    self.state.connecting_in_progress = Some(selected_index);
                    self.state.connecting_animation_frame = 0;
                    self.state.connection_start_time = Some(std::time::Instant::now());

                    // Set status to connecting immediately
                    if let Some(conn) = self
                        .state
                        .db
                        .connections
                        .connections
                        .get_mut(selected_index)
                    {
                        conn.status = crate::database::ConnectionStatus::Connecting;
                        self.state
                            .toast_manager
                            .info(format!("Connecting to {}...", conn.name));
                    }

                    // Clone necessary data for background task
                    let connection_config =
                        self.state.db.connections.connections[selected_index].clone();
                    let connection_manager = self.state.connection_manager.clone();
                    let tx = self.connection_events_tx.clone();

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
                                            error: format!(
                                                "Failed to load database objects: {}",
                                                e
                                            ),
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

                    self.state.ui.exit_connections_search();
                }
                KeyCode::Down => {
                    self.state
                        .ui
                        .connections_selection_down(&self.state.db.connections.connections);
                }
                KeyCode::Up => {
                    self.state
                        .ui
                        .connections_selection_up(&self.state.db.connections.connections);
                }
                KeyCode::Char(c) => {
                    self.state.ui.add_to_connections_search(c);
                    self.state
                        .ui
                        .update_filtered_connections(&self.state.db.connections.connections);
                }
                _ => {}
            }
            return Ok(());
        }

        // Normal mode - direct key bindings
        match key.code {
            // 'a' - Add new connection
            KeyCode::Char('a') => {
                self.state.open_add_connection_modal();
            }
            // 'e' - Edit selected connection
            KeyCode::Char('e') => {
                self.state.open_edit_connection_modal();
            }
            // 'd' - Delete selected connection
            KeyCode::Char('d') => {
                if !self.state.db.connections.connections.is_empty() {
                    let index = self.state.ui.selected_connection;
                    self.state.ui.confirmation_modal = Some(crate::ui::ConfirmationModal {
                        title: "Delete Connection".to_string(),
                        message: format!(
                            "Are you sure you want to delete the connection '{}'?",
                            self.state.db.connections.connections[index].name
                        ),
                        action: crate::ui::ConfirmationAction::DeleteConnection(index),
                    });
                }
            }
            // Enter or Space - Connect to selected database
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Get selected connection index
                let selected_index = if let Some(index) = self
                    .state
                    .ui
                    .get_selected_connection_index(&self.state.db.connections.connections)
                {
                    index
                } else {
                    return Ok(()); // No connection selected
                };

                // Don't start new connection if one is already in progress
                if self.state.connecting_in_progress.is_some() {
                    self.state
                        .toast_manager
                        .warning("Connection attempt already in progress");
                    return Ok(());
                }

                // Mark connection as in progress
                self.state.connecting_in_progress = Some(selected_index);
                self.state.connecting_animation_frame = 0;
                self.state.connection_start_time = Some(std::time::Instant::now());

                // Set status to connecting immediately (for visual feedback)
                if let Some(conn) = self
                    .state
                    .db
                    .connections
                    .connections
                    .get_mut(selected_index)
                {
                    conn.status = crate::database::ConnectionStatus::Connecting;
                    self.state
                        .toast_manager
                        .info(format!("Connecting to {}...", conn.name));
                }

                // Clone necessary data for background task
                let connection_config =
                    self.state.db.connections.connections[selected_index].clone();
                let connection_manager = self.state.connection_manager.clone();
                let tx = self.connection_events_tx.clone();

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
                self.state.toast_manager.info("Connections refreshed");
            }
            // '/' - Enter search mode
            KeyCode::Char('/') => {
                self.state.ui.enter_connections_search();
            }
            // j/k or arrow keys - Navigate
            KeyCode::Char('j') | KeyCode::Down => {
                self.state
                    .ui
                    .connections_selection_down(&self.state.db.connections.connections);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state
                    .ui
                    .connections_selection_up(&self.state.db.connections.connections);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle Tables pane keys - DIRECT KEY BINDINGS
    async fn handle_tables_keys(&mut self, key: KeyEvent) -> Result<()> {
        // Search mode active
        if self.state.ui.tables_search_active {
            match key.code {
                KeyCode::Esc => {
                    self.state.ui.exit_tables_search();
                }
                KeyCode::Backspace => {
                    self.state.ui.backspace_tables_search();
                }
                KeyCode::Enter => {
                    self.state.open_table_for_viewing().await;
                    self.state.ui.exit_tables_search();
                }
                KeyCode::Down => {
                    self.state.ui.table_search_selection_down();
                }
                KeyCode::Up => {
                    self.state.ui.table_search_selection_up();
                }
                KeyCode::Char(c) => {
                    self.state.ui.add_to_tables_search(c);
                }
                _ => {}
            }
            return Ok(());
        }

        // Normal mode
        match key.code {
            // Enter or Space - Open table for viewing
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.state.open_table_for_viewing().await;
            }
            // 'r' - Refresh tables list
            KeyCode::Char('r') => {
                self.state.connect_to_selected_database().await;
                self.state.toast_manager.info("Tables refreshed");
            }
            // '/' - Enter search mode
            KeyCode::Char('/') => {
                self.state.ui.enter_tables_search();
            }
            // j/k - Navigate
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.ui.table_search_selection_down();
                self.state.ui.cancel_pending_gg();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.ui.table_search_selection_up();
                self.state.ui.cancel_pending_gg();
            }
            // 'g' - First press of gg
            KeyCode::Char('g') => {
                self.state.ui.handle_g_key_press();
            }
            // 'G' - Jump to last table
            KeyCode::Char('G') => {
                self.state.ui.table_go_to_last();
            }
            // Ctrl+d - Page down (half page)
            KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                for _ in 0..10 {
                    self.state.ui.table_search_selection_down();
                }
            }
            // Ctrl+u - Page up (half page)
            KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                for _ in 0..10 {
                    self.state.ui.table_search_selection_up();
                }
            }
            // Tab - Toggle group expansion (when on a header)
            KeyCode::Tab if key.modifiers == KeyModifiers::NONE => {
                if let Some(item) = self.state.ui.get_selected_item_raw() {
                    if !item.is_selectable {
                        // It's a group header - extract group name and toggle expansion
                        let group_name = item
                            .display_name
                            .trim_start_matches("▼ ")
                            .trim_start_matches("▶ ")
                            .trim()
                            .to_string();

                        if !group_name.is_empty() {
                            let is_expanded_before =
                                self.state.ui.is_object_group_expanded(&group_name);
                            self.state.ui.toggle_object_group_expansion(&group_name);
                            self.state
                                .ui
                                .build_selectable_table_items(&self.state.db.database_objects);
                            self.state.toast_manager.info(format!(
                                "{} {}",
                                if !is_expanded_before {
                                    "Expanded"
                                } else {
                                    "Collapsed"
                                },
                                group_name
                            ));
                        }
                    }
                    // If it's not a header, Tab is handled by global keys for pane cycling
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle Details pane keys - READ-ONLY (just scrolling)
    fn handle_details_keys(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.move_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.move_up();
            }
            KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
                // Page down
                if self.state.ui.details_viewport_offset + 10
                    < self.state.ui.details_max_scroll_offset
                {
                    self.state.ui.details_viewport_offset += 10;
                } else {
                    self.state.ui.details_viewport_offset = self.state.ui.details_max_scroll_offset;
                }
            }
            KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                // Page up
                self.state.ui.details_viewport_offset =
                    self.state.ui.details_viewport_offset.saturating_sub(10);
            }
            KeyCode::Char('g') => {
                if self.state.ui.pending_gg_command {
                    self.state.ui.details_viewport_offset = 0;
                    self.state.ui.pending_gg_command = false;
                } else {
                    self.state.ui.pending_gg_command = true;
                }
            }
            KeyCode::Char('G') => {
                self.state.ui.details_viewport_offset = self.state.ui.details_max_scroll_offset;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle Query Results pane keys - has its own edit mode
    async fn handle_query_results_keys(&mut self, key: KeyEvent) -> Result<()> {
        // Check if in edit mode
        if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
            if tab.in_edit_mode {
                return self.handle_table_viewer_edit_mode(key).await;
            }
            if tab.in_search_mode {
                return self.handle_table_viewer_search_mode(key).await;
            }
        }

        // Normal navigation mode
        match key.code {
            // 'i' or Enter - Start editing current cell
            KeyCode::Char('i') | KeyCode::Enter => {
                if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                    tab.start_edit();
                }
            }
            // 'd' - Delete current row
            KeyCode::Char('d') => {
                // Trigger delete confirmation
                if let Some(_tab) = self.state.table_viewer_state.current_tab() {
                    // Build delete confirmation - implementation simplified for now
                    self.state
                        .toast_manager
                        .info("Delete row: Press 'dd' to confirm");
                }
            }
            // '/' - Enter search mode
            KeyCode::Char('/') => {
                if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
                    tab.start_search();
                }
            }
            // Ctrl+r - Refresh table data
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                let tab_idx = self.state.table_viewer_state.active_tab;
                if let Err(e) = self.state.load_table_data(tab_idx).await {
                    self.state
                        .toast_manager
                        .error(format!("Failed to refresh: {e}"));
                } else {
                    self.state.toast_manager.success("Table data refreshed");
                }
            }
            // h/j/k/l - Navigate cells
            KeyCode::Char('h') | KeyCode::Left => {
                self.state.move_left();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.move_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.move_up();
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.state.move_right();
            }
            // 'H' - Switch to previous tab
            KeyCode::Char('H') => {
                self.state.table_viewer_state.prev_tab();
            }
            // 'L' - Switch to next tab
            KeyCode::Char('L') => {
                self.state.table_viewer_state.next_tab();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle table viewer edit mode keys
    async fn handle_table_viewer_edit_mode(&mut self, key: KeyEvent) -> Result<()> {
        if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    // Save edit
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
                }
                KeyCode::Char(c) if key.modifiers == KeyModifiers::CONTROL && c == 'c' => {
                    // Cancel edit
                    tab.cancel_edit();
                }
                KeyCode::Char(c) => {
                    tab.edit_buffer.push(c);
                }
                KeyCode::Backspace => {
                    tab.edit_buffer.pop();
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handle table viewer search mode keys
    async fn handle_table_viewer_search_mode(&mut self, key: KeyEvent) -> Result<()> {
        if let Some(tab) = self.state.table_viewer_state.current_tab_mut() {
            match key.code {
                KeyCode::Esc => {
                    tab.cancel_search();
                }
                KeyCode::Enter => {
                    tab.in_search_mode = false;
                }
                KeyCode::Char('n') => {
                    tab.next_search_result();
                }
                KeyCode::Char('N') => {
                    tab.prev_search_result();
                }
                KeyCode::Char(c) if !matches!(c, 'h' | 'j' | 'k' | 'l') => {
                    tab.search_query.push(c);
                    tab.update_search(&tab.search_query.clone());
                }
                KeyCode::Backspace => {
                    tab.search_query.pop();
                    tab.update_search(&tab.search_query.clone());
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handle SQL Files pane keys - DIRECT KEY BINDINGS
    async fn handle_sql_files_keys(&mut self, key: KeyEvent) -> Result<()> {
        // Handle special input modes
        if self.state.ui.sql_files_search_active {
            return self.handle_sql_files_search_mode(key).await;
        }
        if self.state.ui.sql_files_rename_mode {
            return self.handle_sql_files_rename_mode(key).await;
        }
        if self.state.ui.sql_files_create_mode {
            return self.handle_sql_files_create_mode(key).await;
        }

        // Normal mode
        match key.code {
            // Enter - Load selected SQL file
            KeyCode::Enter => {
                if let Err(e) = self.state.load_selected_sql_file() {
                    self.state
                        .toast_manager
                        .error(format!("Failed to load SQL file: {e}"));
                } else {
                    self.state.toast_manager.success("SQL file loaded");
                }
            }
            // 'n' - Create new file
            KeyCode::Char('n') => {
                self.state.ui.enter_sql_files_create();
            }
            // 'r' - Rename file
            KeyCode::Char('r') => {
                if let Some(filename) = self.state.get_selected_sql_file() {
                    self.state.ui.enter_sql_files_rename(&filename);
                }
            }
            // 'd' - Delete file
            KeyCode::Char('d') => {
                if !self.state.saved_sql_files.is_empty() {
                    let index = self.state.get_filtered_sql_file_selection();
                    self.state.ui.confirmation_modal = Some(crate::ui::ConfirmationModal {
                        title: "Delete SQL File".to_string(),
                        message: format!(
                            "Are you sure you want to delete '{}'?",
                            self.state
                                .saved_sql_files
                                .get(index)
                                .unwrap_or(&String::new())
                        ),
                        action: crate::ui::ConfirmationAction::DeleteSqlFile(index),
                    });
                }
            }
            // '/' - Enter search mode
            KeyCode::Char('/') => {
                self.state.ui.enter_sql_files_search();
            }
            // j/k - Navigate files
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.update_sql_file_selection_for_filtered(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.update_sql_file_selection_for_filtered(-1);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle SQL files search mode
    async fn handle_sql_files_search_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.ui.exit_sql_files_search();
            }
            KeyCode::Backspace => {
                self.state.ui.backspace_sql_files_search();
            }
            KeyCode::Enter => {
                if let Err(e) = self.state.load_selected_sql_file() {
                    self.state
                        .toast_manager
                        .error(format!("Failed to load SQL file: {e}"));
                } else {
                    self.state.toast_manager.success("SQL file loaded");
                }
                self.state.ui.exit_sql_files_search();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.update_sql_file_selection_for_filtered(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.update_sql_file_selection_for_filtered(-1);
            }
            KeyCode::Char(c) if !matches!(c, 'j' | 'k') => {
                self.state.ui.add_to_sql_files_search(c);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle SQL files rename mode
    async fn handle_sql_files_rename_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.ui.exit_sql_files_rename();
            }
            KeyCode::Backspace => {
                self.state.ui.backspace_sql_files_rename();
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
                                self.state.rename_sql_file(original_index, &new_name).await
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
            }
            KeyCode::Char(c) => {
                self.state.ui.add_to_sql_files_rename(c);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle SQL files create mode
    async fn handle_sql_files_create_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state.ui.exit_sql_files_create();
            }
            KeyCode::Backspace => {
                self.state.ui.backspace_sql_files_create();
            }
            KeyCode::Enter => {
                let filename = self.state.ui.sql_files_create_buffer.clone();
                if !filename.is_empty() {
                    if let Err(e) = self.state.create_sql_file(&filename).await {
                        self.state
                            .toast_manager
                            .error(format!("Failed to create file: {e}"));
                    } else {
                        self.state
                            .toast_manager
                            .success("File created successfully");
                        // Load the new file
                        let _ = self.state.load_query_file(&filename);
                    }
                }
                self.state.ui.exit_sql_files_create();
            }
            KeyCode::Char(c) => {
                self.state.ui.add_to_sql_files_create(c);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle Query Editor pane keys - ONLY PANE WITH VIM INSERT MODE
    async fn handle_query_editor_keys(&mut self, key: KeyEvent) -> Result<()> {
        // Check if in command mode
        if self.state.query_editor.is_in_command_mode() {
            return self.handle_query_editor_command_mode(key).await;
        }

        // Check if query editor is in insert mode
        if self.state.query_editor.is_insert_mode() {
            return self.handle_query_editor_insert_mode(key).await;
        }

        // Normal mode - vim keybindings
        match key.code {
            // Shift+E - Execute query at cursor (PRIMARY binding, vim-style)
            KeyCode::Char('E') => {
                if let Err(e) = self.state.execute_query_at_cursor().await {
                    self.state
                        .toast_manager
                        .error(format!("Query execution failed: {e}"));
                }
            }
            // Ctrl+Enter - Execute query at cursor (SECONDARY binding, familiar to SQL tool users)
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Err(e) = self.state.execute_query_at_cursor().await {
                    self.state
                        .toast_manager
                        .error(format!("Query execution failed: {e}"));
                }
            }
            // 'i' - Enter insert mode at cursor
            KeyCode::Char('i') => {
                self.state.query_editor.set_insert_mode(true);
            }
            // 'a' - Enter insert mode after cursor
            KeyCode::Char('a') => {
                self.state.query_editor.move_cursor_right();
                self.state.query_editor.set_insert_mode(true);
            }
            // 'o' - New line below + insert mode
            KeyCode::Char('o') => {
                self.state.query_editor.insert_newline();
                self.state.query_editor.set_insert_mode(true);
            }
            // 'O' - New line above + insert mode
            KeyCode::Char('O') => {
                self.state.query_editor.move_cursor_up();
                self.state.query_editor.move_to_line_end();
                self.state.query_editor.insert_newline();
                self.state.query_editor.set_insert_mode(true);
            }
            // Vim motions
            KeyCode::Char('h') | KeyCode::Left => {
                self.state.query_editor.move_cursor_left();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.state.query_editor.move_cursor_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.state.query_editor.move_cursor_up();
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.state.query_editor.move_cursor_right();
            }
            KeyCode::Char('w') => {
                self.state.query_editor.move_to_next_word();
            }
            KeyCode::Char('b') => {
                self.state.query_editor.move_to_prev_word();
            }
            KeyCode::Char('e') => {
                self.state.query_editor.move_to_end_of_word();
            }
            KeyCode::Char('0') => {
                self.state.query_editor.move_to_line_start();
            }
            KeyCode::Char('$') => {
                self.state.query_editor.move_to_line_end();
            }
            KeyCode::Char('g') => {
                if self.state.ui.pending_gg_command {
                    self.state.query_editor.move_to_file_start();
                    self.state.ui.pending_gg_command = false;
                } else {
                    self.state.ui.pending_gg_command = true;
                }
            }
            KeyCode::Char('G') => {
                self.state.query_editor.move_to_file_end();
            }
            // ':' - Enter command mode
            KeyCode::Char(':') => {
                self.state.query_editor.enter_command_mode();
            }
            // Ctrl+d and Ctrl+u for page scrolling - TODO: implement scroll methods
            // KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
            //     self.state.query_editor.scroll_half_page_down();
            // }
            // KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
            //     self.state.query_editor.scroll_half_page_up();
            // }
            _ => {}
        }
        Ok(())
    }

    /// Handle query editor insert mode
    async fn handle_query_editor_insert_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Esc - Exit insert mode
            KeyCode::Esc => {
                if self.state.query_editor.are_suggestions_active() {
                    self.state.query_editor.hide_suggestions();
                } else {
                    self.state.query_editor.set_insert_mode(false);
                }
            }
            // Enter - Insert newline (Ctrl+Enter does NOT work in insert mode - use normal mode)
            KeyCode::Enter => {
                self.state.query_editor.insert_newline();
                self.state.query_content = self.state.query_editor.get_content().to_string();
                self.state.ui.query_modified = true;
            }
            // Ctrl+p - Navigate suggestions up (vim-style) - MUST come before Char(c) pattern
            KeyCode::Char('p') if key.modifiers == KeyModifiers::CONTROL => {
                if self.state.query_editor.are_suggestions_active() {
                    self.state.query_editor.move_suggestion_up();
                }
            }
            // Ctrl+n - Navigate suggestions down (vim-style) - MUST come before Char(c) pattern
            KeyCode::Char('n') if key.modifiers == KeyModifiers::CONTROL => {
                if self.state.query_editor.are_suggestions_active() {
                    self.state.query_editor.move_suggestion_down();
                }
            }
            // Regular typing
            KeyCode::Char(c) => {
                self.state.query_editor.insert_char(c);
                self.state.query_content = self.state.query_editor.get_content().to_string();
                self.state.ui.query_modified = true;
            }
            // Backspace
            KeyCode::Backspace => {
                self.state.query_editor.backspace();
                self.state.query_content = self.state.query_editor.get_content().to_string();
                self.state.ui.query_modified = true;
            }
            // Tab - Accept suggestion if active, otherwise insert tab character
            KeyCode::Tab => {
                if self.state.query_editor.are_suggestions_active() {
                    self.state.query_editor.accept_suggestion();
                    self.state.query_content = self.state.query_editor.get_content().to_string();
                    self.state.ui.query_modified = true;
                } else {
                    self.state.query_editor.insert_char('\t');
                    self.state.query_content = self.state.query_editor.get_content().to_string();
                    self.state.ui.query_modified = true;
                }
            }
            // Up arrow - Navigate suggestions or move cursor
            KeyCode::Up => {
                if self.state.query_editor.are_suggestions_active() {
                    self.state.query_editor.move_suggestion_up();
                } else {
                    self.state.query_editor.move_cursor_up();
                }
            }
            // Down arrow - Navigate suggestions or move cursor
            KeyCode::Down => {
                if self.state.query_editor.are_suggestions_active() {
                    self.state.query_editor.move_suggestion_down();
                } else {
                    self.state.query_editor.move_cursor_down();
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle query editor command mode (vim : commands)
    async fn handle_query_editor_command_mode(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Esc - Exit command mode
            KeyCode::Esc => {
                self.state.query_editor.exit_command_mode();
            }
            // Backspace - Remove character from command buffer
            KeyCode::Backspace => {
                self.state.query_editor.backspace_command_buffer();
            }
            // Enter - Execute command
            KeyCode::Enter => {
                let command = self.state.query_editor.get_command_buffer().to_string();
                self.state.query_editor.exit_command_mode();

                // Parse and execute command
                match command.trim() {
                    ":w" => {
                        // Save file
                        if let Err(e) = self.state.save_sql_file_with_connection().await {
                            self.state
                                .toast_manager
                                .error(format!("Failed to save file: {}", e));
                        } else {
                            self.state.query_editor.mark_saved();
                            self.state.toast_manager.success("File saved successfully");
                        }
                    }
                    ":q" => {
                        // Clear editor (with confirmation if modified)
                        if self.state.query_editor.is_modified() {
                            self.state
                                .toast_manager
                                .warning("No write since last change (use :q! to force)");
                        } else {
                            self.state.query_editor.reset();
                            self.state.toast_manager.info("Editor cleared");
                        }
                    }
                    ":q!" => {
                        // Force clear editor
                        self.state.query_editor.reset();
                        self.state.toast_manager.info("Editor cleared");
                    }
                    ":wq" => {
                        // Save and clear
                        if let Err(e) = self.state.save_sql_file_with_connection().await {
                            self.state
                                .toast_manager
                                .error(format!("Failed to save file: {}", e));
                        } else {
                            self.state.query_editor.mark_saved();
                            self.state.query_editor.reset();
                            self.state
                                .toast_manager
                                .success("File saved and editor cleared");
                        }
                    }
                    cmd if cmd.starts_with(":w ") => {
                        // Save with filename - future enhancement
                        self.state
                            .toast_manager
                            .warning("Save with filename not yet implemented");
                    }
                    _ => {
                        self.state
                            .toast_manager
                            .error(format!("Unknown command: {}", command));
                    }
                }
            }
            // Regular typing - add to command buffer
            KeyCode::Char(c) => {
                self.state.query_editor.add_to_command_buffer(c);
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_connection_modal_key_event(&mut self, key: KeyEvent) -> Result<()> {
        use crate::ui::components::{ConnectionField, PasswordStorageType};

        match key.code {
            // PRIORITY 1: Global shortcuts (work from any field EXCEPT text input fields)
            KeyCode::Char('t')
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !self.state.connection_modal_state.is_text_field() =>
            {
                // Plain 't': Test connection shortcut
                self.test_connection_from_modal().await;
            }
            KeyCode::Char('s') if !self.state.connection_modal_state.is_text_field() => {
                // Save shortcut - works from any field except text input fields
                if let Err(error) = self.state.save_connection_from_modal().await {
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
            KeyCode::Char('c') if !self.state.connection_modal_state.is_text_field() => {
                // Cancel shortcut - works from any field except text input fields
                if self.state.ui.current_view.is_connection_form() {
                    self.state.close_add_connection_modal();
                } else {
                    self.state.close_edit_connection_modal();
                }
            }

            // PRIORITY 2: Navigation and special keys
            KeyCode::Esc => {
                // Close the appropriate modal
                if self.state.ui.current_view.is_connection_form() {
                    self.state.close_add_connection_modal();
                } else {
                    self.state.close_edit_connection_modal();
                }
            }
            KeyCode::Tab => {
                // Tab for next field navigation
                self.state.connection_modal_state.focused_field =
                    self.state.connection_modal_state.get_smart_next_field();
            }
            KeyCode::BackTab => {
                // Shift+Tab for previous field navigation
                self.state.connection_modal_state.focused_field =
                    self.state.connection_modal_state.get_smart_previous_field();
            }
            // Arrow keys for navigation within sections
            KeyCode::Down => {
                // Handle database type and other dropdowns specially
                match self.state.connection_modal_state.focused_field {
                    ConnectionField::DatabaseType => {
                        // Navigate database type dropdown
                        let current = self
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
                        self.state
                            .connection_modal_state
                            .select_database_type(new_index);
                    }
                    ConnectionField::SslMode => {
                        // Navigate SSL mode dropdown
                        let current = self
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
                        self.state.connection_modal_state.select_ssl_mode(new_index);
                    }
                    ConnectionField::PasswordStorageType => {
                        // Cycle through password storage types
                        self.state
                            .connection_modal_state
                            .cycle_password_storage_type();
                    }
                    _ => {
                        // For other fields, move to next field
                        self.state.connection_modal_state.focused_field =
                            self.state.connection_modal_state.get_smart_next_field();
                    }
                }
            }
            KeyCode::Up => {
                // Handle database type and other dropdowns specially
                match self.state.connection_modal_state.focused_field {
                    ConnectionField::DatabaseType => {
                        // Navigate database type dropdown
                        let current = self
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
                        self.state
                            .connection_modal_state
                            .select_database_type(new_index);
                    }
                    ConnectionField::SslMode => {
                        // Navigate SSL mode dropdown
                        let current = self
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
                    _ => {
                        // For other fields, move to previous field
                        self.state.connection_modal_state.focused_field =
                            self.state.connection_modal_state.get_smart_previous_field();
                    }
                }
            }
            KeyCode::Enter => {
                // Handle Enter on button fields specially
                match self.state.connection_modal_state.focused_field {
                    ConnectionField::Test => {
                        // Activate Test button
                        self.test_connection_from_modal().await;
                    }
                    ConnectionField::Save => {
                        // Activate Save button
                        if let Err(error) = self.state.save_connection_from_modal().await {
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
                        // Activate Cancel button
                        if self.state.ui.current_view.is_connection_form() {
                            self.state.close_add_connection_modal();
                        } else {
                            self.state.close_edit_connection_modal();
                        }
                    }
                    _ => {
                        // For all other fields, Enter moves to next field
                        self.state.connection_modal_state.next_field();
                    }
                }
            }
            // Ctrl+T: Toggle between connection string and individual fields
            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.connection_modal_state.using_connection_string =
                    !self.state.connection_modal_state.using_connection_string;

                // Clear the opposite fields when switching
                if self.state.connection_modal_state.using_connection_string {
                    // Clear individual fields when switching to connection string
                    self.state.connection_modal_state.host = "localhost".to_string();
                    self.state.connection_modal_state.port_input =
                        match self.state.connection_modal_state.database_type {
                            crate::database::DatabaseType::PostgreSQL => "5432".to_string(),
                            crate::database::DatabaseType::MySQL
                            | crate::database::DatabaseType::MariaDB => "3306".to_string(),
                            _ => "5432".to_string(),
                        };
                    self.state.connection_modal_state.database.clear();
                    self.state.connection_modal_state.username.clear();
                    self.state.connection_modal_state.password.clear();
                } else {
                    // Clear connection string when switching to individual fields
                    self.state.connection_modal_state.connection_string.clear();
                }

                // Clear any test status when switching input methods
                self.state.connection_modal_state.test_status = None;
            }

            // PRIORITY 3: Text input for text fields (lowest priority, after shortcuts)
            KeyCode::Char(c) if self.state.connection_modal_state.is_text_field() => {
                self.state.connection_modal_state.handle_char_input(c);
            }
            KeyCode::Backspace if self.state.connection_modal_state.is_text_field() => {
                self.state.connection_modal_state.handle_backspace();
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
        // Increment tick counter
        self.tick_counter = self.tick_counter.wrapping_add(1);

        // Handle ongoing connection attempt
        if let Some(connecting_index) = self.state.connecting_in_progress {
            // Animate loading dots every tick (250ms interval)
            self.state.connecting_animation_frame = (self.state.connecting_animation_frame + 1) % 3;

            // Check for timeout
            if let Some(start_time) = self.state.connection_start_time {
                let elapsed = start_time.elapsed().as_secs();
                if elapsed >= self.state.connection_timeout_seconds {
                    // Timeout reached, mark as failed
                    if let Some(conn) = self
                        .state
                        .db
                        .connections
                        .connections
                        .get_mut(connecting_index)
                    {
                        conn.status = crate::database::ConnectionStatus::Failed(format!(
                            "Connection timeout after {} seconds",
                            elapsed
                        ));
                        self.state.toast_manager.error("Connection timeout");
                    }
                    self.state.connecting_in_progress = None;
                    self.state.connection_start_time = None;
                    // Don't process events if we just timed out
                    return Ok(());
                }
            }

            // Check for completion events (NON-BLOCKING)
            if let Ok(event) = self.connection_events_rx.try_recv() {
                match event {
                    ConnectionEvent::Success {
                        connection_index,
                        objects,
                    } => {
                        // Connection succeeded! Update state
                        if let Some(conn) = self
                            .state
                            .db
                            .connections
                            .connections
                            .get_mut(connection_index)
                        {
                            conn.status = crate::database::ConnectionStatus::Connected;
                        }

                        // Update database state
                        self.state.db.database_objects = Some(objects.clone());
                        self.state.db.tables = objects
                            .tables
                            .iter()
                            .map(|t| {
                                if t.schema.as_deref() == Some("public") || t.schema.is_none() {
                                    t.name.clone()
                                } else {
                                    t.qualified_name()
                                }
                            })
                            .collect();

                        // Update UI
                        self.state
                            .ui
                            .build_selectable_table_items(&self.state.db.database_objects);
                        self.state.update_table_selection();

                        // Show success message
                        if let Some(conn) =
                            self.state.db.connections.connections.get(connection_index)
                        {
                            self.state
                                .toast_manager
                                .success(format!("Connected to {}", conn.name));

                            // Update active connection in app state database
                            let _ = self
                                .state
                                .app_state_db
                                .set_active_connection(
                                    &conn.id,
                                    &conn.name,
                                    conn.database_type.display_name(),
                                )
                                .await;
                        }

                        // Refresh SQL files
                        self.state.refresh_sql_files().await;

                        // Clear in-progress flag and start time
                        self.state.connecting_in_progress = None;
                        self.state.connection_start_time = None;
                    }
                    ConnectionEvent::Failed {
                        connection_index,
                        error,
                    } => {
                        // Connection failed
                        if let Some(conn) = self
                            .state
                            .db
                            .connections
                            .connections
                            .get_mut(connection_index)
                        {
                            conn.status = crate::database::ConnectionStatus::Failed(error.clone());
                            self.state
                                .toast_manager
                                .error(format!("Connection failed: {}", error));
                        }
                        self.state.connecting_in_progress = None;
                        self.state.connection_start_time = None;
                    }
                }
            }
        }

        // Handle ongoing test connection attempt
        if self.state.test_connection_in_progress {
            // Animate loading dots every tick (250ms interval)
            self.state.test_animation_frame = (self.state.test_animation_frame + 1) % 3;

            // Check for timeout
            if let Some(start_time) = self.state.test_start_time {
                let elapsed = start_time.elapsed().as_secs();
                if elapsed >= 30 {
                    // 30 second timeout for test connections
                    use crate::ui::components::TestConnectionStatus;
                    self.state.connection_modal_state.test_status =
                        Some(TestConnectionStatus::Failed(format!(
                            "Test timeout after {} seconds",
                            elapsed
                        )));
                    self.state.test_connection_in_progress = false;
                    self.state.test_start_time = None;
                    self.state.toast_manager.error("Test connection timeout");
                    return Ok(());
                }
            }

            // Check for test completion events (NON-BLOCKING)
            if let Ok(event) = self.test_connection_events_rx.try_recv() {
                use crate::ui::components::TestConnectionStatus;

                match event {
                    TestConnectionEvent::Success(msg) => {
                        self.state.connection_modal_state.test_status =
                            Some(TestConnectionStatus::Success(msg));
                        self.state
                            .toast_manager
                            .success("Test connection successful");
                    }
                    TestConnectionEvent::Failed(error) => {
                        self.state.connection_modal_state.test_status =
                            Some(TestConnectionStatus::Failed(error.clone()));
                        self.state
                            .toast_manager
                            .error(format!("Test connection failed: {}", error));
                    }
                }

                // Clear in-progress flag and start time
                self.state.test_connection_in_progress = false;
                self.state.test_start_time = None;
            }
        }

        // Perform connection health check every 100 ticks (approximately every 25 seconds with 250ms intervals)
        if self.tick_counter % 100 == 0 {
            // Only check health if we have an active connection
            if let Some(connection) = self.state.get_selected_connection() {
                if matches!(
                    connection.status,
                    crate::database::ConnectionStatus::Connected
                ) {
                    // Perform health check in background - don't await to avoid blocking UI
                    let _ = self.state.check_connection_health().await;
                }
            }
        }

        Ok(())
    }

    /// Test connection from modal
    async fn test_connection_from_modal(&mut self) {
        use crate::ui::components::TestConnectionStatus;

        // Don't start new test if one is already in progress
        if self.state.test_connection_in_progress {
            self.state.toast_manager.warning("Test already in progress");
            return;
        }

        // Set status to testing and start timer
        self.state.connection_modal_state.test_status = Some(TestConnectionStatus::Testing);
        self.state.test_connection_in_progress = true;
        self.state.test_animation_frame = 0;
        self.state.test_start_time = Some(std::time::Instant::now());

        // Try to create a connection config (no uniqueness check needed for testing)
        let config = match self
            .state
            .connection_modal_state
            .try_create_connection(&[], None)
        {
            Ok(config) => config,
            Err(e) => {
                // Invalid config - send error immediately
                let _ = self
                    .test_connection_events_tx
                    .send(TestConnectionEvent::Failed(format!(
                        "Invalid configuration: {e}"
                    )));
                return;
            }
        };

        // Clone sender for background task
        let tx = self.test_connection_events_tx.clone();

        // Spawn background task to test connection
        tokio::spawn(async move {
            use crate::database::{Connection, DatabaseType};

            let result = match config.database_type {
                DatabaseType::PostgreSQL => {
                    use crate::database::postgres::PostgresConnection;
                    let mut conn = PostgresConnection::new(config);

                    match conn.connect().await {
                        Ok(()) => conn
                            .test_connection()
                            .await
                            .map(|_| "Connection successful!".to_string()),
                        Err(e) => Err(crate::core::error::LazyTablesError::Connection(format!(
                            "Connection failed: {e}"
                        ))),
                    }
                }
                DatabaseType::MySQL | DatabaseType::MariaDB => {
                    use crate::database::mysql::MySqlConnection;
                    let mut conn = MySqlConnection::new(config);

                    match conn.connect().await {
                        Ok(()) => conn
                            .test_connection()
                            .await
                            .map(|_| "Connection successful!".to_string()),
                        Err(e) => Err(crate::core::error::LazyTablesError::Connection(format!(
                            "Connection failed: {e}"
                        ))),
                    }
                }
                DatabaseType::SQLite => {
                    use crate::database::sqlite::SqliteConnection;
                    let mut conn = SqliteConnection::new(config);

                    match conn.connect().await {
                        Ok(()) => conn
                            .test_connection()
                            .await
                            .map(|_| "Connection successful!".to_string()),
                        Err(e) => Err(crate::core::error::LazyTablesError::Connection(format!(
                            "Connection failed: {e}"
                        ))),
                    }
                }
                _ => Err(crate::core::error::LazyTablesError::Connection(
                    "Database type not yet supported".to_string(),
                )),
            };

            // Send result back to main loop
            let event = match result {
                Ok(msg) => TestConnectionEvent::Success(msg),
                Err(e) => TestConnectionEvent::Failed(e.to_string()),
            };

            let _ = tx.send(event);
        });
    }
}
