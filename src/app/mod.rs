// FilePath: src/app/mod.rs

use crate::{
    commands::{CommandAction, CommandContext, CommandId, CommandRegistry, CommandResult},
    config::Config,
    core::error::Result,
    event::{Event, EventHandler},
    ui::UI,
};
use crossterm::event::KeyEvent;
use ratatui::{DefaultTerminal, Frame};
use std::time::Duration;

pub mod handlers;
pub mod state;

pub use state::{AppState, AppView, ConnectionFormMode, FocusedPane, HelpMode, OverlayView, TextInputMode};

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
    /// Task handle for ongoing test connection (for abort capability)
    test_connection_task_handle: Option<tokio::task::JoinHandle<()>>,
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
            test_connection_task_handle: None,
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
                // Note: Query execution is handled directly by key handlers (handle_query_editor_keys)
                // via execute_query_at_cursor(). This action is kept for command system compatibility
                // but actual execution happens in the async key handler.
                self.state.toast_manager.info(format!("Query submitted: {}",
                    query.lines().next().unwrap_or("").chars().take(50).collect::<String>()));
            }
            CommandAction::ExecuteQueryWithContext {
                query: _,
                database_type,
                connection_name,
            } => {
                // Note: Query execution is handled directly by key handlers (handle_query_editor_keys)
                // via execute_query_at_cursor(). This action is kept for command system compatibility
                // but actual execution happens in the async key handler.
                self.state.toast_manager.info(format!(
                    "Query submitted to {} ({})",
                    connection_name,
                    database_type.display_name()
                ));
            }
            CommandAction::LoadFile(path) => {
                // Note: File loading is handled directly by the SQL Files pane via load_selected_sql_file().
                // This action is unused but kept for command system compatibility.
                self.state.toast_manager.info(format!("Load file: {path}"));
            }
            CommandAction::SaveFile(path) => {
                // Note: File saving is handled directly by the Query Editor command mode (:w command)
                // via save_sql_file_with_connection(). This action is unused but kept for compatibility.
                self.state.toast_manager.info(format!("Save file: {path}"));
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
        if handlers::global::handle(self, key)?.is_some() {
            return Ok(());
        }

        // 2. Handle overlays (forms, modals, debug view)
        if self.state.ui.is_in_overlay() {
            return handlers::overlays::handle(self, key).await;
        }

        // 3. Handle confirmation modals
        if self.state.ui.confirmation_modal.is_some() {
            return handlers::overlays::handle_confirmation_modal(self, key).await;
        }

        // 4. Handle table viewer delete confirmation
        if self.state.table_viewer_state.delete_confirmation.is_some() {
            return handlers::overlays::handle_table_delete_confirmation(self, key).await;
        }

        // 4b. Handle table viewer set NULL confirmation
        if self.state.table_viewer_state.set_null_confirmation.is_some() {
            return handlers::overlays::handle_set_null_confirmation(self, key).await;
        }

        // 5. Route to focused pane handler (main view)
        match self.state.ui.focused_pane {
            FocusedPane::Connections => handlers::connections::handle(self, key).await,
            FocusedPane::Tables => handlers::tables::handle(self, key).await,
            FocusedPane::Details => handlers::details::handle(self, key),
            FocusedPane::TabularOutput => handlers::query_results::handle(self, key).await,
            FocusedPane::SqlFiles => handlers::sql_files::handle(self, key).await,
            FocusedPane::QueryWindow => handlers::query_editor::handle(self, key).await,
        }
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
                    self.test_connection_task_handle = None;
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
                self.test_connection_task_handle = None;
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

}
