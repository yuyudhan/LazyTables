// FilePath: src/ui/theme/loader.rs

use super::{Theme, ThemeManager};
use std::fs;
use std::path::{Path, PathBuf};

/// Theme loader for loading themes from various locations
pub struct ThemeLoader;

impl ThemeLoader {
    /// Get all theme directories where themes can be stored
    pub fn theme_directories() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // User themes directory (~/.config/lazytables/themes/)
        if let Some(config_dir) = dirs::config_dir() {
            dirs.push(config_dir.join("lazytables").join("themes"));
        }

        // User data directory (~/.local/share/lazytables/themes/)
        if let Some(data_dir) = dirs::data_dir() {
            dirs.push(data_dir.join("lazytables").join("themes"));
        }

        // System themes directory (/usr/share/lazytables/themes/)
        #[cfg(unix)]
        dirs.push(PathBuf::from("/usr/share/lazytables/themes"));

        // Bundled themes in the executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                dirs.push(exe_dir.join("themes"));
            }
        }

        // Development themes directory (./themes/)
        dirs.push(PathBuf::from("themes"));

        dirs
    }

    /// Load all available themes from all theme directories
    pub fn load_all_themes() -> ThemeManager {
        let mut manager = ThemeManager::new();

        for dir in Self::theme_directories() {
            if dir.exists() && dir.is_dir() {
                Self::load_themes_from_directory(&dir, &mut manager);
            }
        }

        manager
    }

    /// Load themes from a specific directory
    fn load_themes_from_directory(dir: &Path, manager: &mut ThemeManager) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "toml") {
                    if let Ok(theme) = Theme::load_from_file(&path) {
                        tracing::info!("Loaded theme '{}' from {:?}", theme.name, path);
                        manager.add_theme(theme);
                    } else {
                        tracing::warn!("Failed to load theme from {:?}", path);
                    }
                }
            }
        }
    }

    /// Install a theme file to the user themes directory
    pub fn install_theme(theme_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Validate the theme file first
        let theme = Theme::load_from_file(theme_path)?;

        // Get user themes directory
        let user_themes_dir = dirs::config_dir()
            .ok_or("Could not determine config directory")?
            .join("lazytables")
            .join("themes");

        // Create themes directory if it doesn't exist
        fs::create_dir_all(&user_themes_dir)?;

        // Copy theme file to user themes directory
        let dest_filename = format!("{}.toml", theme.name.to_lowercase().replace(' ', "_"));
        let dest_path = user_themes_dir.join(dest_filename);

        fs::copy(theme_path, &dest_path)?;
        tracing::info!("Installed theme '{}' to {:?}", theme.name, dest_path);

        Ok(())
    }

    /// List all available themes with their locations
    pub fn list_available_themes() -> Vec<(String, PathBuf)> {
        let mut themes = Vec::new();

        for dir in Self::theme_directories() {
            if dir.exists() && dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().is_some_and(|ext| ext == "toml") {
                            if let Ok(theme) = Theme::load_from_file(&path) {
                                themes.push((theme.name, path));
                            }
                        }
                    }
                }
            }
        }

        themes
    }

    /// Export built-in themes to a directory
    pub fn export_builtin_themes(export_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(export_dir)?;

        // Export dark theme
        let dark_theme = Theme::dark_theme();
        let dark_content = toml::to_string_pretty(&dark_theme)?;
        fs::write(export_dir.join("dark.toml"), dark_content)?;

        // Export light theme
        let light_theme = Theme::light_theme();
        let light_content = toml::to_string_pretty(&light_theme)?;
        fs::write(export_dir.join("light.toml"), light_content)?;

        tracing::info!("Exported built-in themes to {:?}", export_dir);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_theme_directories() {
        let dirs = ThemeLoader::theme_directories();
        assert!(!dirs.is_empty());
        // Should at least have the development themes directory
        assert!(dirs.iter().any(|d| d.ends_with("themes")));
    }

    #[test]
    fn test_export_builtin_themes() {
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path();

        ThemeLoader::export_builtin_themes(export_path).unwrap();

        assert!(export_path.join("dark.toml").exists());
        assert!(export_path.join("light.toml").exists());
    }

    #[test]
    fn test_load_theme() {
        let temp_dir = TempDir::new().unwrap();
        let theme_path = temp_dir.path().join("test_theme.toml");

        // Create a test theme using the built-in theme as template
        let test_theme = Theme::dark_theme();
        let theme_content = toml::to_string_pretty(&test_theme).unwrap();

        fs::write(&theme_path, theme_content).unwrap();

        // Test that the theme can be loaded
        let loaded_theme = Theme::load_from_file(&theme_path).unwrap();
        assert_eq!(loaded_theme.name, test_theme.name);
    }
}
