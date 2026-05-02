// FilePath: src/ui/components/table_viewer.rs

// Tabbed query results viewer with Data and Schema view modes.
// Renders tabular data with cell navigation, editing support, and scrollable
// schema information including columns, indexes, foreign keys, and statistics.

use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell as TableCell, Clear, Paragraph, Row, Table, Tabs, Wrap},
    Frame,
};

// Re-export all types from lazytables-core::table_viewer
pub use lazytables_core::table_viewer::*;

// REPLACED_MARKER_START - types moved to core
// The following types are now re-exported from lazytables_core:
// TableViewMode, TableTab, ColumnInfo, CellUpdate, TableViewerState,
// DeleteConfirmation, SetNullConfirmation
// REPLACED_MARKER_END

/// Extension trait for TableViewerState methods that depend on clipboard (arboard)
pub trait TableViewerStateExt {
    /// Copy current row to clipboard in CSV format
    fn copy_row_csv(&self) -> Result<(), String>;

    /// Copy current cell to clipboard (raw value)
    fn copy_cell(&self) -> Result<(), String>;
}

impl TableViewerStateExt for TableViewerState {
    fn copy_row_csv(&self) -> Result<(), String> {
        if let Some(tab) = self.current_tab() {
            if let Some(row_data) = tab.rows.get(tab.selected_row) {
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

    fn copy_cell(&self) -> Result<(), String> {
        if let Some(tab) = self.current_tab() {
            if tab.rows.is_empty() {
                return Err("No data in table".to_string());
            }

            let cell_value = tab.get_cell_value(tab.selected_row, tab.selected_col);

            let mut clipboard = arboard::Clipboard::new()
                .map_err(|e| format!("Failed to access clipboard: {e}"))?;
            clipboard
                .set_text(cell_value)
                .map_err(|e| format!("Failed to copy to clipboard: {e}"))?;

            Ok(())
        } else {
            Err("No table open".to_string())
        }
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

    // Layout: Tabs at top, table content below, optional help overlay
    let chunks = if state.show_help {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(10),   // Table content
                Constraint::Length(8), // Help
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Table content (takes remaining space)
            ])
            .split(area)
    };

    // Render tabs
    render_tabs(f, state, chunks[0], theme, is_focused);

    // Render current table
    if let Some(tab) = state.current_tab_mut() {
        render_table_content(f, tab, chunks[1], theme, is_focused);
    }

    // Render help if requested (no persistent status bar)
    if state.show_help {
        render_help(f, chunks[2], theme);
    }

    // Render delete confirmation dialog if active
    if let Some(confirmation) = &state.delete_confirmation {
        render_delete_confirmation(f, confirmation, f.area(), theme);
    }

    // Render set NULL confirmation dialog if active
    if let Some(confirmation) = &state.set_null_confirmation {
        render_set_null_confirmation(f, confirmation, f.area(), theme);
    }
}

fn render_delete_confirmation(
    f: &mut Frame,
    confirmation: &DeleteConfirmation,
    area: Rect,
    theme: &Theme,
) {
    use ratatui::style::Color;

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

    // Clear the area behind the modal for better visibility
    f.render_widget(Clear, modal_area);

    // Use a solid, darker background for better visibility
    let solid_bg = Color::Rgb(20, 20, 30); // Dark blue-gray

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
        .style(Style::default().bg(solid_bg));

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
                Style::default().fg(Color::White).bg(solid_bg),
            ),
            Span::styled(
                format!("#{}", confirmation.row_index + 1),
                Style::default()
                    .fg(Color::Cyan)
                    .bg(solid_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " from table ",
                Style::default().fg(Color::White).bg(solid_bg),
            ),
            Span::styled(
                format!("'{}'", confirmation.table_name),
                Style::default()
                    .fg(Color::Yellow)
                    .bg(solid_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("?", Style::default().fg(Color::White).bg(solid_bg)),
        ]),
        Line::from(""),
        Line::from("─────────────────")
            .fg(Color::Gray)
            .bg(solid_bg)
            .centered(),
        Line::from(vec![
            Span::styled(
                "[Y/Enter] ",
                Style::default()
                    .fg(Color::Green)
                    .bg(solid_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Confirm  ", Style::default().fg(Color::White).bg(solid_bg)),
            Span::styled(
                "[N/Esc] ",
                Style::default()
                    .fg(Color::Red)
                    .bg(solid_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Cancel", Style::default().fg(Color::White).bg(solid_bg)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .style(Style::default().bg(solid_bg))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, inner_area);
}

fn render_set_null_confirmation(
    f: &mut Frame,
    confirmation: &SetNullConfirmation,
    area: Rect,
    theme: &Theme,
) {
    use ratatui::style::Color;

    // Create a compact centered modal
    let modal_width = 60u16.min(area.width - 4);
    let modal_height = 9;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect {
        x,
        y,
        width: modal_width,
        height: modal_height,
    };

    // Clear the area behind the modal for better visibility
    f.render_widget(Clear, modal_area);

    // Use a solid, darker background for better visibility
    let solid_bg = Color::Rgb(20, 20, 30); // Dark blue-gray

    // Create the modal content with proper spacing
    let inner_block = Block::default()
        .borders(Borders::ALL)
        .title(" ⚠ Set NULL Confirmation ")
        .title_alignment(Alignment::Center)
        .border_style(
            Style::default()
                .fg(theme.get_color("warning"))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(solid_bg));

    f.render_widget(inner_block, modal_area);

    // Calculate inner area for content
    let inner_area = Rect {
        x: modal_area.x + 2,
        y: modal_area.y + 1,
        width: modal_area.width.saturating_sub(4),
        height: modal_area.height.saturating_sub(2),
    };

    // Truncate current value if too long
    let display_value = if confirmation.current_value.len() > 30 {
        format!("{}...", &confirmation.current_value[..27])
    } else {
        confirmation.current_value.clone()
    };

    // Build the content lines with proper formatting
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Set column ",
                Style::default().fg(Color::White).bg(solid_bg),
            ),
            Span::styled(
                format!("'{}'", confirmation.column_name),
                Style::default()
                    .fg(Color::Cyan)
                    .bg(solid_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to ", Style::default().fg(Color::White).bg(solid_bg)),
            Span::styled(
                "NULL",
                Style::default()
                    .fg(Color::Yellow)
                    .bg(solid_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("?", Style::default().fg(Color::White).bg(solid_bg)),
        ]),
        Line::from(vec![
            Span::styled(
                "Current value: ",
                Style::default().fg(Color::Gray).bg(solid_bg),
            ),
            Span::styled(
                format!("'{}'", display_value),
                Style::default()
                    .fg(Color::White)
                    .bg(solid_bg)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]),
        Line::from(""),
        Line::from("─────────────────")
            .fg(Color::Gray)
            .bg(solid_bg)
            .centered(),
        Line::from(vec![
            Span::styled(
                "[Y/Enter] ",
                Style::default()
                    .fg(Color::Green)
                    .bg(solid_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Confirm  ", Style::default().fg(Color::White).bg(solid_bg)),
            Span::styled(
                "[N/Esc] ",
                Style::default()
                    .fg(Color::Red)
                    .bg(solid_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Cancel", Style::default().fg(Color::White).bg(solid_bg)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .style(Style::default().bg(solid_bg))
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
    tab: &mut TableTab,
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

fn render_data_view(
    f: &mut Frame,
    tab: &mut TableTab,
    area: Rect,
    theme: &Theme,
    is_focused: bool,
) {
    // Calculate visible columns based on available width
    tab.ensure_column_visible(area.width as usize);
    let visible_column_indices = tab.calculate_visible_columns(area.width as usize);

    // Prepare table headers - only for visible columns
    let headers: Vec<TableCell> = visible_column_indices
        .iter()
        .filter_map(|&idx| tab.columns.get(idx).map(|col| (idx, col)))
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

    // Calculate viewport height for scrolling and update the tab
    let viewport_height = area.height.saturating_sub(4) as usize; // Account for borders (2) + header (2)
    tab.update_viewport_height(area.height as usize);
    tab.ensure_selection_visible_with_height(viewport_height);

    // Prepare table rows - only render visible rows within viewport
    let visible_rows: Vec<_> = tab
        .rows
        .iter()
        .enumerate()
        .skip(tab.scroll_offset_y)
        .take(viewport_height)
        .collect();

    let rows: Vec<Row> = visible_rows
        .iter()
        .map(|(row_idx, row_data)| {
            let cells: Vec<TableCell> = visible_column_indices
                .iter()
                .map(|&col_idx| {
                    let value = row_data.get(col_idx).cloned().unwrap_or_default();
                    let is_selected = *row_idx == tab.selected_row && col_idx == tab.selected_col;
                    let is_modified = tab.modified_cells.contains_key(&(*row_idx, col_idx));
                    let is_search_match = tab.search_results.contains(&(*row_idx, col_idx));
                    let is_current_search = tab.search_results.get(tab.current_search_result)
                        == Some(&(*row_idx, col_idx));

                    let display_value = if is_selected && tab.in_edit_mode {
                        format!(" {}▌ ", tab.edit_buffer)
                    } else if is_modified {
                        let val = tab
                            .modified_cells
                            .get(&(*row_idx, col_idx))
                            .cloned()
                            .unwrap_or_else(|| value.clone());
                        format!(" {val} ")
                    } else {
                        format!(" {value} ")
                    };

                    // Base style with alternating row background
                    let base_style = if *row_idx % 2 == 0 {
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

    // Calculate column widths for visible columns only
    let widths: Vec<Constraint> = visible_column_indices
        .iter()
        .map(|&idx| {
            let col = &tab.columns[idx];
            Constraint::Min(col.max_display_width.min(30) as u16)
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " {} - Data - Page {}/{} ({} rows, {} cols) {} [t] Toggle View{} ",
                    tab.table_name,
                    tab.current_page + 1,
                    (tab.total_rows.saturating_sub(1)) / tab.rows_per_page + 1,
                    tab.total_rows,
                    tab.columns.len(),
                    if visible_column_indices.len() < tab.columns.len() {
                        format!(
                            "[{}-{}/{}]",
                            tab.scroll_offset_x + 1,
                            tab.scroll_offset_x + visible_column_indices.len(),
                            tab.columns.len()
                        )
                    } else {
                        String::new()
                    },
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

fn render_schema_view(
    f: &mut Frame,
    tab: &mut TableTab,
    area: Rect,
    theme: &Theme,
    is_focused: bool,
) {
    use ratatui::widgets::Paragraph;

    // Build comprehensive schema information as text lines
    let mut lines = Vec::new();

    // Title
    lines.push(Line::from(vec![Span::styled(
        format!("📋 TABLE SCHEMA: {}", tab.table_name),
        Style::default()
            .fg(theme.get_color("primary_highlight"))
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    // Section 1: COLUMNS
    lines.push(Line::from(vec![Span::styled(
        "━━━ COLUMNS ━━━",
        Style::default()
            .fg(theme.get_color("accent"))
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    for col in &tab.columns {
        let pk_marker = if col.is_primary_key { " 🔑" } else { "" };
        let nullable = if col.is_nullable { "NULL" } else { "NOT NULL" };

        lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(theme.get_color("success"))),
            Span::styled(
                format!("{}{}", col.name, pk_marker),
                Style::default()
                    .fg(theme.get_color("text_primary"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" : "),
            Span::styled(&col.data_type, Style::default().fg(theme.get_color("info"))),
            Span::raw("  "),
            Span::styled(
                nullable,
                Style::default().fg(if col.is_nullable {
                    theme.get_color("warning")
                } else {
                    theme.get_color("text_secondary")
                }),
            ),
        ]));
    }

    // Section 2: INDEXES (if metadata available)
    if let Some(metadata) = &tab.table_metadata {
        if !metadata.indexes.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "━━━ INDEXES ━━━",
                Style::default()
                    .fg(theme.get_color("accent"))
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));

            for idx in &metadata.indexes {
                let idx_type = idx.index_type.as_deref().unwrap_or("BTREE");
                let unique_marker = if idx.is_unique { " UNIQUE" } else { "" };
                let primary_marker = if idx.is_primary { " PRIMARY" } else { "" };

                lines.push(Line::from(vec![
                    Span::styled("  📑 ", Style::default().fg(theme.get_color("info"))),
                    Span::styled(
                        &idx.name,
                        Style::default()
                            .fg(theme.get_color("text_primary"))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("({}{}{})", idx_type, unique_marker, primary_marker),
                        Style::default().fg(theme.get_color("text_secondary")),
                    ),
                ]));

                lines.push(Line::from(vec![
                    Span::raw("     Columns: "),
                    Span::styled(
                        idx.columns.join(", "),
                        Style::default().fg(theme.get_color("success")),
                    ),
                ]));

                if let Some(size) = idx.size {
                    let size_mb = size as f64 / (1024.0 * 1024.0);
                    lines.push(Line::from(vec![
                        Span::raw("     Size: "),
                        Span::styled(
                            format!("{:.2} MB", size_mb),
                            Style::default().fg(theme.get_color("warning")),
                        ),
                    ]));
                }
            }
        }

        // Section 3: FOREIGN KEYS
        if !metadata.foreign_keys.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "━━━ FOREIGN KEYS ━━━",
                Style::default()
                    .fg(theme.get_color("accent"))
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));

            for fk in &metadata.foreign_keys {
                lines.push(Line::from(vec![
                    Span::styled("  🔗 ", Style::default().fg(theme.get_color("info"))),
                    Span::styled(
                        &fk.constraint_name,
                        Style::default()
                            .fg(theme.get_color("text_primary"))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));

                lines.push(Line::from(vec![
                    Span::raw("     "),
                    Span::styled(
                        fk.column_names.join(", "),
                        Style::default().fg(theme.get_color("success")),
                    ),
                    Span::raw(" → "),
                    Span::styled(
                        &fk.referenced_table,
                        Style::default().fg(theme.get_color("info")),
                    ),
                    Span::raw("("),
                    Span::styled(
                        fk.referenced_columns.join(", "),
                        Style::default().fg(theme.get_color("success")),
                    ),
                    Span::raw(")"),
                ]));

                if let (Some(on_delete), Some(on_update)) = (&fk.on_delete, &fk.on_update) {
                    lines.push(Line::from(vec![
                        Span::raw("     ON DELETE "),
                        Span::styled(on_delete, Style::default().fg(theme.get_color("warning"))),
                        Span::raw(" / ON UPDATE "),
                        Span::styled(on_update, Style::default().fg(theme.get_color("warning"))),
                    ]));
                }
            }
        }

        // Section 4: CONSTRAINTS
        if !metadata.constraints.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "━━━ CONSTRAINTS ━━━",
                Style::default()
                    .fg(theme.get_color("accent"))
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));

            for constraint in &metadata.constraints {
                lines.push(Line::from(vec![
                    Span::styled("  ⚡ ", Style::default().fg(theme.get_color("warning"))),
                    Span::styled(
                        &constraint.constraint_type,
                        Style::default()
                            .fg(theme.get_color("text_primary"))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": "),
                    Span::styled(
                        &constraint.name,
                        Style::default().fg(theme.get_color("info")),
                    ),
                ]));

                if let Some(definition) = &constraint.definition {
                    lines.push(Line::from(vec![
                        Span::raw("     "),
                        Span::styled(
                            definition,
                            Style::default().fg(theme.get_color("text_secondary")),
                        ),
                    ]));
                }
            }
        }

        // Section 5: TABLE STATISTICS
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "━━━ TABLE INFO ━━━",
            Style::default()
                .fg(theme.get_color("accent"))
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        let table_size_mb = metadata.table_size as f64 / (1024.0 * 1024.0);
        let indexes_size_mb = metadata.indexes_size as f64 / (1024.0 * 1024.0);
        let total_size_mb = metadata.total_size as f64 / (1024.0 * 1024.0);

        lines.push(Line::from(vec![
            Span::raw("  • Rows: "),
            Span::styled(
                format!("{}", metadata.row_count),
                Style::default()
                    .fg(theme.get_color("success"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Columns: "),
            Span::styled(
                format!("{}", metadata.column_count),
                Style::default()
                    .fg(theme.get_color("success"))
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::raw("  • Table Size: "),
            Span::styled(
                format!("{:.2} MB", table_size_mb),
                Style::default().fg(theme.get_color("info")),
            ),
            Span::raw("  Index Size: "),
            Span::styled(
                format!("{:.2} MB", indexes_size_mb),
                Style::default().fg(theme.get_color("info")),
            ),
            Span::raw("  Total: "),
            Span::styled(
                format!("{:.2} MB", total_size_mb),
                Style::default()
                    .fg(theme.get_color("warning"))
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        if let Some(last_vacuum) = &metadata.last_vacuum {
            lines.push(Line::from(vec![
                Span::raw("  • Last Vacuum: "),
                Span::styled(
                    last_vacuum,
                    Style::default().fg(theme.get_color("text_secondary")),
                ),
            ]));
        }

        if let Some(last_analyze) = &metadata.last_analyze {
            lines.push(Line::from(vec![
                Span::raw("  • Last Analyze: "),
                Span::styled(
                    last_analyze,
                    Style::default().fg(theme.get_color("text_secondary")),
                ),
            ]));
        }

        if let Some(owner) = &metadata.table_owner {
            lines.push(Line::from(vec![
                Span::raw("  • Owner: "),
                Span::styled(
                    owner,
                    Style::default().fg(theme.get_color("text_secondary")),
                ),
            ]));
        }
    }

    // Render as scrollable paragraph
    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " {} - Schema View [t] Toggle | [j/k] Scroll ",
                    tab.table_name
                ))
                .border_style(if is_focused {
                    Style::default().fg(theme.get_color("active_border"))
                } else {
                    Style::default().fg(theme.get_color("border"))
                }),
        )
        .wrap(Wrap { trim: false })
        .scroll((tab.scroll_offset_y as u16, 0));

    f.render_widget(paragraph, area);
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
            Span::raw("Ctrl+D - Page down | Ctrl+U - Page up"),
        ]),
        Line::from(vec![
            Span::styled(
                "Tabs: ",
                Style::default()
                    .fg(theme.get_color("primary_highlight"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("H - Previous tab | L - Next tab | x - Close current tab"),
        ]),
        Line::from(vec![
            Span::styled(
                "Copy: ",
                Style::default()
                    .fg(theme.get_color("primary_highlight"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("yy - Copy row (CSV) | yc - Copy cell"),
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
