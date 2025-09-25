// FilePath: src/ui/components/connection_mode.rs

use crate::{
    database::{ConnectionConfig, DatabaseType, SslMode},
    state::ui::ConnectionModeType,
    ui::theme::Theme,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Connection mode component for full-screen connection management
#[derive(Debug, Clone)]
pub struct ConnectionMode {
    /// Current form state
    pub form_state: ConnectionFormState,
    /// Connection being edited (None for Add mode)
    pub editing_connection: Option<ConnectionConfig>,
    /// Current focus section
    pub focused_section: ConnectionSection,
    /// Current form field within the focused section
    pub focused_field: usize,
    /// Whether we're in insert mode for text input
    pub insert_mode: bool,
    /// Error message to display
    pub error_message: Option<String>,
}

/// Simplified connection form state
#[derive(Debug, Clone)]
pub struct ConnectionFormState {
    /// Connection name
    pub name: String,
    /// Selected database type
    pub database_type: DatabaseType,
    /// Host/server address
    pub host: String,
    /// Port number
    pub port: String,
    /// Database name
    pub database: String,
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// Connection string (alternative input method)
    pub connection_string: String,
    /// Whether to use connection string or individual fields
    pub use_connection_string: bool,
    /// SSL mode
    pub ssl_mode: SslMode,
}

/// Connection mode sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionSection {
    /// Database type selection
    DatabaseType,
    /// Connection details (fields or connection string)
    ConnectionDetails,
}

impl Default for ConnectionFormState {
    fn default() -> Self {
        Self {
            name: String::new(),
            database_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: "5432".to_string(),
            database: String::new(),
            username: String::new(),
            password: String::new(),
            connection_string: String::new(),
            use_connection_string: false,
            ssl_mode: SslMode::Prefer,
        }
    }
}

impl ConnectionMode {
    /// Create a new connection mode for adding a connection
    pub fn new_add() -> Self {
        Self {
            form_state: ConnectionFormState::default(),
            editing_connection: None,
            focused_section: ConnectionSection::DatabaseType,
            focused_field: 0,
            insert_mode: false,
            error_message: None,
        }
    }

    /// Create a new connection mode for editing a connection
    pub fn new_edit(connection: ConnectionConfig) -> Self {
        let mut form_state = ConnectionFormState {
            name: connection.name.clone(),
            database_type: connection.database_type.clone(),
            host: connection.host.clone(),
            port: connection.port.to_string(),
            database: connection.database.clone().unwrap_or_default(),
            username: connection.username.clone(),
            password: connection.password.clone().unwrap_or_default(),
            connection_string: String::new(),
            use_connection_string: false,
            ssl_mode: connection.ssl_mode.clone(),
        };

        // Generate connection string for display
        form_state.connection_string = Self::generate_connection_string(&form_state);

        Self {
            form_state,
            editing_connection: Some(connection),
            focused_section: ConnectionSection::DatabaseType,
            focused_field: 0,
            insert_mode: false,
            error_message: None,
        }
    }

    /// Render the connection mode as a full-screen overlay
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        mode_type: ConnectionModeType,
        scroll_offset: usize,
    ) {
        // Clear the background
        frame.render_widget(Clear, area);

        // Main block with borders
        let title = match mode_type {
            ConnectionModeType::Add => " Add New Connection ",
            ConnectionModeType::Edit => " Edit Database Connection ",
        };

        let main_block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_alignment(Alignment::Center)
            .style(
                Style::default()
                    .bg(theme.get_color("background"))
                    .fg(theme.get_color("foreground")),
            );

        let inner_area = main_block.inner(area);
        frame.render_widget(main_block, area);

        // Split the area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Database type selection
                Constraint::Min(10),   // Connection details (more space)
                Constraint::Length(2), // Status (smaller)
                Constraint::Length(3), // Help text
            ])
            .split(inner_area);

        // Render each section
        self.render_database_type_section(frame, chunks[0], theme);
        self.render_connection_details_section(frame, chunks[1], theme, scroll_offset);
        self.render_status_section(frame, chunks[2], theme);
        self.render_help_section(frame, chunks[3], theme);
    }

    /// Render database type selection section
    fn render_database_type_section(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let is_focused = self.focused_section == ConnectionSection::DatabaseType;

        let border_color = if is_focused {
            theme.get_color("primary_highlight")
        } else {
            theme.get_color("inactive_pane")
        };

        let db_type_block = Block::default()
            .borders(Borders::ALL)
            .title(" 1. Database Type ")
            .style(
                Style::default()
                    .bg(theme.get_color("background"))
                    .fg(border_color),
            );

        let inner_area = db_type_block.inner(area);
        frame.render_widget(db_type_block, area);

        // Database type options
        let db_types = [
            ("PostgreSQL", DatabaseType::PostgreSQL),
            ("MySQL", DatabaseType::MySQL),
            ("SQLite", DatabaseType::SQLite),
        ];

        let items: Vec<ListItem> = db_types
            .iter()
            .map(|(name, db_type)| {
                let is_selected = *db_type == self.form_state.database_type;
                let symbol = if is_selected { "● " } else { "○ " };

                let style = if is_selected {
                    Style::default().fg(theme.get_color("primary_highlight"))
                } else {
                    Style::default().fg(theme.get_color("foreground"))
                };

                let line = Line::from(vec![
                    Span::styled(symbol, style),
                    Span::styled(*name, style),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).style(Style::default().bg(theme.get_color("background")));

        frame.render_widget(list, inner_area);
    }

    /// Render connection details section
    fn render_connection_details_section(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        _scroll_offset: usize,
    ) {
        let is_focused = self.focused_section == ConnectionSection::ConnectionDetails;

        let border_color = if is_focused {
            theme.get_color("primary_highlight")
        } else {
            theme.get_color("inactive_pane")
        };

        let details_block = Block::default()
            .borders(Borders::ALL)
            .title(" 2. Connection Details ")
            .style(
                Style::default()
                    .bg(theme.get_color("background"))
                    .fg(border_color),
            );

        let inner_area = details_block.inner(area);
        frame.render_widget(details_block, area);

        if self.form_state.use_connection_string {
            self.render_connection_string_form(frame, inner_area, theme);
        } else {
            self.render_individual_fields_form_two_columns(frame, inner_area, theme);
        }
    }

    /// Render connection string form
    fn render_connection_string_form(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Connection Name
                Constraint::Length(4), // Connection String
                Constraint::Min(1),    // Rest
            ])
            .split(area);

        // Connection Name field
        self.render_input_field(
            frame,
            chunks[0],
            theme,
            "Connection Name",
            &self.form_state.name,
            0,
        );

        // Connection String field
        self.render_text_area_field(
            frame,
            chunks[1],
            theme,
            "Connection String (postgresql://user:pass@host:5432/db)",
            &self.form_state.connection_string,
            1,
        );
    }

    /// Render individual fields form with two columns
    fn render_individual_fields_form_two_columns(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
    ) {
        // Split into two columns
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left column fields
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Connection Name
                Constraint::Length(3), // Host
                Constraint::Length(3), // Database
                Constraint::Min(1),    // Rest
            ])
            .split(columns[0]);

        // Right column fields
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Port
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Min(1),    // Rest
            ])
            .split(columns[1]);

        // Render left column fields
        self.render_input_field(
            frame,
            left_chunks[0],
            theme,
            "Connection Name",
            &self.form_state.name,
            0,
        );
        self.render_input_field(
            frame,
            left_chunks[1],
            theme,
            "Host",
            &self.form_state.host,
            1,
        );
        self.render_input_field(
            frame,
            left_chunks[2],
            theme,
            "Database",
            &self.form_state.database,
            3,
        );

        // Render right column fields
        self.render_input_field(
            frame,
            right_chunks[0],
            theme,
            "Port",
            &self.form_state.port,
            2,
        );
        self.render_input_field(
            frame,
            right_chunks[1],
            theme,
            "Username",
            &self.form_state.username,
            4,
        );
        self.render_input_field(
            frame,
            right_chunks[2],
            theme,
            "Password",
            &self.form_state.password,
            5,
        );
    }

    /// Render a single input field
    fn render_input_field(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        label: &str,
        value: &str,
        field_index: usize,
    ) {
        let is_focused = self.focused_section == ConnectionSection::ConnectionDetails
            && self.focused_field == field_index;

        let border_style = if is_focused {
            if self.insert_mode {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(theme.get_color("primary_highlight"))
            }
        } else {
            Style::default().fg(theme.get_color("inactive_pane"))
        };

        let mode_indicator = if is_focused && self.insert_mode {
            " [INSERT]"
        } else {
            ""
        };

        let title = format!(" {}{} ", label, mode_indicator);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(border_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let display_value = if label == "Password" && !value.is_empty() {
            "•".repeat(value.len())
        } else {
            value.to_string()
        };

        let paragraph =
            Paragraph::new(display_value).style(Style::default().fg(theme.get_color("foreground")));

        frame.render_widget(paragraph, inner_area);
    }

    /// Render a text area field (for connection string)
    fn render_text_area_field(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        label: &str,
        value: &str,
        field_index: usize,
    ) {
        let is_focused = self.focused_section == ConnectionSection::ConnectionDetails
            && self.focused_field == field_index;

        let border_style = if is_focused {
            if self.insert_mode {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(theme.get_color("primary_highlight"))
            }
        } else {
            Style::default().fg(theme.get_color("inactive_pane"))
        };

        let mode_indicator = if is_focused && self.insert_mode {
            " [INSERT]"
        } else {
            ""
        };

        let title = format!(" {}{} ", label, mode_indicator);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(border_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let paragraph = Paragraph::new(value)
            .style(Style::default().fg(theme.get_color("foreground")))
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(paragraph, inner_area);
    }

    /// Render status/error section (compact, bottom line)
    fn render_status_section(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let status_text = if let Some(ref error) = self.error_message {
            format!("Error: {}", error)
        } else {
            "Ready. Use keyboard shortcuts: S=Save, T=Toggle method, C=Cancel".to_string()
        };

        let status_color = if self.error_message.is_some() {
            Color::Red
        } else {
            theme.get_color("foreground")
        };

        let paragraph = Paragraph::new(status_text)
            .style(Style::default().fg(status_color))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    /// Render help section
    fn render_help_section(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let help_block = Block::default()
            .borders(Borders::ALL)
            .title(" Navigation ")
            .style(
                Style::default()
                    .bg(theme.get_color("background"))
                    .fg(theme.get_color("inactive_pane")),
            );

        let inner_area = help_block.inner(area);
        frame.render_widget(help_block, area);

        let help_text = "j/k: Navigate sections • Tab: Navigate fields • i: Insert mode • Esc: Exit insert/mode • Enter: Select database type • T: Toggle method • S: Save • C: Cancel";

        let paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(theme.get_color("foreground")))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, inner_area);
    }

    /// Generate connection string from individual fields
    fn generate_connection_string(form_state: &ConnectionFormState) -> String {
        match form_state.database_type {
            DatabaseType::PostgreSQL => {
                format!(
                    "postgresql://{}:{}@{}:{}/{}",
                    form_state.username,
                    form_state.password,
                    form_state.host,
                    form_state.port,
                    form_state.database
                )
            }
            DatabaseType::MySQL | DatabaseType::MariaDB => {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    form_state.username,
                    form_state.password,
                    form_state.host,
                    form_state.port,
                    form_state.database
                )
            }
            DatabaseType::SQLite => {
                format!("sqlite://{}", form_state.database)
            }
            _ => {
                // For unsupported database types, return a generic connection string format
                format!(
                    "{}://{}:{}@{}:{}/{}",
                    form_state.database_type.display_name(),
                    form_state.username,
                    form_state.password,
                    form_state.host,
                    form_state.port,
                    form_state.database
                )
            }
        }
    }

    /// Navigate to next section
    pub fn next_section(&mut self) {
        self.focused_section = match self.focused_section {
            ConnectionSection::DatabaseType => ConnectionSection::ConnectionDetails,
            ConnectionSection::ConnectionDetails => ConnectionSection::DatabaseType,
        };
        self.focused_field = 0;
    }

    /// Navigate to previous section
    pub fn previous_section(&mut self) {
        self.focused_section = match self.focused_section {
            ConnectionSection::DatabaseType => ConnectionSection::ConnectionDetails,
            ConnectionSection::ConnectionDetails => ConnectionSection::DatabaseType,
        };
        self.focused_field = 0;
    }

    /// Navigate to next field within current section
    pub fn next_field(&mut self) {
        match self.focused_section {
            ConnectionSection::DatabaseType => {
                // Database types: PostgreSQL, MySQL, SQLite
                self.focused_field = (self.focused_field + 1) % 3;
            }
            ConnectionSection::ConnectionDetails => {
                if self.form_state.use_connection_string {
                    // Connection string mode: Name, Connection String
                    self.focused_field = (self.focused_field + 1) % 2;
                } else {
                    // Individual fields mode: Name, Host, Port, Database, Username, Password
                    self.focused_field = (self.focused_field + 1) % 6;
                }
            }
        }
    }

    /// Navigate to previous field within current section
    pub fn previous_field(&mut self) {
        match self.focused_section {
            ConnectionSection::DatabaseType => {
                self.focused_field = if self.focused_field > 0 {
                    self.focused_field - 1
                } else {
                    2
                };
            }
            ConnectionSection::ConnectionDetails => {
                if self.form_state.use_connection_string {
                    self.focused_field = if self.focused_field > 0 {
                        self.focused_field - 1
                    } else {
                        1
                    };
                } else {
                    self.focused_field = if self.focused_field > 0 {
                        self.focused_field - 1
                    } else {
                        5
                    };
                }
            }
        }
    }

    /// Handle Enter key press
    pub fn handle_enter(&mut self) -> ConnectionModeAction {
        match self.focused_section {
            ConnectionSection::DatabaseType => {
                // Select database type
                match self.focused_field {
                    0 => self.form_state.database_type = DatabaseType::PostgreSQL,
                    1 => self.form_state.database_type = DatabaseType::MySQL,
                    2 => self.form_state.database_type = DatabaseType::SQLite,
                    _ => {}
                }
                // Update port based on database type
                self.update_default_port();
                ConnectionModeAction::None
            }
            ConnectionSection::ConnectionDetails => {
                if !self.insert_mode {
                    self.insert_mode = true;
                }
                ConnectionModeAction::None
            }
        }
    }

    /// Update default port based on database type
    fn update_default_port(&mut self) {
        if self.form_state.port.is_empty() || self.is_default_port(&self.form_state.port) {
            self.form_state.port = match self.form_state.database_type {
                DatabaseType::PostgreSQL => "5432".to_string(),
                DatabaseType::MySQL | DatabaseType::MariaDB => "3306".to_string(),
                DatabaseType::SQLite => "0".to_string(),
                _ => "0".to_string(), // Default for unsupported types
            };
        }
    }

    /// Check if port is a default port
    fn is_default_port(&self, port: &str) -> bool {
        matches!(port, "5432" | "3306" | "0")
    }

    /// Toggle between connection string and individual fields
    pub fn toggle_input_method(&mut self) {
        self.form_state.use_connection_string = !self.form_state.use_connection_string;

        if self.form_state.use_connection_string {
            // Generate connection string from current fields
            self.form_state.connection_string = Self::generate_connection_string(&self.form_state);
        }

        // Reset field focus
        self.focused_field = 0;
    }

    /// Enter insert mode
    pub fn enter_insert_mode(&mut self) {
        if self.focused_section == ConnectionSection::ConnectionDetails {
            self.insert_mode = true;
        }
    }

    /// Exit insert mode
    pub fn exit_insert_mode(&mut self) {
        self.insert_mode = false;
    }

    /// Add character to current field
    pub fn add_char(&mut self, ch: char) {
        if !self.insert_mode || self.focused_section != ConnectionSection::ConnectionDetails {
            return;
        }

        if self.form_state.use_connection_string {
            match self.focused_field {
                0 => self.form_state.name.push(ch),
                1 => self.form_state.connection_string.push(ch),
                _ => {}
            }
        } else {
            match self.focused_field {
                0 => self.form_state.name.push(ch),
                1 => self.form_state.host.push(ch),
                2 => self.form_state.port.push(ch),
                3 => self.form_state.database.push(ch),
                4 => self.form_state.username.push(ch),
                5 => self.form_state.password.push(ch),
                _ => {}
            }
        }
    }

    /// Remove character from current field
    pub fn backspace(&mut self) {
        if !self.insert_mode || self.focused_section != ConnectionSection::ConnectionDetails {
            return;
        }

        if self.form_state.use_connection_string {
            match self.focused_field {
                0 => {
                    self.form_state.name.pop();
                }
                1 => {
                    self.form_state.connection_string.pop();
                }
                _ => {}
            }
        } else {
            match self.focused_field {
                0 => {
                    self.form_state.name.pop();
                }
                1 => {
                    self.form_state.host.pop();
                }
                2 => {
                    self.form_state.port.pop();
                }
                3 => {
                    self.form_state.database.pop();
                }
                4 => {
                    self.form_state.username.pop();
                }
                5 => {
                    self.form_state.password.pop();
                }
                _ => {}
            }
        }
    }

    /// Set error message
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Convert form state to ConnectionConfig
    pub fn to_connection_config(&self) -> Result<ConnectionConfig, String> {
        if self.form_state.name.trim().is_empty() {
            return Err("Connection name is required".to_string());
        }

        let port = self
            .form_state
            .port
            .parse::<u16>()
            .map_err(|_| "Invalid port number".to_string())?;

        Ok(ConnectionConfig {
            id: uuid::Uuid::new_v4().to_string(),
            name: self.form_state.name.clone(),
            database_type: self.form_state.database_type.clone(),
            host: self.form_state.host.clone(),
            port,
            database: if self.form_state.database.is_empty() {
                None
            } else {
                Some(self.form_state.database.clone())
            },
            username: self.form_state.username.clone(),
            password: if self.form_state.password.is_empty() {
                None
            } else {
                Some(self.form_state.password.clone())
            },
            password_source: None,
            ssl_mode: self.form_state.ssl_mode.clone(),
            timeout: None,
            status: crate::database::ConnectionStatus::Disconnected,
        })
    }
}

/// Actions that can be triggered from connection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionModeAction {
    None,
    TestConnection,
    Save,
    Cancel,
}

impl Default for ConnectionMode {
    fn default() -> Self {
        Self::new_add()
    }
}
