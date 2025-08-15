// FilePath: src/terminal.rs

use crate::core::error::{Error, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, DefaultTerminal};
use std::io::stdout;

/// Initialize the terminal for TUI mode
pub fn init() -> Result<DefaultTerminal> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let terminal = ratatui::Terminal::new(backend).map_err(|e| Error::Terminal(e.to_string()))?;

    Ok(terminal)
}

/// Restore the terminal to normal mode
pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// Install panic hook to restore terminal on panic
pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore();
        original_hook(panic_info);
    }));
}
