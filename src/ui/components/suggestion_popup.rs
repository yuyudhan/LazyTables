// FilePath: src/ui/components/suggestion_popup.rs

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use super::sql_suggestions::{SqlSuggestion, SuggestionType};

#[derive(Debug)]
pub struct SuggestionPopup {
    /// Current suggestions to display
    suggestions: Vec<SqlSuggestion>,
    /// List state for selection tracking
    list_state: ListState,
    /// Whether the popup is visible
    is_visible: bool,
    /// Maximum number of suggestions to show
    max_visible: usize,
}

impl SuggestionPopup {
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            list_state: ListState::default(),
            is_visible: false,
            max_visible: 10,
        }
    }

    /// Update suggestions and show popup
    pub fn show_suggestions(&mut self, suggestions: Vec<SqlSuggestion>) {
        self.suggestions = suggestions;
        self.is_visible = !self.suggestions.is_empty();

        if self.is_visible {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    /// Hide the popup
    pub fn hide(&mut self) {
        self.is_visible = false;
        self.suggestions.clear();
        self.list_state.select(None);
    }

    /// Check if popup is visible
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if !self.is_visible || self.suggestions.is_empty() {
            return;
        }

        let selected = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.suggestions.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(selected));
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.is_visible || self.suggestions.is_empty() {
            return;
        }

        let selected = match self.list_state.selected() {
            Some(i) => (i + 1) % self.suggestions.len(),
            None => 0,
        };
        self.list_state.select(Some(selected));
    }

    /// Get currently selected suggestion
    pub fn get_selected_suggestion(&self) -> Option<&SqlSuggestion> {
        if let Some(selected) = self.list_state.selected() {
            self.suggestions.get(selected)
        } else {
            None
        }
    }

    /// Render the suggestion popup
    pub fn render(&mut self, frame: &mut Frame, cursor_position: (u16, u16), available_area: Rect) {
        if !self.is_visible || self.suggestions.is_empty() {
            return;
        }

        // Calculate popup dimensions
        let popup_width = 50u16.min(available_area.width.saturating_sub(2));
        let popup_height = (self.suggestions.len() as u16 + 2).min(self.max_visible as u16 + 2);

        // Calculate position relative to cursor
        let (cursor_x, cursor_y) = cursor_position;
        let popup_x = if cursor_x + popup_width + 2 <= available_area.width {
            cursor_x + 1 // Position to the right of cursor
        } else {
            cursor_x.saturating_sub(popup_width + 1) // Position to the left if no space on right
        };

        let popup_y = if cursor_y + popup_height <= available_area.height {
            cursor_y + 1 // Position below cursor
        } else {
            cursor_y.saturating_sub(popup_height) // Position above if no space below
        };

        // Ensure popup stays within available area
        let popup_x = popup_x.min(available_area.width.saturating_sub(popup_width));
        let popup_y = popup_y.min(available_area.height.saturating_sub(popup_height));

        let popup_area = Rect {
            x: available_area.x + popup_x,
            y: available_area.y + popup_y,
            width: popup_width,
            height: popup_height,
        };

        self.render_popup(frame, popup_area);
    }

    /// Render the popup content
    fn render_popup(&mut self, frame: &mut Frame, area: Rect) {
        // Create list items from suggestions
        let items: Vec<ListItem> = self
            .suggestions
            .iter()
            .map(|suggestion| {
                let style = self.get_suggestion_style(&suggestion.suggestion_type);

                ListItem::new(Line::from(vec![
                    Span::styled(
                        self.get_suggestion_icon(&suggestion.suggestion_type),
                        style.add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(suggestion.display.clone(), style),
                    if let Some(ref desc) = suggestion.description {
                        Span::styled(format!(" - {}", desc), Style::default().fg(Color::DarkGray))
                    } else {
                        Span::raw("")
                    },
                ]))
            })
            .collect();

        // Create the list widget
        let list = List::new(items)
            .block(
                Block::default()
                    .title(" SQL Suggestions ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        // Clear the area behind the popup to prevent artifacts
        let clear_block = Block::default().style(Style::default().bg(Color::Black));
        frame.render_widget(clear_block, area);

        // Render the list
        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    /// Get appropriate style for suggestion type
    fn get_suggestion_style(&self, suggestion_type: &SuggestionType) -> Style {
        match suggestion_type {
            SuggestionType::Keyword => Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            SuggestionType::Table => Style::default().fg(Color::Green),
            SuggestionType::Column => Style::default().fg(Color::Yellow),
            SuggestionType::Function => Style::default().fg(Color::Magenta),
            SuggestionType::Alias => Style::default().fg(Color::Cyan),
        }
    }

    /// Get icon for suggestion type
    fn get_suggestion_icon(&self, suggestion_type: &SuggestionType) -> &'static str {
        match suggestion_type {
            SuggestionType::Keyword => "K",
            SuggestionType::Table => "T",
            SuggestionType::Column => "C",
            SuggestionType::Function => "F",
            SuggestionType::Alias => "A",
        }
    }
}

impl Default for SuggestionPopup {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to calculate optimal popup position
pub fn calculate_popup_position(
    cursor_pos: (u16, u16),
    popup_size: (u16, u16),
    container_size: (u16, u16),
) -> (u16, u16) {
    let (cursor_x, cursor_y) = cursor_pos;
    let (popup_width, popup_height) = popup_size;
    let (container_width, container_height) = container_size;

    // Try to position below and to the right of cursor
    let mut x = cursor_x + 1;
    let mut y = cursor_y + 1;

    // Adjust if popup would go outside container bounds
    if x + popup_width > container_width {
        x = cursor_x.saturating_sub(popup_width);
    }

    if y + popup_height > container_height {
        y = cursor_y.saturating_sub(popup_height);
    }

    // Ensure minimum bounds
    x = x.min(container_width.saturating_sub(popup_width));
    y = y.min(container_height.saturating_sub(popup_height));

    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_creation() {
        let popup = SuggestionPopup::new();
        assert!(!popup.is_visible());
        assert!(popup.suggestions.is_empty());
    }

    #[test]
    fn test_show_hide_suggestions() {
        let mut popup = SuggestionPopup::new();
        let suggestions = vec![SqlSuggestion {
            text: "SELECT".to_string(),
            display: "SELECT".to_string(),
            suggestion_type: SuggestionType::Keyword,
            description: Some("SQL SELECT statement".to_string()),
        }];

        popup.show_suggestions(suggestions);
        assert!(popup.is_visible());
        assert_eq!(popup.suggestions.len(), 1);

        popup.hide();
        assert!(!popup.is_visible());
        assert!(popup.suggestions.is_empty());
    }

    #[test]
    fn test_navigation() {
        let mut popup = SuggestionPopup::new();
        let suggestions = vec![
            SqlSuggestion {
                text: "SELECT".to_string(),
                display: "SELECT".to_string(),
                suggestion_type: SuggestionType::Keyword,
                description: None,
            },
            SqlSuggestion {
                text: "INSERT".to_string(),
                display: "INSERT".to_string(),
                suggestion_type: SuggestionType::Keyword,
                description: None,
            },
        ];

        popup.show_suggestions(suggestions);
        assert_eq!(popup.list_state.selected(), Some(0));

        popup.select_next();
        assert_eq!(popup.list_state.selected(), Some(1));

        popup.select_next();
        assert_eq!(popup.list_state.selected(), Some(0)); // Wraps around

        popup.select_previous();
        assert_eq!(popup.list_state.selected(), Some(1)); // Wraps around backwards
    }

    #[test]
    fn test_position_calculation() {
        let cursor_pos = (10, 5);
        let popup_size = (20, 8);
        let container_size = (80, 24);

        let (x, y) = calculate_popup_position(cursor_pos, popup_size, container_size);
        assert_eq!((x, y), (11, 6)); // Should position below and to the right

        // Test edge case where popup would go outside bounds
        let cursor_pos = (75, 20);
        let (x, y) = calculate_popup_position(cursor_pos, popup_size, container_size);
        assert!(x + popup_size.0 <= container_size.0);
        assert!(y + popup_size.1 <= container_size.1);
    }
}
