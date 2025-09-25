// FilePath: src/ui/components/sql_suggestions.rs

use crate::database::DatabaseType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SqlSuggestion {
    /// The text to insert
    pub text: String,
    /// Display text (may differ from insert text)
    pub display: String,
    /// Type of suggestion for styling
    pub suggestion_type: SuggestionType,
    /// Optional description
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    Keyword,
    Table,
    Column,
    Function,
    Alias,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SqlContext {
    /// At the beginning or after semicolon
    StartOfStatement,
    /// After SELECT, expecting column names
    SelectColumns,
    /// After FROM, expecting table names
    FromClause,
    /// After WHERE, expecting column names or conditions
    WhereClause,
    /// After JOIN, expecting table names
    JoinClause,
    /// After ON, expecting join conditions
    OnClause,
    /// After ORDER BY, expecting column names
    OrderByClause,
    /// After GROUP BY, expecting column names
    GroupByClause,
    /// General context where any suggestion might apply
    General,
}

#[derive(Debug)]
pub struct SqlSuggestionEngine {
    /// SQL keywords
    keywords: Vec<&'static str>,
    /// SQL functions
    functions: Vec<&'static str>,
    /// Database-specific information
    database_type: Option<DatabaseType>,
    /// Available tables in the current database
    tables: Vec<String>,
    /// Available columns for each table
    table_columns: HashMap<String, Vec<String>>,
}

impl SqlSuggestionEngine {
    pub fn new() -> Self {
        Self {
            keywords: vec![
                // Basic keywords
                "SELECT",
                "FROM",
                "WHERE",
                "ORDER BY",
                "GROUP BY",
                "HAVING",
                "INSERT",
                "INTO",
                "VALUES",
                "UPDATE",
                "SET",
                "DELETE",
                "CREATE",
                "TABLE",
                "ALTER",
                "DROP",
                "INDEX",
                // JOIN keywords
                "JOIN",
                "INNER JOIN",
                "LEFT JOIN",
                "RIGHT JOIN",
                "FULL JOIN",
                "CROSS JOIN",
                "ON",
                "USING",
                // Logical operators
                "AND",
                "OR",
                "NOT",
                "IN",
                "EXISTS",
                "BETWEEN",
                "LIKE",
                "IS",
                "NULL",
                "TRUE",
                "FALSE",
                // Aggregate and comparison
                "DISTINCT",
                "AS",
                "ASC",
                "DESC",
                "LIMIT",
                "OFFSET",
                "CASE",
                "WHEN",
                "THEN",
                "ELSE",
                "END",
                // Data types (common ones)
                "INTEGER",
                "VARCHAR",
                "TEXT",
                "DATE",
                "TIMESTAMP",
                "BOOLEAN",
                "DECIMAL",
                "FLOAT",
                "REAL",
            ],
            functions: vec![
                // Aggregate functions
                "COUNT",
                "SUM",
                "AVG",
                "MIN",
                "MAX",
                // String functions
                "CONCAT",
                "SUBSTRING",
                "LENGTH",
                "UPPER",
                "LOWER",
                "TRIM",
                // Date functions
                "NOW",
                "CURRENT_DATE",
                "CURRENT_TIME",
                "CURRENT_TIMESTAMP",
                "DATE_PART",
                "EXTRACT",
                // Mathematical functions
                "ABS",
                "ROUND",
                "CEIL",
                "FLOOR",
                "POWER",
                "SQRT",
                // Conditional functions
                "COALESCE",
                "NULLIF",
                "GREATEST",
                "LEAST",
            ],
            database_type: None,
            tables: Vec::new(),
            table_columns: HashMap::new(),
        }
    }

    /// Set the current database type for context-aware suggestions
    pub fn set_database_type(&mut self, db_type: Option<DatabaseType>) {
        self.database_type = db_type;
    }

    /// Update available tables
    pub fn set_tables(&mut self, tables: Vec<String>) {
        self.tables = tables;
    }

    /// Update columns for a specific table
    pub fn set_table_columns(&mut self, table: String, columns: Vec<String>) {
        self.table_columns.insert(table, columns);
    }

    /// Get suggestions based on current SQL content and cursor position
    pub fn get_suggestions(
        &self,
        sql_content: &str,
        cursor_line: usize,
        cursor_column: usize,
    ) -> Vec<SqlSuggestion> {
        let context = self.analyze_context(sql_content, cursor_line, cursor_column);
        let partial_word = self.get_partial_word_at_cursor(sql_content, cursor_line, cursor_column);

        let mut suggestions = Vec::new();

        match context {
            SqlContext::StartOfStatement => {
                suggestions.extend(self.get_statement_keywords(&partial_word));
            }
            SqlContext::SelectColumns => {
                suggestions.extend(self.get_column_suggestions(&partial_word));
                suggestions.extend(self.get_function_suggestions(&partial_word));
                suggestions
                    .extend(self.get_keyword_suggestions(&["DISTINCT", "FROM"], &partial_word));
            }
            SqlContext::FromClause => {
                suggestions.extend(self.get_table_suggestions(&partial_word));
            }
            SqlContext::JoinClause => {
                suggestions.extend(self.get_table_suggestions(&partial_word));
            }
            SqlContext::WhereClause | SqlContext::OnClause => {
                suggestions.extend(self.get_column_suggestions(&partial_word));
                suggestions.extend(self.get_keyword_suggestions(
                    &[
                        "AND", "OR", "NOT", "IN", "EXISTS", "BETWEEN", "LIKE", "IS", "NULL",
                    ],
                    &partial_word,
                ));
            }
            SqlContext::OrderByClause | SqlContext::GroupByClause => {
                suggestions.extend(self.get_column_suggestions(&partial_word));
            }
            SqlContext::General => {
                suggestions.extend(self.get_all_suggestions(&partial_word));
            }
        }

        // Filter and sort suggestions
        self.filter_and_sort_suggestions(suggestions, &partial_word)
    }

    /// Analyze SQL context at cursor position
    fn analyze_context(
        &self,
        sql_content: &str,
        cursor_line: usize,
        cursor_column: usize,
    ) -> SqlContext {
        // Get text up to cursor position
        let lines: Vec<&str> = sql_content.lines().collect();
        let mut text_before_cursor = String::new();

        for (i, line) in lines.iter().enumerate() {
            if i < cursor_line {
                text_before_cursor.push_str(line);
                text_before_cursor.push(' ');
            } else if i == cursor_line {
                text_before_cursor.push_str(&line[..cursor_column.min(line.len())]);
                break;
            }
        }

        // Normalize and analyze
        let normalized = text_before_cursor.to_uppercase();
        let tokens: Vec<&str> = normalized.split_whitespace().collect();

        // Find the most recent statement start
        let mut statement_start = 0;
        for (i, token) in tokens.iter().enumerate() {
            if matches!(
                *token,
                "SELECT" | "INSERT" | "UPDATE" | "DELETE" | "CREATE" | "DROP" | "ALTER"
            ) {
                statement_start = i;
            }
        }

        // Analyze context based on recent tokens
        let recent_tokens = &tokens[statement_start..];

        if recent_tokens.is_empty() {
            return SqlContext::StartOfStatement;
        }

        // Look for specific patterns
        if let Some(last_token) = recent_tokens.last() {
            match *last_token {
                "SELECT" => return SqlContext::SelectColumns,
                "FROM" => return SqlContext::FromClause,
                "WHERE" => return SqlContext::WhereClause,
                "JOIN" | "INNER" | "LEFT" | "RIGHT" | "FULL" | "CROSS" => {
                    return SqlContext::JoinClause
                }
                "ON" => return SqlContext::OnClause,
                "BY" if recent_tokens.len() >= 2 => match recent_tokens[recent_tokens.len() - 2] {
                    "ORDER" => return SqlContext::OrderByClause,
                    "GROUP" => return SqlContext::GroupByClause,
                    _ => {}
                },
                _ => {}
            }
        }

        // Check for SELECT context (between SELECT and FROM)
        if recent_tokens.contains(&"SELECT") && !recent_tokens.contains(&"FROM") {
            return SqlContext::SelectColumns;
        }

        // Check for FROM context
        if recent_tokens.contains(&"FROM")
            && !recent_tokens
                .iter()
                .any(|&t| matches!(t, "WHERE" | "GROUP" | "ORDER" | "HAVING"))
        {
            if recent_tokens
                .iter()
                .any(|&t| matches!(t, "JOIN" | "INNER" | "LEFT" | "RIGHT" | "FULL" | "CROSS"))
            {
                return SqlContext::JoinClause;
            }
        }

        SqlContext::General
    }

    /// Get the partial word being typed at cursor position
    fn get_partial_word_at_cursor(
        &self,
        sql_content: &str,
        cursor_line: usize,
        cursor_column: usize,
    ) -> String {
        let lines: Vec<&str> = sql_content.lines().collect();

        if cursor_line >= lines.len() {
            return String::new();
        }

        let line = lines[cursor_line];
        if cursor_column > line.len() {
            return String::new();
        }

        // Find word boundaries around cursor
        let mut start = cursor_column;
        let mut end = cursor_column;

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

    /// Get statement-starting keyword suggestions
    fn get_statement_keywords(&self, partial_word: &str) -> Vec<SqlSuggestion> {
        let statement_keywords = [
            "SELECT", "INSERT", "UPDATE", "DELETE", "CREATE", "ALTER", "DROP",
        ];

        statement_keywords
            .iter()
            .filter(|&keyword| {
                keyword
                    .to_lowercase()
                    .starts_with(&partial_word.to_lowercase())
            })
            .map(|&keyword| SqlSuggestion {
                text: keyword.to_string(),
                display: keyword.to_string(),
                suggestion_type: SuggestionType::Keyword,
                description: Some(self.get_keyword_description(keyword)),
            })
            .collect()
    }

    /// Get table name suggestions
    fn get_table_suggestions(&self, partial_word: &str) -> Vec<SqlSuggestion> {
        self.tables
            .iter()
            .filter(|table| {
                table
                    .to_lowercase()
                    .starts_with(&partial_word.to_lowercase())
            })
            .map(|table| SqlSuggestion {
                text: table.clone(),
                display: table.clone(),
                suggestion_type: SuggestionType::Table,
                description: None,
            })
            .collect()
    }

    /// Get column suggestions based on available tables
    fn get_column_suggestions(&self, partial_word: &str) -> Vec<SqlSuggestion> {
        let mut suggestions = Vec::new();

        // Add columns from all available tables
        for (table, columns) in &self.table_columns {
            for column in columns {
                if column
                    .to_lowercase()
                    .starts_with(&partial_word.to_lowercase())
                {
                    suggestions.push(SqlSuggestion {
                        text: column.clone(),
                        display: format!("{} ({})", column, table),
                        suggestion_type: SuggestionType::Column,
                        description: Some(format!("Column from table {}", table)),
                    });
                }
            }
        }

        suggestions
    }

    /// Get function suggestions
    fn get_function_suggestions(&self, partial_word: &str) -> Vec<SqlSuggestion> {
        self.functions
            .iter()
            .filter(|&func| {
                func.to_lowercase()
                    .starts_with(&partial_word.to_lowercase())
            })
            .map(|&func| SqlSuggestion {
                text: format!("{}()", func),
                display: format!("{}()", func),
                suggestion_type: SuggestionType::Function,
                description: Some(self.get_function_description(func)),
            })
            .collect()
    }

    /// Get keyword suggestions for specific keywords
    fn get_keyword_suggestions(&self, keywords: &[&str], partial_word: &str) -> Vec<SqlSuggestion> {
        keywords
            .iter()
            .filter(|&keyword| {
                keyword
                    .to_lowercase()
                    .starts_with(&partial_word.to_lowercase())
            })
            .map(|&keyword| SqlSuggestion {
                text: keyword.to_string(),
                display: keyword.to_string(),
                suggestion_type: SuggestionType::Keyword,
                description: Some(self.get_keyword_description(keyword)),
            })
            .collect()
    }

    /// Get all suggestions (fallback)
    fn get_all_suggestions(&self, partial_word: &str) -> Vec<SqlSuggestion> {
        let mut suggestions = Vec::new();

        suggestions.extend(self.get_keyword_suggestions(&self.keywords, partial_word));
        suggestions.extend(self.get_function_suggestions(partial_word));
        suggestions.extend(self.get_table_suggestions(partial_word));
        suggestions.extend(self.get_column_suggestions(partial_word));

        suggestions
    }

    /// Filter and sort suggestions based on relevance
    fn filter_and_sort_suggestions(
        &self,
        mut suggestions: Vec<SqlSuggestion>,
        partial_word: &str,
    ) -> Vec<SqlSuggestion> {
        if partial_word.is_empty() {
            suggestions.sort_by(|a, b| a.display.cmp(&b.display));
            suggestions.truncate(20); // Limit to prevent overwhelming UI
            return suggestions;
        }

        let partial_lower = partial_word.to_lowercase();

        // Sort by relevance: exact prefix match first, then contains match
        suggestions.sort_by(|a, b| {
            let a_exact = a.display.to_lowercase().starts_with(&partial_lower);
            let b_exact = b.display.to_lowercase().starts_with(&partial_lower);

            match (a_exact, b_exact) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.display.cmp(&b.display),
            }
        });

        suggestions.truncate(20);
        suggestions
    }

    /// Get description for a keyword
    fn get_keyword_description(&self, keyword: &str) -> String {
        match keyword {
            "SELECT" => "Retrieve data from database tables".to_string(),
            "FROM" => "Specify the source table(s) for the query".to_string(),
            "WHERE" => "Filter rows based on conditions".to_string(),
            "JOIN" => "Combine rows from multiple tables".to_string(),
            "INSERT" => "Add new rows to a table".to_string(),
            "UPDATE" => "Modify existing rows in a table".to_string(),
            "DELETE" => "Remove rows from a table".to_string(),
            "ORDER BY" => "Sort the result set".to_string(),
            "GROUP BY" => "Group rows with the same values".to_string(),
            _ => format!("SQL keyword: {}", keyword),
        }
    }

    /// Get description for a function
    fn get_function_description(&self, function: &str) -> String {
        match function {
            "COUNT" => "Count the number of rows".to_string(),
            "SUM" => "Calculate the sum of values".to_string(),
            "AVG" => "Calculate the average of values".to_string(),
            "MIN" => "Find the minimum value".to_string(),
            "MAX" => "Find the maximum value".to_string(),
            "CONCAT" => "Concatenate strings".to_string(),
            "UPPER" => "Convert text to uppercase".to_string(),
            "LOWER" => "Convert text to lowercase".to_string(),
            "NOW" => "Get the current timestamp".to_string(),
            _ => format!("SQL function: {}", function),
        }
    }
}

impl Default for SqlSuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_keyword_suggestions() {
        let engine = SqlSuggestionEngine::new();
        let suggestions = engine.get_suggestions("SEL", 0, 3);

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.text == "SELECT"));
    }

    #[test]
    fn test_context_analysis() {
        let engine = SqlSuggestionEngine::new();

        // Test SELECT context
        let context = engine.analyze_context("SELECT ", 0, 7);
        assert_eq!(context, SqlContext::SelectColumns);

        // Test FROM context
        let context = engine.analyze_context("SELECT * FROM ", 0, 14);
        assert_eq!(context, SqlContext::FromClause);
    }

    #[test]
    fn test_partial_word_extraction() {
        let engine = SqlSuggestionEngine::new();

        let partial = engine.get_partial_word_at_cursor("SELECT cou", 0, 10);
        assert_eq!(partial, "cou");

        let partial = engine.get_partial_word_at_cursor("FROM user_ta", 0, 12);
        assert_eq!(partial, "user_ta");
    }
}
