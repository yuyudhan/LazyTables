// FilePath: src/ui/components/table_viewer.rs

use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell as TableCell, Paragraph, Row, Table, Tabs, Wrap},
    Frame,
};
use std::collections::HashMap;

/// View mode for the table viewer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableViewMode {
    Data,
    Schema,
}

/// Represents a single table tab
#[derive(Debug, Clone)]
pub struct TableTab {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,
    pub current_page: usize,
    pub rows_per_page: usize,
    pub selected_row: usize,
    pub selected_col: usize,
    pub scroll_offset_x: usize,
    pub scroll_offset_y: usize,
    pub modified_cells: HashMap<(usize, usize), String>,
    pub in_edit_mode: bool,
    pub edit_buffer: String,
    pub primary_key_columns: Vec<usize>,
    pub loading: bool,
    pub error: Option<String>,
    pub search_query: String,
    pub search_results: Vec<(usize, usize)>,
    pub current_search_result: usize,
    pub in_search_mode: bool,
    pub view_mode: TableViewMode,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub max_display_width: usize,
}

impl TableTab {
    pub fn new(table_name: String) -> Self {
        Self {
            table_name,
            columns: Vec::new(),
            rows: Vec::new(),
            total_rows: 0,
            current_page: 0,
            rows_per_page: 20,
            selected_row: 0,
            selected_col: 0,
            scroll_offset_x: 0,
            scroll_offset_y: 0,
            modified_cells: HashMap::new(),
            in_edit_mode: false,
            edit_buffer: String::new(),
            primary_key_columns: Vec::new(),
            loading: true,
            error: None,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_result: 0,
            in_search_mode: false,
            view_mode: TableViewMode::Data,
        }
    }

    /// Toggle between data and schema view
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            TableViewMode::Data => TableViewMode::Schema,
            TableViewMode::Schema => TableViewMode::Data,
        };
        // Reset selection when switching views
        self.selected_row = 0;
        self.selected_col = 0;
    }

    /// Get the current cell value (including any modifications)
    pub fn get_cell_value(&self, row: usize, col: usize) -> String {
        if let Some(modified) = self.modified_cells.get(&(row, col)) {
            modified.clone()
        } else if let Some(row_data) = self.rows.get(row) {
            row_data.get(col).cloned().unwrap_or_default()
        } else {
            String::new()
        }
    }

    /// Start editing the current cell
    pub fn start_edit(&mut self) {
        if !self.in_edit_mode && !self.rows.is_empty() {
            self.in_edit_mode = true;
            self.edit_buffer = self.get_cell_value(self.selected_row, self.selected_col);
        }
    }

    /// Cancel editing
    pub fn cancel_edit(&mut self) {
        self.in_edit_mode = false;
        self.edit_buffer.clear();
    }

    /// Save the current edit
    pub fn save_edit(&mut self) -> Option<CellUpdate> {
        if !self.in_edit_mode {
            return None;
        }

        let row_idx = self.selected_row;
        let col_idx = self.selected_col;
        let new_value = self.edit_buffer.clone();

        // Get the original value
        let original_value = if let Some(row_data) = self.rows.get(row_idx) {
            row_data.get(col_idx).cloned().unwrap_or_default()
        } else {
            String::new()
        };

        // Only save if value changed
        if new_value != original_value {
            self.modified_cells
                .insert((row_idx, col_idx), new_value.clone());

            // Prepare update info for database
            let update = CellUpdate {
                table_name: self.table_name.clone(),
                column_name: self.columns[col_idx].name.clone(),
                new_value,
                row_index: row_idx,
                primary_key_values: self.get_primary_key_values(row_idx),
            };

            self.in_edit_mode = false;
            self.edit_buffer.clear();

            Some(update)
        } else {
            self.in_edit_mode = false;
            self.edit_buffer.clear();
            None
        }
    }

    /// Get primary key values for a row
    fn get_primary_key_values(&self, row_idx: usize) -> Vec<(String, String)> {
        let mut pk_values = Vec::new();

        if let Some(row_data) = self.rows.get(row_idx) {
            for &pk_col_idx in &self.primary_key_columns {
                if let Some(column) = self.columns.get(pk_col_idx) {
                    if let Some(value) = row_data.get(pk_col_idx) {
                        pk_values.push((column.name.clone(), value.clone()));
                    }
                }
            }
        }

        pk_values
    }

    /// Navigate to next page
    pub fn next_page(&mut self) -> bool {
        let max_page = (self.total_rows.saturating_sub(1)) / self.rows_per_page;
        if self.current_page < max_page {
            self.current_page += 1;
            self.selected_row = 0;
            true // Need to reload data
        } else {
            false
        }
    }

    /// Navigate to previous page
    pub fn prev_page(&mut self) -> bool {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_row = 0;
            true // Need to reload data
        } else {
            false
        }
    }

    /// Page down (Ctrl+d)
    pub fn page_down(&mut self) -> bool {
        self.next_page()
    }

    /// Page up (Ctrl+u)
    pub fn page_up(&mut self) -> bool {
        self.prev_page()
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected_row < self.rows.len().saturating_sub(1) {
            self.selected_row += 1;
        }
    }

    /// Move selection left
    pub fn move_left(&mut self) {
        crate::log_debug!(
            "move_left called, current col: {}, total cols: {}",
            self.selected_col,
            self.columns.len()
        );
        if self.selected_col > 0 {
            self.selected_col -= 1;
            crate::log_debug!("moved left to col: {}", self.selected_col);
        } else {
            crate::log_debug!("already at leftmost column");
        }
    }

    /// Move selection right
    pub fn move_right(&mut self) {
        crate::log_debug!(
            "move_right called, current col: {}, total cols: {}",
            self.selected_col,
            self.columns.len()
        );
        if self.selected_col < self.columns.len().saturating_sub(1) {
            self.selected_col += 1;
            crate::log_debug!("moved right to col: {}", self.selected_col);
        } else {
            crate::log_debug!("already at rightmost column");
        }
    }

    /// Jump to first row
    pub fn jump_to_first(&mut self) {
        self.selected_row = 0;
    }

    /// Jump to last row
    pub fn jump_to_last(&mut self) {
        self.selected_row = self.rows.len().saturating_sub(1);
    }

    /// Jump to first column
    pub fn jump_to_first_col(&mut self) {
        self.selected_col = 0;
    }

    /// Jump to last column
    pub fn jump_to_last_col(&mut self) {
        self.selected_col = self.columns.len().saturating_sub(1);
    }

    /// Start search mode
    pub fn start_search(&mut self) {
        self.in_search_mode = true;
        self.search_query.clear();
        self.search_results.clear();
        self.current_search_result = 0;
    }

    /// Cancel search
    pub fn cancel_search(&mut self) {
        self.in_search_mode = false;
        self.search_query.clear();
        self.search_results.clear();
        self.current_search_result = 0;
    }

    /// Update search query and find matches
    pub fn update_search(&mut self, query: &str) {
        self.search_query = query.to_lowercase();
        self.search_results.clear();
        self.current_search_result = 0;

        if self.search_query.is_empty() {
            return;
        }

        // Search through all cells
        for (row_idx, row_data) in self.rows.iter().enumerate() {
            for (col_idx, cell_value) in row_data.iter().enumerate() {
                // Check modified cells first
                let value = if let Some(modified) = self.modified_cells.get(&(row_idx, col_idx)) {
                    modified.clone()
                } else {
                    cell_value.clone()
                };

                if value.to_lowercase().contains(&self.search_query) {
                    self.search_results.push((row_idx, col_idx));
                }
            }
        }
    }

    /// Navigate to next search result
    pub fn next_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_result =
                (self.current_search_result + 1) % self.search_results.len();
            if let Some(&(row, col)) = self.search_results.get(self.current_search_result) {
                self.selected_row = row;
                self.selected_col = col;
            }
        }
    }

    /// Navigate to previous search result
    pub fn prev_search_result(&mut self) {
        if !self.search_results.is_empty() {
            if self.current_search_result == 0 {
                self.current_search_result = self.search_results.len() - 1;
            } else {
                self.current_search_result -= 1;
            }
            if let Some(&(row, col)) = self.search_results.get(self.current_search_result) {
                self.selected_row = row;
                self.selected_col = col;
            }
        }
    }
}

/// Represents a cell update to be applied to the database
#[derive(Debug, Clone)]
pub struct CellUpdate {
    pub table_name: String,
    pub column_name: String,
    pub new_value: String,
    pub row_index: usize,
    pub primary_key_values: Vec<(String, String)>,
}

/// State for the table viewer
#[derive(Debug, Clone)]
pub struct TableViewerState {
    pub tabs: Vec<TableTab>,
    pub active_tab: usize,
    pub show_help: bool,
    pub delete_confirmation: Option<DeleteConfirmation>,
    pub last_d_press: Option<std::time::Instant>,
    pub last_y_press: Option<std::time::Instant>,
}

/// Delete confirmation dialog state
#[derive(Debug, Clone)]
pub struct DeleteConfirmation {
    pub row_index: usize,
    pub table_name: String,
    pub primary_key_values: Vec<(String, String)>,
}

impl TableViewerState {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
            show_help: false,
            delete_confirmation: None,
            last_d_press: None,
            last_y_press: None,
        }
    }

    /// Add a new table tab
    pub fn add_tab(&mut self, table_name: String) -> usize {
        // Check if tab already exists
        for (idx, tab) in self.tabs.iter().enumerate() {
            if tab.table_name == table_name {
                self.active_tab = idx;
                return idx;
            }
        }

        // Add new tab
        self.tabs.push(TableTab::new(table_name));
        self.active_tab = self.tabs.len() - 1;
        self.active_tab
    }

    /// Close current tab
    pub fn close_current_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.tabs.remove(self.active_tab);

            if !self.tabs.is_empty() {
                if self.active_tab >= self.tabs.len() {
                    self.active_tab = self.tabs.len() - 1;
                }
            } else {
                self.active_tab = 0;
            }
        }
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
        }
    }

    /// Switch to previous tab
    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            if self.active_tab > 0 {
                self.active_tab -= 1;
            } else {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    /// Get current tab
    pub fn current_tab(&self) -> Option<&TableTab> {
        self.tabs.get(self.active_tab)
    }

    /// Get current tab mutably
    pub fn current_tab_mut(&mut self) -> Option<&mut TableTab> {
        self.tabs.get_mut(self.active_tab)
    }

    /// Toggle help
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Copy current row to clipboard in CSV format
    pub fn copy_row_csv(&self) -> Result<(), String> {
        if let Some(tab) = self.current_tab() {
            if let Some(row_data) = tab.rows.get(tab.selected_row) {
                // Escape CSV values that contain commas, quotes, or newlines
                let csv_row = row_data
                    .iter()
                    .map(|cell| {
                        if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
                            format!("\"{}\"", cell.replace('"', "\"\""))
                        } else {
                            cell.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(",");

                // Copy to clipboard
                let mut clipboard = arboard::Clipboard::new()
                    .map_err(|e| format!("Failed to access clipboard: {e}"))?;
                clipboard
                    .set_text(csv_row)
                    .map_err(|e| format!("Failed to copy to clipboard: {e}"))?;

                Ok(())
            } else {
                Err("No row selected".to_string())
            }
        } else {
            Err("No table open".to_string())
        }
    }

    /// Prepare delete confirmation for current row
    pub fn prepare_delete_confirmation(&mut self) -> Option<DeleteConfirmation> {
        if let Some(tab) = self.current_tab() {
            if tab.selected_row < tab.rows.len() {
                // Get primary key values for the row
                let mut primary_key_values = Vec::new();
                for &pk_idx in &tab.primary_key_columns {
                    if let Some(pk_col) = tab.columns.get(pk_idx) {
                        if let Some(row) = tab.rows.get(tab.selected_row) {
                            if let Some(value) = row.get(pk_idx) {
                                primary_key_values.push((pk_col.name.clone(), value.clone()));
                            }
                        }
                    }
                }

                if primary_key_values.is_empty() {
                    // Can't delete without primary key
                    return None;
                }

                Some(DeleteConfirmation {
                    row_index: tab.selected_row,
                    table_name: tab.table_name.clone(),
                    primary_key_values,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for TableViewerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the table viewer
pub fn render_table_viewer(
    f: &mut Frame,
    state: &mut TableViewerState,
    area: Rect,
    theme: &Theme,
    is_focused: bool,
) {
    if state.tabs.is_empty() {
        render_empty_state(f, area, theme, is_focused);
        return;
    }

    // Layout: Tabs at top, table content below, help at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                                   // Tabs
            Constraint::Min(10),                                     // Table content
            Constraint::Length(if state.show_help { 8 } else { 3 }), // Help/status
        ])
        .split(area);

    // Render tabs
    render_tabs(f, state, chunks[0], theme, is_focused);

    // Render current table
    if let Some(tab) = state.current_tab() {
        render_table_content(f, tab, chunks[1], theme, is_focused);
    }

    // Render help or status
    if state.show_help {
        render_help(f, chunks[2], theme);
    } else {
        render_status_bar(f, state, chunks[2], theme);
    }

    // Render delete confirmation dialog if active
    if let Some(confirmation) = &state.delete_confirmation {
        render_delete_confirmation(f, confirmation, f.area(), theme);
    }
}

fn render_delete_confirmation(
    f: &mut Frame,
    confirmation: &DeleteConfirmation,
    area: Rect,
    theme: &Theme,
) {
    // No full-screen overlay - just render the modal

    // Create a compact centered modal
    let modal_width = 50u16.min(area.width - 4);
    let modal_height = 7;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect {
        x,
        y,
        width: modal_width,
        height: modal_height,
    };

    // Create the modal content with proper spacing
    let inner_block = Block::default()
        .borders(Borders::ALL)
        .title(" ⚠ Delete Confirmation ")
        .title_alignment(Alignment::Center)
        .border_style(
            Style::default()
                .fg(theme.get_color("danger"))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.get_color("modal_background")));

    f.render_widget(inner_block, modal_area);

    // Calculate inner area for content
    let inner_area = Rect {
        x: modal_area.x + 2,
        y: modal_area.y + 1,
        width: modal_area.width.saturating_sub(4),
        height: modal_area.height.saturating_sub(2),
    };

    // Build the content lines with proper formatting
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Delete row ",
                Style::default().fg(theme.get_color("text_primary")),
            ),
            Span::styled(
                format!("#{}", confirmation.row_index + 1),
                Style::default()
                    .fg(theme.get_color("primary_highlight"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " from table ",
                Style::default().fg(theme.get_color("text_primary")),
            ),
            Span::styled(
                format!("'{}'", confirmation.table_name),
                Style::default()
                    .fg(theme.get_color("secondary_highlight"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("?", Style::default().fg(theme.get_color("text_primary"))),
        ]),
        Line::from(""),
        Line::from("─────────────────")
            .fg(theme.get_color("border_muted"))
            .centered(),
        Line::from(vec![
            Span::styled(
                "[Y/Enter] ",
                Style::default()
                    .fg(theme.get_color("success"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Confirm  ",
                Style::default().fg(theme.get_color("text_secondary")),
            ),
            Span::styled(
                "[N/Esc] ",
                Style::default()
                    .fg(theme.get_color("danger"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Cancel",
                Style::default().fg(theme.get_color("text_secondary")),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, inner_area);
}

fn render_empty_state(f: &mut Frame, area: Rect, theme: &Theme, is_focused: bool) {
    let text = vec![
        Line::from(""),
        Line::from("No tables open").fg(theme.get_color("text_muted")),
        Line::from(""),
        Line::from("Press Enter on a table in the Tables pane to open it")
            .fg(theme.get_color("text_muted")),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Table Viewer ")
                .border_style(if is_focused {
                    Style::default().fg(theme.get_color("active_border"))
                } else {
                    Style::default().fg(theme.get_color("border"))
                }),
        )
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_tabs(
    f: &mut Frame,
    state: &TableViewerState,
    area: Rect,
    theme: &Theme,
    is_focused: bool,
) {
    let tab_titles: Vec<String> = state
        .tabs
        .iter()
        .enumerate()
        .map(|(idx, tab)| {
            let modified = if tab.modified_cells.is_empty() {
                ""
            } else {
                " *"
            };

            if idx == state.active_tab {
                format!(
                    " {} {}{} ",
                    if idx == state.active_tab { "▶" } else { " " },
                    tab.table_name,
                    modified
                )
            } else {
                format!("  {}{}  ", tab.table_name, modified)
            }
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Open Tables ")
                .border_style(if is_focused {
                    Style::default().fg(theme.get_color("active_border"))
                } else {
                    Style::default().fg(theme.get_color("border"))
                }),
        )
        .select(state.active_tab)
        .style(Style::default().fg(theme.get_color("text_primary")))
        .highlight_style(
            Style::default()
                .fg(theme.get_color("secondary_highlight"))
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

fn render_table_content(
    f: &mut Frame,
    tab: &TableTab,
    area: Rect,
    theme: &Theme,
    is_focused: bool,
) {
    if tab.loading {
        let loading_msg = match tab.view_mode {
            TableViewMode::Data => "Loading table data...",
            TableViewMode::Schema => "Loading table schema...",
        };
        let loading = Paragraph::new(loading_msg)
            .style(Style::default().fg(theme.get_color("warning")))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", tab.table_name)),
            )
            .alignment(Alignment::Center);
        f.render_widget(loading, area);
        return;
    }

    if let Some(ref error) = tab.error {
        let error_text = Paragraph::new(error.as_str())
            .style(Style::default().fg(theme.get_color("danger")))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} - Error ", tab.table_name))
                    .border_style(Style::default().fg(theme.get_color("danger"))),
            )
            .alignment(Alignment::Center);
        f.render_widget(error_text, area);
        return;
    }

    // Render based on view mode
    match tab.view_mode {
        TableViewMode::Data => render_data_view(f, tab, area, theme, is_focused),
        TableViewMode::Schema => render_schema_view(f, tab, area, theme, is_focused),
    }
}

fn render_data_view(f: &mut Frame, tab: &TableTab, area: Rect, theme: &Theme, is_focused: bool) {
    // Prepare table headers
    let headers: Vec<TableCell> = tab
        .columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            let style = if idx == tab.selected_col && !tab.in_edit_mode {
                Style::default()
                    .fg(theme.get_color("secondary_highlight"))
                    .add_modifier(Modifier::BOLD)
            } else if col.is_primary_key {
                Style::default()
                    .fg(theme.get_color("primary_highlight"))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.get_color("text_primary"))
            };

            let name = if col.is_primary_key {
                format!(" 🔑 {} ", col.name)
            } else {
                format!(" {} ", col.name)
            };

            TableCell::from(name).style(style)
        })
        .collect();

    let header = Row::new(headers)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    // Prepare table rows
    let rows: Vec<Row> = tab
        .rows
        .iter()
        .enumerate()
        .map(|(row_idx, row_data)| {
            let cells: Vec<TableCell> = row_data
                .iter()
                .enumerate()
                .map(|(col_idx, value)| {
                    let is_selected = row_idx == tab.selected_row && col_idx == tab.selected_col;
                    let is_modified = tab.modified_cells.contains_key(&(row_idx, col_idx));
                    let is_search_match = tab.search_results.contains(&(row_idx, col_idx));
                    let is_current_search = tab.search_results.get(tab.current_search_result)
                        == Some(&(row_idx, col_idx));

                    let display_value = if is_selected && tab.in_edit_mode {
                        format!(" {}▌ ", tab.edit_buffer)
                    } else if is_modified {
                        let val = tab
                            .modified_cells
                            .get(&(row_idx, col_idx))
                            .cloned()
                            .unwrap_or_else(|| value.clone());
                        format!(" {val} ")
                    } else {
                        format!(" {value} ")
                    };

                    // Base style with alternating row background
                    let base_style = if row_idx % 2 == 0 {
                        Style::default().bg(theme.get_color("row_alternate_bg"))
                    } else {
                        Style::default()
                    };

                    let style = if is_selected && tab.in_edit_mode {
                        Style::default()
                            .fg(theme.get_color("edit_mode_text"))
                            .bg(theme.get_color("edit_mode_bg"))
                            .add_modifier(Modifier::BOLD)
                    } else if is_current_search {
                        Style::default()
                            .fg(theme.get_color("search_current_text"))
                            .bg(theme.get_color("search_current_bg"))
                            .add_modifier(Modifier::BOLD)
                    } else if is_selected {
                        Style::default()
                            .fg(theme.get_color("selected_text"))
                            .bg(theme.get_color("selected_bg"))
                    } else if is_search_match {
                        base_style
                            .fg(theme.get_color("search_match"))
                            .add_modifier(Modifier::UNDERLINED)
                    } else if is_modified {
                        base_style
                            .fg(theme.get_color("modified_cell"))
                            .add_modifier(Modifier::ITALIC)
                    } else if value == "NULL" || value.is_empty() {
                        base_style.fg(theme.get_color("null_value"))
                    } else {
                        base_style
                    };

                    TableCell::from(display_value).style(style)
                })
                .collect();

            Row::new(cells).height(1).bottom_margin(0)
        })
        .collect();

    // Calculate column widths
    let widths: Vec<Constraint> = tab
        .columns
        .iter()
        .map(|col| Constraint::Min(col.max_display_width.min(30) as u16))
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " {} - Data - Page {}/{} ({} rows) [t] Toggle View{} ",
                    tab.table_name,
                    tab.current_page + 1,
                    (tab.total_rows.saturating_sub(1)) / tab.rows_per_page + 1,
                    tab.total_rows,
                    if tab.in_search_mode {
                        format!(
                            " | Search: '{}' ({}/{})",
                            tab.search_query,
                            if tab.search_results.is_empty() {
                                0
                            } else {
                                tab.current_search_result + 1
                            },
                            tab.search_results.len()
                        )
                    } else if !tab.search_results.is_empty() {
                        format!(
                            " | Found: {}/{}",
                            tab.current_search_result + 1,
                            tab.search_results.len()
                        )
                    } else {
                        String::new()
                    }
                ))
                .border_style(if tab.in_edit_mode {
                    Style::default().fg(theme.get_color("edit_mode_border"))
                } else if tab.in_search_mode {
                    Style::default().fg(theme.get_color("search_mode_border"))
                } else if is_focused {
                    Style::default().fg(theme.get_color("active_border"))
                } else {
                    Style::default().fg(theme.get_color("border"))
                }),
        )
        .column_spacing(1)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_widget(table, area);
}

fn render_schema_view(f: &mut Frame, tab: &TableTab, area: Rect, theme: &Theme, is_focused: bool) {
    // Create schema rows showing column information
    let schema_headers = vec![
        TableCell::from(" Column Name ").style(
            Style::default()
                .fg(theme.get_color("text_primary"))
                .add_modifier(Modifier::BOLD),
        ),
        TableCell::from(" Data Type ").style(
            Style::default()
                .fg(theme.get_color("text_primary"))
                .add_modifier(Modifier::BOLD),
        ),
        TableCell::from(" Nullable ").style(
            Style::default()
                .fg(theme.get_color("text_primary"))
                .add_modifier(Modifier::BOLD),
        ),
        TableCell::from(" Primary Key ").style(
            Style::default()
                .fg(theme.get_color("text_primary"))
                .add_modifier(Modifier::BOLD),
        ),
    ];

    let header = Row::new(schema_headers)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    // Create rows for each column showing its schema information
    let schema_rows: Vec<Row> = tab
        .columns
        .iter()
        .enumerate()
        .map(|(row_idx, col)| {
            let is_selected = row_idx == tab.selected_row;

            // Base style with alternating row background
            let base_style = if row_idx % 2 == 0 {
                Style::default().bg(theme.get_color("row_alternate_bg"))
            } else {
                Style::default()
            };

            let row_style = if is_selected {
                Style::default()
                    .fg(theme.get_color("selected_text"))
                    .bg(theme.get_color("selected_bg"))
            } else {
                base_style
            };

            let cells = vec![
                TableCell::from(format!(
                    " {} {}",
                    if col.is_primary_key { "🔑" } else { " " },
                    col.name
                ))
                .style(row_style),
                TableCell::from(format!(" {} ", col.data_type)).style(row_style),
                TableCell::from(format!(" {} ", if col.is_nullable { "YES" } else { "NO" }))
                    .style(row_style),
                TableCell::from(format!(
                    " {} ",
                    if col.is_primary_key { "YES" } else { "NO" }
                ))
                .style(row_style),
            ];

            Row::new(cells).height(1).bottom_margin(0)
        })
        .collect();

    // Calculate column widths for schema view
    let widths = vec![
        Constraint::Min(20), // Column Name
        Constraint::Min(15), // Data Type
        Constraint::Min(10), // Nullable
        Constraint::Min(12), // Primary Key
    ];

    let table = Table::new(schema_rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " {} - Schema ({} columns) [t] Toggle View ",
                    tab.table_name,
                    tab.columns.len()
                ))
                .border_style(if is_focused {
                    Style::default().fg(theme.get_color("active_border"))
                } else {
                    Style::default().fg(theme.get_color("border"))
                }),
        )
        .column_spacing(1)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_widget(table, area);
}

fn render_status_bar(f: &mut Frame, state: &TableViewerState, area: Rect, theme: &Theme) {
    let help_text = if let Some(tab) = state.current_tab() {
        if tab.in_edit_mode {
            " [ESC/Enter] Save & Exit | [Ctrl+C] Cancel Edit "
        } else if tab.in_search_mode {
            " Type to search | [Enter] Confirm | [ESC] Cancel | [n/N] Next/Previous "
        } else {
            match tab.view_mode {
                TableViewMode::Data => " [?] Help | [i] Edit Cell | [/] Search | [t] Schema View | [x] Close Tab | [S/D] Switch Tabs ",
                TableViewMode::Schema => " [?] Help | [t] Data View | [x] Close Tab | [S/D] Switch Tabs ",
            }
        }
    } else {
        " [?] Help | Open a table to start "
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(theme.get_color("text_muted")))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme.get_color("border_muted"))),
        )
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}

fn render_help(f: &mut Frame, area: Rect, theme: &Theme) {
    let help_text = vec![
        Line::from(vec![
            Span::styled(
                "Navigation: ",
                Style::default()
                    .fg(theme.get_color("primary_highlight"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(
                "h/j/k/l - Move between cells | gg/G - First/Last row | 0/$ - First/Last column",
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Editing: ",
                Style::default()
                    .fg(theme.get_color("primary_highlight"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("i - Edit cell | ESC - Save changes | Ctrl+C - Cancel edit"),
        ]),
        Line::from(vec![
            Span::styled(
                "Pagination: ",
                Style::default()
                    .fg(theme.get_color("primary_highlight"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Ctrl+D - Page down | Ctrl+U - Page up | n/p - Next/Previous page"),
        ]),
        Line::from(vec![
            Span::styled(
                "Tabs: ",
                Style::default()
                    .fg(theme.get_color("primary_highlight"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("S - Previous tab | D - Next tab | x - Close current tab"),
        ]),
        Line::from(vec![
            Span::styled(
                "Other: ",
                Style::default()
                    .fg(theme.get_color("primary_highlight"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("r - Refresh data | / - Search | ? - Toggle this help"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Table Viewer Help ")
                .border_style(Style::default().fg(theme.get_color("secondary_highlight"))),
        )
        .alignment(Alignment::Left);

    f.render_widget(help, area);
}
