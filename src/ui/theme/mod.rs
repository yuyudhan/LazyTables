// FilePath: src/ui/theme/mod.rs

use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub author: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // Core UI colors
    pub background: String,
    pub foreground: String,
    pub text: String,
    pub selection_bg: String,
    pub cursor: String,
    
    // Pane colors
    pub pane_background: String,
    pub border: String,
    pub active_border: String,
    pub inactive_pane: String,
    
    // Component colors
    pub header_fg: String,
    pub status_bg: String,
    pub status_fg: String,
    pub primary_highlight: String,
    
    // Table colors
    pub table_header_bg: String,
    pub table_header_fg: String,
    pub table_row_bg: String,
    pub table_row_alt_bg: String,
    pub selected_cell_bg: String,
    
    // Modal colors
    pub modal_bg: String,
    pub modal_border: String,
    pub modal_title: String,
    
    // Input field colors
    pub input_bg: String,
    pub input_fg: String,
    pub input_border: String,
    pub input_active_border: String,
    pub input_placeholder: String,
    
    // Button colors
    pub button_bg: String,
    pub button_fg: String,
    pub button_active_bg: String,
    pub button_active_fg: String,
    
    // Status colors
    pub success: String,
    pub error: String,
    pub warning: String,
    pub info: String,
    
    // SQL editor colors
    pub editor_bg: String,
    pub editor_fg: String,
    pub editor_line_number: String,
    pub editor_cursor_line: String,
    pub editor_selection: String,
    
    // Syntax highlighting
    pub syntax_keyword: String,
    pub syntax_string: String,
    pub syntax_number: String,
    pub syntax_comment: String,
    pub syntax_function: String,
    pub syntax_operator: String,
    
    // Toast colors
    pub toast_success_bg: String,
    pub toast_error_bg: String,
    pub toast_warning_bg: String,
    pub toast_info_bg: String,
    
    // Help colors
    pub help_bg: String,
    pub help_fg: String,
    pub help_header: String,
    pub help_key: String,
    pub help_description: String,
}

impl Theme {
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }
    
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Ok(Self::from_toml(&content)?)
    }
    
    pub fn parse_color(hex: &str) -> Color {
        if let Some(hex) = hex.strip_prefix('#') {
            if hex.len() == 6 {
                if let Ok(rgb) = u32::from_str_radix(hex, 16) {
                    return Color::Rgb(
                        ((rgb >> 16) & 0xFF) as u8,
                        ((rgb >> 8) & 0xFF) as u8,
                        (rgb & 0xFF) as u8,
                    );
                }
            }
        }
        Color::White
    }
    
    pub fn get_color(&self, key: &str) -> Color {
        let color_str = match key {
            "background" => &self.colors.background,
            "foreground" => &self.colors.foreground,
            "text" => &self.colors.text,
            "selection_bg" => &self.colors.selection_bg,
            "cursor" => &self.colors.cursor,
            "pane_background" => &self.colors.pane_background,
            "border" => &self.colors.border,
            "active_border" => &self.colors.active_border,
            "inactive_pane" => &self.colors.inactive_pane,
            "header_fg" => &self.colors.header_fg,
            "status_bg" => &self.colors.status_bg,
            "status_fg" => &self.colors.status_fg,
            "primary_highlight" => &self.colors.primary_highlight,
            "table_header_bg" => &self.colors.table_header_bg,
            "table_header_fg" => &self.colors.table_header_fg,
            "table_row_bg" => &self.colors.table_row_bg,
            "table_row_alt_bg" => &self.colors.table_row_alt_bg,
            "selected_cell_bg" => &self.colors.selected_cell_bg,
            "modal_bg" => &self.colors.modal_bg,
            "modal_border" => &self.colors.modal_border,
            "modal_title" => &self.colors.modal_title,
            "input_bg" => &self.colors.input_bg,
            "input_fg" => &self.colors.input_fg,
            "input_border" => &self.colors.input_border,
            "input_active_border" => &self.colors.input_active_border,
            "input_placeholder" => &self.colors.input_placeholder,
            "button_bg" => &self.colors.button_bg,
            "button_fg" => &self.colors.button_fg,
            "button_active_bg" => &self.colors.button_active_bg,
            "button_active_fg" => &self.colors.button_active_fg,
            "success" => &self.colors.success,
            "error" => &self.colors.error,
            "warning" => &self.colors.warning,
            "info" => &self.colors.info,
            "editor_bg" => &self.colors.editor_bg,
            "editor_fg" => &self.colors.editor_fg,
            "editor_line_number" => &self.colors.editor_line_number,
            "editor_cursor_line" => &self.colors.editor_cursor_line,
            "editor_selection" => &self.colors.editor_selection,
            "syntax_keyword" => &self.colors.syntax_keyword,
            "syntax_string" => &self.colors.syntax_string,
            "syntax_number" => &self.colors.syntax_number,
            "syntax_comment" => &self.colors.syntax_comment,
            "syntax_function" => &self.colors.syntax_function,
            "syntax_operator" => &self.colors.syntax_operator,
            "toast_success_bg" => &self.colors.toast_success_bg,
            "toast_error_bg" => &self.colors.toast_error_bg,
            "toast_warning_bg" => &self.colors.toast_warning_bg,
            "toast_info_bg" => &self.colors.toast_info_bg,
            "help_bg" => &self.colors.help_bg,
            "help_fg" => &self.colors.help_fg,
            "help_header" => &self.colors.help_header,
            "help_key" => &self.colors.help_key,
            "help_description" => &self.colors.help_description,
            _ => "#ffffff",
        };
        Self::parse_color(color_str)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark_theme()
    }
}

impl Theme {
    pub fn dark_theme() -> Self {
        Self {
            name: "LazyDark".to_string(),
            author: "LazyTables Team".to_string(),
            colors: ThemeColors {
                // Core UI colors
                background: "#0d0d0d".to_string(),
                foreground: "#cdd6f4".to_string(),
                text: "#ffffff".to_string(),
                selection_bg: "#45475a".to_string(),
                cursor: "#f5e0dc".to_string(),
                
                // Pane colors
                pane_background: "#181825".to_string(),
                border: "#313244".to_string(),
                active_border: "#74c7ec".to_string(),
                inactive_pane: "#45475a".to_string(),
                
                // Component colors
                header_fg: "#cba6f7".to_string(),
                status_bg: "#313244".to_string(),
                status_fg: "#cdd6f4".to_string(),
                primary_highlight: "#74c7ec".to_string(),
                
                // Table colors
                table_header_bg: "#313244".to_string(),
                table_header_fg: "#cba6f7".to_string(),
                table_row_bg: "#181825".to_string(),
                table_row_alt_bg: "#1e1e2e".to_string(),
                selected_cell_bg: "#45475a".to_string(),
                
                // Modal colors
                modal_bg: "#0d0d0d".to_string(),
                modal_border: "#74c7ec".to_string(),
                modal_title: "#74c7ec".to_string(),
                
                // Input field colors
                input_bg: "#1e1e2e".to_string(),
                input_fg: "#ffffff".to_string(),
                input_border: "#45475a".to_string(),
                input_active_border: "#74c7ec".to_string(),
                input_placeholder: "#6c7086".to_string(),
                
                // Button colors
                button_bg: "#0000ff".to_string(),
                button_fg: "#000000".to_string(),
                button_active_bg: "#74c7ec".to_string(),
                button_active_fg: "#000000".to_string(),
                
                // Status colors
                success: "#00ff00".to_string(),
                error: "#ff0000".to_string(),
                warning: "#ffff00".to_string(),
                info: "#00ffff".to_string(),
                
                // SQL editor colors
                editor_bg: "#1e1e2e".to_string(),
                editor_fg: "#cdd6f4".to_string(),
                editor_line_number: "#6c7086".to_string(),
                editor_cursor_line: "#313244".to_string(),
                editor_selection: "#45475a".to_string(),
                
                // Syntax highlighting
                syntax_keyword: "#cba6f7".to_string(),
                syntax_string: "#a6e3a1".to_string(),
                syntax_number: "#fab387".to_string(),
                syntax_comment: "#6c7086".to_string(),
                syntax_function: "#89b4fa".to_string(),
                syntax_operator: "#f5c2e7".to_string(),
                
                // Toast colors
                toast_success_bg: "#285028".to_string(),
                toast_error_bg: "#502828".to_string(),
                toast_warning_bg: "#505028".to_string(),
                toast_info_bg: "#283c50".to_string(),
                
                // Help colors
                help_bg: "#1e1e2e".to_string(),
                help_fg: "#cdd6f4".to_string(),
                help_header: "#cba6f7".to_string(),
                help_key: "#74c7ec".to_string(),
                help_description: "#bac2de".to_string(),
            },
        }
    }
    
    pub fn light_theme() -> Self {
        Self {
            name: "LazyLight".to_string(),
            author: "LazyTables Team".to_string(),
            colors: ThemeColors {
                // Core UI colors
                background: "#ffffff".to_string(),
                foreground: "#4c4f69".to_string(),
                text: "#000000".to_string(),
                selection_bg: "#dce0e8".to_string(),
                cursor: "#dc8a78".to_string(),
                
                // Pane colors
                pane_background: "#eff1f5".to_string(),
                border: "#ccd0da".to_string(),
                active_border: "#1e66f5".to_string(),
                inactive_pane: "#bcc0cc".to_string(),
                
                // Component colors
                header_fg: "#8839ef".to_string(),
                status_bg: "#e6e9ef".to_string(),
                status_fg: "#4c4f69".to_string(),
                primary_highlight: "#1e66f5".to_string(),
                
                // Table colors
                table_header_bg: "#e6e9ef".to_string(),
                table_header_fg: "#8839ef".to_string(),
                table_row_bg: "#ffffff".to_string(),
                table_row_alt_bg: "#f5f5f5".to_string(),
                selected_cell_bg: "#dce0e8".to_string(),
                
                // Modal colors
                modal_bg: "#ffffff".to_string(),
                modal_border: "#1e66f5".to_string(),
                modal_title: "#1e66f5".to_string(),
                
                // Input field colors
                input_bg: "#f5f5f5".to_string(),
                input_fg: "#000000".to_string(),
                input_border: "#ccd0da".to_string(),
                input_active_border: "#1e66f5".to_string(),
                input_placeholder: "#9ca0b0".to_string(),
                
                // Button colors
                button_bg: "#1e66f5".to_string(),
                button_fg: "#ffffff".to_string(),
                button_active_bg: "#0d52bf".to_string(),
                button_active_fg: "#ffffff".to_string(),
                
                // Status colors
                success: "#40a02b".to_string(),
                error: "#d20f39".to_string(),
                warning: "#df8e1d".to_string(),
                info: "#209fb5".to_string(),
                
                // SQL editor colors
                editor_bg: "#f5f5f5".to_string(),
                editor_fg: "#4c4f69".to_string(),
                editor_line_number: "#9ca0b0".to_string(),
                editor_cursor_line: "#e6e9ef".to_string(),
                editor_selection: "#dce0e8".to_string(),
                
                // Syntax highlighting
                syntax_keyword: "#8839ef".to_string(),
                syntax_string: "#40a02b".to_string(),
                syntax_number: "#fe640b".to_string(),
                syntax_comment: "#9ca0b0".to_string(),
                syntax_function: "#1e66f5".to_string(),
                syntax_operator: "#ea76cb".to_string(),
                
                // Toast colors
                toast_success_bg: "#d4f4dd".to_string(),
                toast_error_bg: "#f4d4d4".to_string(),
                toast_warning_bg: "#f4e6d4".to_string(),
                toast_info_bg: "#d4e6f4".to_string(),
                
                // Help colors
                help_bg: "#f5f5f5".to_string(),
                help_fg: "#4c4f69".to_string(),
                help_header: "#8839ef".to_string(),
                help_key: "#1e66f5".to_string(),
                help_description: "#5c5f77".to_string(),
            },
        }
    }
    
    pub fn load_from_config(config_path: Option<&Path>) -> Self {
        if let Some(path) = config_path {
            if let Ok(theme) = Self::load_from_file(path) {
                return theme;
            }
        }
        Self::default()
    }
}

pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    current_theme: String,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        
        let dark = Theme::dark_theme();
        let light = Theme::light_theme();
        
        themes.insert(dark.name.clone(), dark);
        themes.insert(light.name.clone(), light);
        
        Self {
            themes,
            current_theme: "LazyDark".to_string(),
        }
    }
    
    pub fn current(&self) -> &Theme {
        self.themes
            .get(&self.current_theme)
            .unwrap_or(&self.themes["LazyDark"])
    }
    
    pub fn switch_theme(&mut self, name: &str) -> bool {
        if self.themes.contains_key(name) {
            self.current_theme = name.to_string();
            true
        } else {
            false
        }
    }
    
    pub fn add_theme(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), theme);
    }
    
    pub fn list_themes(&self) -> Vec<&str> {
        self.themes.keys().map(|s| s.as_str()).collect()
    }
}