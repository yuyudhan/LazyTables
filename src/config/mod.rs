// FilePath: src/config/mod.rs

use crate::core::error::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Theme configuration
    pub theme: ThemeConfig,
    /// Editor preferences
    pub editor: EditorConfig,
    /// Connection settings
    pub connections: ConnectionsConfig,
    /// Keybindings
    pub keybindings: KeybindingsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub dark_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub tab_size: usize,
    pub show_line_numbers: bool,
    pub highlight_current_line: bool,
    pub auto_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionsConfig {
    pub auto_reconnect: bool,
    pub connection_timeout: u64,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    pub leader_key: String,
}

impl Config {
    /// Load configuration from file or create default
    pub fn load(path: Option<PathBuf>) -> Result<Self> {
        let config_path = path.unwrap_or_else(Self::default_path);

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            let config = Self::default();
            // Try to save default config
            let _ = config.save(&config_path);
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    /// Get default configuration path
    pub fn default_path() -> PathBuf {
        ProjectDirs::from("com", "lazytables", "LazyTables")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from(".config/lazytables/config.toml"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeConfig {
                name: "LazyDark".to_string(),
                dark_mode: true,
            },
            editor: EditorConfig {
                tab_size: 4,
                show_line_numbers: true,
                highlight_current_line: true,
                auto_complete: true,
            },
            connections: ConnectionsConfig {
                auto_reconnect: true,
                connection_timeout: 5000,
                max_connections: 10,
            },
            keybindings: KeybindingsConfig {
                leader_key: " ".to_string(),
            },
        }
    }
}

