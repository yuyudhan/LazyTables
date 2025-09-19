// FilePath: src/ui/components/query_editor.rs

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::{SyntaxSet, SyntaxReference},
    util::LinesWithEndings,
};
use crate::database::DatabaseType;

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
        }
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn set_database_type(&mut self, db_type: Option<DatabaseType>) {
        self.database_type = db_type;
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

        self.content = new_lines.join("\n");
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
        self.adjust_scroll();

        self.content = new_lines.join("\n");
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

        let statement_lines: Vec<&str> = lines[start_line..=end_line].iter().copied().collect();
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
            Some(DatabaseType::MySQL) => {
                self.syntax_set.find_syntax_by_extension("sql")
                    .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
            }
            Some(DatabaseType::PostgreSQL) => {
                self.syntax_set.find_syntax_by_extension("sql")
                    .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
            }
            Some(DatabaseType::SQLite) => {
                self.syntax_set.find_syntax_by_extension("sql")
                    .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
            }
            _ => self.syntax_set.find_syntax_plain_text(),
        }
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
                    if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
                        ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
                    }
                    if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
                        ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
                    }
                    if style.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE) {
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

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(format!(
                "SQL Query Editor{}{}",
                if let Some(ref db_type) = self.database_type {
                    format!(" ({})", match db_type {
                        DatabaseType::PostgreSQL => "PostgreSQL",
                        DatabaseType::MySQL => "MySQL",
                        DatabaseType::MariaDB => "MariaDB",
                        DatabaseType::SQLite => "SQLite",
                        DatabaseType::Oracle => "Oracle",
                        DatabaseType::Redis => "Redis",
                        DatabaseType::MongoDB => "MongoDB",
                    })
                } else {
                    String::new()
                },
                if self.is_insert_mode { " [INSERT]" } else { "" }
            ))
            .borders(Borders::ALL)
            .border_style(if self.is_focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            });

        let inner = block.inner(area);
        f.render_widget(block, area);

        let highlighted_text = self.apply_syntax_highlighting(&self.content);

        let paragraph = Paragraph::new(highlighted_text)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset as u16, 0));

        f.render_widget(paragraph, inner);

        if self.is_focused && self.is_insert_mode {
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

            if cursor_y < inner.height && cursor_x < inner.width {
                f.set_cursor_position((inner.x + cursor_x, inner.y + cursor_y));
            }
        }
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