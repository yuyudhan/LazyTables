// FilePath: src/ui/components/query_editor.rs

use super::{SqlSuggestionEngine, SuggestionPopup};
use crate::database::DatabaseType;
use ratatui::{
    layout::Rect,
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
    /// Pending vim command (for commands like 'dd', 'dw', etc.)
    pending_command: Option<String>,
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
            pending_command: None,
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
            pending_command: None,
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

    // Vim motion methods

    /// Move to beginning of current line (0 key)
    pub fn move_to_line_start(&mut self) {
        self.cursor_col = 0;
    }

    /// Move to end of current line ($ key)
    pub fn move_to_line_end(&mut self) {
        let lines = self.content.lines().collect::<Vec<_>>();
        if self.cursor_line < lines.len() {
            self.cursor_col = lines[self.cursor_line].len();
        }
    }

    /// Move to beginning of file (gg)
    pub fn move_to_file_start(&mut self) {
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    /// Move to end of file (G)
    pub fn move_to_file_end(&mut self) {
        let lines = self.content.lines().collect::<Vec<_>>();
        if !lines.is_empty() {
            self.cursor_line = lines.len() - 1;
            self.cursor_col = lines[self.cursor_line].len();
            self.adjust_scroll();
        }
    }

    /// Move to next word (w key)
    pub fn move_to_next_word(&mut self) {
        let lines = self.content.lines().collect::<Vec<_>>();
        if self.cursor_line >= lines.len() {
            return;
        }

        let current_line = lines[self.cursor_line];
        let chars: Vec<char> = current_line.chars().collect();

        // If at end of line, move to beginning of next line
        if self.cursor_col >= chars.len() {
            if self.cursor_line < lines.len() - 1 {
                self.cursor_line += 1;
                self.cursor_col = 0;
                self.adjust_scroll();
                // Skip leading whitespace
                self.skip_whitespace_forward();
            }
            return;
        }

        // Skip current word
        while self.cursor_col < chars.len() && !chars[self.cursor_col].is_whitespace() {
            self.cursor_col += 1;
        }

        // Skip whitespace
        while self.cursor_col < chars.len() && chars[self.cursor_col].is_whitespace() {
            self.cursor_col += 1;
        }

        // If we reached end of line, move to next line
        if self.cursor_col >= chars.len() && self.cursor_line < lines.len() - 1 {
            self.cursor_line += 1;
            self.cursor_col = 0;
            self.adjust_scroll();
            self.skip_whitespace_forward();
        }
    }

    /// Move to previous word (b key)
    pub fn move_to_prev_word(&mut self) {
        let lines = self.content.lines().collect::<Vec<_>>();
        if self.cursor_line >= lines.len() {
            return;
        }

        // If at beginning of line, move to end of previous line
        if self.cursor_col == 0 {
            if self.cursor_line > 0 {
                self.cursor_line -= 1;
                self.cursor_col = lines[self.cursor_line].len();
                self.adjust_scroll();
                // Move to beginning of last word on previous line
                self.move_to_word_beginning();
            }
            return;
        }

        let current_line = lines[self.cursor_line];
        let chars: Vec<char> = current_line.chars().collect();

        // Move back to find beginning of current or previous word
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }

        // Skip whitespace backwards
        while self.cursor_col > 0
            && self.cursor_col < chars.len()
            && chars[self.cursor_col].is_whitespace()
        {
            self.cursor_col -= 1;
        }

        // Skip word backwards to find its beginning
        while self.cursor_col > 0 && !chars[self.cursor_col - 1].is_whitespace() {
            self.cursor_col -= 1;
        }
    }

    /// Move to end of current word (e key)
    pub fn move_to_end_of_word(&mut self) {
        let lines = self.content.lines().collect::<Vec<_>>();
        if self.cursor_line >= lines.len() {
            return;
        }

        let current_line = lines[self.cursor_line];
        let chars: Vec<char> = current_line.chars().collect();

        if chars.is_empty() {
            return;
        }

        // If we're at a word character, move to end of current word
        if self.cursor_col < chars.len() && !chars[self.cursor_col].is_whitespace() {
            while self.cursor_col < chars.len() - 1 && !chars[self.cursor_col + 1].is_whitespace() {
                self.cursor_col += 1;
            }
        } else {
            // Skip whitespace to find next word
            while self.cursor_col < chars.len() && chars[self.cursor_col].is_whitespace() {
                self.cursor_col += 1;
            }
            // Move to end of that word
            while self.cursor_col < chars.len() - 1 && !chars[self.cursor_col + 1].is_whitespace() {
                self.cursor_col += 1;
            }
        }
    }

    /// Delete current line (dd)
    pub fn delete_current_line(&mut self) {
        if self.is_insert_mode {
            return;
        }

        let lines: Vec<String> = self.content.lines().map(|s| s.to_string()).collect();
        if lines.is_empty() {
            return;
        }

        let mut new_lines = lines;

        // Remove current line
        if self.cursor_line < new_lines.len() {
            new_lines.remove(self.cursor_line);
        }

        // Adjust cursor position
        if new_lines.is_empty() {
            self.cursor_line = 0;
            self.cursor_col = 0;
        } else {
            if self.cursor_line >= new_lines.len() {
                self.cursor_line = new_lines.len() - 1;
            }
            self.adjust_cursor_column();
        }

        self.content = new_lines.join("\n");
        self.is_modified = true;
        self.adjust_scroll();
    }

    /// Delete to end of line (d$ or D)
    pub fn delete_to_line_end(&mut self) {
        if self.is_insert_mode {
            return;
        }

        let lines: Vec<String> = self.content.lines().map(|s| s.to_string()).collect();
        let mut new_lines = lines;

        if self.cursor_line < new_lines.len() {
            let line = &mut new_lines[self.cursor_line];
            if self.cursor_col < line.len() {
                line.truncate(self.cursor_col);
                self.is_modified = true;
            }
        }

        self.content = new_lines.join("\n");
    }

    /// Delete word under cursor (dw)
    pub fn delete_word(&mut self) {
        if self.is_insert_mode {
            return;
        }

        let lines: Vec<String> = self.content.lines().map(|s| s.to_string()).collect();
        let mut new_lines = lines;

        if self.cursor_line < new_lines.len() {
            let line = &mut new_lines[self.cursor_line];
            let mut chars: Vec<char> = line.chars().collect();

            if self.cursor_col < chars.len() {
                let start_col = self.cursor_col;
                let mut end_col = start_col;

                // Find end of word
                while end_col < chars.len() && !chars[end_col].is_whitespace() {
                    end_col += 1;
                }

                // Include trailing whitespace
                while end_col < chars.len() && chars[end_col].is_whitespace() {
                    end_col += 1;
                }

                // Remove the word
                chars.drain(start_col..end_col);
                *line = chars.into_iter().collect();
                self.is_modified = true;

                // Adjust cursor
                if self.cursor_col > line.len() {
                    self.cursor_col = line.len();
                }
            }
        }

        self.content = new_lines.join("\n");
    }

    /// Helper method to skip whitespace forward
    fn skip_whitespace_forward(&mut self) {
        let lines = self.content.lines().collect::<Vec<_>>();
        if self.cursor_line < lines.len() {
            let chars: Vec<char> = lines[self.cursor_line].chars().collect();
            while self.cursor_col < chars.len() && chars[self.cursor_col].is_whitespace() {
                self.cursor_col += 1;
            }
        }
    }

    /// Helper method to move to beginning of word
    fn move_to_word_beginning(&mut self) {
        let lines = self.content.lines().collect::<Vec<_>>();
        if self.cursor_line < lines.len() {
            let chars: Vec<char> = lines[self.cursor_line].chars().collect();

            // Skip whitespace backwards
            while self.cursor_col > 0
                && self.cursor_col <= chars.len()
                && (self.cursor_col == chars.len() || chars[self.cursor_col].is_whitespace())
            {
                self.cursor_col -= 1;
            }

            // Move to beginning of word
            while self.cursor_col > 0 && !chars[self.cursor_col - 1].is_whitespace() {
                self.cursor_col -= 1;
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

    /// Handle vim command input
    pub fn handle_vim_command(&mut self, ch: char) -> bool {
        // If we have a pending command, try to complete it
        if let Some(ref pending) = self.pending_command.clone() {
            let full_command = format!("{}{}", pending, ch);

            match full_command.as_str() {
                "dd" => {
                    self.delete_current_line();
                    self.pending_command = None;
                    return true;
                }
                "dw" => {
                    self.delete_word();
                    self.pending_command = None;
                    return true;
                }
                "d$" => {
                    self.delete_to_line_end();
                    self.pending_command = None;
                    return true;
                }
                _ => {
                    // Invalid command, clear pending
                    self.pending_command = None;
                    return false;
                }
            }
        }

        // Start new command
        match ch {
            'd' => {
                self.pending_command = Some("d".to_string());
                true
            }
            _ => false,
        }
    }

    /// Cancel any pending vim command
    pub fn cancel_pending_command(&mut self) {
        self.pending_command = None;
    }

    /// Check if there's a pending vim command
    pub fn has_pending_command(&self) -> bool {
        self.pending_command.is_some()
    }

    /// Get the current pending command for display
    pub fn get_pending_command(&self) -> Option<&String> {
        self.pending_command.as_ref()
    }

    fn apply_syntax_highlighting_with_line_numbers(&self, text: &str) -> Text<'static> {
        let syntax = self.get_syntax();
        let theme = &self.theme_set.themes["base16-ocean.dark"];

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut styled_lines = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let total_lines = lines.len();
        let line_number_width = format!("{}", total_lines).len().max(3); // At least 3 digits

        for (line_index, line_content) in lines.iter().enumerate() {
            let line_number = line_index + 1;

            // Create line number span with proper formatting
            let line_number_text = format!("{:>width$} │ ", line_number, width = line_number_width);
            let line_number_style = if line_index == self.cursor_line {
                // Highlight current line number
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let mut spans = vec![Span::styled(line_number_text, line_number_style)];

            // Add syntax highlighting for the actual line content
            let line_with_newline = format!("{}\n", line_content);
            if let Ok(ranges) = highlighter.highlight_line(&line_with_newline, &self.syntax_set) {
                for (style, text) in ranges {
                    // Skip the newline character we added and convert to owned string
                    let text_content = text.trim_end_matches('\n').to_string();
                    if !text_content.is_empty() {
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

                        spans.push(Span::styled(text_content, ratatui_style));
                    }
                }
            } else {
                // Fallback for lines that can't be highlighted
                spans.push(Span::raw(line_content.to_string()));
            }

            styled_lines.push(Line::from(spans));
        }

        Text::from(styled_lines)
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // No inline help - all help goes to help modal (accessible with '?')
        let editor_area = area;

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
            // Render syntax-highlighted content with line numbers
            let highlighted_text = self.apply_syntax_highlighting_with_line_numbers(&self.content);

            let paragraph = Paragraph::new(highlighted_text)
                .wrap(Wrap { trim: false })
                .scroll((self.scroll_offset as u16, 0));

            f.render_widget(paragraph, editor_inner);
        }

        // Set cursor position if focused (both insert and normal modes)
        if self.is_focused && !self.content.is_empty() {
            let lines: Vec<&str> = self.content.lines().collect();
            let cursor_y = if self.cursor_line >= self.scroll_offset {
                (self.cursor_line - self.scroll_offset) as u16
            } else {
                0
            };

            // Calculate line number width to offset cursor position
            let total_lines = lines.len();
            let line_number_width = format!("{}", total_lines).len().max(3);
            let line_number_offset = (line_number_width + 3) as u16; // +3 for " │ "

            let cursor_x = if self.cursor_line < lines.len() {
                line_number_offset + self.cursor_col.min(lines[self.cursor_line].len()) as u16
            } else {
                line_number_offset
            };

            if cursor_y < editor_inner.height && cursor_x < editor_inner.width {
                f.set_cursor_position((editor_inner.x + cursor_x, editor_inner.y + cursor_y));
            }
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

                // Calculate line number width to offset cursor position
                let total_lines = lines.len();
                let line_number_width = format!("{}", total_lines).len().max(3);
                let line_number_offset = (line_number_width + 3) as u16; // +3 for " │ "

                let cursor_x = if self.cursor_line < lines.len() {
                    editor_inner.x
                        + line_number_offset
                        + self.cursor_col.min(lines[self.cursor_line].len()) as u16
                } else {
                    editor_inner.x + line_number_offset
                };

                (cursor_x, cursor_y)
            } else {
                (editor_inner.x, editor_inner.y)
            };

            self.suggestion_popup.render(f, cursor_screen_pos, area);
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
