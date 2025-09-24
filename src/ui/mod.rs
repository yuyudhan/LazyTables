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
    DeleteSqlFile(usize),
    ExitApplication,
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

        // Render instructions with highlighted key bindings
        let instructions = Paragraph::new(Line::from(vec![
            Span::raw("Press "),
            Span::styled(
                "Y",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to confirm, "),
            Span::styled(
                "N",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" or "),
            Span::styled(
                "ESC",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" to cancel"),
        ]))
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

        // Get display connections (filtered or all)
        let display_indices = state
            .ui
            .get_display_connections(&state.db.connections.connections);

        // Create list items from connections to display
        let mut items: Vec<ListItem> = display_indices
            .iter()
            .filter_map(|&index| state.db.connections.connections.get(index))
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

                // Get database type icon (AC5 requirement)
                let db_type_icon = match connection.database_type {
                    crate::database::DatabaseType::PostgreSQL => "🐘",
                    crate::database::DatabaseType::MySQL => "🐬",
                    crate::database::DatabaseType::MariaDB => "🗄️",
                    crate::database::DatabaseType::SQLite => "📁",
                    crate::database::DatabaseType::Oracle => "🏛️",
                    crate::database::DatabaseType::Redis => "🔴",
                    crate::database::DatabaseType::MongoDB => "🍃",
                };

                // Format: "🐘 ✓ ConnectionName (postgresql) [DB: database_name] Connected"
                let db_name = connection.database.as_deref().unwrap_or("default");
                let db_type_name = connection.database_type.display_name();

                let line = Line::from(vec![
                    Span::styled(
                        format!("{} ", db_type_icon),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(format!("{} ", connection.status_symbol()), symbol_style),
                    Span::styled(
                        &connection.name,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" ({})", db_type_name),
                        Style::default().fg(Color::Blue),
                    ),
                    Span::styled(" [DB: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(db_name, Style::default().fg(Color::Cyan)),
                    Span::styled("] ", Style::default().fg(Color::DarkGray)),
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
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("Press ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        "/",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" to search connections", Style::default().fg(Color::Gray)),
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

        // Create title with search indicator
        let title = if state.ui.connections_search_active {
            format!(
                " Connections [SEARCH: {}] ",
                state.ui.connections_search_query
            )
        } else {
            " Connections ".to_string()
        };

        let connections = List::new(items)
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
        let mut list_state = state.ui.connections_list_state.clone();
        frame.render_stateful_widget(connections, area, &mut list_state);

        // Update the state with any changes
        state.ui.connections_list_state = list_state;
    }

    /// Draw the tables/views pane
    fn draw_tables_pane(&self, frame: &mut Frame, area: Rect, state: &mut AppState) {
        // Use the dedicated TablesPane component with database-adaptive features
        components::render_tables_pane(frame, area, state, &self.theme);
    }

    /// Draw the enhanced table details pane with comprehensive metadata
    fn draw_details_pane(&self, frame: &mut Frame, area: Rect, state: &mut AppState) {
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
        } else if let Some(selected_table_name) = state.ui.get_selected_table_name() {
            self.build_comprehensive_table_details(
                selected_table_name,
                &state.db,
                &state.ui,
                is_focused,
            )
        } else {
            vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "No table selected",
                    Style::default().fg(Color::Gray),
                )]),
            ]
        };

        // Apply scrolling if content is too long
        let content_height = details_text.len();
        let available_height = area.height.saturating_sub(2) as usize; // Account for borders

        // Store content dimensions for scroll bounds checking
        state.ui.details_content_height = content_height;
        state.ui.details_viewport_height = available_height;
        state.ui.details_max_scroll_offset = content_height.saturating_sub(available_height);

        let visible_lines = if content_height > available_height {
            let start = state
                .ui
                .details_viewport_offset
                .min(content_height.saturating_sub(available_height));
            let end = (start + available_height).min(content_height);
            details_text[start..end].to_vec()
        } else {
            details_text
        };

        // Create title with scroll indicator
        let title = if content_height > available_height {
            let scroll_info = format!(
                "Table Details [{}/{}]",
                state.ui.details_viewport_offset + 1,
                content_height.saturating_sub(available_height) + 1
            );
            scroll_info
        } else {
            "Table Details".to_string()
        };

        let details = Paragraph::new(visible_lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .style(Style::default().fg(self.theme.get_color("text")));

        frame.render_widget(details, area);
    }

    /// Build comprehensive table details with all available metadata
    fn build_comprehensive_table_details(
        &self,
        table_name: String,
        db_state: &crate::state::DatabaseState,
        _ui_state: &crate::state::UIState,
        is_focused: bool,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Define colors based on focus state
        let label_color = if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        let text_color = if is_focused {
            Color::White
        } else {
            Color::Gray
        };

        // === HEADER SECTION ===
        lines.push(Line::from(vec![
            Span::styled("Object: ".to_string(), Style::default().fg(label_color)),
            Span::styled(
                table_name.clone(),
                Style::default().fg(text_color).add_modifier(if is_focused {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
            ),
        ]));

        // Determine table type
        let table_type = if let Some(ref db_objects) = db_state.database_objects {
            if db_objects
                .tables
                .iter()
                .any(|t| t.name == table_name || t.qualified_name() == table_name)
            {
                "Table"
            } else if db_objects
                .views
                .iter()
                .any(|v| v.name == table_name || v.qualified_name() == table_name)
            {
                "View"
            } else if db_objects
                .materialized_views
                .iter()
                .any(|mv| mv.name == table_name || mv.qualified_name() == table_name)
            {
                "Materialized View"
            } else {
                "Unknown"
            }
        } else {
            "Table"
        };

        lines.push(Line::from(vec![
            Span::styled("Type: ".to_string(), Style::default().fg(label_color)),
            Span::styled(
                table_type.to_string(),
                Style::default().fg(if is_focused {
                    match table_type {
                        "Table" => Color::Blue,
                        "View" => Color::Green,
                        "Materialized View" => Color::Magenta,
                        _ => Color::Gray,
                    }
                } else {
                    Color::DarkGray
                }),
            ),
        ]));

        lines.push(Line::from("".to_string()));

        // === METADATA SECTION ===
        if let Some(metadata) = &db_state.current_table_metadata {
            let section_color = if is_focused {
                Color::Yellow
            } else {
                Color::DarkGray
            };

            // Basic metrics
            lines.push(Line::from(vec![Span::styled(
                "📊 Metrics".to_string(),
                Style::default()
                    .fg(section_color)
                    .add_modifier(if is_focused {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            )]));

            lines.push(Line::from(vec![
                Span::styled("  Rows: ".to_string(), Style::default().fg(label_color)),
                Span::styled(
                    metadata.row_count.to_string(),
                    Style::default().fg(text_color),
                ),
            ]));

            lines.push(Line::from(vec![
                Span::styled("  Columns: ".to_string(), Style::default().fg(label_color)),
                Span::styled(
                    metadata.column_count.to_string(),
                    Style::default().fg(text_color),
                ),
            ]));

            // Storage information
            lines.push(Line::from("".to_string()));
            lines.push(Line::from(vec![Span::styled(
                "💾 Storage".to_string(),
                Style::default()
                    .fg(section_color)
                    .add_modifier(if is_focused {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            )]));

            lines.push(Line::from(vec![
                Span::styled(
                    "  Total Size: ".to_string(),
                    Style::default().fg(label_color),
                ),
                Span::styled(
                    crate::database::TableMetadata::format_size(metadata.total_size),
                    Style::default().fg(text_color),
                ),
            ]));

            lines.push(Line::from(vec![
                Span::styled(
                    "  Table Size: ".to_string(),
                    Style::default().fg(label_color),
                ),
                Span::styled(
                    crate::database::TableMetadata::format_size(metadata.table_size),
                    Style::default().fg(text_color),
                ),
            ]));

            lines.push(Line::from(vec![
                Span::styled(
                    "  Indexes Size: ".to_string(),
                    Style::default().fg(label_color),
                ),
                Span::styled(
                    crate::database::TableMetadata::format_size(metadata.indexes_size),
                    Style::default().fg(text_color),
                ),
            ]));

            // Schema relationships
            lines.push(Line::from("".to_string()));
            lines.push(Line::from(vec![Span::styled(
                "🔗 Relationships".to_string(),
                Style::default()
                    .fg(section_color)
                    .add_modifier(if is_focused {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            )]));

            if !metadata.primary_keys.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(
                        "  Primary Keys: ".to_string(),
                        Style::default().fg(label_color),
                    ),
                    Span::styled(
                        metadata.primary_keys.join(", "),
                        Style::default().fg(text_color),
                    ),
                ]));
            }

            if !metadata.foreign_keys.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(
                        "  Foreign Keys: ".to_string(),
                        Style::default().fg(label_color),
                    ),
                    Span::styled(
                        format!("{} relationships", metadata.foreign_keys.len()),
                        Style::default().fg(text_color),
                    ),
                ]));
            }

            if !metadata.indexes.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("  Indexes: ".to_string(), Style::default().fg(label_color)),
                    Span::styled(
                        format!("{} total", metadata.indexes.len()),
                        Style::default().fg(text_color),
                    ),
                ]));
            }

            // Add comment if any
            if let Some(ref comment) = metadata.comment {
                lines.push(Line::from("".to_string()));
                lines.push(Line::from(vec![Span::styled(
                    "💬 Comment".to_string(),
                    Style::default()
                        .fg(section_color)
                        .add_modifier(if is_focused {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                )]));
                lines.push(Line::from(vec![
                    Span::styled("  ".to_string(), Style::default()),
                    Span::styled(
                        comment.clone(),
                        Style::default()
                            .fg(if is_focused {
                                Color::Gray
                            } else {
                                Color::DarkGray
                            })
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }
        } else {
            // No metadata loaded yet
            lines.push(Line::from(vec![Span::styled(
                "Press Enter to load detailed metadata".to_string(),
                Style::default().fg(if is_focused {
                    Color::Yellow
                } else {
                    Color::DarkGray
                }),
            )]));
        }

        // === ACTIONS SECTION ===
        lines.push(Line::from("".to_string()));
        lines.push(Line::from("".to_string()));
        lines.push(Line::from(vec![Span::styled(
            "⌨️  Actions".to_string(),
            Style::default()
                .fg(if is_focused {
                    Color::DarkGray
                } else {
                    Color::Black
                })
                .add_modifier(if is_focused {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        )]));

        let actions = vec![("r", "Refresh metadata")];

        for (key, desc) in actions {
            lines.push(Line::from(vec![
                Span::styled("  ".to_string(), Style::default()),
                Span::styled(
                    format!("{}: ", key),
                    Style::default().fg(if is_focused {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::styled(
                    desc.to_string(),
                    Style::default().fg(if is_focused {
                        Color::Gray
                    } else {
                        Color::DarkGray
                    }),
                ),
            ]));
        }

        lines
    }

    /// Draw the tabular output area
    fn draw_tabular_output(&self, frame: &mut Frame, area: Rect, state: &mut AppState) {
        // Use table viewer if tables are open
        if !state.table_viewer_state.tabs.is_empty() {
            let is_focused = state.ui.focused_pane == FocusedPane::TabularOutput;
            crate::ui::components::render_table_viewer(
                frame,
                &mut state.table_viewer_state,
                area,
                &self.theme,
                is_focused,
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

        // Get filtered files list for display
        let display_files = state.get_filtered_sql_files();
        let selected_index = state.get_filtered_sql_file_selection();

        // Create list items from SQL files
        let mut items: Vec<ListItem> = display_files
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
                } else if i == selected_index && is_focused {
                    Style::default().fg(self.theme.get_color("primary_highlight"))
                } else {
                    Style::default().fg(self.theme.get_color("text"))
                };

                // Add file metadata if focused and not in input mode
                let file_display = if is_focused
                    && !state.ui.sql_files_search_active
                    && !state.ui.sql_files_rename_mode
                    && !state.ui.sql_files_create_mode
                {
                    // Get file size and modification time
                    let connection_name = if let Some(connection) = state
                        .db
                        .connections
                        .connections
                        .get(state.ui.selected_connection)
                    {
                        connection.name.clone()
                    } else {
                        "default".to_string()
                    };

                    let connection_dir =
                        crate::config::Config::sql_files_dir().join(&connection_name);
                    let root_dir = crate::config::Config::sql_files_dir();

                    let connection_path = connection_dir.join(format!("{filename}.sql"));
                    let root_path = root_dir.join(format!("{filename}.sql"));

                    let (size_str, modified_str) = if connection_path.exists() {
                        self.get_file_metadata(&connection_path)
                    } else if root_path.exists() {
                        self.get_file_metadata(&root_path)
                    } else {
                        ("?".to_string(), "?".to_string())
                    };

                    format!("{prefix}{filename}.sql  [{size_str}] {modified_str}")
                } else {
                    format!("{prefix}{filename}.sql")
                };

                ListItem::new(Line::from(vec![Span::styled(file_display, style)]))
            })
            .collect();

        // Handle input modes
        if state.ui.sql_files_search_active && is_focused {
            items.insert(
                0,
                ListItem::new(Line::from(vec![
                    Span::styled("Search: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        &state.ui.sql_files_search_query,
                        Style::default().fg(Color::White),
                    ),
                    Span::styled("_", Style::default().fg(Color::Gray)),
                ])),
            );
            items.insert(1, ListItem::new(""));
        } else if state.ui.sql_files_rename_mode && is_focused {
            items.insert(
                0,
                ListItem::new(Line::from(vec![
                    Span::styled("Rename to: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        &state.ui.sql_files_rename_buffer,
                        Style::default().fg(Color::White),
                    ),
                    Span::styled("_", Style::default().fg(Color::Gray)),
                ])),
            );
            items.insert(1, ListItem::new(""));
        } else if state.ui.sql_files_create_mode && is_focused {
            items.insert(
                0,
                ListItem::new(Line::from(vec![
                    Span::styled("New file: ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        &state.ui.sql_files_create_buffer,
                        Style::default().fg(Color::White),
                    ),
                    Span::styled("_", Style::default().fg(Color::Gray)),
                ])),
            );
            items.insert(1, ListItem::new(""));
        }

        // Add instruction text if no files exist
        if display_files.is_empty() && !state.ui.sql_files_create_mode {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "No SQL files found",
                Style::default().fg(Color::Gray),
            )])));
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "Press 'n' to create files",
                Style::default().fg(Color::Gray),
            )])));
        } else if is_focused
            && !state.ui.sql_files_search_active
            && !state.ui.sql_files_rename_mode
            && !state.ui.sql_files_create_mode
        {
            // Add keybinding help when focused
            items.push(ListItem::new(""));
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" load | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "n",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" new | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "r",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" rename", Style::default().fg(Color::Gray)),
            ])));
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    "d",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" delete | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "c",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" copy | ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "/",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" search", Style::default().fg(Color::Gray)),
            ])));
        }

        // Create title with search/mode indicator
        let title = if state.ui.sql_files_search_active {
            format!(" SQL Files [SEARCH: {}] ", state.ui.sql_files_search_query)
        } else if state.ui.sql_files_rename_mode {
            " SQL Files [RENAME] ".to_string()
        } else if state.ui.sql_files_create_mode {
            " SQL Files [CREATE] ".to_string()
        } else {
            format!(" SQL Files ({}) ", display_files.len())
        };

        let sql_files = List::new(items)
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

        frame.render_widget(sql_files, area);
    }

    /// Get file metadata as formatted strings
    fn get_file_metadata(&self, path: &std::path::Path) -> (String, String) {
        use std::fs;
        use std::time::SystemTime;

        match fs::metadata(path) {
            Ok(metadata) => {
                // Format file size
                let size = metadata.len();
                let size_str = if size < 1024 {
                    format!("{}B", size)
                } else if size < 1024 * 1024 {
                    format!("{:.1}KB", size as f64 / 1024.0)
                } else {
                    format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
                };

                // Format modification time
                let modified_str = metadata
                    .modified()
                    .map(|time| {
                        let duration = SystemTime::now().duration_since(time).unwrap_or_default();
                        let secs = duration.as_secs();
                        if secs < 60 {
                            "now".to_string()
                        } else if secs < 3600 {
                            format!("{}m", secs / 60)
                        } else if secs < 86400 {
                            format!("{}h", secs / 3600)
                        } else {
                            format!("{}d", secs / 86400)
                        }
                    })
                    .unwrap_or_else(|_| "?".to_string());

                (size_str, modified_str)
            }
            Err(_) => ("?".to_string(), "?".to_string()),
        }
    }

    /// Draw the query window area
    fn draw_query_window(&self, frame: &mut Frame, area: Rect, state: &mut AppState) {
        let is_focused = state.ui.focused_pane == FocusedPane::QueryWindow;
        let border_style = if is_focused {
            Style::default().fg(self.theme.get_color("active_border"))
        } else {
            Style::default().fg(self.theme.get_color("border"))
        };

        // Calculate the available height for the editor (accounting for borders and status)
        // Reserve lines for: border(2) + mode/file info(4 if focused)
        let reserved_lines = if is_focused { 6 } else { 2 };
        let available_height = area.height.saturating_sub(reserved_lines) as usize;

        // Update the viewport height in state
        state.ui.query_viewport_height = available_height;

        // Check if there's an active connection for better help messages
        let has_active_connection = state
            .db
            .connections
            .connections
            .iter()
            .any(|conn| conn.is_connected());

        // Get query content lines with viewport
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
            // Collect all lines into a vector for viewport calculation
            let all_lines: Vec<&str> = state.query_content.lines().collect();
            let total_lines = all_lines.len();

            // Calculate the visible range
            let viewport_start = state.ui.query_viewport_offset;
            let viewport_end = (viewport_start + available_height).min(total_lines);

            // Get only the visible lines
            all_lines[viewport_start..viewport_end]
                .iter()
                .enumerate()
                .map(|(relative_idx, line)| {
                    let actual_line_idx = viewport_start + relative_idx;

                    // Add line numbers for better navigation visibility
                    let line_number = format!("{:>4} ", actual_line_idx + 1);
                    let line_number_style = if actual_line_idx == state.ui.query_cursor_line {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let mut spans = vec![Span::styled(line_number, line_number_style)];

                    // Render the line content with cursor if needed
                    if actual_line_idx == state.ui.query_cursor_line && is_focused {
                        // Highlight current line with cursor
                        if state.ui.query_cursor_column > 0
                            && state.ui.query_cursor_column <= line.len()
                        {
                            spans.push(Span::raw(&line[..state.ui.query_cursor_column]));
                        }

                        // Render cursor
                        let cursor_char = if state.ui.query_cursor_column < line.len() {
                            &line[state.ui.query_cursor_column..state.ui.query_cursor_column + 1]
                        } else {
                            " "
                        };

                        let cursor_style = if state.ui.query_edit_mode
                            == crate::app::state::QueryEditMode::Insert
                        {
                            Style::default().bg(Color::Green).fg(Color::Black)
                        } else {
                            Style::default().bg(Color::Gray).fg(Color::Black)
                        };

                        spans.push(Span::styled(cursor_char, cursor_style));

                        if state.ui.query_cursor_column + 1 < line.len() {
                            spans.push(Span::raw(&line[state.ui.query_cursor_column + 1..]));
                        } else if state.ui.query_cursor_column > 0
                            && state.ui.query_cursor_column == line.len()
                        {
                            // Cursor is at end of line - already handled above
                        }
                    } else {
                        // Normal line without cursor
                        spans.push(Span::raw(*line));
                    }

                    Line::from(spans)
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

        // Create title with scroll indicator
        let total_lines = state.query_content.lines().count();
        let title = if total_lines > 0 {
            let current_line = state.ui.query_cursor_line + 1;
            let scroll_percent = if total_lines > 1 {
                ((state.ui.query_viewport_offset as f32
                    / (total_lines.saturating_sub(available_height).max(1)) as f32)
                    * 100.0) as u32
            } else {
                0
            };
            format!(
                " SQL Query Editor [{}:{}/{}] {}% ",
                current_line,
                state.ui.query_cursor_column + 1,
                total_lines,
                scroll_percent
            )
        } else {
            " SQL Query Editor ".to_string()
        };

        let query_editor = Paragraph::new(query_lines)
            .block(
                Block::default()
                    .title(title)
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

        // Get real position/context info with explicit pane name
        let position_text = match state.ui.focused_pane {
            FocusedPane::Connections => format!(
                "[CONNECTIONS] Connection {}/{}",
                state.ui.selected_connection + 1,
                state.db.connections.connections.len()
            ),
            FocusedPane::Tables => {
                if state.db.tables.is_empty() {
                    "[TABLES] No tables".to_string()
                } else {
                    format!(
                        "[TABLES] Table {}/{}",
                        state.ui.selected_table + 1,
                        state.db.tables.len()
                    )
                }
            }
            FocusedPane::TabularOutput => {
                if let Some(tab) = state.table_viewer_state.current_tab() {
                    format!(
                        "[TABLE_VIEWER] Row {} Col {} | {}",
                        tab.selected_row + 1,
                        tab.selected_col + 1,
                        if tab.in_edit_mode {
                            "EDITING"
                        } else {
                            "READ-ONLY"
                        }
                    )
                } else {
                    "[TABLE_VIEWER] No table open".to_string()
                }
            }
            FocusedPane::QueryWindow => "[QUERY_EDITOR] Active".to_string(),
            FocusedPane::SqlFiles => format!("[SQL_FILES] {} files", state.saved_sql_files.len()),
            FocusedPane::Details => "[DETAILS] Table Details".to_string(),
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
