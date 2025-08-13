// FilePath: src/themes/mod.rs

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Theme definition for LazyTables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub author: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub selection: String,
    pub cursor: String,
    pub pane_background: String,
    pub primary_highlight: String,
    pub selected_cell_background: String,
    pub table_header_text: String,
    pub status_bar_background: String,
}

impl Theme {
    /// Load theme from TOML string
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Convert hex color to Ratatui Color
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
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "LazyDark".to_string(),
            author: "LazyTables Team".to_string(),
            colors: ThemeColors {
                background: "#1e1e2e".to_string(),
                foreground: "#cdd6f4".to_string(),
                selection: "#45475a".to_string(),
                cursor: "#f5e0dc".to_string(),
                pane_background: "#181825".to_string(),
                primary_highlight: "#74c7ec".to_string(),
                selected_cell_background: "#45475a".to_string(),
                table_header_text: "#cba6f7".to_string(),
                status_bar_background: "#313244".to_string(),
            },
        }
    }
}
