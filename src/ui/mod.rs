// FilePath: src/ui/mod.rs

use crate::{
    app::{AppState, FocusedPane},
    config::Config,
    constants,
    core::error::Result,
    database::ConnectionStatus,
};
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

pub mod components;
pub mod help;
pub mod layout;
pub mod theme;
pub mod widgets;

use layout::LayoutManager;
use theme::Theme;

/// Confirmation modal for destructive actions
#[derive(Debug, Clone)]
pub struct ConfirmationModal {
    pub title: String,
    pub message: String,
    pub action: ConfirmationAction,
}

/// Actions that can be confirmed
#[derive(Debug, Clone)]
pub enum ConfirmationAction {
    DeleteConnection(usize),
    DeleteTable(String),
    // Add more actions as needed
}

/// Main UI structure
pub struct UI {
    layout_manager: LayoutManager,
    pub theme: Theme,
}

impl UI {
    /// Create a new UI instance
    pub fn new(config: &Config) -> Result<Self> {
        let layout_manager = LayoutManager::new();

        // Load theme based on config or use default
        let theme = if !config.theme.name.is_empty() {
            // Try to load theme from available themes
            let themes = theme::ThemeLoader::list_available_themes();
            if let Some((_, path)) = themes.iter().find(|(name, _)| name == &config.theme.name) {
                Theme::load_from_file(path).unwrap_or_else(|e| {
                    tracing::warn!("Failed to load theme '{}': {}", config.theme.name, e);
                    Theme::default()
                })
            } else {
                tracing::warn!("Theme '{}' not found, using default", config.theme.name);
                Theme::default()
            }
        } else {
            Theme::default()
        };

        Ok(Self {
            layout_manager,
            theme,
        })
    }

    /// Calculate centered modal area
    fn render_confirmation_modal(&self, frame: &mut Frame, modal: &ConfirmationModal, area: Rect) {
        use ratatui::layout::{Direction, Layout, Margin};
        use ratatui::widgets::Clear;

        // Center the modal
        let modal_area = self.center_modal(area, 50, 30);

        // Clear the background
        frame.render_widget(Clear, modal_area);

        // Draw modal border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(format!(" {} ", modal.title))
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(block.clone(), modal_area);

        // Layout for modal content
        let inner = modal_area.inner(Margin::new(2, 1));
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Message
                Constraint::Length(1), // Empty line
                Constraint::Length(1), // Instructions
            ])
            .split(inner);

        // Render message
        let message = Paragraph::new(modal.message.clone())
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));
        frame.render_widget(message, chunks[0]);

        // Render instructions
        let instructions = Paragraph::new("Press Y to confirm, N or ESC to cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(instructions, chunks[2]);
    }

    fn center_modal(&self, area: Rect, width_percent: u16, height_percent: u16) -> Rect {
        let width = (area.width * width_percent / 100).min(area.width);
        let height = (area.height * height_percent / 100).min(area.height);
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;

        Rect {
            x: area.x + x,
            y: area.y + y,
            width,
            height,
        }
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
        use crate::ui::help::HelpSystem;
        HelpSystem::render_help(frame, state.ui.help_mode);

        // Cleanup expired toasts
        state.toast_manager.cleanup();

        // Draw toast notifications
        components::toast::render_toasts(frame, &state.toast_manager, frame.area(), &self.theme);

        // Command mode is handled internally, not shown in UI

        // Draw confirmation modal if active
        if let Some(modal) = &state.ui.confirmation_modal {
            self.render_confirmation_modal(frame, modal, frame.area());
        }

        // Draw connection modal if active (either add or edit)
        if state.ui.show_add_connection_modal || state.ui.show_edit_connection_modal {
            crate::ui::components::render_connection_modal(
                frame,
                &state.connection_modal_state,
                frame.area(),
            );
        }

        // Draw table creator if active
        if state.ui.show_table_creator {
            crate::ui::components::render_table_creator(
                frame,
                &mut state.table_creator_state,
                self.center_modal(frame.area(), 90, 80),
            );
        }

        // Draw table editor if active
        if state.ui.show_table_editor {
            crate::ui::components::render_table_editor(
                frame,
                &mut state.table_editor_state,
                self.center_modal(frame.area(), 90, 80),
            );
        }
    }

    /// Draw the header bar
    fn draw_header(&self, frame: &mut Frame, area: Rect, _state: &AppState) {
        let header = Paragraph::new(constants::version_string())
            .style(Style::default().fg(self.theme.get_color("header_fg")))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(self.theme.get_color("border"))),
            )
            .centered();

        frame.render_widget(header, area);
    }

    /// Draw the connections pane
    fn draw_connections_pane(&self, frame: &mut Frame, area: Rect, state: &mut AppState) {
        let is_focused = state.ui.focused_pane == FocusedPane::Connections;
        let border_style = if is_focused {
            Style::default().fg(self.theme.get_color("active_border"))
        } else {
            Style::default().fg(self.theme.get_color("border"))
        };

        // Create list items from stored connections
        let mut items: Vec<ListItem> = state
            .db
            .connections
            .connections
            .iter()
            .map(|connection| {
                // Get status symbol and color based on connection status
                let (symbol_style, text_style) = match &connection.status {
                    ConnectionStatus::Connected => (
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                        Style::default().fg(Color::Green),
                    ),
                    ConnectionStatus::Connecting => (
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                        Style::default().fg(Color::Yellow),
                    ),
                    ConnectionStatus::Failed(_) => (
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        Style::default().fg(Color::Red),
                    ),
                    ConnectionStatus::Disconnected => (
                        Style::default().fg(Color::DarkGray),
                        Style::default().fg(Color::Gray),
                    ),
                };

                // Format: "✓ CONN: ConnectionName, DB: database_name: Connected"
                let db_name = connection.database.as_deref().unwrap_or("default");
                let line = Line::from(vec![
                    Span::styled(format!("{} ", connection.status_symbol()), symbol_style),
                    Span::styled("CONN: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&connection.name, Style::default().fg(Color::White)),
                    Span::styled(", ", Style::default().fg(Color::DarkGray)),
                    Span::styled("DB: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(db_name, Style::default().fg(Color::Cyan)),
                    Span::styled(": ", Style::default().fg(Color::DarkGray)),
                    Span::styled(connection.status_text(), text_style),
                ]);

                ListItem::new(line)
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
                    "Enter/Space",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to connect", Style::default().fg(Color::Gray)),
            ])));
            items.push(ListItem::new(Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "x",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to disconnect", Style::default().fg(Color::Gray)),
            ])));
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
            if !state.db.connections.connections.is_empty() {
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
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "d",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" to delete connection", Style::default().fg(Color::Gray)),
                ])));

                // Show error message if the selected connection has failed
                if let Some(connection) = state
                    .db
                    .connections
                    .connections
                    .get(state.ui.selected_connection)
                {
                    if let Some(error) = connection.get_error() {
                        items.push(ListItem::new(""));
                        items.push(ListItem::new(Line::from(vec![
                            Span::styled(
                                "Error: ",
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(error, Style::default().fg(Color::Red)),
                        ])));
                    }
                }
            }
        }

        // Add legend for status symbols at the bottom if there are connections
        if !state.db.connections.connections.is_empty() && !is_focused {
            items.push(ListItem::new(""));
            items.push(ListItem::new(Line::from(vec![
                Span::styled("✓ ", Style::default().fg(Color::Green)),
                Span::styled(
                    "Connected  ",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled("— ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "Not connected  ",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled("✗ ", Style::default().fg(Color::Red)),
                Span::styled(
                    "Failed",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                ),
            ])));
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
                    .bg(self.theme.get_color("selection_bg"))
                    .add_modifier(Modifier::BOLD),
            );

        // Use stateful widget to show selection
        let mut list_state = state.ui.connections_list_state.clone();
        frame.render_stateful_widget(connections, area, &mut list_state);

        // Update the state with any changes
        state.ui.connections_list_state = list_state;
    }

    /// Draw the tables/views pane
    fn draw_tables_pane(&self, frame: &mut Frame, area: Rect, state: &mut AppState) {
        let is_focused = state.ui.focused_pane == FocusedPane::Tables;
        let border_style = if is_focused {
            Style::default().fg(self.theme.get_color("active_border"))
        } else {
            Style::default().fg(self.theme.get_color("border"))
        };

        // Check if there's an active connection
        let has_active_connection = state
            .db
            .connections
            .connections
            .iter()
            .any(|conn| conn.is_connected());

        let items: Vec<ListItem> = if !has_active_connection {
            // Show "not connected" message
            vec![
                ListItem::new(Line::from(vec![Span::styled(
                    "Choose a connection",
                    Style::default().fg(Color::Gray),
                )])),
                ListItem::new(""),
                ListItem::new(Line::from(vec![Span::styled(
                    "from the Connections pane",
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
                            "Ctrl+h",
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
        } else if state.db.tables.is_empty() {
            // Check connection status to show appropriate message
            let message = if let Some(connection) = state
                .db
                .connections
                .connections
                .get(state.ui.selected_connection)
            {
                match &connection.status {
                    ConnectionStatus::Connected => "No tables in database",
                    ConnectionStatus::Connecting => "Connecting to database...",
                    ConnectionStatus::Failed(_error) => "Connection failed (see status bar)",
                    ConnectionStatus::Disconnected => "Not connected",
                }
            } else {
                "No connection selected"
            };

            vec![ListItem::new(Line::from(vec![Span::styled(
                message,
                Style::default().fg(if message.contains("failed") {
                    Color::Red
                } else {
                    Color::Yellow
                }),
            )]))]
        } else {
            // Build list items - only actual selectable objects
            let mut table_items = Vec::new();

            // Only add actual table/view items that can be selected
            for table in &state.db.tables {
                // Determine icon and color based on database objects if available
                let (icon, color) = if let Some(ref db_objects) = state.db.database_objects {
                    // Find the object to get its type
                    if let Some(obj) = db_objects.tables.iter().find(|o| o.name == *table) {
                        (obj.object_type.icon(), Color::Blue)
                    } else if let Some(obj) = db_objects.views.iter().find(|o| o.name == *table) {
                        (obj.object_type.icon(), Color::Green)
                    } else if let Some(obj) = db_objects.materialized_views.iter().find(|o| o.name == *table) {
                        (obj.object_type.icon(), Color::Magenta)
                    } else {
                        ("📋", Color::Blue) // Default icon
                    }
                } else {
                    ("📋", Color::Blue) // Default icon
                };

                table_items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("  {} ", icon), Style::default().fg(color)),
                    Span::styled(table, Style::default().fg(Color::White)),
                ])));
            }

            // Add navigation help if focused
            if is_focused && !state.db.tables.is_empty() {
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
                table_items.push(ListItem::new(Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "n",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" to create new table", Style::default().fg(Color::Gray)),
                ])));
            }

            table_items
        };

        // Build title with object counts and schema info
        let title = if let Some(ref db_objects) = state.db.database_objects {
            let mut title_parts = Vec::new();

            // Add schema info if multiple schemas
            if state.db.schemas.len() > 1 {
                let schema = state.db.selected_schema.as_deref().unwrap_or("all");
                title_parts.push(format!("Schema: {}", schema));
            }

            // Add object counts
            let mut counts = Vec::new();
            if db_objects.tables.len() > 0 {
                counts.push(format!("{} tables", db_objects.tables.len()));
            }
            if db_objects.views.len() > 0 {
                counts.push(format!("{} views", db_objects.views.len()));
            }
            if db_objects.materialized_views.len() > 0 {
                counts.push(format!("{} mat. views", db_objects.materialized_views.len()));
            }

            if !counts.is_empty() {
                title_parts.push(counts.join(", "));
            }

            if !title_parts.is_empty() {
                format!(" Tables/Views ({}) ", title_parts.join(" | "))
            } else {
                " Tables/Views ".to_string()
            }
        } else {
            " Tables/Views ".to_string()
        };

        let tables = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .bg(self.theme.get_color("selection_bg"))
                    .add_modifier(Modifier::BOLD),
            );

        // Use stateful widget to show selection
        // Direct selection without offset since we're only showing selectable items
        if !state.db.tables.is_empty() && has_active_connection {
            state
                .ui
                .tables_list_state
                .select(Some(state.ui.selected_table));
        }
        frame.render_stateful_widget(tables, area, &mut state.ui.tables_list_state);
    }

    /// Draw the table details pane
    fn draw_details_pane(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.ui.focused_pane == FocusedPane::Details;
        let border_style = if is_focused {
            Style::default().fg(self.theme.get_color("active_border"))
        } else {
            Style::default().fg(self.theme.get_color("border"))
        };

        // Check if there's an active connection
        let has_active_connection = state
            .db
            .connections
            .connections
            .iter()
            .any(|conn| conn.is_connected());

        let details_text = if !has_active_connection {
            vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "No database connected",
                    Style::default().fg(Color::Gray),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Connect to database first",
                    Style::default().fg(Color::Gray),
                )]),
            ]
        } else if state.db.tables.is_empty() {
            vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "No tables in database",
                    Style::default().fg(Color::Yellow),
                )]),
            ]
        } else if state.ui.selected_table < state.db.tables.len() {
            // Get selected table info
            let selected_table = &state.db.tables[state.ui.selected_table];
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Selected: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        selected_table,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
            ];

            // Add object type if we have database objects
            if let Some(ref db_objects) = state.db.database_objects {
                // Find the object to determine its type
                let obj_type = if db_objects.tables.iter().any(|o| o.name == *selected_table) {
                    Some(("Table", Color::Blue))
                } else if db_objects.views.iter().any(|o| o.name == *selected_table) {
                    Some(("View", Color::Green))
                } else if db_objects.materialized_views.iter().any(|o| o.name == *selected_table) {
                    Some(("Materialized View", Color::Magenta))
                } else {
                    None
                };

                if let Some((type_name, color)) = obj_type {
                    lines.push(Line::from(vec![
                        Span::styled("Type: ", Style::default().fg(Color::Cyan)),
                        Span::styled(type_name, Style::default().fg(color)),
                    ]));
                }
            }

            // Show metadata if available
            if let Some(metadata) = &state.db.current_table_metadata {
            // Show actual table metadata
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Table: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        &metadata.table_name,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Rows: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("{}", metadata.row_count),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Columns: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        metadata.column_count.to_string(),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Total Size: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format_bytes(metadata.total_size),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Table Size: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format_bytes(metadata.table_size),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Indexes Size: ", Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format_bytes(metadata.indexes_size),
                        Style::default().fg(Color::White),
                    ),
                ]),
            ];

            // Add primary keys summary
            if !metadata.primary_keys.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("PKs: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        metadata.primary_keys.join(", "),
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            // Add indexes count
            if !metadata.indexes.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Indexes: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{} total", metadata.indexes.len()),
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            // Add comment if any
            if let Some(comment) = &metadata.comment {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    "Comment:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]));
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        comment,
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }

            } else {
                // Table selected but metadata not loaded yet
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    "Press Enter to load details",
                    Style::default().fg(Color::Yellow),
                )]));
            }

            // Add keyboard shortcuts
            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Actions:",
                Style::default().fg(Color::DarkGray),
            )]));
            lines.push(Line::from(vec![
                Span::styled("Enter: ", Style::default().fg(Color::Yellow)),
                Span::styled("View data", Style::default().fg(Color::Gray)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("e: ", Style::default().fg(Color::Yellow)),
                Span::styled("Edit structure", Style::default().fg(Color::Gray)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("d: ", Style::default().fg(Color::Yellow)),
                Span::styled("Drop table", Style::default().fg(Color::Gray)),
            ]));

            lines
        } else {
            vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "No table selected",
                    Style::default().fg(Color::Gray),
                )]),
            ]
        };

        let details = Paragraph::new(details_text)
            .block(
                Block::default()
                    .title(" Table Details ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .style(Style::default().fg(self.theme.get_color("text")));

        frame.render_widget(details, area);
    }

    /// Draw the tabular output area
    fn draw_tabular_output(&self, frame: &mut Frame, area: Rect, state: &mut AppState) {
        // Use table viewer if tables are open
        if !state.table_viewer_state.tabs.is_empty() {
            crate::ui::components::render_table_viewer(
                frame,
                &mut state.table_viewer_state,
                area,
                &self.theme,
            );
            return;
        }

        // Check if table creator is active
        if state.ui.show_table_creator {
            crate::ui::components::render_table_creator(
                frame,
                &mut state.table_creator_state,
                area,
            );
            return;
        }

        let is_focused = state.ui.focused_pane == FocusedPane::TabularOutput;
        let border_style = if is_focused {
            Style::default().fg(self.theme.get_color("active_border"))
        } else {
            Style::default().fg(self.theme.get_color("border"))
        };

        // Check if there's an active connection and table selected
        let has_active_connection = state
            .db
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
                .style(Style::default().fg(self.theme.get_color("text")))
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
                .style(Style::default().fg(self.theme.get_color("text")))
                .alignment(Alignment::Center);

            frame.render_widget(placeholder, area);
        } else {
            // Show actual table data (sample data for now)
            let header = Row::new(vec!["id", "name", "email", "created"])
                .style(Style::default().fg(self.theme.get_color("header_fg")))
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
                .row_highlight_style(Style::default().bg(self.theme.get_color("selection_bg")));

            frame.render_widget(table, area);
        }
    }

    /// Draw the SQL files browser pane
    fn draw_sql_files_pane(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.ui.focused_pane == FocusedPane::SqlFiles;
        let border_style = if is_focused {
            Style::default().fg(self.theme.get_color("active_border"))
        } else {
            Style::default().fg(self.theme.get_color("border"))
        };

        // Create list items from SQL files
        let mut items: Vec<ListItem> = state
            .saved_sql_files
            .iter()
            .enumerate()
            .map(|(i, filename)| {
                let prefix = if Some(filename) == state.ui.current_sql_file.as_ref() {
                    "● " // Indicate currently loaded file
                } else {
                    "  "
                };

                let style = if Some(filename) == state.ui.current_sql_file.as_ref() {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if i == state.ui.selected_sql_file && is_focused {
                    Style::default().fg(self.theme.get_color("primary_highlight"))
                } else {
                    Style::default().fg(self.theme.get_color("text"))
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
                    .bg(self.theme.get_color("selection_bg"))
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(sql_files, area);
    }

    /// Draw the query window area
    fn draw_query_window(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.ui.focused_pane == FocusedPane::QueryWindow;
        let border_style = if is_focused {
            Style::default().fg(self.theme.get_color("active_border"))
        } else {
            Style::default().fg(self.theme.get_color("border"))
        };

        // Check if there's an active connection for better help messages
        let has_active_connection = state
            .db
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
                    if i == state.ui.query_cursor_line {
                        // Highlight current line
                        let mut spans = Vec::new();
                        if state.ui.query_cursor_column > 0 {
                            spans.push(Span::raw(&line[..state.ui.query_cursor_column]));
                        }
                        if is_focused {
                            spans.push(Span::styled(
                                if state.ui.query_cursor_column < line.len() {
                                    &line[state.ui.query_cursor_column
                                        ..state.ui.query_cursor_column + 1]
                                } else {
                                    " "
                                },
                                Style::default().bg(Color::Gray).fg(Color::Black),
                            ));
                        }
                        if state.ui.query_cursor_column + 1 < line.len() {
                            spans.push(Span::raw(&line[state.ui.query_cursor_column + 1..]));
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
            let file_info = if let Some(ref filename) = state.ui.current_sql_file {
                let modified_indicator = if state.ui.query_modified { " [+]" } else { "" };
                format!("File: {filename}{modified_indicator}")
            } else {
                "New file (unsaved)".to_string()
            };

            query_lines.push(Line::from(vec![Span::styled(
                file_info,
                Style::default().fg(Color::Gray),
            )]));

            // Show vim mode and command
            let mode_info = if state.ui.in_vim_command {
                format!(":{}", state.ui.vim_command_buffer)
            } else {
                match state.ui.query_edit_mode {
                    crate::app::state::QueryEditMode::Normal => "-- NORMAL --".to_string(),
                    crate::app::state::QueryEditMode::Insert => "-- INSERT --".to_string(),
                }
            };

            query_lines.push(Line::from(vec![Span::styled(
                mode_info,
                Style::default()
                    .fg(
                        if state.ui.query_edit_mode == crate::app::state::QueryEditMode::Insert {
                            Color::Green
                        } else {
                            Color::Yellow
                        },
                    )
                    .add_modifier(Modifier::BOLD),
            )]));

            // Add keybinding help
            query_lines.push(Line::from(""));
            query_lines.push(Line::from(vec![
                Span::styled(
                    "i",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" insert | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    ":w",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" save | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    ":q",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" quit | ", Style::default().fg(Color::Gray)),
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
            .style(Style::default().fg(self.theme.get_color("text")))
            .wrap(Wrap { trim: false });

        frame.render_widget(query_editor, area);
    }

    /// Draw the status bar
    fn draw_status_bar(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let brand = format!("{} v{}", constants::APP_NAME, constants::VERSION);

        // Get real connection info
        let connection_text = if let Some(connection) = state
            .db
            .connections
            .connections
            .get(state.ui.selected_connection)
        {
            match &connection.status {
                ConnectionStatus::Connected => format!(
                    "Connected: {}@{}:{}",
                    connection.username, connection.host, connection.port
                ),
                ConnectionStatus::Connecting => format!("Connecting to {}...", connection.name),
                ConnectionStatus::Failed(_) => format!("Failed: {}", connection.name),
                ConnectionStatus::Disconnected => "Not connected".to_string(),
            }
        } else {
            "No connection selected".to_string()
        };

        // Get real position/context info
        let position_text = match state.ui.focused_pane {
            FocusedPane::Connections => format!(
                "Connection {}/{}",
                state.ui.selected_connection + 1,
                state.db.connections.connections.len()
            ),
            FocusedPane::Tables => {
                if state.db.tables.is_empty() {
                    "No tables".to_string()
                } else {
                    format!(
                        "Table {}/{}",
                        state.ui.selected_table + 1,
                        state.db.tables.len()
                    )
                }
            }
            FocusedPane::TabularOutput => {
                if let Some(tab) = state.table_viewer_state.current_tab() {
                    format!(
                        "Row {} Col {} | {}",
                        tab.selected_row + 1,
                        tab.selected_col + 1,
                        if tab.in_edit_mode {
                            "EDITING"
                        } else {
                            "READ-ONLY"
                        }
                    )
                } else {
                    "No table open".to_string()
                }
            }
            FocusedPane::QueryWindow => "Query Editor".to_string(),
            FocusedPane::SqlFiles => format!("SQL Files: {}", state.saved_sql_files.len()),
            FocusedPane::Details => "Table Details".to_string(),
        };

        // Get current date and time
        let now = chrono::Local::now();
        let datetime_text = now.format("%b %d, %Y  %H:%M:%S").to_string();

        // Add help hint when not showing help
        let help_hint = if state.ui.help_mode == crate::app::state::HelpMode::None {
            " | Press ? for help or q to quit"
        } else {
            ""
        };

        // Calculate the width of left side content
        let left_content = format!("{brand} | {connection_text} | {position_text}{help_hint}");

        // Calculate padding needed to right-align the date/time
        let available_width = area.width as usize;
        let left_width = left_content.len();
        let datetime_width = datetime_text.len();
        let padding_width = available_width.saturating_sub(left_width + datetime_width + 2); // 2 for margins

        let status_line = Line::from(vec![
            Span::styled(
                brand.as_str(),
                Style::default()
                    .fg(self.theme.get_color("primary_highlight"))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::raw(&connection_text),
            Span::raw(" | "),
            Span::raw(&position_text),
            Span::raw(help_hint),
            Span::raw(" ".repeat(padding_width)),
            Span::styled(
                datetime_text,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]);

        let status_bar = Paragraph::new(status_line).style(
            Style::default()
                .fg(self.theme.get_color("status_fg"))
                .bg(self.theme.get_color("status_bg")),
        );

        frame.render_widget(status_bar, area);
    }
}

/// Format bytes as human-readable size
fn format_bytes(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes = bytes.abs() as f64;
    let i = (bytes.ln() / 1024_f64.ln()).floor() as usize;
    let i = i.min(UNITS.len() - 1);
    let size = bytes / 1024_f64.powi(i as i32);

    if i == 0 {
        format!("{:.0} {}", size, UNITS[i])
    } else {
        format!("{:.2} {}", size, UNITS[i])
    }
}
