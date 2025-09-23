// FilePath: src/main.rs

use clap::Parser;
use lazytables::{app::App, cli::Cli, config::Config};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // Initialize error handling
    color_eyre::install()?;

    // Parse command line arguments
    let cli = Cli::parse();

    // Handle theme commands if present
    if let Some(lazytables::cli::Commands::Theme { command }) = &cli.theme {
        return command
            .execute()
            .map_err(|e| color_eyre::eyre::eyre!("Theme command failed: {}", e));
    }

    // Initialize logging
    lazytables::logging::init(cli.log_level)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to init logging: {}", e))?;

    // Load configuration
    let config = Config::load(cli.config)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to load config: {}", e))?;

    // Initialize terminal
    let terminal = lazytables::terminal::init()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to init terminal: {}", e))?;

    // Install panic hook to restore terminal on panic
    lazytables::terminal::install_panic_hook();

    // Create and run the application
    let mut app =
        App::new(config).map_err(|e| color_eyre::eyre::eyre!("Failed to create app: {}", e))?;
    let result = app
        .run(terminal)
        .await
        .map_err(|e| color_eyre::eyre::eyre!("Application error: {}", e));

    // Restore terminal
    lazytables::terminal::restore()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to restore terminal: {}", e))?;

    result
}
