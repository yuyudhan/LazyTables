// FilePath: src/app/mod.rs

use crate::{
    config::Config,
    core::error::Result,
    event::{Event, EventHandler},
    ui::UI,
};
use crossterm::event::KeyEvent;
use ratatui::{DefaultTerminal, Frame};
use std::time::Duration;

pub mod state;

pub use state::{AppState, FocusedPane, Mode};

/// Main application structure
pub struct App {
    /// Application state
    pub state: AppState,
    /// Event handler
    event_handler: EventHandler,
    /// User interface
    ui: UI,
    /// Configuration
    _config: Config,
    /// Flag to quit the application
    should_quit: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(config: Config) -> Result<Self> {
        let state = AppState::new();
        let event_handler = EventHandler::new(Duration::from_millis(250));
        let ui = UI::new(&config)?;

        Ok(Self {
            state,
            event_handler,
            ui,
            _config: config,
            should_quit: false,
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

    /// Handle keyboard events based on current mode
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        // Handle ESC key globally to close help overlay
        if key.code == KeyCode::Esc && self.state.show_help {
            self.state.show_help = false;
            return Ok(());
        }

        match self.state.mode {
            Mode::Normal => {
                match (key.modifiers, key.code) {
                    // Enter command mode with ':'
                    (KeyModifiers::NONE, KeyCode::Char(':')) => {
                        self.state.mode = Mode::Command;
                        self.state.command_buffer.clear();
                    }
                    // Pane navigation with Ctrl+h/j/k/l
                    (KeyModifiers::CONTROL, KeyCode::Char('h')) => {
                        self.state.focused_pane = FocusedPane::Connections;
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('j')) => {
                        self.state.focused_pane = FocusedPane::Tables;
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                        self.state.focused_pane = FocusedPane::Details;
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                        self.state.focused_pane = FocusedPane::MainContent;
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
                    // Enter insert mode
                    (KeyModifiers::NONE, KeyCode::Char('i')) => {
                        self.state.mode = Mode::Insert;
                    }
                    // Enter visual mode
                    (KeyModifiers::NONE, KeyCode::Char('v')) => {
                        self.state.mode = Mode::Visual;
                    }
                    // Show help overlay
                    (KeyModifiers::NONE, KeyCode::Char('?')) => {
                        self.state.show_help = !self.state.show_help;
                    }
                    // Leader key (Space) commands
                    (KeyModifiers::NONE, KeyCode::Char(' ')) => {
                        // Track that space was pressed and wait for next key
                        self.state.leader_pressed = true;
                    }
                    _ => {
                        // Handle leader key combinations
                        if self.state.leader_pressed {
                            self.state.leader_pressed = false;
                            // Leader key combinations can be added here for future features
                            // For now, just reset the leader state
                        }
                    }
                }
            }
            Mode::Insert => {
                match key.code {
                    KeyCode::Esc => {
                        self.state.mode = Mode::Normal;
                    }
                    _ => {
                        // Handle insert mode input
                    }
                }
            }
            Mode::Visual => {
                match key.code {
                    KeyCode::Esc => {
                        self.state.mode = Mode::Normal;
                    }
                    _ => {
                        // Handle visual mode selection
                    }
                }
            }
            Mode::Command => {
                match key.code {
                    KeyCode::Esc => {
                        self.state.command_buffer.clear();
                        self.state.mode = Mode::Normal;
                    }
                    KeyCode::Enter => {
                        // Execute command
                        let command = self.state.command_buffer.trim();
                        if command == "q" || command == "quit" {
                            self.should_quit = true;
                        }
                        self.state.command_buffer.clear();
                        self.state.mode = Mode::Normal;
                    }
                    KeyCode::Char(c) => {
                        self.state.command_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        self.state.command_buffer.pop();
                    }
                    _ => {}
                }
            }
            Mode::Query => {
                match key.code {
                    KeyCode::Esc => {
                        self.state.mode = Mode::Normal;
                    }
                    _ => {
                        // Handle query mode input
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle periodic updates
    async fn tick(&mut self) -> Result<()> {
        // Update any time-based state here
        Ok(())
    }
}

