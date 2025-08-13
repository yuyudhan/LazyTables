// FilePath: src/config/mod.rs

use crate::core::error::Result;
// Removed directories crate, using dirs crate for home_dir
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

    /// Get default configuration path - uses ~/.config/lazytables/config.toml  
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .map(|config| config.join("lazytables").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from(".config/lazytables/config.toml"))
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
            keybindings: KeybindingsConfig {
                leader_key: " ".to_string(),
            },
        }
    }
}

