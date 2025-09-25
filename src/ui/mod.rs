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

    /// Render modal overlay background
    fn render_modal_overlay(&self, frame: &mut Frame, area: Rect) {
        // Create a dimmed overlay effect using the theme's background color
        // This maintains the dark theme elegance without the whitish artifact
        let overlay =
            Block::default().style(Style::default().bg(self.theme.get_color("background")));
        frame.render_widget(overlay, area);
    }

    /// Calculate centered modal area
    fn render_confirmation_modal(&self, frame: &mut Frame, modal: &ConfirmationModal, area: Rect) {
        use ratatui::layout::{Direction, Layout, Margin};
        use ratatui::widgets::Clear;

        // Render modal overlay background first
        self.render_modal_overlay(frame, area);

        // Center the modal
        let modal_area = self.center_modal(area, 50, 30);

        // Clear the modal area specifically
        frame.render_widget(Clear, modal_area);

        // Draw modal border with proper background
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.get_color("modal_border")))
            .style(
                Style::default()
                    .bg(self.theme.get_color("modal_bg"))
                    .fg(Color::White),
            )
            .title(format!(" {} ", modal.title))
            .title_style(
                Style::default()
                    .fg(self.theme.get_color("modal_title"))
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
        // Clear the frame to prevent artifacts
        frame.render_widget(ratatui::widgets::Clear, frame.area());

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
        HelpSystem::render_help(frame, &state.ui);

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
                state.ui.show_edit_connection_modal, // Pass edit mode flag
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

        // Draw connection mode if active (full-screen overlay)
        if state.ui.show_connection_mode {
            if let Some(connection_mode) = &state.connection_mode {
                connection_mode.render(
                    frame,
                    frame.area(),
                    &self.theme,
                    state.ui.connection_mode_type,
                    state.ui.connection_mode_scroll_offset,
                );
            }
        }

        // Draw debug view if active (full-screen overlay)
        if state.ui.show_debug_view {
            let debug_messages = crate::logging::get_debug_messages();
            state.debug_view.render(
                frame,
                frame.area(),
                &self.theme,
                &debug_messages,
                state.ui.debug_view_scroll_offset,
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

        // Show error message if the selected connection has failed and we're focused
        if is_focused {
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
                "No metadata loaded yet".to_string(),
                Style::default().fg(Color::Gray),
            )]));
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
        let sql_panes_enabled = state.are_sql_panes_enabled();

        let border_style = if !sql_panes_enabled {
            // Show disabled state with gray border
            Style::default().fg(Color::DarkGray)
        } else if is_focused {
            Style::default().fg(self.theme.get_color("active_border"))
        } else {
            Style::default().fg(self.theme.get_color("border"))
        };

        // Get filtered files list for display (empty if disabled)
        let display_files = if sql_panes_enabled {
            state.get_filtered_sql_files()
        } else {
            Vec::new()
        };
        let selected_index = state.get_filtered_sql_file_selection();

        // Create list items from SQL files
        let mut items: Vec<ListItem> = if sql_panes_enabled {
            display_files
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
                .collect()
        } else {
            Vec::new()
        };

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

        // Add message if no files exist or SQL panes are disabled
        if !sql_panes_enabled {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "🔒 Connect to database",
                Style::default().fg(Color::DarkGray),
            )])));
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "   to access SQL files",
                Style::default().fg(Color::DarkGray),
            )])));
        } else if display_files.is_empty() && !state.ui.sql_files_create_mode {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "No SQL files found",
                Style::default().fg(Color::Gray),
            )])));
        }

        // Create title with search/mode/disabled indicator
        let title = if !sql_panes_enabled {
            " SQL Files [DISABLED] ".to_string()
        } else if state.ui.sql_files_search_active {
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

    /// Draw the query window area using the QueryEditor component
    fn draw_query_window(&self, frame: &mut Frame, area: Rect, state: &mut AppState) {
        let is_focused = state.ui.focused_pane == FocusedPane::QueryWindow;
        let sql_panes_enabled = state.are_sql_panes_enabled();
        let query_editor_enabled = state.is_query_editor_enabled();

        if !query_editor_enabled {
            let disabled_block = Block::default()
                .title(" SQL Query Editor [DISABLED] ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));

            let disabled_message = if !sql_panes_enabled {
                // No connection
                Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "🔒 Connect to a database to enable SQL editing",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Select a connection and press Enter to connect",
                        Style::default().fg(Color::DarkGray),
                    )]),
                ])
            } else {
                // Connected but no file selected
                Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "📄 Select an SQL file to start editing",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Navigate to SQL Files pane and press Enter on a file",
                        Style::default().fg(Color::DarkGray),
                    )]),
                    Line::from(vec![Span::styled(
                        "or press 'n' to create a new SQL file",
                        Style::default().fg(Color::DarkGray),
                    )]),
                ])
            };

            let final_message = disabled_message
                .block(disabled_block)
                .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(final_message, area);
            return;
        }

        // Update the QueryEditor component state
        state.query_editor.set_focused(is_focused);
        state
            .query_editor
            .set_insert_mode(state.ui.query_edit_mode == crate::app::state::QueryEditMode::Insert);

        // Set database type if we have an active connection
        if let Some(connection) = state.get_selected_connection() {
            state
                .query_editor
                .set_database_type(Some(connection.database_type.clone()));
        }

        // Set file info
        state
            .query_editor
            .set_current_file(state.ui.current_sql_file.clone());

        // Sync content between legacy state and QueryEditor
        if state.query_editor.get_content() != state.query_content {
            state.query_editor.set_content(state.query_content.clone());
        }

        // Set available tables and columns for suggestions
        if let Some(db_objects) = &state.db.database_objects {
            let table_names: Vec<String> =
                db_objects.tables.iter().map(|t| t.name.clone()).collect();
            state.query_editor.set_tables(table_names);
        }

        // Set columns for current table if available
        if let Some(metadata) = &state.db.current_table_metadata {
            let column_names: Vec<String> = metadata
                .columns_summary
                .iter()
                .map(|c| c.name.clone())
                .collect();
            state
                .query_editor
                .set_table_columns(metadata.table_name.clone(), column_names);
        }

        // Render the QueryEditor component
        state.query_editor.render(frame, area);

        // Sync content back to legacy state if it was modified
        let new_content = state.query_editor.get_content().to_string();
        if new_content != state.query_content {
            state.query_content = new_content;
            state.ui.query_modified = state.query_editor.is_modified();
        }
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
