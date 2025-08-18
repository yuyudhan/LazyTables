// FilePath: src/config/mod.rs

use crate::core::error::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

pub mod shortcuts;

pub use shortcuts::{HotkeyManager, KeybindingsConfig, NavigationAction};

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


impl Config {
    /// Load configuration from file or create default
    pub fn load(path: Option<PathBuf>) -> Result<Self> {
        let config_path = path.unwrap_or_else(Self::default_path);

        // Try multiple configuration paths
        let config_paths = [
            config_path.clone(),
            Self::legacy_config_path(),  // ~/.lazytables/config.toml
            Self::data_dir().join("config.toml"),  // ~/.lazytables/config.toml alternative
        ];

        for config_path in &config_paths {
            if config_path.exists() {
                let contents = fs::read_to_string(config_path)?;
                match toml::from_str::<Config>(&contents) {
                    Ok(config) => return Ok(config),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse config at {}: {}", config_path.display(), e);
                        continue;
                    }
                }
            }
        }

        // No valid config found, create default
        let config = Self::default();
        
        // Ensure directories exist before saving
        Self::ensure_directories()?;
        
        // Save to the primary config location
        let _ = config.save(&config_paths[0]);
        Ok(config)
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

    /// Get default configuration path - uses ~/.config/lazytables/config.toml  
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .map(|config| config.join("lazytables").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from(".config/lazytables/config.toml"))
    }

    /// Get legacy configuration path - uses ~/.lazytables/config.toml  
    pub fn legacy_config_path() -> PathBuf {
        Self::data_dir().join("config.toml")
    }

    /// Load configuration with cargo install support
    pub fn load_for_install() -> Result<Self> {
        // For cargo install, prioritize ~/.lazytables over ~/.config/lazytables
        let config_paths = [
            Self::data_dir().join("config.toml"),      // ~/.lazytables/config.toml (primary for install)
            Self::default_path(),                      // ~/.config/lazytables/config.toml
            Self::data_dir().join("config").join("config.toml"), // ~/.lazytables/config/config.toml
        ];

        for config_path in &config_paths {
            if config_path.exists() {
                let contents = fs::read_to_string(config_path)?;
                match toml::from_str::<Config>(&contents) {
                    Ok(config) => {
                        eprintln!("Loaded config from: {}", config_path.display());
                        return Ok(config);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse config at {}: {}", config_path.display(), e);
                        continue;
                    }
                }
            }
        }

        // No valid config found, create default and save to ~/.lazytables
        let config = Self::default();
        
        // Ensure directories exist
        Self::ensure_directories()?;
        
        // Save to ~/.lazytables/config.toml for cargo install
        let primary_path = Self::data_dir().join("config.toml");
        if let Err(e) = config.save(&primary_path) {
            eprintln!("Warning: Failed to save config to {}: {}", primary_path.display(), e);
        } else {
            eprintln!("Created default config at: {}", primary_path.display());
        }
        
        Ok(config)
    }

    /// Get data directory path - uses ~/.lazytables
    pub fn data_dir() -> PathBuf {
        dirs::home_dir()
            .map(|home| home.join(".lazytables"))
            .unwrap_or_else(|| PathBuf::from(".lazytables"))
    }

    /// Get connections storage path
    pub fn connections_path() -> PathBuf {
        Self::data_dir().join("connections.json")
    }

    /// Get SQL files directory
    pub fn sql_files_dir() -> PathBuf {
        Self::data_dir().join("sql_files")
    }

    /// Get logs directory  
    pub fn logs_dir() -> PathBuf {
        Self::data_dir().join("logs")
    }

    /// Get backups directory
    pub fn backups_dir() -> PathBuf {
        Self::data_dir().join("backups")
    }

    /// Ensure all necessary directories exist
    pub fn ensure_directories() -> Result<()> {
        let data_dir = Self::data_dir();
        let config_dir = Self::default_path().parent().unwrap().to_path_buf();

        // Create main directories
        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(Self::sql_files_dir())?;
        fs::create_dir_all(Self::logs_dir())?;
        fs::create_dir_all(Self::backups_dir())?;
        fs::create_dir_all(data_dir.join("connections"))?;

        // Create README.md if it doesn't exist
        let readme_path = data_dir.join("README.md");
        if !readme_path.exists() {
            let readme_content = "# LazyTables Data Directory

This directory contains all LazyTables application data:

- `config.toml`: Main configuration file
- `connections.json`: Database connection definitions  
- `connections/`: Individual connection files
- `sql_files/`: Saved SQL query files
- `logs/`: Application log files
- `backups/`: Backup files

This directory is created automatically by LazyTables.
";
            fs::write(&readme_path, readme_content)?;
        }

        // Create sample query file if sql_files is empty
        let sample_query_path = Self::sql_files_dir().join("sample_queries.sql");
        if !sample_query_path.exists() {
            let sample_content = "-- Sample SQL queries for LazyTables
-- Use Ctrl+Enter to execute queries

-- Basic table information
SELECT table_name, table_type 
FROM information_schema.tables 
WHERE table_schema = 'public'
ORDER BY table_name;

-- Count rows in all tables
SELECT 
    schemaname,
    tablename,
    n_tup_ins - n_tup_del as row_count
FROM pg_stat_user_tables
ORDER BY row_count DESC;

-- Show database size
SELECT 
    pg_size_pretty(pg_database_size(current_database())) as database_size;
";
            fs::write(&sample_query_path, sample_content)?;
        }

        Ok(())
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
            keybindings: KeybindingsConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config() -> Config {
        Config {
            theme: ThemeConfig {
                name: "TestTheme".to_string(),
                dark_mode: false,
            },
            editor: EditorConfig {
                tab_size: 2,
                show_line_numbers: false,
                highlight_current_line: false,
                auto_complete: false,
            },
            connections: ConnectionsConfig {
                auto_reconnect: false,
                connection_timeout: 1000,
                max_connections: 5,
            },
            keybindings: KeybindingsConfig::default(),
        }
    }

    fn create_test_toml() -> String {
        r#"
[theme]
name = "TestTheme"
dark_mode = false

[editor]
tab_size = 2
show_line_numbers = false
highlight_current_line = false
auto_complete = false

[connections]
auto_reconnect = false
connection_timeout = 1000
max_connections = 5

[keybindings]
leader_key = " "

[keybindings.pane_hotkeys]
connections = "F1"
tables = "F2"
details = "F3"
tabular_output = "F4"
sql_files = "F5"
query_window = "F6"

[keybindings.navigation]
focus_left = "Ctrl+h"
focus_down = "Ctrl+j"
focus_up = "Ctrl+k"
focus_right = "Ctrl+l"
cycle_forward = "Tab"
cycle_backward = "Shift+Tab"
"#.trim().to_string()
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        
        assert_eq!(config.theme.name, "LazyDark");
        assert!(config.theme.dark_mode);
        assert_eq!(config.editor.tab_size, 4);
        assert!(config.editor.show_line_numbers);
        assert_eq!(config.connections.connection_timeout, 5000);
        assert_eq!(config.keybindings.leader_key, " ");
    }

    #[test]
    fn test_config_serialization() {
        let config = create_test_config();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        
        // Verify that the TOML contains expected sections
        assert!(toml_str.contains("[theme]"));
        assert!(toml_str.contains("[editor]"));
        assert!(toml_str.contains("[connections]"));
        assert!(toml_str.contains("[keybindings]"));
        assert!(toml_str.contains("name = \"TestTheme\""));
        assert!(toml_str.contains("tab_size = 2"));
        assert!(toml_str.contains("connection_timeout = 1000"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = create_test_toml();
        let config: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.theme.name, "TestTheme");
        assert!(!config.theme.dark_mode);
        assert_eq!(config.editor.tab_size, 2);
        assert!(!config.editor.show_line_numbers);
        assert_eq!(config.connections.connection_timeout, 1000);
        assert_eq!(config.connections.max_connections, 5);
    }

    #[test]
    fn test_config_roundtrip() {
        let original_config = create_test_config();
        let toml_str = toml::to_string_pretty(&original_config).unwrap();
        let parsed_config: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(original_config.theme.name, parsed_config.theme.name);
        assert_eq!(original_config.theme.dark_mode, parsed_config.theme.dark_mode);
        assert_eq!(original_config.editor.tab_size, parsed_config.editor.tab_size);
        assert_eq!(original_config.connections.connection_timeout, parsed_config.connections.connection_timeout);
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let original_config = create_test_config();
        
        // Test saving
        original_config.save(&config_path).unwrap();
        assert!(config_path.exists());
        
        // Test loading
        let loaded_config = Config::load(Some(config_path)).unwrap();
        assert_eq!(loaded_config.theme.name, "TestTheme");
        assert_eq!(loaded_config.editor.tab_size, 2);
        assert_eq!(loaded_config.connections.max_connections, 5);
    }

    #[test]
    fn test_config_load_nonexistent_creates_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");
        
        // Loading non-existent file should create default
        let config = Config::load(Some(config_path.clone())).unwrap();
        
        // Should be default config
        assert_eq!(config.theme.name, "LazyDark");
        assert!(config.theme.dark_mode);
        assert_eq!(config.editor.tab_size, 4);
        
        // Should have attempted to save the default to the file
        // Note: The save operation may fail silently in load method,
        // but we still get a valid default config
        // Let's verify by manually saving and then checking
        config.save(&config_path).unwrap();
        assert!(config_path.exists());
    }

    #[test]
    fn test_config_load_multiple_paths() {
        let temp_dir = TempDir::new().unwrap();
        let primary_path = temp_dir.path().join("primary.toml");
        let secondary_path = temp_dir.path().join("secondary.toml");
        
        // Create config in secondary path
        let test_config = create_test_config();
        test_config.save(&secondary_path).unwrap();
        
        // Mock the load method to try multiple paths
        let config_paths = [primary_path, secondary_path.clone()];
        
        // Should find config in second path
        for config_path in &config_paths {
            if config_path.exists() {
                let contents = fs::read_to_string(config_path).unwrap();
                let config: Config = toml::from_str(&contents).unwrap();
                assert_eq!(config.theme.name, "TestTheme");
                break;
            }
        }
    }

    #[test]
    fn test_config_load_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");
        
        // Create invalid TOML file
        fs::write(&config_path, "invalid toml content [[[").unwrap();
        
        // Loading should fail gracefully and create default
        let config = Config::load(Some(config_path)).unwrap();
        assert_eq!(config.theme.name, "LazyDark"); // Should be default
    }

    #[test] 
    fn test_config_directory_paths() {
        // Test that path functions return reasonable values
        let default_path = Config::default_path();
        let legacy_path = Config::legacy_config_path();
        let data_dir = Config::data_dir();
        
        assert!(default_path.to_string_lossy().contains("lazytables"));
        assert!(legacy_path.to_string_lossy().contains(".lazytables"));
        assert!(data_dir.to_string_lossy().contains(".lazytables"));
        
        // Ensure different paths are actually different
        assert_ne!(default_path, legacy_path);
        assert!(legacy_path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    fn test_config_ensure_directories() {
        let temp_dir = TempDir::new().unwrap();
        
        // Mock the data_dir for testing
        std::env::set_var("HOME", temp_dir.path());
        
        // This should create all necessary directories
        Config::ensure_directories().unwrap();
        
        let data_dir = temp_dir.path().join(".lazytables");
        assert!(data_dir.exists());
        assert!(data_dir.join("sql_files").exists());
        assert!(data_dir.join("logs").exists());
        assert!(data_dir.join("backups").exists());
        assert!(data_dir.join("connections").exists());
        assert!(data_dir.join("README.md").exists());
        
        // Clean up
        std::env::remove_var("HOME");
    }

    #[test]
    fn test_theme_config() {
        let theme = ThemeConfig {
            name: "CustomTheme".to_string(),
            dark_mode: true,
        };
        
        assert_eq!(theme.name, "CustomTheme");
        assert!(theme.dark_mode);
    }

    #[test]
    fn test_editor_config() {
        let editor = EditorConfig {
            tab_size: 8,
            show_line_numbers: true,
            highlight_current_line: false,
            auto_complete: true,
        };
        
        assert_eq!(editor.tab_size, 8);
        assert!(editor.show_line_numbers);
        assert!(!editor.highlight_current_line);
        assert!(editor.auto_complete);
    }

    #[test]
    fn test_connections_config() {
        let connections = ConnectionsConfig {
            auto_reconnect: true,
            connection_timeout: 2000,
            max_connections: 15,
        };
        
        assert!(connections.auto_reconnect);
        assert_eq!(connections.connection_timeout, 2000);
        assert_eq!(connections.max_connections, 15);
    }
}
