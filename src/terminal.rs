// FilePath: src/terminal.rs

use crate::core::error::{Error, Result};
use crossterm::{
    cursor,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
};
use ratatui::{backend::CrosstermBackend, DefaultTerminal};
use std::io::{stdout, Write};

/// Initialize the terminal for TUI mode
pub fn init() -> Result<DefaultTerminal> {
    enable_raw_mode()?;
    execute!(
        stdout(),
        EnterAlternateScreen,
        Clear(ClearType::All),
        cursor::Hide
    )?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = ratatui::Terminal::new(backend).map_err(|e| Error::Terminal(e.to_string()))?;

    // Clear the terminal to ensure clean state
    terminal.clear().map_err(|e| Error::Terminal(e.to_string()))?;

    // Flush to ensure all operations are applied
    stdout().flush().map_err(|e| Error::Terminal(e.to_string()))?;

    Ok(terminal)
}

/// Restore the terminal to normal mode
pub fn restore() -> Result<()> {
    execute!(
        stdout(),
        cursor::Show,
        Clear(ClearType::All),
        LeaveAlternateScreen
    )?;
    stdout().flush().map_err(|e| Error::Terminal(e.to_string()))?;
    disable_raw_mode()?;
    Ok(())
}

/// Clear the entire terminal screen
pub fn clear_screen() -> Result<()> {
    execute!(stdout(), Clear(ClearType::All))?;
    stdout().flush().map_err(|e| Error::Terminal(e.to_string()))?;
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
