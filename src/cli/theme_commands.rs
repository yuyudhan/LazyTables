// FilePath: src/cli/theme_commands.rs

use crate::ui::theme::ThemeLoader;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum ThemeCommand {
    /// List all available themes
    List,
    
    /// Install a theme from a TOML file
    Install {
        /// Path to the theme TOML file
        path: PathBuf,
    },
    
    /// Export built-in themes to a directory
    Export {
        /// Directory to export themes to (defaults to ./themes)
        #[arg(default_value = "./themes")]
        dir: PathBuf,
    },
    
    /// Show theme directories
    Dirs,
}

impl ThemeCommand {
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            ThemeCommand::List => {
                println!("Available themes:");
                println!();
                
                let themes = ThemeLoader::list_available_themes();
                if themes.is_empty() {
                    println!("No themes found. You can:");
                    println!("  1. Export built-in themes: lazytables theme export");
                    println!("  2. Install custom themes: lazytables theme install <path>");
                } else {
                    for (name, path) in themes {
                        println!("  • {} ({})", name, path.display());
                    }
                }
            }
            
            ThemeCommand::Install { path } => {
                println!("Installing theme from: {}", path.display());
                ThemeLoader::install_theme(path)?;
                println!("✓ Theme installed successfully!");
                println!();
                println!("To use the theme, add it to your config file:");
                println!("  ~/.config/lazytables/config.toml");
                println!();
                println!("  [ui]");
                println!("  theme = \"<theme_name>\"");
            }
            
            ThemeCommand::Export { dir } => {
                println!("Exporting built-in themes to: {}", dir.display());
                ThemeLoader::export_builtin_themes(dir)?;
                println!("✓ Themes exported successfully!");
                println!();
                println!("Exported themes:");
                println!("  • {}/dark.toml", dir.display());
                println!("  • {}/light.toml", dir.display());
            }
            
            ThemeCommand::Dirs => {
                println!("Theme directories (in order of priority):");
                println!();
                
                for (i, dir) in ThemeLoader::theme_directories().iter().enumerate() {
                    let exists = if dir.exists() { "✓" } else { "✗" };
                    println!("  {}. [{}] {}", i + 1, exists, dir.display());
                }
                
                println!();
                println!("Legend: ✓ = exists, ✗ = does not exist");
            }
        }
        
        Ok(())
    }
}