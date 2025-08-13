// FilePath: src/ui/mod.rs

use crate::{
    app::{AppState, FocusedPane, Mode},
    config::Config,
    constants,
    core::error::Result,
};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table},
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

        // Draw main content area
        self.draw_main_content(frame, areas.main_content, state);

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
    fn draw_connections_pane(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.focused_pane == FocusedPane::Connections;
        let border_style = if is_focused {
            Style::default().fg(self.theme.active_border)
        } else {
            Style::default().fg(self.theme.border)
        };

        let items: Vec<ListItem> = vec![
            ListItem::new("▼ production"),
            ListItem::new("  ● localhost"),
            ListItem::new("  ○ staging_db"),
            ListItem::new("▶ development"),
            ListItem::new("  local_dev"),
            ListItem::new(""),
            ListItem::new("[+] Add New"),
        ];

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

        frame.render_widget(connections, area);
    }

    /// Draw the tables/views pane
    fn draw_tables_pane(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.focused_pane == FocusedPane::Tables;
        let border_style = if is_focused {
            Style::default().fg(self.theme.active_border)
        } else {
            Style::default().fg(self.theme.border)
        };

        let items: Vec<ListItem> = vec![
            ListItem::new("▼ public"),
            ListItem::new("  ▶ users"),
            ListItem::new("  ▶ products"),
            ListItem::new("  ▶ orders"),
            ListItem::new("  ▶ payments"),
            ListItem::new("▼ analytics"),
            ListItem::new("  events"),
            ListItem::new("  sessions"),
            ListItem::new("  metrics"),
        ];

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

        frame.render_widget(tables, area);
    }

    /// Draw the table details pane
    fn draw_details_pane(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.focused_pane == FocusedPane::Details;
        let border_style = if is_focused {
            Style::default().fg(self.theme.active_border)
        } else {
            Style::default().fg(self.theme.border)
        };

        let details_text = vec![
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
        ];

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

    /// Draw the main content area
    fn draw_main_content(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let is_focused = state.focused_pane == FocusedPane::MainContent;
        let border_style = if is_focused {
            Style::default().fg(self.theme.active_border)
        } else {
            Style::default().fg(self.theme.border)
        };

        // Sample table data
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
                    .title(" Main Content ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .row_highlight_style(Style::default().bg(self.theme.selection_bg));

        frame.render_widget(table, area);
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
            Line::from(vec![
                Span::styled(format!("{} Help", constants::version_string()), Style::default().fg(self.theme.header_fg).add_modifier(Modifier::BOLD)),
            ]).alignment(Alignment::Center),
            Line::from(""),
            Line::from(vec![Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))]),
            Line::from("  h/j/k/l        Navigate within panes"),
            Line::from("  Ctrl+h/j/k/l   Switch between panes"),
            Line::from("  Tab            Cycle through panes forward"),
            Line::from("  Shift+Tab      Cycle through panes backward"),
            Line::from("  gg/G           Jump to first/last row"),
            Line::from("  0/$            Jump to first/last column"),
            Line::from(""),
            Line::from(vec![Span::styled("Modes", Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))]),
            Line::from("  i              Enter Insert mode (edit cells)"),
            Line::from("  v              Enter Visual mode (selection)"),
            Line::from("  :              Enter Command mode"),
            Line::from("  Space z q      Enter Query mode (SQL editor)"),
            Line::from("  ESC            Return to Normal mode"),
            Line::from(""),
            Line::from(vec![Span::styled("Leader Commands (Space)", Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))]),
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
            Line::from(vec![Span::styled("General", Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))]),
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
            ]).alignment(Alignment::Center),
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
                    Style::default().fg(self.theme.text).add_modifier(Modifier::DIM)
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
                    suggestions.push(format!("{} - {}", cmd, desc));
                }
            }
            
            // If no exact prefix matches, try fuzzy matching
            if suggestions.is_empty() {
                for (cmd, desc) in &all_commands {
                    if cmd.contains(input) {
                        suggestions.push(format!("{} - {}", cmd, desc));
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
            background: Color::Rgb(13, 13, 13),       // #0d0d0d
            text: Color::Rgb(255, 255, 255),          // #ffffff
            header_fg: Color::Rgb(203, 166, 247),     // #cba6f7
            border: Color::Rgb(49, 50, 68),           // #313244
            active_border: Color::Rgb(116, 199, 236), // #74c7ec
            selection_bg: Color::Rgb(69, 71, 90),     // #45475a
            status_bg: Color::Rgb(49, 50, 68),        // #313244
            status_fg: Color::Rgb(205, 214, 244),     // #cdd6f4
            primary_highlight: Color::Rgb(116, 199, 236), // #74c7ec
        }
    }
}

