// FilePath: src/ui/components/connection_modal.rs

use crate::database::connection::{ConnectionConfig, DatabaseType, SslMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Type alias for connection string parsing result
type ParseResult = Result<(String, u16, String, Option<String>, Option<String>), String>;

/// State for the connection creation modal
#[derive(Debug, Clone)]
pub struct ConnectionModalState {
    /// Current modal step
    pub current_step: ModalStep,
    /// Currently focused field
    pub focused_field: ConnectionField,
    /// Connection name input
    pub name: String,
    /// Selected database type
    pub database_type: DatabaseType,
    /// Database type selection state
    pub db_type_list_state: ListState,
    /// Connection string input
    pub connection_string: String,
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
    /// Whether using connection string instead of individual fields
    pub using_connection_string: bool,
}

/// Fields in the connection modal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionField {
    Name,
    DatabaseType,
    ConnectionString,
    Host,
    Port,
    Database,
    Username,
    Password,
    SslMode,
    Save,
    Cancel,
}

/// Modal flow states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalStep {
    /// First step: select database type
    DatabaseTypeSelection,
    /// Second step: connection details
    ConnectionDetails,
}

impl ConnectionField {
    /// Get the next field in tab order based on current step and mode
    pub fn next(&self, step: ModalStep, using_connection_string: bool) -> Self {
        match step {
            ModalStep::DatabaseTypeSelection => match self {
                Self::DatabaseType => Self::Save,
                Self::Save => Self::Cancel,
                Self::Cancel => Self::DatabaseType,
                _ => Self::DatabaseType,
            },
            ModalStep::ConnectionDetails => {
                if using_connection_string {
                    match self {
                        Self::Name => Self::ConnectionString,
                        Self::ConnectionString => Self::Save,
                        Self::Save => Self::Cancel,
                        Self::Cancel => Self::Name,
                        _ => Self::Name,
                    }
                } else {
                    match self {
                        Self::Name => Self::ConnectionString,
                        Self::ConnectionString => Self::Host,
                        Self::Host => Self::Port,
                        Self::Port => Self::Database,
                        Self::Database => Self::Username,
                        Self::Username => Self::Password,
                        Self::Password => Self::SslMode,
                        Self::SslMode => Self::Save,
                        Self::Save => Self::Cancel,
                        Self::Cancel => Self::Name,
                        _ => Self::Name,
                    }
                }
            }
        }
    }

    /// Get the previous field in tab order based on current step and mode
    pub fn previous(&self, step: ModalStep, using_connection_string: bool) -> Self {
        match step {
            ModalStep::DatabaseTypeSelection => match self {
                Self::DatabaseType => Self::Cancel,
                Self::Save => Self::DatabaseType,
                Self::Cancel => Self::Save,
                _ => Self::DatabaseType,
            },
            ModalStep::ConnectionDetails => {
                if using_connection_string {
                    match self {
                        Self::Name => Self::Cancel,
                        Self::ConnectionString => Self::Name,
                        Self::Save => Self::ConnectionString,
                        Self::Cancel => Self::Save,
                        _ => Self::Name,
                    }
                } else {
                    match self {
                        Self::Name => Self::Cancel,
                        Self::ConnectionString => Self::Name,
                        Self::Host => Self::ConnectionString,
                        Self::Port => Self::Host,
                        Self::Database => Self::Port,
                        Self::Username => Self::Database,
                        Self::Password => Self::Username,
                        Self::SslMode => Self::Password,
                        Self::Save => Self::SslMode,
                        Self::Cancel => Self::Save,
                        _ => Self::Name,
                    }
                }
            }
        }
    }

    /// Get display name for the field
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Name => "Connection Name",
            Self::DatabaseType => "Database Type",
            Self::ConnectionString => "Connection String",
            Self::Host => "Host",
            Self::Port => "Port",
            Self::Database => "Database",
            Self::Username => "Username",
            Self::Password => "Password",
            Self::SslMode => "SSL Mode",
            Self::Save => "Save (s)",
            Self::Cancel => "Cancel (c)",
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
            current_step: ModalStep::DatabaseTypeSelection,
            focused_field: ConnectionField::DatabaseType,
            name: String::new(),
            database_type: DatabaseType::PostgreSQL,
            db_type_list_state,
            connection_string: String::new(),
            host: "localhost".to_string(),
            port_input: "5432".to_string(),
            database: String::new(),
            username: String::new(),
            password: String::new(),
            ssl_mode: SslMode::Prefer,
            ssl_list_state,
            error_message: None,
            using_connection_string: false,
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
        self.focused_field = self
            .focused_field
            .next(self.current_step, self.using_connection_string);
    }

    /// Move to previous field
    pub fn previous_field(&mut self) {
        self.focused_field = self
            .focused_field
            .previous(self.current_step, self.using_connection_string);
    }

    /// Advance to next step or execute action
    pub fn advance_step(&mut self) -> bool {
        match self.current_step {
            ModalStep::DatabaseTypeSelection => {
                self.current_step = ModalStep::ConnectionDetails;
                self.focused_field = ConnectionField::Name;
                true
            }
            ModalStep::ConnectionDetails => false, // Ready for action (save/cancel)
        }
    }

    /// Go back to previous step
    pub fn go_back(&mut self) {
        match self.current_step {
            ModalStep::DatabaseTypeSelection => {} // Can't go back further
            ModalStep::ConnectionDetails => {
                self.current_step = ModalStep::DatabaseTypeSelection;
                self.focused_field = ConnectionField::DatabaseType;
            }
        }
    }

    /// Handle character input for the current field
    pub fn handle_char_input(&mut self, c: char) {
        match self.focused_field {
            ConnectionField::Name => {
                self.name.push(c);
            }
            ConnectionField::ConnectionString => {
                self.connection_string.push(c);
                // When connection string is being typed, switch to connection string mode
                if !self.connection_string.is_empty() {
                    self.using_connection_string = true;
                    self.clear_individual_fields();
                }
            }
            ConnectionField::Host => {
                if !self.using_connection_string {
                    self.host.push(c);
                }
            }
            ConnectionField::Port => {
                if !self.using_connection_string && c.is_ascii_digit() {
                    self.port_input.push(c);
                }
            }
            ConnectionField::Database => {
                if !self.using_connection_string {
                    self.database.push(c);
                }
            }
            ConnectionField::Username => {
                if !self.using_connection_string {
                    self.username.push(c);
                }
            }
            ConnectionField::Password => {
                if !self.using_connection_string {
                    self.password.push(c);
                }
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
            ConnectionField::ConnectionString => {
                self.connection_string.pop();
                // If connection string becomes empty, switch back to individual fields mode
                if self.connection_string.is_empty() {
                    self.using_connection_string = false;
                }
            }
            ConnectionField::Host => {
                if !self.using_connection_string {
                    self.host.pop();
                }
            }
            ConnectionField::Port => {
                if !self.using_connection_string {
                    self.port_input.pop();
                }
            }
            ConnectionField::Database => {
                if !self.using_connection_string {
                    self.database.pop();
                }
            }
            ConnectionField::Username => {
                if !self.using_connection_string {
                    self.username.pop();
                }
            }
            ConnectionField::Password => {
                if !self.using_connection_string {
                    self.password.pop();
                }
            }
            _ => {}
        }
    }

    /// Clear individual connection fields
    fn clear_individual_fields(&mut self) {
        self.host = "localhost".to_string();
        self.port_input = match self.database_type {
            DatabaseType::PostgreSQL => "5432".to_string(),
            DatabaseType::MySQL | DatabaseType::MariaDB => "3306".to_string(),
            DatabaseType::SQLite => "".to_string(),
            _ => "5432".to_string(),
        };
        self.database.clear();
        self.username.clear();
        self.password.clear();
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

    /// Parse connection string and extract connection details
    fn parse_connection_string(
        &self,
    ) -> ParseResult {
        let conn_str = self.connection_string.trim();

        // Handle different connection string formats based on database type
        match self.database_type {
            DatabaseType::PostgreSQL => {
                // PostgreSQL format: postgresql://username:password@host:port/database
                // or postgres://username:password@host:port/database
                self.parse_uri_connection_string(conn_str, &["postgresql", "postgres"], 5432)
            }
            DatabaseType::MySQL | DatabaseType::MariaDB => {
                // MySQL format: mysql://username:password@host:port/database
                self.parse_uri_connection_string(conn_str, &["mysql"], 3306)
            }
            DatabaseType::SQLite => {
                // SQLite format: sqlite:///path/to/database.db or sqlite://./relative/path.db
                if conn_str.starts_with("sqlite://") {
                    let path = conn_str.strip_prefix("sqlite://").unwrap_or(conn_str);
                    Ok((path.to_string(), 0, String::new(), None, None))
                } else {
                    Err("Invalid SQLite connection string format. Expected: sqlite:///path/to/database.db".to_string())
                }
            }
            _ => Err("Connection strings not yet supported for this database type".to_string()),
        }
    }

    /// Parse URI-style connection string (for PostgreSQL, MySQL, etc.)
    fn parse_uri_connection_string(
        &self,
        conn_str: &str,
        schemes: &[&str],
        default_port: u16,
    ) -> ParseResult {
        // Find the scheme
        let scheme_end = conn_str
            .find("://")
            .ok_or("Invalid connection string: missing ://".to_string())?;
        let scheme = &conn_str[..scheme_end];

        if !schemes.contains(&scheme) {
            return Err(format!(
                "Invalid scheme '{}'. Expected one of: {}",
                scheme,
                schemes.join(", ")
            ));
        }

        let rest = &conn_str[scheme_end + 3..]; // Skip "://"

        // Split by @ to separate auth from host
        let (auth_part, host_part) = if let Some(at_pos) = rest.find('@') {
            (Some(&rest[..at_pos]), &rest[at_pos + 1..])
        } else {
            (None, rest)
        };

        // Parse auth part (username:password)
        let (username, password) = if let Some(auth) = auth_part {
            if let Some(colon_pos) = auth.find(':') {
                (
                    auth[..colon_pos].to_string(),
                    Some(auth[colon_pos + 1..].to_string()),
                )
            } else {
                (auth.to_string(), None)
            }
        } else {
            (String::new(), None)
        };

        // Parse host part (host:port/database)
        let (host_port, database) = if let Some(slash_pos) = host_part.find('/') {
            (
                &host_part[..slash_pos],
                Some(host_part[slash_pos + 1..].to_string()),
            )
        } else {
            (host_part, None)
        };

        // Parse host and port
        let (host, port) = if let Some(colon_pos) = host_port.rfind(':') {
            let host = host_port[..colon_pos].to_string();
            let port_str = &host_port[colon_pos + 1..];
            let port = port_str
                .parse::<u16>()
                .map_err(|_| format!("Invalid port number: {port_str}"))?;
            (host, port)
        } else {
            (host_port.to_string(), default_port)
        };

        // Default host to localhost if empty
        let host = if host.is_empty() {
            "localhost".to_string()
        } else {
            host
        };

        Ok((host, port, username, password, database))
    }

    /// Validate the current input and create a connection config
    pub fn try_create_connection(&self) -> Result<ConnectionConfig, String> {
        // Validate required fields
        if self.name.trim().is_empty() {
            return Err("Connection name is required".to_string());
        }

        if self.using_connection_string {
            // Parse connection string
            if self.connection_string.trim().is_empty() {
                return Err("Connection string is required".to_string());
            }

            let (host, port, username, password, database) = self.parse_connection_string()?;

            // Create connection config from parsed string
            let mut connection = ConnectionConfig::new(
                self.name.trim().to_string(),
                self.database_type.clone(),
                host,
                port,
                username,
            );

            if let Some(pwd) = password {
                if !pwd.is_empty() {
                    connection.password = Some(pwd);
                }
            }

            if let Some(db) = database {
                if !db.is_empty() {
                    connection.database = Some(db);
                }
            }

            connection.ssl_mode = self.ssl_mode.clone();
            Ok(connection)
        } else {
            // Use individual fields
            if self.host.trim().is_empty() {
                return Err("Host is required".to_string());
            }

            if self.username.trim().is_empty() {
                return Err("Username is required".to_string());
            }

            // Parse port
            let port: u16 = if self.port_input.trim().is_empty()
                && self.database_type == DatabaseType::SQLite
            {
                0 // SQLite doesn't use ports
            } else {
                self.port_input
                    .trim()
                    .parse()
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
    }

    /// Clear all fields
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Populate modal state from existing connection for editing
    pub fn populate_from_connection(&mut self, connection: &ConnectionConfig) {
        self.name = connection.name.clone();
        self.database_type = connection.database_type.clone();
        self.host = connection.host.clone();
        self.port_input = connection.port.to_string();
        self.database = connection.database.as_deref().unwrap_or("").to_string();
        self.username = connection.username.clone();
        self.password = connection.password.as_deref().unwrap_or("").to_string();
        self.ssl_mode = connection.ssl_mode.clone();
        
        // Set up list state for database type
        let db_types = get_database_types();
        if let Some(index) = db_types.iter().position(|db| {
            db.to_lowercase() == connection.database_type.display_name()
        }) {
            self.db_type_list_state.select(Some(index));
        }
        
        // Set up SSL mode list state
        let ssl_modes = [
            SslMode::Disable,
            SslMode::Allow,
            SslMode::Prefer,
            SslMode::Require,
            SslMode::VerifyCA,
            SslMode::VerifyFull,
        ];
        if let Some(index) = ssl_modes.iter().position(|mode| {
            std::mem::discriminant(mode) == std::mem::discriminant(&connection.ssl_mode)
        }) {
            self.ssl_list_state.select(Some(index));
        }
        
        // Start in connection details step for editing
        self.current_step = ModalStep::ConnectionDetails;
        self.focused_field = ConnectionField::Name;
        self.error_message = None;
        self.using_connection_string = false;
        self.connection_string.clear();
    }
}

/// Render the connection creation modal
pub fn render_connection_modal(f: &mut Frame, modal_state: &ConnectionModalState, area: Rect) {
    // Create centered modal area with better proportions
    let modal_area = centered_rect(55, 75, area);

    // Clear the background
    f.render_widget(Clear, modal_area);

    // Main modal block with elegant styling
    let modal_block = Block::default()
        .title(" 🗄️  New Database Connection ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(116, 199, 236)) // LazyTables brand color
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(116, 199, 236)))
        .style(
            Style::default()
                .bg(Color::Rgb(13, 13, 13)) // Dark background
                .fg(Color::Rgb(255, 255, 255)),
        ); // White text

    f.render_widget(modal_block, modal_area);

    // Inner area for content with better margins
    let inner_area = modal_area.inner(Margin::new(3, 2));

    // Split into header, main content, and buttons
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header/instructions
            Constraint::Min(0),    // Form fields
            Constraint::Length(4), // Buttons area
        ])
        .split(inner_area);

    // Render header/instructions
    render_modal_header(f, main_chunks[0]);

    // Render different content based on current step
    match modal_state.current_step {
        ModalStep::DatabaseTypeSelection => {
            render_database_type_selection(f, modal_state, main_chunks[1]);
        }
        ModalStep::ConnectionDetails => {
            render_connection_details(f, modal_state, main_chunks[1]);
        }
    }

    // Render buttons and error area
    render_modal_footer(f, modal_state, main_chunks[2]);
}

/// Render the modal header with step-specific instructions
fn render_modal_header(f: &mut Frame, area: Rect) {
    let instructions = vec![
        Line::from(vec![
            Span::styled("Navigate: ", Style::default().fg(Color::Gray)),
            Span::styled(
                "j/k",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" or ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  •  Select: ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  •  Back: ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Esc",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    let header = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Rgb(205, 214, 244)))
        .alignment(Alignment::Center);

    f.render_widget(header, area);
}

/// Render database type selection step
fn render_database_type_selection(f: &mut Frame, modal_state: &ConnectionModalState, area: Rect) {
    // Create layout for instructions and database type selector
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Instructions
            Constraint::Min(0),    // Database type selector
        ])
        .split(area);

    // Instructions
    let instructions = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Step 1 of 2: ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Choose your database type",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let instruction_paragraph = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Rgb(205, 214, 244)))
        .alignment(Alignment::Center);

    f.render_widget(instruction_paragraph, chunks[0]);

    // Database type selector (large and prominent)
    render_dropdown_field(
        f,
        "Database Type",
        &get_database_types(),
        modal_state.focused_field == ConnectionField::DatabaseType,
        &modal_state.db_type_list_state,
        chunks[1],
    );
}

/// Render connection details step
fn render_connection_details(f: &mut Frame, modal_state: &ConnectionModalState, area: Rect) {
    // Create layout for instructions and form fields
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Instructions
            Constraint::Min(0),    // Form fields
        ])
        .split(area);

    // Step instructions
    let db_name = match modal_state.database_type {
        DatabaseType::PostgreSQL => "PostgreSQL",
        DatabaseType::MySQL => "MySQL",
        DatabaseType::MariaDB => "MariaDB",
        DatabaseType::SQLite => "SQLite",
        _ => "Database",
    };

    let instructions = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Step 2 of 2: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("Configure {db_name} connection"),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Use ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Connection String",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" or fill ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Individual Fields",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let instruction_paragraph = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Rgb(205, 214, 244)))
        .alignment(Alignment::Center);

    f.render_widget(instruction_paragraph, chunks[0]);

    // Form fields
    render_form_fields(f, modal_state, chunks[1]);
}

/// Render the form fields
fn render_form_fields(f: &mut Frame, modal_state: &ConnectionModalState, area: Rect) {
    // Create layout: Connection name + Connection string at top, then individual fields if not using connection string
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Connection Name
            Constraint::Length(4), // Connection String (taller for better visibility)
            Constraint::Min(0),    // Individual fields or spacer
        ])
        .split(area);

    // Always show connection name
    render_text_field(
        f,
        "Connection Name",
        &modal_state.name,
        modal_state.focused_field == ConnectionField::Name,
        main_chunks[0],
    );

    // Connection string field with example
    let conn_string_label = format!(
        "Connection String ({})",
        get_connection_string_example(&modal_state.database_type)
    );
    let conn_string_focused = modal_state.focused_field == ConnectionField::ConnectionString;

    render_connection_string_field(
        f,
        &conn_string_label,
        &modal_state.connection_string,
        conn_string_focused,
        modal_state.using_connection_string,
        main_chunks[1],
    );

    // Show individual fields only if not using connection string
    if !modal_state.using_connection_string {
        render_individual_fields(f, modal_state, main_chunks[2]);
    } else {
        // Show a message when using connection string
        let message = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "✓ Using connection string",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "Clear the connection string to use individual fields",
                Style::default().fg(Color::Gray),
            )]),
        ];

        let message_paragraph = Paragraph::new(message)
            .style(Style::default().fg(Color::Rgb(205, 214, 244)))
            .alignment(Alignment::Center);

        f.render_widget(message_paragraph, main_chunks[2]);
    }
}

/// Render individual connection fields
fn render_individual_fields(f: &mut Frame, modal_state: &ConnectionModalState, area: Rect) {
    // Split into two columns for better layout
    let main_columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left column fields
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
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
    render_text_field(
        f,
        "Host",
        &modal_state.host,
        modal_state.focused_field == ConnectionField::Host,
        left_chunks[0],
    );
    render_text_field(
        f,
        "Port",
        &modal_state.port_input,
        modal_state.focused_field == ConnectionField::Port,
        left_chunks[1],
    );

    // Render right column fields
    render_text_field(
        f,
        "Database (Optional)",
        &modal_state.database,
        modal_state.focused_field == ConnectionField::Database,
        right_chunks[0],
    );
    render_text_field(
        f,
        "Username",
        &modal_state.username,
        modal_state.focused_field == ConnectionField::Username,
        right_chunks[1],
    );
    render_password_field(
        f,
        "Password (Optional)",
        &modal_state.password,
        modal_state.focused_field == ConnectionField::Password,
        right_chunks[2],
    );
    render_dropdown_field(
        f,
        "SSL Mode",
        &get_ssl_modes(),
        modal_state.focused_field == ConnectionField::SslMode,
        &modal_state.ssl_list_state,
        right_chunks[3],
    );
}

/// Render connection string field with special styling
fn render_connection_string_field(
    f: &mut Frame,
    label: &str,
    value: &str,
    focused: bool,
    using_connection_string: bool,
    area: Rect,
) {
    let (border_style, title_style) = if focused {
        (
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    } else if using_connection_string {
        (
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            Style::default().fg(Color::Rgb(69, 71, 90)),
            Style::default().fg(Color::Gray),
        )
    };

    let block = Block::default()
        .title(format!(" {label} "))
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style);

    let display_value = if focused && !value.is_empty() {
        format!("{value}│") // Add cursor indicator
    } else if focused {
        "│".to_string() // Just cursor when empty
    } else if using_connection_string {
        format!("✓ {value}") // Show checkmark when using connection string
    } else {
        value.to_string()
    };

    let paragraph = Paragraph::new(display_value)
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

/// Get connection string example for database type
fn get_connection_string_example(db_type: &DatabaseType) -> &'static str {
    match db_type {
        DatabaseType::PostgreSQL => "postgresql://user:pass@host:5432/db",
        DatabaseType::MySQL | DatabaseType::MariaDB => "mysql://user:pass@host:3306/db",
        DatabaseType::SQLite => "sqlite:///path/to/database.db",
        _ => "scheme://user:pass@host:port/db",
    }
}

/// Render a text input field
fn render_text_field(f: &mut Frame, label: &str, value: &str, focused: bool, area: Rect) {
    let (border_style, title_style) = if focused {
        (
            Style::default()
                .fg(Color::Rgb(116, 199, 236))
                .add_modifier(Modifier::BOLD),
            Style::default()
                .fg(Color::Rgb(116, 199, 236))
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            Style::default().fg(Color::Rgb(69, 71, 90)),
            Style::default().fg(Color::Gray),
        )
    };

    let block = Block::default()
        .title(format!(" {label} "))
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style);

    let display_value = if focused && !value.is_empty() {
        format!("{value}│") // Add cursor indicator
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
            Style::default()
                .fg(Color::Rgb(116, 199, 236))
                .add_modifier(Modifier::BOLD),
            Style::default()
                .fg(Color::Rgb(116, 199, 236))
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            Style::default().fg(Color::Rgb(69, 71, 90)),
            Style::default().fg(Color::Gray),
        )
    };

    let block = Block::default()
        .title(format!(" {label} "))
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(masked_value)
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

/// Render a dropdown field
fn render_dropdown_field(
    f: &mut Frame,
    label: &str,
    items: &[String],
    focused: bool,
    list_state: &ListState,
    area: Rect,
) {
    let (border_style, title_style) = if focused {
        (
            Style::default()
                .fg(Color::Rgb(116, 199, 236))
                .add_modifier(Modifier::BOLD),
            Style::default()
                .fg(Color::Rgb(116, 199, 236))
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            Style::default().fg(Color::Rgb(69, 71, 90)),
            Style::default().fg(Color::Gray),
        )
    };

    let block = Block::default()
        .title(format!(" {label} "))
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
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(116, 199, 236))
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

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

    // Render buttons based on current step
    match modal_state.current_step {
        ModalStep::DatabaseTypeSelection => {
            render_database_selection_buttons(f, modal_state, footer_chunks[0]);
        }
        ModalStep::ConnectionDetails => {
            render_connection_details_buttons(f, modal_state, footer_chunks[0]);
        }
    }

    // Render error message if present
    if let Some(error) = &modal_state.error_message {
        let error_paragraph = Paragraph::new(format!("❗ {error}"))
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(error_paragraph, footer_chunks[1]);
    }
}

/// Render buttons for database type selection step
fn render_database_selection_buttons(
    f: &mut Frame,
    modal_state: &ConnectionModalState,
    area: Rect,
) {
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ])
        .split(area);

    let next_style = if modal_state.focused_field == ConnectionField::Save {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    };

    let cancel_style = if modal_state.focused_field == ConnectionField::Cancel {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    };

    let next_button = Paragraph::new("▶️ Next Step (s)")
        .style(next_style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(
            if modal_state.focused_field == ConnectionField::Save {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::Rgb(69, 71, 90))
            },
        ));

    let cancel_button = Paragraph::new("❌ Cancel (c)")
        .style(cancel_style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(
            if modal_state.focused_field == ConnectionField::Cancel {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Rgb(69, 71, 90))
            },
        ));

    f.render_widget(next_button, button_chunks[0]);
    f.render_widget(cancel_button, button_chunks[2]);
}

/// Render buttons for connection details step
fn render_connection_details_buttons(
    f: &mut Frame,
    modal_state: &ConnectionModalState,
    area: Rect,
) {
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(10),
            Constraint::Percentage(30),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
        ])
        .split(area);

    let save_style = if modal_state.focused_field == ConnectionField::Save {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    };

    let cancel_style = if modal_state.focused_field == ConnectionField::Cancel {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    };

    let save_button = Paragraph::new("💾 Save (s)")
        .style(save_style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(
            if modal_state.focused_field == ConnectionField::Save {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Rgb(69, 71, 90))
            },
        ));

    let cancel_button = Paragraph::new("❌ Cancel (c)")
        .style(cancel_style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(
            if modal_state.focused_field == ConnectionField::Cancel {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Rgb(69, 71, 90))
            },
        ));

    let back_button = Paragraph::new("◀️ Back (b)")
        .style(
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(69, 71, 90))),
        );

    f.render_widget(save_button, button_chunks[0]);
    f.render_widget(cancel_button, button_chunks[2]);
    f.render_widget(back_button, button_chunks[4]);
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
