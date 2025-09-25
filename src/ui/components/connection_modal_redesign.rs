// FilePath: src/ui/components/connection_modal_redesign.rs

use crate::database::connection::{ConnectionConfig, DatabaseType, SslMode};
use crate::security::PasswordSource;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Modal operation mode
#[derive(Debug, Clone)]
pub enum ModalMode {
    Add,
    Edit(String), // Connection ID
}

/// Form step in the modal flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormStep {
    DatabaseType,
    ConnectionDetails,
}

/// Connection input method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMethod {
    ConnectionString,
    IndividualFields,
}

/// Password configuration
#[derive(Debug, Clone)]
pub enum PasswordConfig {
    None,
    PlainText(String),
    Environment(String),        // env var name
    Encrypted(String, String),  // password, encryption key
}

/// Currently focused form field
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    DatabaseType,
    Name,
    ConnectionString,
    Host,
    Port,
    Database,
    Username,
    PasswordType,
    PasswordValue,
    PasswordEnvVar,
    EncryptionKey,
    SslMode,
    TestConnection,
    Save,
    Cancel,
}

/// Connection form data
#[derive(Debug, Clone)]
pub struct ConnectionForm {
    pub name: String,
    pub database_type: DatabaseType,
    pub connection_method: ConnectionMethod,
    pub connection_string: String,
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password_config: PasswordConfig,
    pub ssl_mode: SslMode,
}

impl Default for ConnectionForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            database_type: DatabaseType::PostgreSQL,
            connection_method: ConnectionMethod::IndividualFields,
            connection_string: String::new(),
            host: "localhost".to_string(),
            port: "5432".to_string(),
            database: String::new(),
            username: String::new(),
            password_config: PasswordConfig::None,
            ssl_mode: SslMode::Prefer,
        }
    }
}

impl ConnectionForm {
    /// Update port when database type changes
    pub fn update_port_for_db_type(&mut self) {
        self.port = match self.database_type {
            DatabaseType::PostgreSQL => "5432".to_string(),
            DatabaseType::MySQL | DatabaseType::MariaDB => "3306".to_string(),
            DatabaseType::SQLite => "".to_string(),
            _ => "5432".to_string(),
        };
    }

    /// Switch to connection string mode and clear individual fields
    pub fn switch_to_connection_string(&mut self) {
        self.connection_method = ConnectionMethod::ConnectionString;
        self.clear_individual_fields();
    }

    /// Switch to individual fields mode and clear connection string
    pub fn switch_to_individual_fields(&mut self) {
        self.connection_method = ConnectionMethod::IndividualFields;
        self.connection_string.clear();
    }

    /// Clear individual connection fields
    fn clear_individual_fields(&mut self) {
        self.host = "localhost".to_string();
        self.update_port_for_db_type();
        self.database.clear();
        self.username.clear();
        self.password_config = PasswordConfig::None;
    }

    /// Validate form and create connection config
    pub fn to_connection_config(&self) -> Result<ConnectionConfig, String> {
        if self.name.trim().is_empty() {
            return Err("Connection name is required".to_string());
        }

        let mut connection = match self.connection_method {
            ConnectionMethod::ConnectionString => {
                if self.connection_string.trim().is_empty() {
                    return Err("Connection string is required".to_string());
                }
                // Parse connection string and create config
                self.parse_connection_string()?
            }
            ConnectionMethod::IndividualFields => {
                if self.host.trim().is_empty() {
                    return Err("Host is required".to_string());
                }
                if self.username.trim().is_empty() {
                    return Err("Username is required".to_string());
                }

                let port: u16 = if self.database_type == DatabaseType::SQLite {
                    0
                } else {
                    self.port.trim().parse()
                        .map_err(|_| "Invalid port number".to_string())?
                };

                let mut config = ConnectionConfig::new(
                    self.name.trim().to_string(),
                    self.database_type.clone(),
                    self.host.trim().to_string(),
                    port,
                    self.username.trim().to_string(),
                );

                if !self.database.trim().is_empty() {
                    config.database = Some(self.database.trim().to_string());
                }

                config
            }
        };

        // Set password source
        match &self.password_config {
            PasswordConfig::None => {}
            PasswordConfig::PlainText(password) => {
                if !password.trim().is_empty() {
                    connection.set_plain_password(password.trim().to_string());
                }
            }
            PasswordConfig::Environment(env_var) => {
                if !env_var.trim().is_empty() {
                    use crate::security::PasswordManager;
                    let source = PasswordManager::from_environment(env_var.trim().to_string());
                    connection.set_password_source(source);
                }
            }
            PasswordConfig::Encrypted(password, key) => {
                if !password.trim().is_empty() && !key.trim().is_empty() {
                    use crate::security::PasswordManager;
                    let source = PasswordManager::create_encrypted(password.trim(), key.trim(), None)
                        .map_err(|e| format!("Failed to encrypt password: {e}"))?;
                    connection.set_password_source(source);
                }
            }
        }

        connection.ssl_mode = self.ssl_mode.clone();
        Ok(connection)
    }

    /// Parse connection string (simplified)
    fn parse_connection_string(&self) -> Result<ConnectionConfig, String> {
        // For now, return error - this would need proper URI parsing
        Err("Connection string parsing not yet implemented".to_string())
    }

    /// Load from existing connection for editing
    pub fn from_connection_config(config: &ConnectionConfig) -> Self {
        let mut form = Self {
            name: config.name.clone(),
            database_type: config.database_type.clone(),
            connection_method: ConnectionMethod::IndividualFields,
            connection_string: String::new(),
            host: config.host.clone(),
            port: config.port.to_string(),
            database: config.database.as_deref().unwrap_or("").to_string(),
            username: config.username.clone(),
            password_config: PasswordConfig::None,
            ssl_mode: config.ssl_mode.clone(),
        };

        // Set password config based on source
        form.password_config = if let Some(ref password_source) = config.password_source {
            match password_source {
                PasswordSource::PlainText(password) => PasswordConfig::PlainText(password.clone()),
                PasswordSource::Environment { var_name } => PasswordConfig::Environment(var_name.clone()),
                PasswordSource::Encrypted(_) => PasswordConfig::Encrypted(String::new(), String::new()),
            }
        } else if let Some(ref legacy_password) = config.password {
            PasswordConfig::PlainText(legacy_password.clone())
        } else {
            PasswordConfig::None
        };

        form
    }
}

/// Simplified connection modal state
#[derive(Debug, Clone)]
pub struct ConnectionModalState {
    pub mode: ModalMode,
    pub step: FormStep,
    pub focused_field: FormField,
    pub form: ConnectionForm,
    pub errors: Vec<String>,
    pub test_status: Option<TestStatus>,

    // UI state
    pub database_type_list_state: ListState,
    pub ssl_mode_list_state: ListState,
    pub is_insert_mode: bool,
}

#[derive(Debug, Clone)]
pub enum TestStatus {
    Testing,
    Success(String),
    Failed(String),
}

impl ConnectionModalState {
    pub fn new_add() -> Self {
        let mut db_list_state = ListState::default();
        db_list_state.select(Some(0));

        let mut ssl_list_state = ListState::default();
        ssl_list_state.select(Some(2)); // Prefer

        Self {
            mode: ModalMode::Add,
            step: FormStep::DatabaseType,
            focused_field: FormField::DatabaseType,
            form: ConnectionForm::default(),
            errors: Vec::new(),
            test_status: None,
            database_type_list_state: db_list_state,
            ssl_mode_list_state: ssl_list_state,
            is_insert_mode: false,
        }
    }

    pub fn new_edit(connection: &ConnectionConfig) -> Self {
        let mut state = Self::new_add();
        state.mode = ModalMode::Edit(connection.id.clone());
        state.form = ConnectionForm::from_connection_config(connection);
        state.step = FormStep::ConnectionDetails;
        state.focused_field = FormField::Name;
        state.setup_database_type_selection();
        state.setup_ssl_mode_selection();
        state
    }

    /// Set up database type list selection
    fn setup_database_type_selection(&mut self) {
        let db_types = [
            DatabaseType::PostgreSQL,
            DatabaseType::MySQL,
            DatabaseType::MariaDB,
            DatabaseType::SQLite,
        ];

        if let Some(index) = db_types.iter().position(|db_type| {
            std::mem::discriminant(db_type) == std::mem::discriminant(&self.form.database_type)
        }) {
            self.database_type_list_state.select(Some(index));
        }
    }

    /// Set up SSL mode list selection
    fn setup_ssl_mode_selection(&mut self) {
        let ssl_modes = [
            SslMode::Disable,
            SslMode::Allow,
            SslMode::Prefer,
            SslMode::Require,
            SslMode::VerifyCA,
            SslMode::VerifyFull,
        ];

        if let Some(index) = ssl_modes.iter().position(|mode| {
            std::mem::discriminant(mode) == std::mem::discriminant(&self.form.ssl_mode)
        }) {
            self.ssl_mode_list_state.select(Some(index));
        }
    }

    /// Move to next field
    pub fn next_field(&mut self) {
        self.focused_field = match self.step {
            FormStep::DatabaseType => match self.focused_field {
                FormField::DatabaseType => FormField::Save,
                FormField::Save => FormField::Cancel,
                _ => FormField::DatabaseType,
            },
            FormStep::ConnectionDetails => match self.form.connection_method {
                ConnectionMethod::ConnectionString => match self.focused_field {
                    FormField::Name => FormField::ConnectionString,
                    FormField::ConnectionString => FormField::SslMode,
                    FormField::SslMode => FormField::TestConnection,
                    FormField::TestConnection => FormField::Save,
                    FormField::Save => FormField::Cancel,
                    _ => FormField::Name,
                },
                ConnectionMethod::IndividualFields => match self.focused_field {
                    FormField::Name => FormField::Host,
                    FormField::Host => FormField::Port,
                    FormField::Port => FormField::Database,
                    FormField::Database => FormField::Username,
                    FormField::Username => FormField::PasswordType,
                    FormField::PasswordType => self.next_password_field(),
                    FormField::PasswordValue | FormField::PasswordEnvVar | FormField::EncryptionKey => FormField::SslMode,
                    FormField::SslMode => FormField::TestConnection,
                    FormField::TestConnection => FormField::Save,
                    FormField::Save => FormField::Cancel,
                    _ => FormField::Name,
                },
            },
        };
    }

    /// Get next password field based on password config type
    fn next_password_field(&self) -> FormField {
        match self.form.password_config {
            PasswordConfig::None => FormField::SslMode,
            PasswordConfig::PlainText(_) => FormField::PasswordValue,
            PasswordConfig::Environment(_) => FormField::PasswordEnvVar,
            PasswordConfig::Encrypted(_, _) => FormField::PasswordValue,
        }
    }

    /// Move to previous field
    pub fn previous_field(&mut self) {
        // Implementation would be similar to next_field but in reverse
        // Simplified for now
        self.focused_field = match self.focused_field {
            FormField::Save => FormField::TestConnection,
            FormField::Cancel => FormField::Save,
            _ => FormField::Name,
        };
    }

    /// Advance to next step
    pub fn advance_step(&mut self) {
        if self.step == FormStep::DatabaseType {
            self.step = FormStep::ConnectionDetails;
            self.focused_field = FormField::Name;
        }
    }

    /// Go back to previous step
    pub fn go_back(&mut self) {
        if self.step == FormStep::ConnectionDetails {
            self.step = FormStep::DatabaseType;
            self.focused_field = FormField::DatabaseType;
        }
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) {
        if !self.is_insert_mode {
            return;
        }

        match self.focused_field {
            FormField::Name => self.form.name.push(ch),
            FormField::ConnectionString => {
                self.form.connection_string.push(ch);
                if !self.form.connection_string.is_empty() {
                    self.form.switch_to_connection_string();
                }
            }
            FormField::Host => self.form.host.push(ch),
            FormField::Port => {
                if ch.is_ascii_digit() {
                    self.form.port.push(ch);
                }
            }
            FormField::Database => self.form.database.push(ch),
            FormField::Username => self.form.username.push(ch),
            FormField::PasswordValue => {
                match &mut self.form.password_config {
                    PasswordConfig::PlainText(ref mut pw) => pw.push(ch),
                    PasswordConfig::Encrypted(ref mut pw, _) => pw.push(ch),
                    _ => {
                        self.form.password_config = PasswordConfig::PlainText(ch.to_string());
                    }
                }
            }
            FormField::PasswordEnvVar => {
                if let PasswordConfig::Environment(ref mut var) = self.form.password_config {
                    var.push(ch);
                } else {
                    self.form.password_config = PasswordConfig::Environment(ch.to_string());
                }
            }
            FormField::EncryptionKey => {
                if let PasswordConfig::Encrypted(_, ref mut key) = self.form.password_config {
                    key.push(ch);
                }
            }
            _ => {}
        }
        self.clear_errors();
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        if !self.is_insert_mode {
            return;
        }

        match self.focused_field {
            FormField::Name => { self.form.name.pop(); }
            FormField::ConnectionString => {
                self.form.connection_string.pop();
                if self.form.connection_string.is_empty() {
                    self.form.switch_to_individual_fields();
                }
            }
            FormField::Host => { self.form.host.pop(); }
            FormField::Port => { self.form.port.pop(); }
            FormField::Database => { self.form.database.pop(); }
            FormField::Username => { self.form.username.pop(); }
            FormField::PasswordValue => {
                match &mut self.form.password_config {
                    PasswordConfig::PlainText(ref mut pw) => { pw.pop(); }
                    PasswordConfig::Encrypted(ref mut pw, _) => { pw.pop(); }
                    _ => {}
                }
            }
            FormField::PasswordEnvVar => {
                if let PasswordConfig::Environment(ref mut var) = self.form.password_config {
                    var.pop();
                }
            }
            FormField::EncryptionKey => {
                if let PasswordConfig::Encrypted(_, ref mut key) = self.form.password_config {
                    key.pop();
                }
            }
            _ => {}
        }
    }

    /// Toggle insert mode
    pub fn toggle_insert_mode(&mut self) {
        self.is_insert_mode = !self.is_insert_mode;
    }

    /// Enter insert mode for text fields
    pub fn enter_insert_mode(&mut self) {
        if self.is_text_field() {
            self.is_insert_mode = true;
        }
    }

    /// Exit insert mode
    pub fn exit_insert_mode(&mut self) {
        self.is_insert_mode = false;
    }

    /// Check if current field is a text input field
    pub fn is_text_field(&self) -> bool {
        matches!(
            self.focused_field,
            FormField::Name
                | FormField::ConnectionString
                | FormField::Host
                | FormField::Port
                | FormField::Database
                | FormField::Username
                | FormField::PasswordValue
                | FormField::PasswordEnvVar
                | FormField::EncryptionKey
        )
    }

    /// Clear errors
    pub fn clear_errors(&mut self) {
        self.errors.clear();
        self.test_status = None;
    }

    /// Add error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Create connection config from form
    pub fn create_connection(&mut self) -> Result<ConnectionConfig, String> {
        self.clear_errors();
        match self.form.to_connection_config() {
            Ok(config) => Ok(config),
            Err(e) => {
                self.add_error(e.clone());
                Err(e)
            }
        }
    }

    /// Select database type
    pub fn select_database_type(&mut self, index: usize) {
        let types = [
            DatabaseType::PostgreSQL,
            DatabaseType::MySQL,
            DatabaseType::MariaDB,
            DatabaseType::SQLite,
        ];

        if let Some(db_type) = types.get(index) {
            self.form.database_type = db_type.clone();
            self.form.update_port_for_db_type();
            self.database_type_list_state.select(Some(index));
        }
    }

    /// Select SSL mode
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
            self.form.ssl_mode = mode.clone();
            self.ssl_mode_list_state.select(Some(index));
        }
    }

    /// Cycle password config type
    pub fn cycle_password_type(&mut self) {
        self.form.password_config = match &self.form.password_config {
            PasswordConfig::None => PasswordConfig::PlainText(String::new()),
            PasswordConfig::PlainText(_) => PasswordConfig::Environment(String::new()),
            PasswordConfig::Environment(_) => PasswordConfig::Encrypted(String::new(), String::new()),
            PasswordConfig::Encrypted(_, _) => PasswordConfig::None,
        };
    }

    /// Get password type display name
    pub fn password_type_display(&self) -> &'static str {
        match self.form.password_config {
            PasswordConfig::None => "No Password",
            PasswordConfig::PlainText(_) => "Plain Text",
            PasswordConfig::Environment(_) => "Environment Variable",
            PasswordConfig::Encrypted(_, _) => "Encrypted",
        }
    }

    /// Get modal title based on mode
    pub fn title(&self) -> &'static str {
        match self.mode {
            ModalMode::Add => " 🗄️  New Database Connection ",
            ModalMode::Edit(_) => " ✏️  Edit Database Connection ",
        }
    }

    /// Check if in edit mode
    pub fn is_edit_mode(&self) -> bool {
        matches!(self.mode, ModalMode::Edit(_))
    }
}

/// Render the redesigned connection modal
pub fn render_connection_modal(f: &mut Frame, state: &ConnectionModalState, area: Rect) {
    // Render overlay
    f.render_widget(Clear, area);
    let overlay = Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0)));
    f.render_widget(overlay, area);

    // Center the modal
    let modal_area = centered_rect(80, 70, area);
    f.render_widget(Clear, modal_area);

    // Main modal block
    let modal_block = Block::default()
        .title(state.title())
        .title_style(
            Style::default()
                .fg(Color::Rgb(116, 199, 236))
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(116, 199, 236)))
        .style(Style::default().bg(Color::Rgb(13, 13, 13)).fg(Color::White));

    f.render_widget(modal_block, modal_area);

    let inner_area = modal_area.inner(Margin::new(2, 1));

    // Split into header, content, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer/errors
        ])
        .split(inner_area);

    // Render header
    render_header(f, state, chunks[0]);

    // Render content based on step
    match state.step {
        FormStep::DatabaseType => render_database_type_step(f, state, chunks[1]),
        FormStep::ConnectionDetails => render_connection_details_step(f, state, chunks[1]),
    }

    // Render footer
    render_footer(f, state, chunks[2]);
}

/// Render modal header
fn render_header(f: &mut Frame, state: &ConnectionModalState, area: Rect) {
    let step_text = match state.step {
        FormStep::DatabaseType => "Step 1 of 2: Choose Database Type",
        FormStep::ConnectionDetails => "Step 2 of 2: Connection Details",
    };

    let mode_indicator = if state.is_insert_mode {
        " [INSERT] "
    } else {
        " [NORMAL] "
    };

    let header_lines = vec![
        Line::from(vec![
            Span::styled(step_text, Style::default().fg(Color::Cyan)),
            Span::styled(mode_indicator, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("Tab/j/k: Navigate  •  Enter: Select/Next  •  i: Insert  •  Esc: Exit Insert/Back"),
    ];

    let paragraph = Paragraph::new(header_lines).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

/// Render database type selection step
fn render_database_type_step(f: &mut Frame, state: &ConnectionModalState, area: Rect) {
    let db_types = vec!["PostgreSQL".to_string(), "MySQL".to_string(), "MariaDB".to_string(), "SQLite".to_string()];
    render_list(f, "Database Type", &db_types, state.focused_field == FormField::DatabaseType, &state.database_type_list_state, area);
}

/// Render connection details step
fn render_connection_details_step(f: &mut Frame, state: &ConnectionModalState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Min(0),    // Connection fields
        ])
        .split(area);

    // Connection name (always shown)
    render_text_field(f, "Connection Name", &state.form.name, state.focused_field == FormField::Name, state.is_insert_mode, chunks[0]);

    // Connection method fields
    match state.form.connection_method {
        ConnectionMethod::ConnectionString => {
            render_text_field(f, "Connection String", &state.form.connection_string,
                state.focused_field == FormField::ConnectionString, state.is_insert_mode, chunks[1]);
        }
        ConnectionMethod::IndividualFields => {
            render_individual_fields(f, state, chunks[1]);
        }
    }
}

/// Render individual connection fields
fn render_individual_fields(f: &mut Frame, state: &ConnectionModalState, area: Rect) {
    let fields_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Host
            Constraint::Length(3), // Port
            Constraint::Length(3), // Database
            Constraint::Length(3), // Username
            Constraint::Length(3), // Password type
            Constraint::Min(0),    // Password value/SSL
        ])
        .split(area);

    render_text_field(f, "Host", &state.form.host, state.focused_field == FormField::Host, state.is_insert_mode, fields_chunks[0]);
    render_text_field(f, "Port", &state.form.port, state.focused_field == FormField::Port, state.is_insert_mode, fields_chunks[1]);
    render_text_field(f, "Database", &state.form.database, state.focused_field == FormField::Database, state.is_insert_mode, fields_chunks[2]);
    render_text_field(f, "Username", &state.form.username, state.focused_field == FormField::Username, state.is_insert_mode, fields_chunks[3]);

    // Password configuration
    render_password_config(f, state, fields_chunks[4]);
}

/// Render password configuration
fn render_password_config(f: &mut Frame, state: &ConnectionModalState, area: Rect) {
    let password_type = state.password_type_display();
    let focused = state.focused_field == FormField::PasswordType;
    let style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(" Password Configuration ")
        .borders(Borders::ALL)
        .border_style(style);

    let content = format!("{} {}", password_type, if focused { "[Space to change]" } else { "" });
    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

/// Render footer with errors/status
fn render_footer(f: &mut Frame, state: &ConnectionModalState, area: Rect) {
    if !state.errors.is_empty() {
        let error_text = state.errors.join(", ");
        let paragraph = Paragraph::new(format!("❌ {}", error_text))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    } else if let Some(test_status) = &state.test_status {
        let (text, style) = match test_status {
            TestStatus::Testing => ("🔄 Testing connection...".to_string(), Style::default().fg(Color::Yellow)),
            TestStatus::Success(msg) => (format!("✅ {}", msg), Style::default().fg(Color::Green)),
            TestStatus::Failed(msg) => (format!("❌ {}", msg), Style::default().fg(Color::Red)),
        };
        let paragraph = Paragraph::new(text).style(style).alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

/// Render a text input field
fn render_text_field(f: &mut Frame, label: &str, value: &str, focused: bool, insert_mode: bool, area: Rect) {
    let style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let display_value = if focused && insert_mode {
        format!("{}│", value)
    } else {
        value.to_string()
    };

    let block = Block::default()
        .title(format!(" {} ", label))
        .borders(Borders::ALL)
        .border_style(style);

    let paragraph = Paragraph::new(display_value)
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

/// Render a list widget
fn render_list(f: &mut Frame, title: &str, items: &[String], focused: bool, list_state: &ListState, area: Rect) {
    let style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let list_items: Vec<ListItem> = items.iter().map(|item| ListItem::new(item.as_str())).collect();

    let mut state = list_state.clone();
    let list = List::new(list_items)
        .block(Block::default().title(format!(" {} ", title)).borders(Borders::ALL).border_style(style))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, area, &mut state);
}

/// Create a centered rectangle
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