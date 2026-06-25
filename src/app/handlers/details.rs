// FilePath: src/app/handlers/details.rs

// Event handler for the Details pane (read-only scrolling of table metadata)

#![forbid(unsafe_code)]

use crate::{app::App, core::error::Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle Details pane keys - READ-ONLY (just scrolling)
pub(crate) fn handle(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.state.move_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.state.move_up();
        }
        KeyCode::Char('d') if key.modifiers == KeyModifiers::CONTROL => {
            // Page down
            if app.state.ui.details_viewport_offset + 10 < app.state.ui.details_max_scroll_offset {
                app.state.ui.details_viewport_offset += 10;
            } else {
                app.state.ui.details_viewport_offset = app.state.ui.details_max_scroll_offset;
            }
        }
        KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
            // Page up
            app.state.ui.details_viewport_offset =
                app.state.ui.details_viewport_offset.saturating_sub(10);
        }
        KeyCode::Char('g') => {
            if app.state.ui.pending_gg_command {
                app.state.ui.details_viewport_offset = 0;
                app.state.ui.pending_gg_command = false;
            } else {
                app.state.ui.pending_gg_command = true;
            }
        }
        KeyCode::Char('G') => {
            app.state.ui.details_viewport_offset = app.state.ui.details_max_scroll_offset;
        }
        _ => {}
    }
    Ok(())
}
