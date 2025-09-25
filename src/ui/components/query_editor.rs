// FilePath: src/ui/components/query_editor.rs

use super::{SqlSuggestionEngine, SuggestionPopup};
use crate::database::DatabaseType;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::{SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};

#[derive(Debug)]
pub struct QueryEditor {
    content: String,
    cursor_line: usize,
    cursor_col: usize,
    scroll_offset: usize,
    is_focused: bool,
    is_insert_mode: bool,
    database_type: Option<DatabaseType>,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    /// SQL suggestion engine
    suggestion_engine: SqlSuggestionEngine,
    /// Suggestion popup
    suggestion_popup: SuggestionPopup,
    /// Whether suggestions are currently active
    suggestions_active: bool,
    /// Available tables for suggestions
    tables: Vec<String>,
    /// Table columns for suggestions
    table_columns: HashMap<String, Vec<String>>,
    /// Current SQL file name
    current_file: Option<String>,
    /// Whether content has been modified
    is_modified: bool,
}

impl Clone for QueryEditor {
    fn clone(&self) -> Self {
        Self {
            content: self.content.clone(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
            scroll_offset: self.scroll_offset,
            is_focused: self.is_focused,
            is_insert_mode: self.is_insert_mode,
            database_type: self.database_type.clone(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            suggestion_engine: SqlSuggestionEngine::new(),
            suggestion_popup: SuggestionPopup::new(),
            suggestions_active: false,
            tables: self.tables.clone(),
            table_columns: self.table_columns.clone(),
            current_file: self.current_file.clone(),
            is_modified: self.is_modified,
        }
    }
}

impl Default for QueryEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryEditor {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            is_focused: false,
            is_insert_mode: false,
            database_type: None,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            suggestion_engine: SqlSuggestionEngine::new(),
            suggestion_popup: SuggestionPopup::new(),
            suggestions_active: false,
            tables: Vec::new(),
            table_columns: HashMap::new(),
            current_file: None,
            is_modified: false,
        }
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
        self.is_modified = false;
        self.hide_suggestions();
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn set_database_type(&mut self, db_type: Option<DatabaseType>) {
        self.database_type = db_type.clone();
        self.suggestion_engine.set_database_type(db_type);
    }

    pub fn get_database_type(&self) -> Option<DatabaseType> {
        self.database_type.clone()
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn toggle_insert_mode(&mut self) {
        self.is_insert_mode = !self.is_insert_mode;
    }

    pub fn set_insert_mode(&mut self, insert_mode: bool) {
        self.is_insert_mode = insert_mode;
    }

    pub fn is_insert_mode(&self) -> bool {
        self.is_insert_mode
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.adjust_cursor_column();
            self.adjust_scroll();
        }
    }

    pub fn move_cursor_down(&mut self) {
        let lines = self.content.lines().collect::<Vec<_>>();
        if self.cursor_line < lines.len().saturating_sub(1) {
            self.cursor_line += 1;
            self.adjust_cursor_column();
            self.adjust_scroll();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            let lines = self.content.lines().collect::<Vec<_>>();
            if self.cursor_line < lines.len() {
                self.cursor_col = lines[self.cursor_line].len();
            }
            self.adjust_scroll();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let lines = self.content.lines().collect::<Vec<_>>();
        if self.cursor_line < lines.len() {
            let current_line = lines[self.cursor_line];
            if self.cursor_col < current_line.len() {
                self.cursor_col += 1;
            } else if self.cursor_line < lines.len() - 1 {
                self.cursor_line += 1;
                self.cursor_col = 0;
                self.adjust_scroll();
            }
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        if !self.is_insert_mode {
            return;
        }

        let lines: Vec<String> = self.content.lines().map(|s| s.to_string()).collect();
        let mut new_lines = lines;

        if new_lines.is_empty() {
            new_lines.push(String::new());
        }

        while self.cursor_line >= new_lines.len() {
            new_lines.push(String::new());
        }

        let line = &mut new_lines[self.cursor_line];
        if self.cursor_col > line.len() {
            self.cursor_col = line.len();
        }

        line.insert(self.cursor_col, ch);
        self.cursor_col += 1;
        self.is_modified = true;

        self.content = new_lines.join("\n");

        // Trigger suggestions after character insertion
        self.update_suggestions();
    }

    pub fn insert_newline(&mut self) {
        if !self.is_insert_mode {
            return;
        }

        let lines: Vec<String> = self.content.lines().map(|s| s.to_string()).collect();
        let mut new_lines = lines;

        if new_lines.is_empty() {
            new_lines.push(String::new());
        }

        while self.cursor_line >= new_lines.len() {
            new_lines.push(String::new());
        }

        let line = new_lines[self.cursor_line].clone();
        let (before, after) = line.split_at(self.cursor_col.min(line.len()));

        new_lines[self.cursor_line] = before.to_string();
        new_lines.insert(self.cursor_line + 1, after.to_string());

        self.cursor_line += 1;
        self.cursor_col = 0;
        self.is_modified = true;
        self.adjust_scroll();

        self.content = new_lines.join("\n");
        self.hide_suggestions();
    }

    pub fn backspace(&mut self) {
        if !self.is_insert_mode {
            return;
        }

        if self.cursor_col > 0 {
            let lines: Vec<String> = self.content.lines().map(|s| s.to_string()).collect();
            let mut new_lines = lines;

            if self.cursor_line < new_lines.len() {
                let line = &mut new_lines[self.cursor_line];
                if self.cursor_col <= line.len() {
                    line.remove(self.cursor_col - 1);
                    self.cursor_col -= 1;
                }
            }

            self.content = new_lines.join("\n");
        } else if self.cursor_line > 0 {
            let lines: Vec<String> = self.content.lines().map(|s| s.to_string()).collect();
            let mut new_lines = lines;

            if self.cursor_line < new_lines.len() {
                let current_line = new_lines.remove(self.cursor_line);
                self.cursor_line -= 1;
                if self.cursor_line < new_lines.len() {
                    self.cursor_col = new_lines[self.cursor_line].len();
                    new_lines[self.cursor_line].push_str(&current_line);
                }
            }

            self.adjust_scroll();
            self.content = new_lines.join("\n");
        }
    }

    pub fn get_statement_at_cursor(&self) -> Option<String> {
        let lines: Vec<&str> = self.content.lines().collect();
        if lines.is_empty() || self.cursor_line >= lines.len() {
            return None;
        }

        let mut start_line = self.cursor_line;
        let mut end_line = self.cursor_line;

        while start_line > 0 {
            let line = lines[start_line - 1].trim();
            if line.ends_with(';') || line.is_empty() {
                break;
            }
            start_line -= 1;
        }

        while end_line < lines.len() - 1 {
            let line = lines[end_line].trim();
            if line.ends_with(';') {
                break;
            }
            end_line += 1;
        }

        let statement_lines: Vec<&str> = lines[start_line..=end_line].to_vec();
        let statement = statement_lines.join("\n").trim().to_string();

        if statement.is_empty() {
            None
        } else {
            Some(statement)
        }
    }

    fn adjust_cursor_column(&mut self) {
        let lines = self.content.lines().collect::<Vec<_>>();
        if self.cursor_line < lines.len() {
            let line_len = lines[self.cursor_line].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        }
    }

    fn adjust_scroll(&mut self) {
        // Simple scroll adjustment - can be enhanced later
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        }
    }

    fn get_syntax(&self) -> &SyntaxReference {
        match self.database_type {
            Some(DatabaseType::MySQL) => self
                .syntax_set
                .find_syntax_by_extension("sql")
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text()),
            Some(DatabaseType::PostgreSQL) => self
                .syntax_set
                .find_syntax_by_extension("sql")
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text()),
            Some(DatabaseType::SQLite) => self
                .syntax_set
                .find_syntax_by_extension("sql")
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text()),
            _ => self.syntax_set.find_syntax_plain_text(),
        }
    }

    // Suggestion-related methods

    /// Set available tables for suggestions
    pub fn set_tables(&mut self, tables: Vec<String>) {
        self.tables = tables.clone();
        self.suggestion_engine.set_tables(tables);
    }

    /// Set columns for a specific table
    pub fn set_table_columns(&mut self, table: String, columns: Vec<String>) {
        self.table_columns.insert(table.clone(), columns.clone());
        self.suggestion_engine.set_table_columns(table, columns);
    }

    /// Set current file name
    pub fn set_current_file(&mut self, filename: Option<String>) {
        self.current_file = filename;
    }

    /// Get current file name
    pub fn get_current_file(&self) -> Option<&String> {
        self.current_file.as_ref()
    }

    /// Check if content has been modified
    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    /// Mark content as saved (not modified)
    pub fn mark_saved(&mut self) {
        self.is_modified = false;
    }

    /// Update suggestions based on current cursor position
    fn update_suggestions(&mut self) {
        if !self.is_insert_mode || !self.is_focused {
            self.hide_suggestions();
            return;
        }

        let suggestions = self.suggestion_engine.get_suggestions(
            &self.content,
            self.cursor_line,
            self.cursor_col,
        );

        if suggestions.is_empty() {
            self.hide_suggestions();
        } else {
            self.suggestion_popup.show_suggestions(suggestions);
            self.suggestions_active = true;
        }
    }

    /// Hide suggestions popup
    pub fn hide_suggestions(&mut self) {
        self.suggestion_popup.hide();
        self.suggestions_active = false;
    }

    /// Move suggestion selection up
    pub fn move_suggestion_up(&mut self) {
        if self.suggestions_active {
            self.suggestion_popup.select_previous();
        }
    }

    /// Move suggestion selection down
    pub fn move_suggestion_down(&mut self) {
        if self.suggestions_active {
            self.suggestion_popup.select_next();
        }
    }

    /// Accept current suggestion
    pub fn accept_suggestion(&mut self) {
        if !self.suggestions_active {
            return;
        }

        if let Some(suggestion) = self.suggestion_popup.get_selected_suggestion() {
            // Get the partial word being replaced
            let partial_word = self.get_partial_word_at_cursor();

            // Clone the suggestion text to avoid borrowing issues
            let insert_text = suggestion.text.clone();

            // Replace the partial word with the suggestion
            self.replace_word_at_cursor(&partial_word, &insert_text);

            self.hide_suggestions();
        }
    }

    /// Get partial word at cursor position
    fn get_partial_word_at_cursor(&self) -> String {
        let lines: Vec<&str> = self.content.lines().collect();

        if self.cursor_line >= lines.len() {
            return String::new();
        }

        let line = lines[self.cursor_line];
        if self.cursor_col > line.len() {
            return String::new();
        }

        // Find word boundaries around cursor
        let mut start = self.cursor_col;
        let mut end = self.cursor_col;

        let chars: Vec<char> = line.chars().collect();

        // Find start of word
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Find end of word
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        line[start..end].to_string()
    }

    /// Replace word at cursor with new text
    fn replace_word_at_cursor(&mut self, old_word: &str, new_text: &str) {
        let lines: Vec<String> = self.content.lines().map(|s| s.to_string()).collect();
        let mut new_lines = lines;

        if self.cursor_line < new_lines.len() {
            let line = &mut new_lines[self.cursor_line];
            let chars: Vec<char> = line.chars().collect();

            // Find start of the word to replace
            let mut start = self.cursor_col;
            while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
                start -= 1;
            }

            let end = start + old_word.len();

            if start < line.len() && end <= line.len() {
                let before = &line[..start];
                let after = &line[end..];

                *line = format!("{}{}{}", before, new_text, after);
                self.cursor_col = start + new_text.len();
                self.is_modified = true;
            }
        }

        self.content = new_lines.join("\n");
    }

    /// Check if suggestions are active
    pub fn are_suggestions_active(&self) -> bool {
        self.suggestions_active
    }

    fn apply_syntax_highlighting<'a>(&self, text: &'a str) -> Text<'a> {
        let syntax = self.get_syntax();
        let theme = &self.theme_set.themes["base16-ocean.dark"];

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut styled_lines = Vec::new();

        for line in LinesWithEndings::from(text) {
            if let Ok(ranges) = highlighter.highlight_line(line, &self.syntax_set) {
                let mut spans = Vec::new();

                for (style, text) in ranges {
                    let fg_color = style.foreground;
                    let color = Color::Rgb(fg_color.r, fg_color.g, fg_color.b);

                    let mut ratatui_style = Style::default().fg(color);
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::BOLD)
                    {
                        ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
                    }
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::ITALIC)
                    {
                        ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
                    }
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::UNDERLINE)
                    {
                        ratatui_style = ratatui_style.add_modifier(Modifier::UNDERLINED);
                    }

                    spans.push(Span::styled(text, ratatui_style));
                }

                styled_lines.push(Line::from(spans));
            } else {
                styled_lines.push(Line::from(line.to_string()));
            }
        }

        Text::from(styled_lines)
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // Create layout with help at the bottom
        let help_height = if self.is_focused { 4 } else { 0 }; // Space for file info, mode, and keybindings

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),              // Editor area (at least 3 lines)
                Constraint::Length(help_height), // Help area at bottom
            ])
            .split(area);

        let editor_area = chunks[0];
        let help_area = if help_height > 0 {
            Some(chunks[1])
        } else {
            None
        };

        // Create title with database type and mode info
        let title = format!(
            "SQL Query Editor{}{}{}",
            if let Some(ref db_type) = self.database_type {
                format!(
                    " ({})",
                    match db_type {
                        DatabaseType::PostgreSQL => "PostgreSQL",
                        DatabaseType::MySQL => "MySQL",
                        DatabaseType::MariaDB => "MariaDB",
                        DatabaseType::SQLite => "SQLite",
                        DatabaseType::Oracle => "Oracle",
                        DatabaseType::Redis => "Redis",
                        DatabaseType::MongoDB => "MongoDB",
                    }
                )
            } else {
                String::new()
            },
            if self.is_modified { " [+]" } else { "" },
            if self.is_insert_mode {
                " [INSERT]"
            } else {
                " [NORMAL]"
            }
        );

        // Create editor block
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if self.is_focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            });

        let editor_inner = block.inner(editor_area);
        f.render_widget(block, editor_area);

        // Render editor content
        if self.content.is_empty() {
            // Show welcome message when empty
            let welcome_text = Text::from(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "-- Welcome to LazyTables SQL Query Editor --",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 'i' to enter INSERT mode and start typing SQL",
                    Style::default().fg(Color::Gray),
                )),
                Line::from(Span::styled(
                    "Press Ctrl+Enter to execute your query",
                    Style::default().fg(Color::Gray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Example: SELECT * FROM users LIMIT 10;",
                    Style::default().fg(Color::DarkGray),
                )),
            ]);

            let welcome_paragraph = Paragraph::new(welcome_text).wrap(Wrap { trim: false });

            f.render_widget(welcome_paragraph, editor_inner);
        } else {
            // Render syntax-highlighted content
            let highlighted_text = self.apply_syntax_highlighting(&self.content);

            let paragraph = Paragraph::new(highlighted_text)
                .wrap(Wrap { trim: false })
                .scroll((self.scroll_offset as u16, 0));

            f.render_widget(paragraph, editor_inner);
        }

        // Set cursor position if in insert mode and focused
        if self.is_focused && self.is_insert_mode && !self.content.is_empty() {
            let lines: Vec<&str> = self.content.lines().collect();
            let cursor_y = if self.cursor_line >= self.scroll_offset {
                (self.cursor_line - self.scroll_offset) as u16
            } else {
                0
            };

            let cursor_x = if self.cursor_line < lines.len() {
                self.cursor_col.min(lines[self.cursor_line].len()) as u16
            } else {
                0
            };

            if cursor_y < editor_inner.height && cursor_x < editor_inner.width {
                f.set_cursor_position((editor_inner.x + cursor_x, editor_inner.y + cursor_y));
            }
        }

        // Render help area at bottom if focused
        if let Some(help_area) = help_area {
            self.render_help(f, help_area);
        }

        // Render suggestions popup if active
        if self.suggestions_active {
            let cursor_screen_pos = if self.is_focused && !self.content.is_empty() {
                let lines: Vec<&str> = self.content.lines().collect();
                let cursor_y = if self.cursor_line >= self.scroll_offset {
                    editor_inner.y + (self.cursor_line - self.scroll_offset) as u16
                } else {
                    editor_inner.y
                };

                let cursor_x = if self.cursor_line < lines.len() {
                    editor_inner.x + self.cursor_col.min(lines[self.cursor_line].len()) as u16
                } else {
                    editor_inner.x
                };

                (cursor_x, cursor_y)
            } else {
                (editor_inner.x, editor_inner.y)
            };

            self.suggestion_popup.render(f, cursor_screen_pos, area);
        }
    }

    /// Render help information at the bottom
    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // File info
                Constraint::Length(1), // Mode info
                Constraint::Length(2), // Keybindings
            ])
            .split(area);

        // File info
        let file_info = if let Some(ref filename) = self.current_file {
            format!(
                "File: {}{}",
                filename,
                if self.is_modified { " [modified]" } else { "" }
            )
        } else {
            "New file (unsaved)".to_string()
        };

        let file_paragraph = Paragraph::new(file_info).style(Style::default().fg(Color::Gray));
        f.render_widget(file_paragraph, help_chunks[0]);

        // Mode info
        let mode_info = if self.is_insert_mode {
            "-- INSERT --"
        } else {
            "-- NORMAL --"
        };

        let mode_paragraph = Paragraph::new(mode_info).style(
            Style::default()
                .fg(if self.is_insert_mode {
                    Color::Green
                } else {
                    Color::Yellow
                })
                .add_modifier(Modifier::BOLD),
        );
        f.render_widget(mode_paragraph, help_chunks[1]);

        // Keybindings
        let keybindings = Line::from(vec![
            Span::styled(
                "i",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" insert | ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" normal | ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Ctrl+Enter",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" execute | ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" accept suggestion", Style::default().fg(Color::Gray)),
        ]);

        let keybindings_paragraph = Paragraph::new(keybindings);
        f.render_widget(keybindings_paragraph, help_chunks[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_editor_creation() {
        let editor = QueryEditor::new();
        assert_eq!(editor.get_content(), "");
        assert!(!editor.is_focused());
        assert!(!editor.is_insert_mode());
        assert_eq!(editor.get_database_type(), None);
    }

    #[test]
    fn test_content_setting() {
        let mut editor = QueryEditor::new();
        let content = "SELECT * FROM users;";
        editor.set_content(content.to_string());
        assert_eq!(editor.get_content(), content);
    }

    #[test]
    fn test_database_type_setting() {
        let mut editor = QueryEditor::new();
        editor.set_database_type(Some(DatabaseType::PostgreSQL));
        assert_eq!(editor.get_database_type(), Some(DatabaseType::PostgreSQL));
    }

    #[test]
    fn test_insert_mode_toggle() {
        let mut editor = QueryEditor::new();
        assert!(!editor.is_insert_mode());
        editor.toggle_insert_mode();
        assert!(editor.is_insert_mode());
        editor.toggle_insert_mode();
        assert!(!editor.is_insert_mode());
    }

    #[test]
    fn test_character_insertion() {
        let mut editor = QueryEditor::new();
        editor.set_insert_mode(true);
        editor.insert_char('S');
        editor.insert_char('E');
        editor.insert_char('L');
        assert_eq!(editor.get_content(), "SEL");
    }

    #[test]
    fn test_statement_extraction() {
        let mut editor = QueryEditor::new();
        editor.set_content("SELECT * FROM users;\nSELECT name FROM products;".to_string());

        let statement = editor.get_statement_at_cursor();
        assert!(statement.is_some());
        assert!(statement.unwrap().contains("SELECT * FROM users"));
    }

    #[test]
    fn test_cursor_movement() {
        let mut editor = QueryEditor::new();
        editor.set_content("Line 1\nLine 2\nLine 3".to_string());

        assert_eq!(editor.cursor_line, 0);
        editor.move_cursor_down();
        assert_eq!(editor.cursor_line, 1);
        editor.move_cursor_up();
        assert_eq!(editor.cursor_line, 0);
    }
}
