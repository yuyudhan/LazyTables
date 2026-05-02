// FilePath: src/ui/components/toast.rs

// Re-exports toast types from lazytables-core. Keeps rendering functions and
// the get_style extension trait that depends on the Theme type.

use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::time::Duration;

// Re-export all types from lazytables-core::toast
pub use lazytables_core::toast::*;

/// Extension trait for Toast to get theme-dependent styles
pub trait ToastExt {
    /// Get the style for this toast type
    fn get_style(&self, theme: &Theme) -> (Color, &str, Color);
}

impl ToastExt for Toast {
    fn get_style(&self, theme: &Theme) -> (Color, &str, Color) {
        match self.toast_type {
            ToastType::Success => (
                theme.get_color("success"),
                "\u{2713}",
                theme.get_color("toast_success_bg"),
            ),
            ToastType::Error => (
                theme.get_color("error"),
                "\u{2717}",
                theme.get_color("toast_error_bg"),
            ),
            ToastType::Warning => (
                theme.get_color("warning"),
                "\u{26a0}",
                theme.get_color("toast_warning_bg"),
            ),
            ToastType::Info => (
                theme.get_color("info"),
                "\u{2139}",
                theme.get_color("toast_info_bg"),
            ),
        }
    }
}

/// Render toasts in the top-right corner
pub fn render_toasts(f: &mut Frame, manager: &ToastManager, area: Rect, theme: &Theme) {
    if !manager.has_toasts() {
        return;
    }

    // Calculate toast area - make it more visible
    let toast_width = 50u16.min(area.width.saturating_sub(4));
    let toast_height = 3; // Give more space for content
    let padding = 1;

    // Position toasts in the top-right corner with better positioning
    let x = area.x + area.width.saturating_sub(toast_width + padding);

    for (idx, toast) in manager.toasts().iter().enumerate() {
        let y = area.y + padding + (idx as u16 * (toast_height + 1));

        // Don't render if we're out of vertical space
        if y + toast_height > area.y + area.height {
            break;
        }

        let toast_area = Rect {
            x,
            y,
            width: toast_width,
            height: toast_height,
        };

        render_single_toast(f, toast, toast_area, theme);
    }
}

/// Render a single toast notification
fn render_single_toast(f: &mut Frame, toast: &Toast, area: Rect, theme: &Theme) {
    let (border_color, prefix, bg_color) = toast.get_style(theme);

    // Calculate fade based on time remaining
    let elapsed = toast.created_at.elapsed();
    let fade_start = toast.duration.saturating_sub(Duration::from_secs(1));
    let is_fading = elapsed > fade_start;

    let border_style = if is_fading {
        Style::default()
            .fg(border_color)
            .add_modifier(Modifier::DIM)
    } else {
        Style::default()
            .fg(border_color)
            .add_modifier(Modifier::BOLD)
    };

    // Format the message with prefix on the same line
    let content = vec![Line::from(vec![
        Span::styled(
            format!("{prefix} "),
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(&toast.message, Style::default().fg(theme.get_color("text"))),
    ])];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(bg_color).fg(theme.get_color("text")));

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
