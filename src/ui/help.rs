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
            format!("🎯 {} Commands", pane_name),
            Style::default()
                .fg(Color::Rgb(120, 180, 255))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
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
            "🌍 Global Commands",
            Style::default()
                .fg(Color::Rgb(100, 220, 180))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]));
        lines.push(Line::from(""));

        // Global commands
        Self::add_command(&mut lines, "q", "Quit LazyTables");
        Self::add_command(&mut lines, "?", "Toggle help");
        Self::add_command(&mut lines, ":", "Enter command mode");
        Self::add_command(&mut lines, "C-B", "Toggle debug view");
        lines.push(Line::from(""));
        Self::add_command(&mut lines, "Tab", "Next pane");
        Self::add_command(&mut lines, "S-Tab", "Previous pane");
        Self::add_command(&mut lines, "C-h", "Focus left");
        Self::add_command(&mut lines, "C-j", "Focus down");
        Self::add_command(&mut lines, "C-k", "Focus up");
        Self::add_command(&mut lines, "C-l", "Focus right");

        lines
    }

    /// Create the right column content (global commands)
    pub fn create_right_column(_current_mode: HelpMode) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "🌐 Global Commands",
                Style::default()
                    .fg(Color::Rgb(255, 150, 200))
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )]),
            Line::from(""),
        ];

        // Application-level commands
        lines.push(Line::from(vec![Span::styled(
            "💾 Application",
            Style::default()
                .fg(Color::Rgb(120, 180, 255))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]));
        lines.push(Line::from(""));
        Self::add_command(&mut lines, "q", "Quit LazyTables");
        Self::add_command(&mut lines, "?", "Toggle help guide");
        Self::add_command(&mut lines, ":", "Enter command mode");
        Self::add_command(&mut lines, "C-B", "Toggle debug view");
        lines.push(Line::from(""));

        // Navigation commands
        lines.push(Line::from(vec![Span::styled(
            "🧭 Navigation",
            Style::default()
                .fg(Color::Rgb(100, 220, 180))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]));
        lines.push(Line::from(""));
        Self::add_command(&mut lines, "Tab", "Next pane");
        Self::add_command(&mut lines, "S-Tab", "Previous pane");
        Self::add_command(&mut lines, "C-h", "Focus left");
        Self::add_command(&mut lines, "C-j", "Focus down");
        Self::add_command(&mut lines, "C-k", "Focus up");
        Self::add_command(&mut lines, "C-l", "Focus right");
        lines.push(Line::from(""));

        // Data operations
        lines.push(Line::from(vec![Span::styled(
            "📊 Data Operations",
            Style::default()
                .fg(Color::Rgb(255, 200, 100))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]));
        lines.push(Line::from(""));
        Self::add_command(&mut lines, "C-Enter", "Execute SQL at cursor");
        Self::add_command(&mut lines, "C-S", "Save current query");
        Self::add_command(&mut lines, "C-O", "Refresh current view");
        Self::add_command(&mut lines, "C-N", "New timestamped query");
        lines.push(Line::from(""));

        // Quick reference
        lines.push(Line::from(vec![Span::styled(
            "📖 Quick Reference",
            Style::default()
                .fg(Color::Rgb(180, 140, 255))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Rgb(100, 220, 180))),
            Span::raw("Use vim-style navigation (h/j/k/l)"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Rgb(100, 220, 180))),
            Span::raw("Press 'i' to enter insert mode in forms"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Rgb(100, 220, 180))),
            Span::raw("ESC exits insert mode and cancels"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Rgb(100, 220, 180))),
            Span::raw("All changes require connection to database"),
        ]));

        lines
    }

    /// Helper to add a command line with proper formatting
    fn add_command(lines: &mut Vec<Line<'static>>, key: &str, desc: &str) {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("⌨️  {key:<12}"),
                Style::default()
                    .fg(Color::Rgb(170, 220, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                desc.to_string(),
                Style::default().fg(Color::Rgb(240, 245, 250)),
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
        Self::add_command(lines, "/", "Search connections");
        Self::add_command(lines, "ESC", "Exit search mode");
        Self::add_command(lines, "↑/↓", "Navigate search results");
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Connection Status:",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(vec![
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::raw("Connected"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("— ", Style::default().fg(Color::DarkGray)),
            Span::raw("Not connected"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("✗ ", Style::default().fg(Color::Red)),
            Span::raw("Failed"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("⟳ ", Style::default().fg(Color::Yellow)),
            Span::raw("Connecting"),
        ]));
    }

    fn add_tables_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "j/k", "Navigate up/down");
        Self::add_command(lines, "gg/G", "First/last table");
        Self::add_command(lines, "Enter/Space", "Open table");
        Self::add_command(lines, "n", "Create table");
        Self::add_command(lines, "e", "Edit structure");
        Self::add_command(lines, "/", "Search tables");
        Self::add_command(lines, "ESC", "Exit search mode");
        Self::add_command(lines, "↑/↓", "Navigate search results");
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Objects Displayed:",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(Span::raw("• Tables with row counts and sizes")));
        lines.push(Line::from(Span::raw("• Views and materialized views")));
        lines.push(Line::from(Span::raw("• Foreign tables (if supported)")));
        lines.push(Line::from(Span::raw(
            "• Schema information (multi-schema databases)",
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Status Messages:",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(Span::raw(
            "• Choose a connection from Connections pane",
        )));
        lines.push(Line::from(Span::raw("• No tables in database")));
        lines.push(Line::from(Span::raw(
            "• Connection failed (see status bar)",
        )));
        lines.push(Line::from(Span::raw("• Search results with filter count")));
    }

    fn add_details_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "j/k", "Scroll up/down");
        Self::add_command(lines, "Enter/Space", "Load detailed metadata");
        Self::add_command(lines, "r", "Refresh metadata");
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Information Displayed:",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(Span::raw(
            "• Object type (Table/View/Materialized View)",
        )));
        lines.push(Line::from(Span::raw("• Row count and column count")));
        lines.push(Line::from(Span::raw(
            "• Storage size (total, table, indexes)",
        )));
        lines.push(Line::from(Span::raw("• Primary keys and foreign keys")));
        lines.push(Line::from(Span::raw("• Index information")));
        lines.push(Line::from(Span::raw("• Table comments and metadata")));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Status Messages:",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(Span::raw("• No database connected")));
        lines.push(Line::from(Span::raw("• No tables in database")));
        lines.push(Line::from(Span::raw("• No table selected")));
    }

    fn add_tabular_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "h/j/k/l", "Navigate cells");
        Self::add_command(lines, "gg/G", "First/last row");
        Self::add_command(lines, "0/$", "First/last col");
        Self::add_command(lines, "i", "Edit cell (insert mode)");
        Self::add_command(lines, "ESC/Enter", "Save cell changes");
        Self::add_command(lines, "C-c", "Cancel cell edit");
        lines.push(Line::from(""));
        Self::add_command(lines, "dd", "Delete row (with confirm)");
        Self::add_command(lines, "yy", "Copy row (CSV)");
        Self::add_command(lines, "/", "Search in table");
        Self::add_command(lines, "n/N", "Next/prev match");
        Self::add_command(lines, "r", "Refresh table data");
        Self::add_command(lines, "t", "Toggle Data/Schema view");
        lines.push(Line::from(""));
        Self::add_command(lines, "S/D", "Prev/next tab");
        Self::add_command(lines, "x", "Close current tab");
        Self::add_command(lines, "C-d/u", "Page down/up");
    }

    fn add_sql_files_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "j/k", "Navigate files");
        Self::add_command(lines, "Enter/Space", "Load file");
        Self::add_command(lines, "n", "Create new file");
        Self::add_command(lines, "r", "Rename file");
        Self::add_command(lines, "d", "Delete file (with confirmation)");
        Self::add_command(lines, "c", "Copy/duplicate file");
        Self::add_command(lines, "/", "Search files");
        Self::add_command(lines, "ESC", "Exit input modes");
        Self::add_command(lines, "i", "Enter Query mode");
        lines.push(Line::from(""));
        Self::add_command(lines, "C-s", "Save current query");
        Self::add_command(lines, "C-o", "Refresh file list");
        Self::add_command(lines, "C-n", "New timestamped query");
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "File Operations:",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(Span::raw(
            "• Files stored per connection in ~/.lazytables/",
        )));
        lines.push(Line::from(Span::raw(
            "• File metadata shown (size, modified time)",
        )));
        lines.push(Line::from(Span::raw(
            "• Current file indicated with ● symbol",
        )));
        lines.push(Line::from(Span::raw("• Search mode shows current query")));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Input Modes:",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(Span::raw("• Search: Type to filter files")));
        lines.push(Line::from(Span::raw("• Rename: Enter new filename")));
        lines.push(Line::from(Span::raw(
            "• Create: Enter filename for new file",
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Status Messages:",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(Span::raw(
            "• No SQL files found (create with 'n')",
        )));
        lines.push(Line::from(Span::raw(
            "• [SEARCH], [RENAME], [CREATE] mode indicators",
        )));
    }

    fn add_query_window_commands(lines: &mut Vec<Line<'static>>) {
        Self::add_command(lines, "i", "Enter Query mode (full editor)");
        Self::add_command(lines, "h/j/k/l", "Navigate cursor");
        Self::add_command(lines, "C-Enter", "Execute query at cursor");
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Query Mode (Vim-style):",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )));
        Self::add_command(lines, "ESC", "Exit Query mode");
        Self::add_command(lines, "i", "Insert mode (within Query)");
        Self::add_command(lines, ":", "Vim command mode (:w, :q)");
        Self::add_command(lines, "w/e", "Word navigation");
        Self::add_command(lines, "0/$", "Line start/end");
        Self::add_command(lines, "gg/G", "File start/end");
        Self::add_command(lines, "C-d/u", "Half page scroll");
        lines.push(Line::from(""));
        Self::add_command(lines, "C-s", "Save query");
        Self::add_command(lines, "C-o", "Refresh file list");
        Self::add_command(lines, "C-n", "New timestamped query");
    }

    /// Render the help overlay
    pub fn render_help(f: &mut Frame, ui_state: &crate::state::ui::UIState) {
        let help_mode = ui_state.help_mode;
        if help_mode == HelpMode::None {
            return;
        }

        // First, clear the entire screen to eliminate any transparency
        f.render_widget(Clear, f.area());

        // Then render a full-screen solid black background
        let fullscreen_overlay = Block::default().style(Style::default().bg(Color::Rgb(8, 10, 12)));
        f.render_widget(fullscreen_overlay, f.area());

        // Create a larger, more spacious modal
        let area = centered_rect(78, 65, f.area());

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

        // Create a solid dark overlay for the modal area (slightly lighter than the background)
        let overlay_block = Block::default().style(Style::default().bg(Color::Rgb(15, 18, 22)));
        f.render_widget(overlay_block, area);

        // Main block with elegant solid styling
        let main_block = Block::default()
            .title(format!(" ❓ Help Guide • {} ", pane_name))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Rgb(120, 150, 220))
                    .add_modifier(Modifier::BOLD),
            )
            .border_type(ratatui::widgets::BorderType::Rounded)
            .style(Style::default().bg(Color::Rgb(12, 15, 18)));

        let inner_area = main_block.inner(area);
        f.render_widget(main_block, area);

        // Create layout with more padding and two columns
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Increased top padding
                Constraint::Min(0),    // Content area
                Constraint::Length(3), // Increased bottom padding for footer
            ])
            .split(inner_area);

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(46), // Left column (slightly smaller for more padding)
                Constraint::Length(4),      // More separator space for padding
                Constraint::Percentage(50), // Right column
            ])
            .split(main_layout[1]);

        // Left column - current pane commands + global
        let left_content = Self::create_left_column(help_mode);
        let left_focused = ui_state.help_pane_focus == crate::state::ui::HelpPaneFocus::Left;
        let left_border_style = if left_focused {
            Style::default()
                .fg(Color::Rgb(120, 180, 255))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(80, 100, 150))
        };
        let left_title = if left_focused {
            format!(
                " 🎯 {} Help (focused) ",
                match help_mode {
                    HelpMode::Connections => "Connections",
                    HelpMode::Tables => "Tables",
                    HelpMode::Details => "Table Details",
                    HelpMode::TabularOutput => "Table Viewer",
                    HelpMode::SqlFiles => "SQL Files",
                    HelpMode::QueryWindow => "Query Editor",
                    HelpMode::None => "LazyTables",
                }
            )
        } else {
            format!(
                " {} Help ",
                match help_mode {
                    HelpMode::Connections => "Connections",
                    HelpMode::Tables => "Tables",
                    HelpMode::Details => "Table Details",
                    HelpMode::TabularOutput => "Table Viewer",
                    HelpMode::SqlFiles => "SQL Files",
                    HelpMode::QueryWindow => "Query Editor",
                    HelpMode::None => "LazyTables",
                }
            )
        };
        let left_widget = Paragraph::new(left_content)
            .style(Style::default().fg(Color::Rgb(240, 245, 250)))
            .wrap(Wrap { trim: true })
            .scroll((ui_state.help_left_scroll_offset as u16, 0))
            .block(
                Block::default()
                    .title(left_title)
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(left_border_style)
                    .style(Style::default().bg(Color::Rgb(18, 22, 26))),
            );

        f.render_widget(left_widget, columns[0]);

        // Right column - global commands
        let right_content = Self::create_right_column(help_mode);
        let right_focused = ui_state.help_pane_focus == crate::state::ui::HelpPaneFocus::Right;
        let right_border_style = if right_focused {
            Style::default()
                .fg(Color::Rgb(120, 180, 255))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(80, 100, 150))
        };
        let right_title = if right_focused {
            " 🌐 Global Commands (focused) ".to_string()
        } else {
            " 🌐 Global Commands ".to_string()
        };
        let right_widget = Paragraph::new(right_content)
            .style(Style::default().fg(Color::Rgb(240, 245, 250)))
            .wrap(Wrap { trim: true })
            .scroll((ui_state.help_right_scroll_offset as u16, 0))
            .block(
                Block::default()
                    .title(right_title)
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(right_border_style)
                    .style(Style::default().bg(Color::Rgb(18, 22, 26))),
            );

        f.render_widget(right_widget, columns[2]);

        // Draw elegant vertical separator
        let separator_chars = "│".repeat(columns[1].height as usize);
        let separator_paragraph = Paragraph::new(separator_chars)
            .style(Style::default().fg(Color::Rgb(80, 95, 140)))
            .alignment(Alignment::Center);
        f.render_widget(separator_paragraph, columns[1]);

        // Add elegant footer with instructions
        let footer_text = "💡 Press ESC or ? to close • ←/→ or Tab to switch panes • ↑/↓ or j/k to scroll • PgUp/PgDown for faster scrolling";
        let footer = Paragraph::new(footer_text)
            .style(
                Style::default()
                    .fg(Color::Rgb(140, 160, 200))
                    .add_modifier(Modifier::ITALIC),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::Rgb(80, 100, 150)))
                    .style(Style::default().bg(Color::Rgb(12, 15, 18))),
            );

        f.render_widget(footer, main_layout[2]);
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
