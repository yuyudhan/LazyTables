// FilePath: src/ui/components/table_creator.rs

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, ListState, Paragraph, Row, Table},
    Frame,
};
use serde::{Deserialize, Serialize};

/// PostgreSQL data types organized by category
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PostgresDataType {
    // Numeric Types
    SmallInt,
    Integer,
    BigInt,
    Decimal,
    Numeric,
    Real,
    DoublePrecision,
    SmallSerial,
    Serial,
    BigSerial,

    // Monetary Types
    Money,

    // Character Types
    Character(Option<u32>),        // char(n)
    CharacterVarying(Option<u32>), // varchar(n)
    Text,

    // Binary Data Types
    Bytea,

    // Date/Time Types
    Date,
    Time,
    TimeWithTimeZone,
    Timestamp,
    TimestampWithTimeZone,
    Interval,

    // Boolean Type
    Boolean,

    // Geometric Types
    Point,
    Line,
    Lseg,
    Box,
    Path,
    Polygon,
    Circle,

    // Network Address Types
    Cidr,
    Inet,
    MacAddr,
    MacAddr8,

    // Bit String Types
    Bit(Option<u32>),
    BitVarying(Option<u32>),

    // Text Search Types
    TsVector,
    TsQuery,

    // UUID Type
    Uuid,

    // XML Type
    Xml,

    // JSON Types
    Json,
    Jsonb,

    // Array Types
    Array(Box<PostgresDataType>),

    // Range Types
    Int4Range,
    Int8Range,
    NumRange,
    TsRange,
    TsTzRange,
    DateRange,

    // Other Types
    Custom(String),
}

impl PostgresDataType {
    /// Get the SQL type name for CREATE TABLE statements
    pub fn to_sql(&self) -> String {
        match self {
            Self::SmallInt => "SMALLINT".to_string(),
            Self::Integer => "INTEGER".to_string(),
            Self::BigInt => "BIGINT".to_string(),
            Self::Decimal => "DECIMAL".to_string(),
            Self::Numeric => "NUMERIC".to_string(),
            Self::Real => "REAL".to_string(),
            Self::DoublePrecision => "DOUBLE PRECISION".to_string(),
            Self::SmallSerial => "SMALLSERIAL".to_string(),
            Self::Serial => "SERIAL".to_string(),
            Self::BigSerial => "BIGSERIAL".to_string(),
            Self::Money => "MONEY".to_string(),
            Self::Character(n) => match n {
                Some(len) => format!("CHARACTER({len})"),
                None => "CHARACTER".to_string(),
            },
            Self::CharacterVarying(n) => match n {
                Some(len) => format!("CHARACTER VARYING({len})"),
                None => "CHARACTER VARYING".to_string(),
            },
            Self::Text => "TEXT".to_string(),
            Self::Bytea => "BYTEA".to_string(),
            Self::Date => "DATE".to_string(),
            Self::Time => "TIME".to_string(),
            Self::TimeWithTimeZone => "TIME WITH TIME ZONE".to_string(),
            Self::Timestamp => "TIMESTAMP".to_string(),
            Self::TimestampWithTimeZone => "TIMESTAMP WITH TIME ZONE".to_string(),
            Self::Interval => "INTERVAL".to_string(),
            Self::Boolean => "BOOLEAN".to_string(),
            Self::Point => "POINT".to_string(),
            Self::Line => "LINE".to_string(),
            Self::Lseg => "LSEG".to_string(),
            Self::Box => "BOX".to_string(),
            Self::Path => "PATH".to_string(),
            Self::Polygon => "POLYGON".to_string(),
            Self::Circle => "CIRCLE".to_string(),
            Self::Cidr => "CIDR".to_string(),
            Self::Inet => "INET".to_string(),
            Self::MacAddr => "MACADDR".to_string(),
            Self::MacAddr8 => "MACADDR8".to_string(),
            Self::Bit(n) => match n {
                Some(len) => format!("BIT({len})"),
                None => "BIT".to_string(),
            },
            Self::BitVarying(n) => match n {
                Some(len) => format!("BIT VARYING({len})"),
                None => "BIT VARYING".to_string(),
            },
            Self::TsVector => "TSVECTOR".to_string(),
            Self::TsQuery => "TSQUERY".to_string(),
            Self::Uuid => "UUID".to_string(),
            Self::Xml => "XML".to_string(),
            Self::Json => "JSON".to_string(),
            Self::Jsonb => "JSONB".to_string(),
            Self::Array(inner) => format!("{}[]", inner.to_sql()),
            Self::Int4Range => "INT4RANGE".to_string(),
            Self::Int8Range => "INT8RANGE".to_string(),
            Self::NumRange => "NUMRANGE".to_string(),
            Self::TsRange => "TSRANGE".to_string(),
            Self::TsTzRange => "TSTZRANGE".to_string(),
            Self::DateRange => "DATERANGE".to_string(),
            Self::Custom(name) => name.clone(),
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> String {
        match self {
            Self::SmallInt => "SMALLINT".to_string(),
            Self::Integer => "INTEGER".to_string(),
            Self::BigInt => "BIGINT".to_string(),
            Self::Serial => "SERIAL".to_string(),
            Self::BigSerial => "BIGSERIAL".to_string(),
            Self::CharacterVarying(_) => "VARCHAR".to_string(),
            Self::Character(_) => "CHAR".to_string(),
            Self::Text => "TEXT".to_string(),
            Self::Boolean => "BOOLEAN".to_string(),
            Self::Date => "DATE".to_string(),
            Self::Timestamp => "TIMESTAMP".to_string(),
            Self::TimestampWithTimeZone => "TIMESTAMPTZ".to_string(),
            Self::Json => "JSON".to_string(),
            Self::Jsonb => "JSONB".to_string(),
            Self::Uuid => "UUID".to_string(),
            _ => self.to_sql(),
        }
    }

    /// Get common data types for quick selection
    pub fn common_types() -> Vec<Self> {
        vec![
            Self::Integer,
            Self::BigInt,
            Self::Serial,
            Self::BigSerial,
            Self::Text,
            Self::CharacterVarying(Some(255)),
            Self::Boolean,
            Self::Date,
            Self::Timestamp,
            Self::TimestampWithTimeZone,
            Self::Json,
            Self::Jsonb,
            Self::Uuid,
            Self::Numeric,
            Self::Real,
            Self::DoublePrecision,
        ]
    }
}

/// Column definition for table creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: PostgresDataType,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub default_value: Option<String>,
    pub check_constraint: Option<String>,
    pub references: Option<ForeignKeyReference>,
}

impl ColumnDefinition {
    pub fn new(name: String) -> Self {
        Self {
            name,
            data_type: PostgresDataType::Integer,
            is_nullable: true,
            is_primary_key: false,
            is_unique: false,
            default_value: None,
            check_constraint: None,
            references: None,
        }
    }

    /// Generate SQL for this column definition
    pub fn to_sql(&self) -> String {
        let mut sql = format!("{} {}", self.name, self.data_type.to_sql());

        if !self.is_nullable {
            sql.push_str(" NOT NULL");
        }

        if self.is_primary_key {
            sql.push_str(" PRIMARY KEY");
        } else if self.is_unique {
            sql.push_str(" UNIQUE");
        }

        if let Some(ref default) = self.default_value {
            sql.push_str(&format!(" DEFAULT {default}"));
        }

        if let Some(ref check) = self.check_constraint {
            sql.push_str(&format!(" CHECK ({check})"));
        }

        if let Some(ref fk) = self.references {
            sql.push_str(&format!(
                " REFERENCES {}({})",
                fk.table_name, fk.column_name
            ));
            if let Some(ref on_delete) = fk.on_delete {
                sql.push_str(&format!(" ON DELETE {on_delete}"));
            }
            if let Some(ref on_update) = fk.on_update {
                sql.push_str(&format!(" ON UPDATE {on_update}"));
            }
        }

        sql
    }
}

/// Foreign key reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyReference {
    pub table_name: String,
    pub column_name: String,
    pub on_delete: Option<String>, // CASCADE, SET NULL, RESTRICT, etc.
    pub on_update: Option<String>,
}

/// Index definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDefinition {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub index_type: IndexType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexType {
    BTree,
    Hash,
    Gist,
    SpGist,
    Gin,
    Brin,
}

impl IndexType {
    pub fn to_sql(&self) -> &str {
        match self {
            Self::BTree => "BTREE",
            Self::Hash => "HASH",
            Self::Gist => "GIST",
            Self::SpGist => "SPGIST",
            Self::Gin => "GIN",
            Self::Brin => "BRIN",
        }
    }
}

/// Fields that can be focused in table creator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TableCreatorField {
    TableName,
    Column(usize, ColumnField),
    AddColumn,
    Save,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnField {
    Name,
    DataType,
    Length,
    Nullable,
    PrimaryKey,
    Unique,
    Default,
    Delete,
}

/// State for table creation
#[derive(Debug, Clone)]
pub struct TableCreatorState {
    pub table_name: String,
    pub columns: Vec<ColumnDefinition>,
    pub indexes: Vec<IndexDefinition>,
    pub focused_field: TableCreatorField,
    pub data_type_list_state: ListState,
    pub show_data_type_dropdown: bool,
    pub error_message: Option<String>,
    pub current_column_index: usize,
    pub in_insert_mode: bool,
}

impl TableCreatorState {
    pub fn new() -> Self {
        // Start with one default column
        let default_column = ColumnDefinition::new("id".to_string());

        Self {
            table_name: String::new(),
            columns: vec![default_column],
            indexes: Vec::new(),
            focused_field: TableCreatorField::TableName,
            data_type_list_state: ListState::default(),
            show_data_type_dropdown: false,
            error_message: None,
            current_column_index: 0,
            in_insert_mode: false,
        }
    }

    /// Check if current field is a text input field
    pub fn is_text_field(&self) -> bool {
        matches!(
            self.focused_field,
            TableCreatorField::TableName
                | TableCreatorField::Column(_, ColumnField::Name)
                | TableCreatorField::Column(_, ColumnField::Default)
                | TableCreatorField::Column(_, ColumnField::Length)
        )
    }

    /// Enter insert mode (for text fields)
    pub fn enter_insert_mode(&mut self) {
        if self.is_text_field() {
            self.in_insert_mode = true;
        }
    }

    /// Exit insert mode
    pub fn exit_insert_mode(&mut self) {
        self.in_insert_mode = false;
    }

    /// Add a new column
    pub fn add_column(&mut self) {
        let column_number = self.columns.len() + 1;
        let new_column = ColumnDefinition::new(format!("column_{column_number}"));
        self.columns.push(new_column);
        self.current_column_index = self.columns.len() - 1;
        self.focused_field =
            TableCreatorField::Column(self.current_column_index, ColumnField::Name);
    }

    /// Delete current column
    pub fn delete_current_column(&mut self) {
        if self.columns.len() > 1 {
            self.columns.remove(self.current_column_index);
            if self.current_column_index >= self.columns.len() {
                self.current_column_index = self.columns.len() - 1;
            }
        }
    }

    /// Navigate to next field
    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
            TableCreatorField::TableName => {
                if !self.columns.is_empty() {
                    TableCreatorField::Column(0, ColumnField::Name)
                } else {
                    TableCreatorField::AddColumn
                }
            }
            TableCreatorField::Column(idx, field) => match field {
                ColumnField::Name => TableCreatorField::Column(idx, ColumnField::DataType),
                ColumnField::DataType => TableCreatorField::Column(idx, ColumnField::Length),
                ColumnField::Length => TableCreatorField::Column(idx, ColumnField::Nullable),
                ColumnField::Nullable => TableCreatorField::Column(idx, ColumnField::PrimaryKey),
                ColumnField::PrimaryKey => TableCreatorField::Column(idx, ColumnField::Unique),
                ColumnField::Unique => TableCreatorField::Column(idx, ColumnField::Default),
                ColumnField::Default => TableCreatorField::Column(idx, ColumnField::Delete),
                ColumnField::Delete => {
                    if idx + 1 < self.columns.len() {
                        TableCreatorField::Column(idx + 1, ColumnField::Name)
                    } else {
                        TableCreatorField::AddColumn
                    }
                }
            },
            TableCreatorField::AddColumn => TableCreatorField::Save,
            TableCreatorField::Save => TableCreatorField::Cancel,
            TableCreatorField::Cancel => TableCreatorField::TableName,
        };
    }

    /// Navigate to previous field
    pub fn previous_field(&mut self) {
        self.focused_field = match self.focused_field {
            TableCreatorField::TableName => TableCreatorField::Cancel,
            TableCreatorField::Column(idx, field) => match field {
                ColumnField::Name => {
                    if idx > 0 {
                        TableCreatorField::Column(idx - 1, ColumnField::Delete)
                    } else {
                        TableCreatorField::TableName
                    }
                }
                ColumnField::DataType => TableCreatorField::Column(idx, ColumnField::Name),
                ColumnField::Length => TableCreatorField::Column(idx, ColumnField::DataType),
                ColumnField::Nullable => TableCreatorField::Column(idx, ColumnField::Length),
                ColumnField::PrimaryKey => TableCreatorField::Column(idx, ColumnField::Nullable),
                ColumnField::Unique => TableCreatorField::Column(idx, ColumnField::PrimaryKey),
                ColumnField::Default => TableCreatorField::Column(idx, ColumnField::Unique),
                ColumnField::Delete => TableCreatorField::Column(idx, ColumnField::Default),
            },
            TableCreatorField::AddColumn => {
                if !self.columns.is_empty() {
                    TableCreatorField::Column(self.columns.len() - 1, ColumnField::Delete)
                } else {
                    TableCreatorField::TableName
                }
            }
            TableCreatorField::Save => TableCreatorField::AddColumn,
            TableCreatorField::Cancel => TableCreatorField::Save,
        };
    }

    /// Handle character input
    pub fn handle_char_input(&mut self, c: char) {
        // Only allow text input in insert mode
        if !self.in_insert_mode {
            return;
        }

        self.error_message = None;

        match self.focused_field {
            TableCreatorField::TableName => {
                self.table_name.push(c);
            }
            TableCreatorField::Column(idx, ColumnField::Name) => {
                if let Some(column) = self.columns.get_mut(idx) {
                    column.name.push(c);
                }
            }
            TableCreatorField::Column(idx, ColumnField::Default) => {
                if let Some(column) = self.columns.get_mut(idx) {
                    if column.default_value.is_none() {
                        column.default_value = Some(String::new());
                    }
                    if let Some(ref mut default) = column.default_value {
                        default.push(c);
                    }
                }
            }
            TableCreatorField::Column(idx, ColumnField::Length) => {
                if c.is_ascii_digit() {
                    if let Some(column) = self.columns.get_mut(idx) {
                        match &mut column.data_type {
                            PostgresDataType::Character(ref mut len)
                            | PostgresDataType::CharacterVarying(ref mut len) => {
                                if let Some(l) = len {
                                    let new_len = format!("{l}{c}");
                                    if let Ok(parsed) = new_len.parse::<u32>() {
                                        *len = Some(parsed);
                                    }
                                } else if let Ok(parsed) = c.to_string().parse::<u32>() {
                                    *len = Some(parsed);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        // Only allow backspace in insert mode
        if !self.in_insert_mode {
            return;
        }
        match self.focused_field {
            TableCreatorField::TableName => {
                self.table_name.pop();
            }
            TableCreatorField::Column(idx, ColumnField::Name) => {
                if let Some(column) = self.columns.get_mut(idx) {
                    column.name.pop();
                }
            }
            TableCreatorField::Column(idx, ColumnField::Default) => {
                if let Some(column) = self.columns.get_mut(idx) {
                    if let Some(ref mut default) = column.default_value {
                        if default.is_empty() {
                            column.default_value = None;
                        } else {
                            default.pop();
                        }
                    }
                }
            }
            TableCreatorField::Column(idx, ColumnField::Length) => {
                if let Some(column) = self.columns.get_mut(idx) {
                    match &mut column.data_type {
                        PostgresDataType::Character(ref mut len)
                        | PostgresDataType::CharacterVarying(ref mut len) => {
                            if let Some(l) = len {
                                let current = l.to_string();
                                if current.len() > 1 {
                                    let new_str = &current[..current.len() - 1];
                                    if let Ok(parsed) = new_str.parse::<u32>() {
                                        *len = Some(parsed);
                                    }
                                } else {
                                    *len = None;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    /// Toggle boolean fields
    pub fn toggle_boolean_field(&mut self) {
        match self.focused_field {
            TableCreatorField::Column(idx, ColumnField::Nullable) => {
                if let Some(column) = self.columns.get_mut(idx) {
                    column.is_nullable = !column.is_nullable;
                }
            }
            TableCreatorField::Column(idx, ColumnField::PrimaryKey) => {
                if let Some(column) = self.columns.get_mut(idx) {
                    column.is_primary_key = !column.is_primary_key;
                    // If setting as primary key, make it NOT NULL and remove nullable
                    if column.is_primary_key {
                        column.is_nullable = false;
                    }
                }
            }
            TableCreatorField::Column(idx, ColumnField::Unique) => {
                if let Some(column) = self.columns.get_mut(idx) {
                    column.is_unique = !column.is_unique;
                }
            }
            _ => {}
        }
    }

    /// Generate CREATE TABLE SQL
    pub fn generate_create_table_sql(&self) -> Result<String, String> {
        if self.table_name.is_empty() {
            return Err("Table name is required".to_string());
        }

        if self.columns.is_empty() {
            return Err("At least one column is required".to_string());
        }

        // Check for empty column names
        for (idx, column) in self.columns.iter().enumerate() {
            if column.name.is_empty() {
                return Err(format!("Column {} name is required", idx + 1));
            }
        }

        let mut sql = format!("CREATE TABLE {} (\n", self.table_name);

        let column_definitions: Vec<String> = self
            .columns
            .iter()
            .map(|col| format!("    {}", col.to_sql()))
            .collect();

        sql.push_str(&column_definitions.join(",\n"));
        sql.push_str("\n);");

        // Add index creation statements
        for index in &self.indexes {
            sql.push_str(&format!(
                "\n\nCREATE {} INDEX {} ON {} USING {} ({});",
                if index.is_unique { "UNIQUE" } else { "" },
                index.name,
                self.table_name,
                index.index_type.to_sql(),
                index.columns.join(", ")
            ));
        }

        Ok(sql)
    }

    /// Clear all fields
    pub fn clear(&mut self) {
        self.table_name.clear();
        self.columns = vec![ColumnDefinition::new("id".to_string())];
        self.indexes.clear();
        self.focused_field = TableCreatorField::TableName;
        self.error_message = None;
        self.current_column_index = 0;
        self.in_insert_mode = false;
    }
}

/// Render the table creator view
pub fn render_table_creator(f: &mut Frame, state: &mut TableCreatorState, area: Rect) {
    // Main block
    let block = Block::default()
        .title(" 🗂️  Create New Table ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(block, area);

    let inner = area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    // Layout: Table name at top, columns in middle, actions at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Table name
            Constraint::Min(10),   // Columns
            Constraint::Length(3), // Actions
        ])
        .split(inner);

    // Render table name input
    render_table_name_input(f, state, chunks[0]);

    // Render columns in transposed view
    render_columns_table(f, state, chunks[1]);

    // Render actions
    render_actions(f, state, chunks[2]);

    // Render error message if any
    if let Some(ref error) = state.error_message {
        let error_text = vec![Line::from(vec![
            Span::styled("❌ ", Style::default().fg(Color::Red)),
            Span::raw(error),
        ])];

        let error_paragraph = Paragraph::new(error_text)
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);

        let error_area = Rect {
            x: area.x,
            y: area.y + area.height - 2,
            width: area.width,
            height: 1,
        };

        f.render_widget(error_paragraph, error_area);
    }
}

impl Default for TableCreatorState {
    fn default() -> Self {
        Self::new()
    }
}

fn render_table_name_input(f: &mut Frame, state: &TableCreatorState, area: Rect) {
    let is_focused = matches!(state.focused_field, TableCreatorField::TableName);

    let style = if is_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let title = if is_focused && state.in_insert_mode {
        "Table Name [INSERT]"
    } else if is_focused {
        "Table Name [Press 'i' to edit]"
    } else {
        "Table Name"
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .title(title);

    let input_text = if state.table_name.is_empty() && !is_focused {
        Paragraph::new("Enter table name...").style(Style::default().fg(Color::DarkGray))
    } else {
        let text = if is_focused && state.in_insert_mode {
            format!("{}▌", state.table_name) // Show cursor in insert mode
        } else {
            state.table_name.clone()
        };
        Paragraph::new(text).style(Style::default().fg(Color::White))
    };

    f.render_widget(input_block, area);
    f.render_widget(
        input_text,
        area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
    );
}

fn render_columns_table(f: &mut Frame, state: &mut TableCreatorState, area: Rect) {
    // Headers for the transposed table
    let headers = vec![
        "Column Name",
        "Data Type",
        "Length",
        "Nullable",
        "Primary Key",
        "Unique",
        "Default",
        "Actions",
    ];

    let header_row = Row::new(headers)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

    // Create rows for each column
    let rows: Vec<Row> = state
        .columns
        .iter()
        .enumerate()
        .map(|(idx, column)| {
            let is_current_column =
                if let TableCreatorField::Column(col_idx, _) = state.focused_field {
                    col_idx == idx
                } else {
                    false
                };

            let cells = vec![
                // Column Name
                if is_current_column
                    && matches!(
                        state.focused_field,
                        TableCreatorField::Column(_, ColumnField::Name)
                    )
                {
                    if state.in_insert_mode {
                        format!("▶ {}▌", column.name)
                    } else {
                        format!("▶ {}", column.name)
                    }
                } else {
                    column.name.clone()
                },
                // Data Type
                column.data_type.display_name(),
                // Length
                match &column.data_type {
                    PostgresDataType::Character(len) | PostgresDataType::CharacterVarying(len) => {
                        len.map_or("-".to_string(), |l| l.to_string())
                    }
                    _ => "-".to_string(),
                },
                // Nullable
                if column.is_nullable { "✓" } else { "✗" }.to_string(),
                // Primary Key
                if column.is_primary_key { "✓" } else { "✗" }.to_string(),
                // Unique
                if column.is_unique { "✓" } else { "✗" }.to_string(),
                // Default
                column.default_value.as_deref().unwrap_or("-").to_string(),
                // Actions
                "[d]elete".to_string(),
            ];

            let style = if is_current_column {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            Row::new(cells).style(style).height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(20), // Column Name
        Constraint::Length(15), // Data Type
        Constraint::Length(8),  // Length
        Constraint::Length(10), // Nullable
        Constraint::Length(12), // Primary Key
        Constraint::Length(8),  // Unique
        Constraint::Length(15), // Default
        Constraint::Length(10), // Actions
    ];

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Columns")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_widget(table, area);
}

fn render_actions(f: &mut Frame, state: &TableCreatorState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Add Column button
    let add_style = if matches!(state.focused_field, TableCreatorField::AddColumn) {
        Style::default().fg(Color::Black).bg(Color::Green)
    } else {
        Style::default().fg(Color::Green)
    };

    let add_button = Paragraph::new("[a] Add Column")
        .style(add_style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(add_button, chunks[1]);

    // Save button
    let save_style = if matches!(state.focused_field, TableCreatorField::Save) {
        Style::default().fg(Color::Black).bg(Color::Green)
    } else {
        Style::default().fg(Color::Green)
    };

    let save_button = Paragraph::new("[s] Save")
        .style(save_style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(save_button, chunks[2]);

    // Cancel button
    let cancel_style = if matches!(state.focused_field, TableCreatorField::Cancel) {
        Style::default().fg(Color::Black).bg(Color::Red)
    } else {
        Style::default().fg(Color::Red)
    };

    let cancel_button = Paragraph::new("[c] Cancel")
        .style(cancel_style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(cancel_button, chunks[3]);
}
