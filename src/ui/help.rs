// FilePath: src/ui/help.rs

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
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
            format!("─── {pane_name} Commands "),
            Style::default()
                .fg(Color::Rgb(180, 180, 100))
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
            "─── Global Commands ",
            Style::default()
                .fg(Color::Rgb(100, 180, 180))
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
            "─── All Panes Overview ",
            Style::default()
                .fg(Color::Rgb(180, 100, 180))
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

    /// Add a pane overview section with key descriptions
    fn add_pane_overview(
        lines: &mut Vec<Line<'static>>,
        name: &str,
        mode: HelpMode,
        current_mode: HelpMode,
    ) {
        let is_current = mode == current_mode;
        let header_style = if is_current {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(150, 150, 150))
        };

        lines.push(Line::from(vec![
            Span::styled(if is_current { "▶ " } else { "  " }, header_style),
            Span::styled(
                name.to_string(),
                header_style.add_modifier(Modifier::UNDERLINED),
            ),
        ]));

        let key_style = if is_current {
            Style::default().fg(Color::Rgb(100, 200, 200))
        } else {
            Style::default().fg(Color::Rgb(100, 100, 100))
        };

        let desc_style = if is_current {
            Style::default().fg(Color::Rgb(200, 200, 200))
        } else {
            Style::default().fg(Color::Rgb(120, 120, 120))
        };

        // Add key commands with descriptions for each pane
        match mode {
            HelpMode::Connections => {
                Self::add_overview_command(lines, "Enter", "Connect", key_style, desc_style);
                Self::add_overview_command(lines, "x", "Disconnect", key_style, desc_style);
                Self::add_overview_command(
                    lines,
                    "a/e/d",
                    "Add/Edit/Delete",
                    key_style,
                    desc_style,
                );
            }
            HelpMode::Tables => {
                Self::add_overview_command(lines, "Enter", "Open", key_style, desc_style);
                Self::add_overview_command(lines, "n", "Create", key_style, desc_style);
                Self::add_overview_command(lines, "e", "Edit", key_style, desc_style);
            }
            HelpMode::Details => {
                Self::add_overview_command(lines, "Enter", "Load metadata", key_style, desc_style);
            }
            HelpMode::TabularOutput => {
                Self::add_overview_command(lines, "i", "Edit", key_style, desc_style);
                Self::add_overview_command(lines, "dd/yy", "Delete/Copy", key_style, desc_style);
                Self::add_overview_command(lines, "S/D", "Switch tabs", key_style, desc_style);
                Self::add_overview_command(lines, "/", "Search", key_style, desc_style);
            }
            HelpMode::SqlFiles => {
                Self::add_overview_command(lines, "Enter", "Load file", key_style, desc_style);
                Self::add_overview_command(lines, "C-o", "Refresh", key_style, desc_style);
                Self::add_overview_command(lines, "C-n", "New file", key_style, desc_style);
            }
            HelpMode::QueryWindow => {
                Self::add_overview_command(lines, "i", "Edit mode", key_style, desc_style);
                Self::add_overview_command(
                    lines,
                    "C-Enter",
                    "Execute at cursor",
                    key_style,
                    desc_style,
                );
                Self::add_overview_command(lines, "C-s", "Save", key_style, desc_style);
            }
            _ => {}
        }
        lines.push(Line::from(""));
    }

    /// Helper to add a command line in the overview section
    fn add_overview_command(
        lines: &mut Vec<Line<'static>>,
        key: &str,
        desc: &str,
        key_style: Style,
        desc_style: Style,
    ) {
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("{key:8}"), key_style),
            Span::styled(desc.to_string(), desc_style),
        ]));
    }

    /// Helper to add a command line with proper formatting
    fn add_command(lines: &mut Vec<Line<'static>>, key: &str, desc: &str) {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{key:10}"),
                Style::default()
                    .fg(Color::Rgb(100, 200, 200))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                desc.to_string(),
                Style::default().fg(Color::Rgb(200, 200, 200)),
            ),
        ]));
    }

    fn add_connections_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "j/k", "Navigate up/down");
        Self::add_command(lines, "Enter/Space", "Connect to database");
        Self::add_command(lines, "x", "Disconnect connection");
        Self::add_command(lines, "a", "Add connection");
        Self::add_command(lines, "e", "Edit connection");
        Self::add_command(lines, "d", "Delete connection");
    }

    fn add_tables_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "j/k", "Navigate up/down");
        Self::add_command(lines, "Enter", "Open table");
        Self::add_command(lines, "n", "Create table");
        Self::add_command(lines, "e", "Edit structure");
    }

    fn add_details_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "Enter", "Load metadata");
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
        Self::add_command(lines, "C-o", "Refresh list");
        Self::add_command(lines, "C-n", "New query file");
    }

    fn add_query_window_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "i", "Insert mode");
        Self::add_command(lines, "ESC", "Exit insert");
        Self::add_command(lines, "C-Enter", "Execute query at cursor");
        Self::add_command(lines, "C-s", "Save query");
        Self::add_command(lines, "C-o", "Refresh file list");
        Self::add_command(lines, "C-n", "New query");
    }

    /// Render the help overlay
    pub fn render_help(f: &mut Frame, help_mode: HelpMode) {
        if help_mode == HelpMode::None {
            return;
        }

        // Create a more elegant modal size - smaller and centered
        let area = centered_rect(65, 50, f.area());

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

        // Create an elegant dark overlay without completely clearing the background
        let overlay_block = Block::default().style(Style::default().bg(Color::Rgb(20, 20, 20)));
        f.render_widget(overlay_block, area);

        let main_block = Block::default()
            .title(format!(" Help • {pane_name} "))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(80, 80, 100)))
            .border_type(ratatui::widgets::BorderType::Rounded)
            .style(Style::default().bg(Color::Rgb(20, 20, 20)));

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
