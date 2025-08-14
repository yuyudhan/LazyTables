// FilePath: src/ui/help.rs

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::state::HelpMode;

/// Help content for each pane
pub struct HelpSystem;

impl HelpSystem {
    /// Create the left column content (current pane + global)
    pub fn create_left_column(mode: HelpMode) -> Vec<Line<'static>> {
        let mut lines = vec![];

        // Current pane header
        let pane_name = match mode {
            HelpMode::Connections => "Connections",
            HelpMode::Tables => "Tables",
            HelpMode::Details => "Table Details",
            HelpMode::TabularOutput => "Table Viewer",
            HelpMode::SqlFiles => "SQL Files",
            HelpMode::QueryWindow => "Query Editor",
            HelpMode::None => "LazyTables",
        };

        lines.push(Line::from(vec![Span::styled(
            format!("━━ {} Commands ", pane_name),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        // Add pane-specific commands for current pane
        match mode {
            HelpMode::Connections => Self::add_connections_commands(&mut lines),
            HelpMode::Tables => Self::add_tables_commands(&mut lines),
            HelpMode::Details => Self::add_details_commands(&mut lines),
            HelpMode::TabularOutput => Self::add_tabular_commands(&mut lines),
            HelpMode::SqlFiles => Self::add_sql_files_commands(&mut lines),
            HelpMode::QueryWindow => Self::add_query_window_commands(&mut lines),
            HelpMode::None => {}
        }

        // Add separator
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "━━ Global Commands ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        // Global commands
        Self::add_command(&mut lines, "q", "Quit LazyTables");
        Self::add_command(&mut lines, "?", "Toggle help");
        Self::add_command(&mut lines, "Tab", "Next pane");
        Self::add_command(&mut lines, "S-Tab", "Previous pane");
        lines.push(Line::from(""));
        Self::add_command(&mut lines, "C-h", "Focus left");
        Self::add_command(&mut lines, "C-j", "Focus down");
        Self::add_command(&mut lines, "C-k", "Focus up");
        Self::add_command(&mut lines, "C-l", "Focus right");

        lines
    }

    /// Create the right column content (all panes overview)
    pub fn create_right_column(current_mode: HelpMode) -> Vec<Line<'static>> {
        let mut lines = vec![];

        lines.push(Line::from(vec![Span::styled(
            "━━ All Panes Overview ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        // Add each pane's key commands
        Self::add_pane_overview(
            &mut lines,
            "Connections",
            HelpMode::Connections,
            current_mode,
        );
        Self::add_pane_overview(&mut lines, "Tables", HelpMode::Tables, current_mode);
        Self::add_pane_overview(&mut lines, "Details", HelpMode::Details, current_mode);
        Self::add_pane_overview(
            &mut lines,
            "Table Viewer",
            HelpMode::TabularOutput,
            current_mode,
        );
        Self::add_pane_overview(&mut lines, "SQL Files", HelpMode::SqlFiles, current_mode);
        Self::add_pane_overview(
            &mut lines,
            "Query Editor",
            HelpMode::QueryWindow,
            current_mode,
        );

        lines
    }

    /// Add a pane overview section
    fn add_pane_overview(
        lines: &mut Vec<Line<'static>>,
        name: &str,
        mode: HelpMode,
        current_mode: HelpMode,
    ) {
        let is_current = mode == current_mode;
        let style = if is_current {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        lines.push(Line::from(vec![
            Span::styled(if is_current { "▶ " } else { "  " }, style),
            Span::styled(name.to_string(), style.add_modifier(Modifier::UNDERLINED)),
        ]));

        let key_style = if is_current {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        match mode {
            HelpMode::Connections => {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled("Enter a e d", key_style),
                ]));
            }
            HelpMode::Tables => {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled("Enter n e d r", key_style),
                ]));
            }
            HelpMode::Details => {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled("Enter r", key_style),
                ]));
            }
            HelpMode::TabularOutput => {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled("i / S D x r", key_style),
                ]));
            }
            HelpMode::SqlFiles => {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled("Enter d r", key_style),
                ]));
            }
            HelpMode::QueryWindow => {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled("i C-Enter C-s", key_style),
                ]));
            }
            _ => {}
        }
        lines.push(Line::from(""));
    }

    /// Helper to add a command line with proper formatting
    fn add_command(lines: &mut Vec<Line<'static>>, key: &str, desc: &str) {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:10}", key),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(desc.to_string()),
        ]));
    }

    fn add_connections_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "j/k", "Navigate up/down");
        Self::add_command(lines, "Enter", "Connect/disconnect");
        Self::add_command(lines, "a", "Add connection");
        Self::add_command(lines, "e", "Edit connection");
        Self::add_command(lines, "d", "Delete connection");
        Self::add_command(lines, "C-r", "Refresh status");
    }

    fn add_tables_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "j/k", "Navigate up/down");
        Self::add_command(lines, "Enter", "Open table");
        Self::add_command(lines, "n", "Create table");
        Self::add_command(lines, "e", "Edit structure");
        Self::add_command(lines, "d", "Drop table");
        Self::add_command(lines, "r", "Refresh list");
    }

    fn add_details_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "Enter", "Load metadata");
        Self::add_command(lines, "r", "Refresh");
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Displays:",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(Span::raw("• Size & row count")));
        lines.push(Line::from(Span::raw("• Keys & indexes")));
        lines.push(Line::from(Span::raw("• Comments")));
    }

    fn add_tabular_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "h/j/k/l", "Navigate cells");
        Self::add_command(lines, "i", "Edit cell");
        Self::add_command(lines, "ESC", "Save changes");
        Self::add_command(lines, "dd", "Delete row");
        Self::add_command(lines, "yy", "Copy row (CSV)");
        Self::add_command(lines, "/", "Search");
        Self::add_command(lines, "n/N", "Next/prev match");
        Self::add_command(lines, "S/D", "Prev/next tab");
        Self::add_command(lines, "x", "Close tab");
        Self::add_command(lines, "r", "Refresh data");
        Self::add_command(lines, "C-d/u", "Page down/up");
        Self::add_command(lines, "gg/G", "First/last row");
        Self::add_command(lines, "0/$", "First/last col");
    }

    fn add_sql_files_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "j/k", "Navigate files");
        Self::add_command(lines, "Enter", "Load file");
        Self::add_command(lines, "d", "Delete file");
        Self::add_command(lines, "r", "Refresh list");
    }

    fn add_query_window_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "i", "Insert mode");
        Self::add_command(lines, "ESC", "Exit insert");
        Self::add_command(lines, "C-Enter", "Execute SQL");
        Self::add_command(lines, "C-s", "Save query");
        Self::add_command(lines, "C-o", "Open file");
        Self::add_command(lines, "C-n", "New query");
    }

    /// Render the help overlay
    pub fn render_help(f: &mut Frame, help_mode: HelpMode) {
        if help_mode == HelpMode::None {
            return;
        }

        // Create a more elegant modal size
        let area = centered_rect(70, 60, f.area());

        // Clear the background
        f.render_widget(Clear, area);

        // Create the main block with title
        let pane_name = match help_mode {
            HelpMode::Connections => "Connections",
            HelpMode::Tables => "Tables",
            HelpMode::Details => "Table Details",
            HelpMode::TabularOutput => "Table Viewer",
            HelpMode::SqlFiles => "SQL Files",
            HelpMode::QueryWindow => "Query Editor",
            HelpMode::None => "LazyTables",
        };

        let main_block = Block::default()
            .title(format!(" LazyTables Help • {} ", pane_name))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .border_type(ratatui::widgets::BorderType::Rounded)
            .style(Style::default().bg(Color::Black));

        let inner_area = main_block.inner(area);
        f.render_widget(main_block, area);

        // Create two columns layout
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner_area);

        // Left column - current pane commands + global
        let left_content = Self::create_left_column(help_mode);
        let left_widget = Paragraph::new(left_content)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        // Add a subtle border between columns
        let left_area = Rect {
            x: columns[0].x,
            y: columns[0].y,
            width: columns[0].width.saturating_sub(1),
            height: columns[0].height,
        };
        f.render_widget(left_widget, left_area);

        // Right column - all panes overview
        let right_content = Self::create_right_column(help_mode);
        let right_widget = Paragraph::new(right_content)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        let right_area = Rect {
            x: columns[1].x + 1,
            y: columns[1].y,
            width: columns[1].width.saturating_sub(1),
            height: columns[1].height,
        };
        f.render_widget(right_widget, right_area);

        // Draw vertical separator
        let separator_area = Rect {
            x: columns[1].x,
            y: columns[1].y,
            width: 1,
            height: columns[1].height,
        };

        let separator = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(Color::DarkGray));
        f.render_widget(separator, separator_area);

        // Add footer hint
        let footer_text = " ESC or ? to close ";
        let footer = Paragraph::new(footer_text)
            .style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )
            .alignment(Alignment::Center);

        let footer_area = Rect {
            x: area.x + 1,
            y: area.y + area.height - 1,
            width: area.width - 2,
            height: 1,
        };

        f.render_widget(footer, footer_area);
    }
}

/// Helper function to create a centered rect
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
