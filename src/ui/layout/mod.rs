// FilePath: src/ui/layout/mod.rs

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Areas for each pane in the layout
#[derive(Debug, Clone, Copy)]
pub struct LayoutAreas {
    pub header: Rect,
    pub connections: Rect,
    pub tables: Rect,
    pub details: Rect,
    pub main_content: Rect,
    pub status_bar: Rect,
}

/// Manages the four-pane layout
pub struct LayoutManager {
    /// Width percentage for left section (connections, tables, details)
    left_width_percent: u16,
    /// Height percentages for left panes
    connections_height_percent: u16,
    tables_height_percent: u16,
    details_height_percent: u16,
}

impl LayoutManager {
    /// Create a new layout manager with default proportions
    pub fn new() -> Self {
        Self {
            left_width_percent: 25,
            connections_height_percent: 40,
            tables_height_percent: 40,
            details_height_percent: 20,
        }
    }

    /// Calculate the layout areas for the given terminal size
    pub fn calculate_layout(&self, area: Rect) -> LayoutAreas {
        // First, split vertically into header, body, and status bar
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Min(0),    // Body
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        let header = main_chunks[0];
        let body = main_chunks[1];
        let status_bar = main_chunks[2];

        // Split body horizontally into left section and main content
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.left_width_percent),
                Constraint::Min(0), // Main content takes remaining space
            ])
            .split(body);

        let left_section = body_chunks[0];
        let main_content = body_chunks[1];

        // Split left section vertically into three panes
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(self.connections_height_percent),
                Constraint::Percentage(self.tables_height_percent),
                Constraint::Percentage(self.details_height_percent),
            ])
            .split(left_section);

        let connections = left_chunks[0];
        let tables = left_chunks[1];
        let details = left_chunks[2];

        LayoutAreas {
            header,
            connections,
            tables,
            details,
            main_content,
            status_bar,
        }
    }

    /// Check if the terminal size meets minimum requirements
    pub fn is_size_valid(&self, area: Rect) -> bool {
        area.width >= 120 && area.height >= 30
    }

    /// Get a warning message for small terminal size
    pub fn size_warning_message() -> &'static str {
        "Terminal size too small. Minimum: 120x30"
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

