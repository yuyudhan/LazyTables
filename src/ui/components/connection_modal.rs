// FilePath: src/ui/components/connection_modal.rs

use crate::database::connection::{ConnectionConfig, DatabaseType, SslMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

/// State for the connection creation modal
#[derive(Debug, Clone)]
pub struct ConnectionModalState {
    /// Currently focused field
    pub focused_field: ConnectionField,
    /// Connection name input
    pub name: String,
    /// Selected database type
    pub database_type: DatabaseType,
    /// Database type selection state
    pub db_type_list_state: ListState,
    /// Host input
    pub host: String,
    /// Port input
    pub port_input: String,
    /// Database name input
    pub database: String,
    /// Username input
    pub username: String,
    /// Password input (not stored in plain text)
    pub password: String,
    /// SSL mode selection
    pub ssl_mode: SslMode,
    /// SSL mode selection state
    pub ssl_list_state: ListState,
    /// Error message to display
    pub error_message: Option<String>,
}

/// Fields in the connection modal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionField {
    Name,
    DatabaseType,
    Host,
    Port,
    Database,
    Username,
    Password,
    SslMode,
    Save,
    Cancel,
}

impl ConnectionField {
    /// Get the next field in tab order
    pub fn next(&self) -> Self {
        match self {
            Self::Name => Self::DatabaseType,
            Self::DatabaseType => Self::Host,
            Self::Host => Self::Port,
            Self::Port => Self::Database,
            Self::Database => Self::Username,
            Self::Username => Self::Password,
            Self::Password => Self::SslMode,
            Self::SslMode => Self::Save,
            Self::Save => Self::Cancel,
            Self::Cancel => Self::Name,
        }
    }

    /// Get the previous field in tab order
    pub fn previous(&self) -> Self {
        match self {
            Self::Name => Self::Cancel,
            Self::DatabaseType => Self::Name,
            Self::Host => Self::DatabaseType,
            Self::Port => Self::Host,
            Self::Database => Self::Port,
            Self::Username => Self::Database,
            Self::Password => Self::Username,
            Self::SslMode => Self::Password,
            Self::Save => Self::SslMode,
            Self::Cancel => Self::Save,
        }
    }

    /// Get display name for the field
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Name => "Connection Name",
            Self::DatabaseType => "Database Type",
            Self::Host => "Host",
            Self::Port => "Port",
            Self::Database => "Database",
            Self::Username => "Username",
            Self::Password => "Password",
            Self::SslMode => "SSL Mode",
            Self::Save => "Save",
            Self::Cancel => "Cancel",
        }
    }
}

impl Default for ConnectionModalState {
    fn default() -> Self {
        let mut db_type_list_state = ListState::default();
        db_type_list_state.select(Some(0)); // Default to PostgreSQL

        let mut ssl_list_state = ListState::default();
        ssl_list_state.select(Some(2)); // Default to Prefer

        Self {
            focused_field: ConnectionField::Name,
            name: String::new(),
            database_type: DatabaseType::PostgreSQL,
            db_type_list_state,
            host: "localhost".to_string(),
            port_input: "5432".to_string(),
            database: String::new(),
            username: String::new(),
            password: String::new(),
            ssl_mode: SslMode::Prefer,
            ssl_list_state,
            error_message: None,
        }
    }
}

impl ConnectionModalState {
    /// Create a new modal state
    pub fn new() -> Self {
        Self::default()
    }

    /// Move to next field
    pub fn next_field(&mut self) {
        self.focused_field = self.focused_field.next();
    }

    /// Move to previous field
    pub fn previous_field(&mut self) {
        self.focused_field = self.focused_field.previous();
    }

    /// Handle character input for the current field
    pub fn handle_char_input(&mut self, c: char) {
        match self.focused_field {
            ConnectionField::Name => {
                self.name.push(c);
            }
            ConnectionField::Host => {
                self.host.push(c);
            }
            ConnectionField::Port => {
                if c.is_ascii_digit() {
                    self.port_input.push(c);
                }
            }
            ConnectionField::Database => {
                self.database.push(c);
            }
            ConnectionField::Username => {
                self.username.push(c);
            }
            ConnectionField::Password => {
                self.password.push(c);
            }
            _ => {}
        }
        self.error_message = None; // Clear error on input
    }

    /// Handle backspace for the current field
    pub fn handle_backspace(&mut self) {
        match self.focused_field {
            ConnectionField::Name => {
                self.name.pop();
            }
            ConnectionField::Host => {
                self.host.pop();
            }
            ConnectionField::Port => {
                self.port_input.pop();
            }
            ConnectionField::Database => {
                self.database.pop();
            }
            ConnectionField::Username => {
                self.username.pop();
            }
            ConnectionField::Password => {
                self.password.pop();
            }
            _ => {}
        }
    }

    /// Select database type from dropdown
    pub fn select_database_type(&mut self, index: usize) {
        let types = [
            DatabaseType::PostgreSQL,
            DatabaseType::MySQL,
            DatabaseType::MariaDB,
            DatabaseType::SQLite,
        ];
        
        if let Some(db_type) = types.get(index) {
            self.database_type = db_type.clone();
            self.db_type_list_state.select(Some(index));
            
            // Update default port based on database type
            self.port_input = match db_type {
                DatabaseType::PostgreSQL => "5432".to_string(),
                DatabaseType::MySQL | DatabaseType::MariaDB => "3306".to_string(),
                DatabaseType::SQLite => "".to_string(),
                _ => self.port_input.clone(),
            };
        }
    }

    /// Select SSL mode from dropdown
    pub fn select_ssl_mode(&mut self, index: usize) {
        let modes = [
            SslMode::Disable,
            SslMode::Allow,
            SslMode::Prefer,
            SslMode::Require,
            SslMode::VerifyCA,
            SslMode::VerifyFull,
        ];
        
        if let Some(mode) = modes.get(index) {
            self.ssl_mode = mode.clone();
            self.ssl_list_state.select(Some(index));
        }
    }

    /// Validate the current input and create a connection config
    pub fn try_create_connection(&self) -> Result<ConnectionConfig, String> {
        // Validate required fields
        if self.name.trim().is_empty() {
            return Err("Connection name is required".to_string());
        }
        
        if self.host.trim().is_empty() {
            return Err("Host is required".to_string());
        }
        
        if self.username.trim().is_empty() {
            return Err("Username is required".to_string());
        }

        // Parse port
        let port: u16 = if self.port_input.trim().is_empty() && self.database_type == DatabaseType::SQLite {
            0 // SQLite doesn't use ports
        } else {
            self.port_input.trim().parse()
                .map_err(|_| "Invalid port number".to_string())?
        };

        // Create connection config
        let mut connection = ConnectionConfig::new(
            self.name.trim().to_string(),
            self.database_type.clone(),
            self.host.trim().to_string(),
            port,
            self.username.trim().to_string(),
        );

        // Set optional fields
        if !self.database.trim().is_empty() {
            connection.database = Some(self.database.trim().to_string());
        }
        
        if !self.password.trim().is_empty() {
            connection.password = Some(self.password.trim().to_string());
        }
        
        connection.ssl_mode = self.ssl_mode.clone();

        Ok(connection)
    }

    /// Clear all fields
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

/// Render the connection creation modal
pub fn render_connection_modal(f: &mut Frame, modal_state: &ConnectionModalState, area: Rect) {
    // Create centered modal area with better proportions
    let modal_area = centered_rect(70, 85, area);
    
    // Clear the background
    f.render_widget(Clear, modal_area);
    
    // Main modal block with elegant styling
    let modal_block = Block::default()
        .title(" 🗄️  New Database Connection ")
        .title_style(Style::default()
            .fg(Color::Rgb(116, 199, 236)) // LazyTables brand color
            .add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(116, 199, 236)))
        .style(Style::default()
            .bg(Color::Rgb(13, 13, 13))    // Dark background
            .fg(Color::Rgb(255, 255, 255))); // White text
        
    f.render_widget(modal_block, modal_area);
    
    // Inner area for content with better margins
    let inner_area = modal_area.inner(Margin::new(3, 2));
    
    // Split into header, main content, and buttons
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header/instructions
            Constraint::Min(0),     // Form fields
            Constraint::Length(4),  // Buttons area
        ])
        .split(inner_area);
    
    // Render header/instructions
    render_modal_header(f, main_chunks[0]);
    
    // Render form fields
    render_form_fields(f, modal_state, main_chunks[1]);
    
    // Render buttons and error area
    render_modal_footer(f, modal_state, main_chunks[2]);
}

/// Render the modal header with instructions
fn render_modal_header(f: &mut Frame, area: Rect) {
    let instructions = vec![
        Line::from(vec![
            Span::styled("Navigate: ", Style::default().fg(Color::Gray)),
            Span::styled("Tab/Shift+Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" or ", Style::default().fg(Color::Gray)),
            Span::styled("j/k", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("  •  Save: ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("  •  Cancel: ", Style::default().fg(Color::Gray)),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];
    
    let header = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Rgb(205, 214, 244)))
        .alignment(Alignment::Center);
    
    f.render_widget(header, area);
}

/// Render the form fields
fn render_form_fields(f: &mut Frame, modal_state: &ConnectionModalState, area: Rect) {
    // Split into two columns for better layout
    let main_columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left column fields
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Database Type
            Constraint::Length(3), // Host  
            Constraint::Length(3), // Port
        ])
        .split(main_columns[0]);

    // Right column fields
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Database
            Constraint::Length(3), // Username
            Constraint::Length(3), // Password
            Constraint::Length(3), // SSL Mode
        ])
        .split(main_columns[1]);

    // Render left column fields
    render_text_field(f, "Connection Name", &modal_state.name, modal_state.focused_field == ConnectionField::Name, left_chunks[0]);
    render_dropdown_field(f, "Database Type", &get_database_types(), modal_state.focused_field == ConnectionField::DatabaseType, &modal_state.db_type_list_state, left_chunks[1]);
    render_text_field(f, "Host", &modal_state.host, modal_state.focused_field == ConnectionField::Host, left_chunks[2]);
    render_text_field(f, "Port", &modal_state.port_input, modal_state.focused_field == ConnectionField::Port, left_chunks[3]);

    // Render right column fields
    render_text_field(f, "Database (Optional)", &modal_state.database, modal_state.focused_field == ConnectionField::Database, right_chunks[0]);
    render_text_field(f, "Username", &modal_state.username, modal_state.focused_field == ConnectionField::Username, right_chunks[1]);
    render_password_field(f, "Password (Optional)", &modal_state.password, modal_state.focused_field == ConnectionField::Password, right_chunks[2]);
    render_dropdown_field(f, "SSL Mode", &get_ssl_modes(), modal_state.focused_field == ConnectionField::SslMode, &modal_state.ssl_list_state, right_chunks[3]);
}

/// Render a text input field
fn render_text_field(f: &mut Frame, label: &str, value: &str, focused: bool, area: Rect) {
    let (border_style, title_style) = if focused {
        (
            Style::default().fg(Color::Rgb(116, 199, 236)).add_modifier(Modifier::BOLD),
            Style::default().fg(Color::Rgb(116, 199, 236)).add_modifier(Modifier::BOLD)
        )
    } else {
        (
            Style::default().fg(Color::Rgb(69, 71, 90)),
            Style::default().fg(Color::Gray)
        )
    };
    
    let block = Block::default()
        .title(format!(" {} ", label))
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style);
        
    let display_value = if focused && !value.is_empty() {
        format!("{}│", value) // Add cursor indicator
    } else if focused {
        "│".to_string() // Just cursor when empty
    } else {
        value.to_string()
    };
        
    let paragraph = Paragraph::new(display_value)
        .block(block)
        .style(Style::default().fg(Color::White));
        
    f.render_widget(paragraph, area);
}

/// Render a password field (masked)
fn render_password_field(f: &mut Frame, label: &str, value: &str, focused: bool, area: Rect) {
    let masked_value = if focused && !value.is_empty() {
        format!("{}│", "*".repeat(value.len())) // Show cursor after masking
    } else if focused {
        "│".to_string() // Just cursor when empty
    } else {
        "*".repeat(value.len())
    };
    
    let (border_style, title_style) = if focused {
        (
            Style::default().fg(Color::Rgb(116, 199, 236)).add_modifier(Modifier::BOLD),
            Style::default().fg(Color::Rgb(116, 199, 236)).add_modifier(Modifier::BOLD)
        )
    } else {
        (
            Style::default().fg(Color::Rgb(69, 71, 90)),
            Style::default().fg(Color::Gray)
        )
    };
    
    let block = Block::default()
        .title(format!(" {} ", label))
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style);
        
    let paragraph = Paragraph::new(masked_value)
        .block(block)
        .style(Style::default().fg(Color::White));
        
    f.render_widget(paragraph, area);
}

/// Render a dropdown field
fn render_dropdown_field(f: &mut Frame, label: &str, items: &[String], focused: bool, list_state: &ListState, area: Rect) {
    let (border_style, title_style) = if focused {
        (
            Style::default().fg(Color::Rgb(116, 199, 236)).add_modifier(Modifier::BOLD),
            Style::default().fg(Color::Rgb(116, 199, 236)).add_modifier(Modifier::BOLD)
        )
    } else {
        (
            Style::default().fg(Color::Rgb(69, 71, 90)),
            Style::default().fg(Color::Gray)
        )
    };
    
    let block = Block::default()
        .title(format!(" {} ", label))
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style);
        
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| ListItem::new(item.as_str()))
        .collect();
        
    let mut list_state = list_state.clone();
    let list = List::new(list_items)
        .block(block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default()
            .bg(Color::Rgb(116, 199, 236))
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD));
        
    f.render_stateful_widget(list, area, &mut list_state);
}

/// Render the modal footer with buttons and error messages
fn render_modal_footer(f: &mut Frame, modal_state: &ConnectionModalState, area: Rect) {
    // Split footer into buttons and error area
    let footer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Buttons
            Constraint::Length(1), // Error message
        ])
        .split(area);

    // Render buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ])
        .split(footer_chunks[0]);
    
    let save_style = if modal_state.focused_field == ConnectionField::Save {
        Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    };
    
    let cancel_style = if modal_state.focused_field == ConnectionField::Cancel {
        Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    };
    
    let save_button = Paragraph::new("💾 Save Connection")
        .style(save_style)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(if modal_state.focused_field == ConnectionField::Save {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Rgb(69, 71, 90))
            }));
        
    let cancel_button = Paragraph::new("❌ Cancel")
        .style(cancel_style)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(if modal_state.focused_field == ConnectionField::Cancel {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Rgb(69, 71, 90))
            }));
        
    f.render_widget(save_button, button_chunks[0]);
    f.render_widget(cancel_button, button_chunks[2]);

    // Render error message if present
    if let Some(error) = &modal_state.error_message {
        let error_paragraph = Paragraph::new(format!("❗ {}", error))
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(error_paragraph, footer_chunks[1]);
    }
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Get available database types for dropdown
fn get_database_types() -> Vec<String> {
    vec![
        "PostgreSQL".to_string(),
        "MySQL".to_string(),
        "MariaDB".to_string(),
        "SQLite".to_string(),
    ]
}

/// Get available SSL modes for dropdown
fn get_ssl_modes() -> Vec<String> {
    vec![
        "Disable".to_string(),
        "Allow".to_string(),
        "Prefer".to_string(),
        "Require".to_string(),
        "Verify CA".to_string(),
        "Verify Full".to_string(),
    ]
}