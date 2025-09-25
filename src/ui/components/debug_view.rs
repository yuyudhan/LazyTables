// FilePath: src/ui/components/debug_view.rs

use crate::{logging::DebugMessage, ui::theme::Theme};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};
use std::collections::HashMap;

/// Debug view component for displaying logs and diagnostics
#[derive(Debug, Clone)]
pub struct DebugView {
    /// Current application performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Cached statistics to prevent flickering
    cached_stats: Option<CachedStatistics>,
}

/// Cached statistics to reduce flickering
#[derive(Debug, Clone)]
struct CachedStatistics {
    stats_text: String,
    last_update: std::time::Instant,
    last_message_count: usize,
}

/// Performance metrics for the debug view
#[derive(Debug, Default, Clone)]
pub struct PerformanceMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub database_connections: usize,
    pub active_queries: usize,
    pub fps: u32,
    pub render_time_ms: f64,
}

impl DebugView {
    /// Create a new debug view
    pub fn new() -> Self {
        Self {
            performance_metrics: PerformanceMetrics::default(),
            cached_stats: None,
        }
    }

    /// Render the debug view as a full-screen overlay
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        debug_messages: &[DebugMessage],
        scroll_offset: usize,
    ) {
        // Clear the background
        frame.render_widget(Clear, area);

        // Main block with borders
        let main_block = Block::default()
            .borders(Borders::ALL)
            .title(" Debug View (Ctrl+B to toggle) ")
            .title_alignment(Alignment::Center)
            .style(
                Style::default()
                    .bg(theme.get_color("background"))
                    .fg(theme.get_color("foreground")),
            );

        let inner_area = main_block.inner(area);
        frame.render_widget(main_block, area);

        // Split the area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Performance metrics
                Constraint::Length(3), // Statistics
                Constraint::Min(10),   // Log messages
                Constraint::Length(3), // Help text
            ])
            .split(inner_area);

        // Render performance metrics
        self.render_performance_metrics(frame, chunks[0], theme);

        // Render statistics
        self.render_statistics(frame, chunks[1], theme, debug_messages);

        // Render log messages
        self.render_log_messages(frame, chunks[2], theme, debug_messages, scroll_offset);

        // Render help text
        self.render_help_text(frame, chunks[3], theme);
    }

    /// Render performance metrics section
    fn render_performance_metrics(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let metrics_block = Block::default()
            .borders(Borders::ALL)
            .title(" Performance Metrics ")
            .style(
                Style::default()
                    .bg(theme.get_color("background"))
                    .fg(theme.get_color("primary_highlight")),
            );

        let inner_area = metrics_block.inner(area);
        frame.render_widget(metrics_block, area);

        // Split into two columns
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner_area);

        // Left column metrics
        let left_metrics = vec![
            format!(
                "Memory Usage: {:.1} MB",
                self.performance_metrics.memory_usage_mb
            ),
            format!(
                "CPU Usage: {:.1}%",
                self.performance_metrics.cpu_usage_percent
            ),
            format!("FPS: {}", self.performance_metrics.fps),
        ];

        let left_text = Text::from(
            left_metrics
                .into_iter()
                .map(|metric| {
                    Line::from(vec![Span::styled(
                        metric,
                        Style::default().fg(theme.get_color("foreground")),
                    )])
                })
                .collect::<Vec<_>>(),
        );

        let left_paragraph =
            Paragraph::new(left_text).style(Style::default().bg(theme.get_color("background")));
        frame.render_widget(left_paragraph, columns[0]);

        // Right column metrics
        let right_metrics = vec![
            format!(
                "DB Connections: {}",
                self.performance_metrics.database_connections
            ),
            format!(
                "Active Queries: {}",
                self.performance_metrics.active_queries
            ),
            format!(
                "Render Time: {:.2}ms",
                self.performance_metrics.render_time_ms
            ),
        ];

        let right_text = Text::from(
            right_metrics
                .into_iter()
                .map(|metric| {
                    Line::from(vec![Span::styled(
                        metric,
                        Style::default().fg(theme.get_color("foreground")),
                    )])
                })
                .collect::<Vec<_>>(),
        );

        let right_paragraph =
            Paragraph::new(right_text).style(Style::default().bg(theme.get_color("background")));
        frame.render_widget(right_paragraph, columns[1]);
    }

    /// Render statistics section with caching to prevent flickering
    fn render_statistics(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        debug_messages: &[DebugMessage],
    ) {
        let stats_block = Block::default()
            .borders(Borders::ALL)
            .title(" Log Statistics ")
            .style(
                Style::default()
                    .bg(theme.get_color("background"))
                    .fg(theme.get_color("primary_highlight")),
            );

        let inner_area = stats_block.inner(area);
        frame.render_widget(stats_block, area);

        let current_message_count = debug_messages.len();
        let now = std::time::Instant::now();

        // Check if we need to update cached statistics
        let should_update = match &self.cached_stats {
            None => true,
            Some(cached) => {
                // Update if:
                // 1. Message count changed significantly (>= 10 new messages)
                // 2. It's been more than 500ms since last update
                // 3. No messages and we had messages before
                (current_message_count >= cached.last_message_count + 10)
                    || (now.duration_since(cached.last_update).as_millis() > 500)
                    || (current_message_count == 0 && cached.last_message_count > 0)
            }
        };

        let stats_text = if should_update {
            // Recalculate statistics
            let new_stats_text = if debug_messages.is_empty() {
                "No log messages captured yet".to_string()
            } else {
                // Count messages by level
                let mut level_counts = HashMap::new();
                for message in debug_messages {
                    *level_counts.entry(&message.level).or_insert(0) += 1;
                }

                let mut parts = vec![format!("Total: {}", debug_messages.len())];
                for (level, count) in &level_counts {
                    parts.push(format!("{}: {}", level, count));
                }
                parts.join(" | ")
            };

            // Update cache
            self.cached_stats = Some(CachedStatistics {
                stats_text: new_stats_text.clone(),
                last_update: now,
                last_message_count: current_message_count,
            });

            new_stats_text
        } else {
            // Use cached statistics
            self.cached_stats
                .as_ref()
                .map(|cached| cached.stats_text.clone())
                .unwrap_or_else(|| "No log messages captured yet".to_string())
        };

        let paragraph = Paragraph::new(stats_text)
            .style(
                Style::default()
                    .fg(theme.get_color("foreground"))
                    .bg(theme.get_color("background")),
            )
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, inner_area);
    }

    /// Render log messages section
    fn render_log_messages(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        debug_messages: &[DebugMessage],
        scroll_offset: usize,
    ) {
        let logs_block = Block::default()
            .borders(Borders::ALL)
            .title(" Recent Log Messages ")
            .style(
                Style::default()
                    .bg(theme.get_color("background"))
                    .fg(theme.get_color("primary_highlight")),
            );

        let inner_area = logs_block.inner(area);
        frame.render_widget(logs_block, area);

        if debug_messages.is_empty() {
            let empty_text = Paragraph::new(
                "No log messages to display.\nTry using the application to generate some logs!",
            )
            .style(
                Style::default()
                    .fg(theme.get_color("inactive_pane"))
                    .bg(theme.get_color("background")),
            )
            .alignment(Alignment::Center);
            frame.render_widget(empty_text, inner_area);
            return;
        }

        // Create list items from log messages
        let visible_height = inner_area.height as usize;
        let start_idx = scroll_offset;
        let _end_idx = (start_idx + visible_height).min(debug_messages.len());

        let items: Vec<ListItem> = debug_messages
            .iter()
            .skip(start_idx)
            .take(visible_height)
            .map(|message| self.format_log_message(message, theme))
            .collect();

        let list = List::new(items).style(Style::default().bg(theme.get_color("background")));

        frame.render_widget(list, inner_area);

        // Render scrollbar if needed
        if debug_messages.len() > visible_height {
            let scrollbar_area = Rect {
                x: inner_area.x + inner_area.width - 1,
                y: inner_area.y,
                width: 1,
                height: inner_area.height,
            };

            let mut scrollbar_state = ScrollbarState::default()
                .content_length(debug_messages.len())
                .viewport_content_length(visible_height)
                .position(scroll_offset);

            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(theme.get_color("primary_highlight")));

            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }
    }

    /// Format a single log message as a list item
    fn format_log_message(&self, message: &DebugMessage, theme: &Theme) -> ListItem<'static> {
        let level_color = match message.level.as_str() {
            "ERROR" => Color::Red,
            "WARN" => Color::Yellow,
            "INFO" => Color::Green,
            "DEBUG" => Color::Cyan,
            "TRACE" => Color::Magenta,
            _ => theme.get_color("foreground"),
        };

        let timestamp = message.timestamp.format("%H:%M:%S%.3f").to_string();

        let location_info = if let Some(ref location) = message.location {
            format!(" [{}]", location)
        } else {
            String::new()
        };

        let line = Line::from(vec![
            Span::styled(
                timestamp,
                Style::default().fg(theme.get_color("inactive_pane")),
            ),
            Span::raw(" "),
            Span::styled(
                format!("{:5}", message.level),
                Style::default().fg(level_color),
            ),
            Span::raw(" "),
            Span::styled(
                message.target.clone(),
                Style::default().fg(theme.get_color("primary_highlight")),
            ),
            Span::styled(
                location_info,
                Style::default().fg(theme.get_color("inactive_pane")),
            ),
            Span::raw(": "),
            Span::styled(
                message.message.clone(),
                Style::default().fg(theme.get_color("foreground")),
            ),
        ]);

        ListItem::new(line)
    }

    /// Render help text section
    fn render_help_text(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let help_block = Block::default()
            .borders(Borders::ALL)
            .title(" Navigation ")
            .style(
                Style::default()
                    .bg(theme.get_color("background"))
                    .fg(theme.get_color("primary_highlight")),
            );

        let inner_area = help_block.inner(area);
        frame.render_widget(help_block, area);

        let help_text = "j/k: Scroll • PgUp/PgDn: Page scroll • gg/G: Top/Bottom • c: Clear logs • Ctrl+B: Close debug view";

        let paragraph = Paragraph::new(help_text)
            .style(
                Style::default()
                    .fg(theme.get_color("foreground"))
                    .bg(theme.get_color("background")),
            )
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, inner_area);
    }

    /// Update performance metrics
    pub fn update_performance_metrics(&mut self, metrics: PerformanceMetrics) {
        self.performance_metrics = metrics;
    }
}

impl Default for DebugView {
    fn default() -> Self {
        Self::new()
    }
}
