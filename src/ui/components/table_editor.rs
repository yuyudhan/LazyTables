// FilePath: src/ui/components/table_editor.rs

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, ListState, Paragraph, Row, Table},
    Frame,
};

use super::table_creator::{
    ColumnDefinition, ColumnField, IndexDefinition, PostgresDataType, TableCreatorField,
};

/// State for table editing
#[derive(Debug, Clone)]
pub struct TableEditorState {
    pub table_name: String,
    pub original_table_name: String,
    pub columns: Vec<ColumnDefinition>,
    pub original_columns: Vec<ColumnDefinition>,
    pub indexes: Vec<IndexDefinition>,
    pub focused_field: TableCreatorField,
    pub data_type_list_state: ListState,
    pub show_data_type_dropdown: bool,
    pub error_message: Option<String>,
    pub current_column_index: usize,
    pub in_insert_mode: bool,
    pub columns_to_drop: Vec<String>,
    pub columns_to_add: Vec<ColumnDefinition>,
    pub columns_to_modify: Vec<(String, ColumnDefinition)>,
}

impl TableEditorState {
    pub fn new(table_name: String) -> Self {
        Self {
            table_name: table_name.clone(),
            original_table_name: table_name,
            columns: Vec::new(),
            original_columns: Vec::new(),
            indexes: Vec::new(),
            focused_field: TableCreatorField::TableName,
            data_type_list_state: ListState::default(),
            show_data_type_dropdown: false,
            error_message: None,
            current_column_index: 0,
            in_insert_mode: false,
            columns_to_drop: Vec::new(),
            columns_to_add: Vec::new(),
            columns_to_modify: Vec::new(),
        }
    }

    /// Load table schema from database
    pub async fn load_table_schema(&mut self, table_name: &str) -> Result<(), String> {
        // This will be populated by querying the database for table structure
        // For now, we'll just set the table name
        self.table_name = table_name.to_string();
        self.original_table_name = table_name.to_string();
        Ok(())
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
        self.columns_to_add.push(new_column.clone());
        self.columns.push(new_column);
        self.current_column_index = self.columns.len() - 1;
        self.focused_field =
            TableCreatorField::Column(self.current_column_index, ColumnField::Name);
    }

    /// Mark column for deletion
    pub fn delete_current_column(&mut self) {
        if let TableCreatorField::Column(idx, _) = self.focused_field {
            if idx < self.columns.len() {
                let column = &self.columns[idx];

                // Check if this is an original column
                if self.original_columns.iter().any(|c| c.name == column.name) {
                    self.columns_to_drop.push(column.name.clone());
                } else {
                    // It's a newly added column, remove from add list
                    self.columns_to_add.retain(|c| c.name != column.name);
                }

                self.columns.remove(idx);

                if self.current_column_index >= self.columns.len() && !self.columns.is_empty() {
                    self.current_column_index = self.columns.len() - 1;
                }
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

    /// Generate ALTER TABLE SQL statements
    pub fn generate_alter_table_sql(&self) -> Result<Vec<String>, String> {
        let mut statements = Vec::new();

        // Rename table if needed
        if self.table_name != self.original_table_name {
            statements.push(format!(
                "ALTER TABLE {} RENAME TO {};",
                self.original_table_name, self.table_name
            ));
        }

        let table_name = &self.table_name;

        // Drop columns
        for column_name in &self.columns_to_drop {
            statements.push(format!(
                "ALTER TABLE {table_name} DROP COLUMN {column_name};"
            ));
        }

        // Add new columns
        for column in &self.columns_to_add {
            let column_sql = column.to_sql();
            statements.push(format!("ALTER TABLE {table_name} ADD COLUMN {column_sql};"));
        }

        // Modify existing columns
        for (original_name, column) in &self.columns_to_modify {
            // If column name changed
            if original_name != &column.name {
                let new_name = &column.name;
                statements.push(format!(
                    "ALTER TABLE {table_name} RENAME COLUMN {original_name} TO {new_name};"
                ));
            }

            let column_name = &column.name;
            let data_type = column.data_type.to_sql();

            // Alter column type if changed
            statements.push(format!(
                "ALTER TABLE {table_name} ALTER COLUMN {column_name} TYPE {data_type};"
            ));

            // Set NULL/NOT NULL
            if column.is_nullable {
                statements.push(format!(
                    "ALTER TABLE {table_name} ALTER COLUMN {column_name} DROP NOT NULL;"
                ));
            } else {
                statements.push(format!(
                    "ALTER TABLE {table_name} ALTER COLUMN {column_name} SET NOT NULL;"
                ));
            }

            // Set/drop default
            if let Some(ref default) = column.default_value {
                statements.push(format!(
                    "ALTER TABLE {table_name} ALTER COLUMN {column_name} SET DEFAULT {default};"
                ));
            } else {
                statements.push(format!(
                    "ALTER TABLE {table_name} ALTER COLUMN {column_name} DROP DEFAULT;"
                ));
            }
        }

        if statements.is_empty() {
            return Err("No changes to apply".to_string());
        }

        Ok(statements)
    }

    /// Clear all fields
    pub fn clear(&mut self) {
        self.table_name = self.original_table_name.clone();
        self.columns.clear();
        self.original_columns.clear();
        self.indexes.clear();
        self.focused_field = TableCreatorField::TableName;
        self.error_message = None;
        self.current_column_index = 0;
        self.in_insert_mode = false;
        self.columns_to_drop.clear();
        self.columns_to_add.clear();
        self.columns_to_modify.clear();
    }
}

impl Default for TableEditorState {
    fn default() -> Self {
        Self::new("table".to_string())
    }
}

/// Render the table editor view
pub fn render_table_editor(f: &mut Frame, state: &mut TableEditorState, area: Rect) {
    // Main block
    let block = Block::default()
        .title(" ✏️  Edit Table ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

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

fn render_table_name_input(f: &mut Frame, state: &TableEditorState, area: Rect) {
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

    let text = if is_focused && state.in_insert_mode {
        format!("{}▌", state.table_name)
    } else {
        state.table_name.clone()
    };

    let input_text = Paragraph::new(text).style(Style::default().fg(Color::White));

    f.render_widget(input_block, area);
    f.render_widget(
        input_text,
        area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
    );
}

fn render_columns_table(f: &mut Frame, state: &mut TableEditorState, area: Rect) {
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
            } else if state.columns_to_drop.contains(&column.name) {
                Style::default().fg(Color::Red).add_modifier(Modifier::DIM)
            } else if state.columns_to_add.iter().any(|c| c.name == column.name) {
                Style::default().fg(Color::Green)
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

fn render_actions(f: &mut Frame, state: &TableEditorState, area: Rect) {
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

    let save_button = Paragraph::new("[s] Save Changes")
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
