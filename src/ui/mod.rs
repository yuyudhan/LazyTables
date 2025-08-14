// FilePath: src/ui/mod.rs

use crate::{
    app::{AppState, FocusedPane, Mode},
    config::Config,
    constants,
    core::error::Result,
    database::ConnectionStatus,
};
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

pub mod components;
pub mod layout;
pub mod widgets;

use layout::LayoutManager;

/// Main UI structure
pub struct UI {
    layout_manager: LayoutManager,
    theme: Theme,
}

impl UI {
    /// Create a new UI instance
    pub fn new(_config: &Config) -> Result<Self> {
        let layout_manager = LayoutManager::new();
        let theme = Theme::default();

        Ok(Self {
            layout_manager,
            theme,
        })
    }

    /// Draw the entire UI
    pub fn draw(&mut self, frame: &mut Frame, state: &mut AppState) {
        let areas = self.layout_manager.calculate_layout(frame.area());

        // Draw header
        self.draw_header(frame, areas.header, state);

        // Draw connections pane
        self.draw_connections_pane(frame, areas.connections, state);

        // Draw tables pane
        self.draw_tables_pane(frame, areas.tables, state);

        // Draw details pane
        self.draw_details_pane(frame, areas.details, state);

        // Draw tabular output area
        self.draw_tabular_output(frame, areas.tabular_output, state);

        // Draw SQL files browser
        self.draw_sql_files_pane(frame, areas.sql_files, state);

        // Draw query window area
        self.draw_query_window(frame, areas.query_window, state);

        // Draw status bar
        self.draw_status_bar(frame, areas.status_bar, state);

        // Draw help overlay if active
        if state.show_help {
            self.draw_help_overlay(frame, frame.area());
        }

        // Draw command mode modal if in command mode
        if state.mode == Mode::Command {
            self.draw_command_modal(frame, frame.area(), state);
        }

        // Draw connection modal if active (either add or edit)
        if state.show_add_connection_modal || state.show_edit_connection_modal {
            crate::ui::components::render_connection_modal(
                frame,
                &state.connection_modal_state,
                frame.area(),
            );
        }
    }

    /// Draw the header bar
    fn draw_header(&self, frame: &mut Frame, area: Rect, _state: &AppState) {
        let header = Paragraph::new(constants::version_string())
            .style(Style::default().fg(self.theme.header_fg))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(self.theme.border)),
            )
            .centered();

        frame.render_widget(header, area);
    }

    /// Draw the connections pane
    fn draw_connections_pane(&self, frame: &mut Frame, area: Rect, state: &mut AppState) {
        let is_focused = state.focused_pane == FocusedPane::Connections;
        let border_style = if is_focused {
            Style::default().fg(self.theme.active_border)
        } else {
            Style::default().fg(self.theme.border)
        };

        // Create list items from stored connections
        let mut items: Vec<ListItem> = state
            .connections
            .connections
            .iter()
            .map(|connection| {
                let (status_indicator, style) = match &connection.status {
                    ConnectionStatus::Connected => ("✓", Style::default().fg(Color::Green)),
                    ConnectionStatus::Connecting => ("⟳", Style::default().fg(Color::Yellow)),
                    ConnectionStatus::Failed(_) => ("✗", Style::default().fg(Color::Red)),
                    ConnectionStatus::Disconnected => ("○", Style::default().fg(Color::Gray)),
                };
                let display_text = format!("{} {}", status_indicator, connection.display_string());
                ListItem::new(Line::from(vec![Span::styled(display_text, style)]))
            })
            .collect();

        // Add instruction text if no connections exist
        if items.is_empty() {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "No connections configured",
                Style::default().fg(Color::Gray),
            )])));
            items.push(ListItem::new(""));
        }

        // Add keybinding help
        if is_focused {
            items.push(ListItem::new(""));
            items.push(ListItem::new(Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "a",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to add connection", Style::default().fg(Color::Gray)),
            ])));
            items.push(ListItem::new(Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to connect/disconnect", Style::default().fg(Color::Gray)),
            ])));
            if !state.connections.connections.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "e",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" to edit connection", Style::default().fg(Color::Gray)),
                ])));
            }
        }

        let connections = List::new(items)
            .block(
                Block::default()
                    .title(" Connections ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .bg(self.theme.selection_bg)
                    .add_modifier(Modifier::BOLD),
            );

        // Use stateful widget to show selection
        let mut list_state = state.connections_list_state.clone();
        frame.render_stateful_widget(connections, area, &mut list_state);
        
        // Update the state with any changes
        state.connections_list_state = list_state;
    }

    /// Draw the tables/views pane
    fn draw_tables_pane(&self, frame: &mut Frame, area: Rect, state: &mut AppState) {
        let is_focused = state.focused_pane == FocusedPane::Tables;
        let border_style = if is_focused {
            Style::default().fg(self.theme.active_border)
        } else {
            Style::default().fg(self.theme.border)
        };

        // Check if there's an active connection
        let has_active_connection = state
            .connections
            .connections
            .iter()
            .any(|conn| conn.is_connected());

        let items: Vec<ListItem> = if !has_active_connection {
            // Show "not connected" message
            vec![
                ListItem::new(Line::from(vec![Span::styled(
                    "No database connected",
                    Style::default().fg(Color::Gray),
                )])),
                ListItem::new(""),
                ListItem::new(Line::from(vec![Span::styled(
                    "Connect to a database",
                    Style::default().fg(Color::Gray),
                )])),
                ListItem::new(Line::from(vec![Span::styled(
                    "to view tables and views",
                    Style::default().fg(Color::Gray),
                )])),
                ListItem::new(""),
                if is_focused {
                    ListItem::new(Line::from(vec![
                        Span::styled("Press ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            "c",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(" to go to connections", Style::default().fg(Color::Gray)),
                    ]))
                } else {
                    ListItem::new("")
                },
            ]
        } else if state.tables.is_empty() {
            // Show loading or no tables message
            vec![
                ListItem::new(Line::from(vec![Span::styled(
                    "Loading tables...",
                    Style::default().fg(Color::Yellow),
                )])),
            ]
        } else {
            // Show actual tables from connected database
            let mut table_items = vec![
                ListItem::new(Line::from(vec![Span::styled(
                    "▼ Tables",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )])),
            ];
            
            for table in &state.tables {
                table_items.push(ListItem::new(Line::from(vec![
                    Span::styled("  📋 ", Style::default().fg(Color::Blue)),
                    Span::styled(table, Style::default().fg(Color::White)),
                ])));
            }
            
            // Add navigation help if focused
            if is_focused && !state.tables.is_empty() {
                table_items.push(ListItem::new(""));
                table_items.push(ListItem::new(Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "j/k",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" to navigate tables", Style::default().fg(Color::Gray)),
                ])));
                table_items.push(ListItem::new(Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "Enter",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" to view table data", Style::default().fg(Color::Gray)),
                ])));
            }
            
            table_items
        };

        let tables = List::new(items)
            .block(
                Block::default()
                    .title(" Tables/Views ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .bg(self.theme.selection_bg)
                    .add_modifier(Modifier::BOLD),
            );

        // Use stateful widget to show selection
        let mut list_state = state.tables_list_state.clone();
        frame.render_stateful_widget(tables, area, &mut list_state);
        
        // Update the state with any changes
        state.tables_list_state = list_state;
    }

    /// Draw the table details pane
    fn draw_details_pane(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.focused_pane == FocusedPane::Details;
        let border_style = if is_focused {
            Style::default().fg(self.theme.active_border)
        } else {
            Style::default().fg(self.theme.border)
        };

        // Check if there's an active connection and table selected
        let has_active_connection = state
            .connections
            .connections
            .iter()
            .any(|conn| conn.is_connected());

        // For now, we'll assume no table is selected until database integration is complete
        let has_selected_table = false;

        let details_text = if !has_active_connection {
            vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "No database connected",
                    Style::default().fg(Color::Gray),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Connect to view table details",
                    Style::default().fg(Color::Gray),
                )]),
            ]
        } else if !has_selected_table {
            vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "No table selected",
                    Style::default().fg(Color::Gray),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Select a table to view",
                    Style::default().fg(Color::Gray),
                )]),
                Line::from(vec![Span::styled(
                    "its structure and metadata",
                    Style::default().fg(Color::Gray),
                )]),
            ]
        } else {
            // Show actual table details (sample data for now)
            vec![
                Line::from("Schema: public"),
                Line::from("Table: users"),
                Line::from(""),
                Line::from("Rows: 15,234"),
                Line::from("Size: 2.4 MB"),
                Line::from("Created: Jan'24"),
                Line::from("Modified: Today"),
                Line::from(""),
                Line::from("Indexes: 3"),
                Line::from("Constraints: 2"),
                Line::from("Relations: 5"),
            ]
        };

        let details = Paragraph::new(details_text)
            .block(
                Block::default()
                    .title(" Table Details ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .style(Style::default().fg(self.theme.text));

        frame.render_widget(details, area);
    }

    /// Draw the tabular output area
    fn draw_tabular_output(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.focused_pane == FocusedPane::TabularOutput;
        let border_style = if is_focused {
            Style::default().fg(self.theme.active_border)
        } else {
            Style::default().fg(self.theme.border)
        };

        // Check if there's an active connection and table selected
        let has_active_connection = state
            .connections
            .connections
            .iter()
            .any(|conn| conn.is_connected());

        // For now, we'll assume no table is selected until database integration is complete
        let has_selected_table = false;

        if !has_active_connection {
            // Show "no connection" message
            let message = vec![
                Line::from(""),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "No database connection active",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Connect to a database from the Connections pane",
                    Style::default().fg(Color::Gray),
                )]),
                Line::from(vec![Span::styled(
                    "Press 'c' to focus connections, then Enter to connect",
                    Style::default().fg(Color::Gray),
                )]),
            ];

            let placeholder = Paragraph::new(message)
                .block(
                    Block::default()
                        .title(" Query Results ")
                        .borders(Borders::ALL)
                        .border_style(border_style),
                )
                .style(Style::default().fg(self.theme.text))
                .alignment(Alignment::Center);

            frame.render_widget(placeholder, area);
        } else if !has_selected_table {
            // Show "no table selected" message
            let message = vec![
                Line::from(""),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "No table selected",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Select a table from the Tables pane to view its data",
                    Style::default().fg(Color::Gray),
                )]),
                Line::from(vec![Span::styled(
                    "Or execute a query from the SQL editor below",
                    Style::default().fg(Color::Gray),
                )]),
            ];

            let placeholder = Paragraph::new(message)
                .block(
                    Block::default()
                        .title(" Query Results ")
                        .borders(Borders::ALL)
                        .border_style(border_style),
                )
                .style(Style::default().fg(self.theme.text))
                .alignment(Alignment::Center);

            frame.render_widget(placeholder, area);
        } else {
            // Show actual table data (sample data for now)
            let header = Row::new(vec!["id", "name", "email", "created"])
                .style(Style::default().fg(self.theme.header_fg))
                .height(1);

            let rows = vec![
                Row::new(vec!["1", "John", "john@example.com", "2024-01-15"]),
                Row::new(vec!["2", "Jane", "jane@example.com", "2024-01-16"]),
                Row::new(vec!["3", "Bob", "bob@example.com", "2024-01-17"]),
                Row::new(vec!["4", "Alice", "alice@example.com", "2024-01-18"]),
                Row::new(vec!["5", "Eve", "eve@example.com", "2024-01-19"]),
            ];

            let widths = [
                Constraint::Length(5),
                Constraint::Length(15),
                Constraint::Length(25),
                Constraint::Length(12),
            ];

            let table = Table::new(rows, widths)
                .header(header)
                .block(
                    Block::default()
                        .title(" Query Results ")
                        .borders(Borders::ALL)
                        .border_style(border_style),
                )
                .row_highlight_style(Style::default().bg(self.theme.selection_bg));

            frame.render_widget(table, area);
        }
    }

    /// Draw the SQL files browser pane
    fn draw_sql_files_pane(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.focused_pane == FocusedPane::SqlFiles;
        let border_style = if is_focused {
            Style::default().fg(self.theme.active_border)
        } else {
            Style::default().fg(self.theme.border)
        };

        // Create list items from SQL files
        let mut items: Vec<ListItem> = state
            .saved_sql_files
            .iter()
            .enumerate()
            .map(|(i, filename)| {
                let prefix = if Some(filename) == state.current_sql_file.as_ref() {
                    "● " // Indicate currently loaded file
                } else {
                    "  "
                };

                let style = if Some(filename) == state.current_sql_file.as_ref() {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if i == state.selected_sql_file && is_focused {
                    Style::default().fg(self.theme.primary_highlight)
                } else {
                    Style::default().fg(self.theme.text)
                };

                ListItem::new(Line::from(vec![Span::styled(
                    format!("{prefix}{filename}.sql"),
                    style,
                )]))
            })
            .collect();

        // Add instruction text if no files exist
        if items.is_empty() {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "No SQL files found",
                Style::default().fg(Color::Gray),
            )])));
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "Create files with Ctrl+N",
                Style::default().fg(Color::Gray),
            )])));
        } else if is_focused {
            // Add keybinding help when focused
            items.push(ListItem::new(""));
            items.push(ListItem::new(Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to load file", Style::default().fg(Color::Gray)),
            ])));
        }

        let sql_files = List::new(items)
            .block(
                Block::default()
                    .title(" SQL Files ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .bg(self.theme.selection_bg)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(sql_files, area);
    }

    /// Draw the query window area
    fn draw_query_window(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.focused_pane == FocusedPane::QueryWindow;
        let border_style = if is_focused {
            Style::default().fg(self.theme.active_border)
        } else {
            Style::default().fg(self.theme.border)
        };

        // Check if there's an active connection for better help messages
        let has_active_connection = state
            .connections
            .connections
            .iter()
            .any(|conn| conn.is_connected());

        // Get query content lines
        let mut query_lines: Vec<Line> = if state.query_content.is_empty() {
            if !has_active_connection {
                vec![
                    Line::from(Span::styled(
                        "-- SQL Query Editor",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "-- Connect to a database first",
                        Style::default().fg(Color::Gray),
                    )),
                    Line::from(Span::styled(
                        "-- Then write your SQL queries here",
                        Style::default().fg(Color::Gray),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "-- Example:",
                        Style::default().fg(Color::Gray),
                    )),
                    Line::from(Span::styled(
                        "-- SELECT * FROM users LIMIT 10;",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]
            } else {
                vec![
                    Line::from(Span::styled(
                        "-- SQL Query Editor",
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "-- Write your SQL queries here",
                        Style::default().fg(Color::Gray),
                    )),
                    Line::from(Span::styled(
                        "-- Press Ctrl+Enter to execute",
                        Style::default().fg(Color::Gray),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "-- Example:",
                        Style::default().fg(Color::Gray),
                    )),
                    Line::from(Span::styled(
                        "-- SELECT * FROM users LIMIT 10;",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]
            }
        } else {
            state
                .query_content
                .lines()
                .enumerate()
                .map(|(i, line)| {
                    if i == state.query_cursor_line {
                        // Highlight current line
                        let mut spans = Vec::new();
                        if state.query_cursor_column > 0 {
                            spans.push(Span::raw(&line[..state.query_cursor_column]));
                        }
                        if is_focused {
                            spans.push(Span::styled(
                                if state.query_cursor_column < line.len() {
                                    &line[state.query_cursor_column..state.query_cursor_column + 1]
                                } else {
                                    " "
                                },
                                Style::default().bg(Color::Gray).fg(Color::Black),
                            ));
                        }
                        if state.query_cursor_column + 1 < line.len() {
                            spans.push(Span::raw(&line[state.query_cursor_column + 1..]));
                        }
                        Line::from(spans)
                    } else {
                        Line::from(line)
                    }
                })
                .collect()
        };

        // Add file info and keybinding help if focused
        if is_focused {
            query_lines.push(Line::from(""));

            // Show current file info
            let file_info = if let Some(ref filename) = state.current_sql_file {
                let modified_indicator = if state.query_modified { " [+]" } else { "" };
                format!("File: {filename}.sql{modified_indicator}")
            } else {
                "New file (unsaved)".to_string()
            };

            query_lines.push(Line::from(vec![Span::styled(
                file_info,
                Style::default().fg(Color::Gray),
            )]));

            // Add keybinding help
            query_lines.push(Line::from(""));
            query_lines.push(Line::from(vec![
                Span::styled(
                    "Ctrl+S",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" save | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Ctrl+O",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" open | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Ctrl+N",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" new | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Ctrl+Enter",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" execute", Style::default().fg(Color::Gray)),
            ]));
        }

        let query_editor = Paragraph::new(query_lines)
            .block(
                Block::default()
                    .title(" SQL Query Editor ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .style(Style::default().fg(self.theme.text))
            .wrap(Wrap { trim: false });

        frame.render_widget(query_editor, area);
    }

    /// Draw the status bar
    fn draw_status_bar(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let brand = format!("{} v{}", constants::APP_NAME, constants::VERSION);

        let mode_text = format!(
            "[{}]",
            match state.mode {
                Mode::Normal => "NORMAL",
                Mode::Insert => "INSERT",
                Mode::Visual => "VISUAL",
                Mode::Command => "COMMAND",
                Mode::Query => "QUERY",
            }
        );

        let connection_text = "Connected: production@localhost";
        let position_text = "Cell: B2 | Mode: Read-Only";

        // Add help hint when not showing help
        let help_hint = if !state.show_help {
            " | Press ? for help"
        } else {
            ""
        };

        let status_line = Line::from(vec![
            Span::styled(
                brand.as_str(),
                Style::default()
                    .fg(self.theme.primary_highlight)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                mode_text,
                Style::default()
                    .fg(self.theme.mode_color(state.mode))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::raw(connection_text),
            Span::raw(" | "),
            Span::raw(position_text),
            Span::raw(help_hint),
        ]);

        let status_bar = Paragraph::new(status_line).style(
            Style::default()
                .fg(self.theme.status_fg)
                .bg(self.theme.status_bg),
        );

        frame.render_widget(status_bar, area);
    }

    /// Draw help overlay
    fn draw_help_overlay(&self, frame: &mut Frame, area: Rect) {
        use ratatui::layout::Alignment;

        // Calculate centered overlay area
        let overlay_width = 80.min(area.width - 4);
        let overlay_height = 40.min(area.height - 4);
        let overlay_x = (area.width - overlay_width) / 2;
        let overlay_y = (area.height - overlay_height) / 2;

        let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

        // Clear the background for the overlay
        frame.render_widget(Clear, overlay_area);

        let help_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("{} Help", constants::version_string()),
                Style::default()
                    .fg(self.theme.header_fg)
                    .add_modifier(Modifier::BOLD),
            )])
            .alignment(Alignment::Center),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Navigation",
                Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )]),
            Line::from("  h/j/k/l        Navigate within panes"),
            Line::from("  Ctrl+h/j/k/l   Switch between panes"),
            Line::from("  Tab            Cycle through panes forward"),
            Line::from("  Shift+Tab      Cycle through panes backward"),
            Line::from("  gg/G           Jump to first/last row"),
            Line::from("  0/$            Jump to first/last column"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Modes",
                Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )]),
            Line::from("  i              Enter Insert mode (edit cells)"),
            Line::from("  v              Enter Visual mode (selection)"),
            Line::from("  :              Enter Command mode"),
            Line::from("  Space z q      Enter Query mode (SQL editor)"),
            Line::from("  ESC            Return to Normal mode"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Leader Commands (Space)",
                Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )]),
            Line::from("  Space c c      Change connection"),
            Line::from("  Space c d      Change database"),
            Line::from("  Space c r      Refresh connection"),
            Line::from("  Space t n      New table"),
            Line::from("  Space t d      Drop table"),
            Line::from("  Space t r      Rename table"),
            Line::from("  Space q r      Run query"),
            Line::from("  Space q h      Query history"),
            Line::from("  Space e x      Export data"),
            Line::from("  Space e i      Import data"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "General",
                Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )]),
            Line::from("  ?              Toggle this help"),
            Line::from("  :q or :quit    Quit LazyTables (Command mode only)"),
            Line::from("  /              Search in current view"),
            Line::from("  n/N            Next/previous search result"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default()),
                Span::styled("?", Style::default().fg(self.theme.primary_highlight)),
                Span::styled(" or ", Style::default()),
                Span::styled("ESC", Style::default().fg(self.theme.primary_highlight)),
                Span::styled(" to close this help", Style::default()),
            ])
            .alignment(Alignment::Center),
        ];

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.active_border))
                    .style(Style::default().bg(self.theme.background)),
            )
            .style(Style::default().fg(self.theme.text))
            .alignment(Alignment::Left);

        frame.render_widget(help, overlay_area);
    }

    /// Draw command mode modal
    fn draw_command_modal(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Calculate modal position (center top, like lazyvim)
        let modal_width = 60.min(area.width - 4);
        let modal_height = 10;
        let modal_x = (area.width - modal_width) / 2;
        let modal_y = 2; // Near the top

        let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

        // Clear the background for the modal
        frame.render_widget(Clear, modal_area);

        // Prepare command display with ':' prefix
        let command_text = format!(":{}", state.command_buffer);

        // Get command suggestions based on current input
        let suggestions = self.get_command_suggestions(&state.command_buffer);

        // Build the modal content
        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    command_text.as_str(),
                    Style::default()
                        .fg(self.theme.text)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        // Add separator if there are suggestions
        if !suggestions.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Suggestions:",
                    Style::default()
                        .fg(self.theme.header_fg)
                        .add_modifier(Modifier::DIM),
                ),
            ]));

            // Add suggestions (max 5)
            for (i, suggestion) in suggestions.iter().take(5).enumerate() {
                let prefix = if i == 0 { "  › " } else { "    " };
                let style = if i == 0 {
                    Style::default().fg(self.theme.primary_highlight)
                } else {
                    Style::default()
                        .fg(self.theme.text)
                        .add_modifier(Modifier::DIM)
                };

                lines.push(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(suggestion.as_str(), style),
                ]));
            }
        }

        let command_modal = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Command ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.active_border))
                    .style(Style::default().bg(self.theme.background)),
            )
            .style(Style::default().fg(self.theme.text));

        frame.render_widget(command_modal, modal_area);

        // Show cursor position
        let cursor_x = modal_area.x + 3 + command_text.len() as u16;
        let cursor_y = modal_area.y + 2;
        frame.set_cursor_position((cursor_x, cursor_y));
    }

    /// Get command suggestions based on current input
    fn get_command_suggestions(&self, input: &str) -> Vec<String> {
        let all_commands = vec![
            ("q", "Quit LazyTables"),
            ("quit", "Quit LazyTables"),
            ("w", "Write/Save current data"),
            ("write", "Write/Save current data"),
            ("wq", "Write and quit"),
            ("e", "Edit connection"),
            ("edit", "Edit connection"),
            ("connect", "Connect to database"),
            ("disconnect", "Disconnect from database"),
            ("tables", "List all tables"),
            ("refresh", "Refresh current view"),
            ("export", "Export current data"),
            ("import", "Import data from file"),
            ("set", "Set configuration option"),
            ("help", "Show help"),
            ("version", "Show version information"),
            ("schema", "Show table schema"),
            ("query", "Execute SQL query"),
            ("history", "Show command history"),
            ("clear", "Clear current view"),
        ];

        let mut suggestions = Vec::new();

        // If input is empty, show common commands
        if input.is_empty() {
            suggestions.push("q - Quit".to_string());
            suggestions.push("w - Write/Save".to_string());
            suggestions.push("help - Show help".to_string());
            suggestions.push("connect - Connect to database".to_string());
            suggestions.push("tables - List tables".to_string());
        } else {
            // Filter commands that start with the input
            for (cmd, desc) in &all_commands {
                if cmd.starts_with(input) {
                    suggestions.push(format!("{cmd} - {desc}"));
                }
            }

            // If no exact prefix matches, try fuzzy matching
            if suggestions.is_empty() {
                for (cmd, desc) in &all_commands {
                    if cmd.contains(input) {
                        suggestions.push(format!("{cmd} - {desc}"));
                    }
                }
            }
        }

        suggestions
    }
}

/// Theme colors
#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub text: Color,
    pub header_fg: Color,
    pub border: Color,
    pub active_border: Color,
    pub selection_bg: Color,
    pub status_bg: Color,
    pub status_fg: Color,
    pub primary_highlight: Color,
}

impl Theme {
    /// Get mode-specific color
    pub fn mode_color(&self, mode: Mode) -> Color {
        match mode {
            Mode::Normal => Color::Cyan,
            Mode::Insert => Color::Green,
            Mode::Visual => Color::Yellow,
            Mode::Command => Color::Magenta,
            Mode::Query => Color::Blue,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::Rgb(13, 13, 13),           // #0d0d0d
            text: Color::Rgb(255, 255, 255),              // #ffffff
            header_fg: Color::Rgb(203, 166, 247),         // #cba6f7
            border: Color::Rgb(49, 50, 68),               // #313244
            active_border: Color::Rgb(116, 199, 236),     // #74c7ec
            selection_bg: Color::Rgb(69, 71, 90),         // #45475a
            status_bg: Color::Rgb(49, 50, 68),            // #313244
            status_fg: Color::Rgb(205, 214, 244),         // #cdd6f4
            primary_highlight: Color::Rgb(116, 199, 236), // #74c7ec
        }
    }
}
