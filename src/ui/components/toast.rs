// FilePath: src/ui/components/toast.rs

use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use std::time::{Duration, Instant};

/// Toast notification types
#[derive(Debug, Clone, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

/// A single toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    /// Create a new toast notification
    pub fn new(message: impl Into<String>, toast_type: ToastType) -> Self {
        Self {
            message: message.into(),
            toast_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(3), // Default 3 seconds
        }
    }

    /// Create a success toast
    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Success)
    }

    /// Create an error toast
    pub fn error(message: impl Into<String>) -> Self {
        let mut toast = Self::new(message, ToastType::Error);
        toast.duration = Duration::from_secs(5); // Errors stay longer
        toast
    }

    /// Create a warning toast
    pub fn warning(message: impl Into<String>) -> Self {
        let mut toast = Self::new(message, ToastType::Warning);
        toast.duration = Duration::from_secs(4);
        toast
    }

    /// Create an info toast
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Info)
    }

    /// Check if the toast has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }

    /// Get the style for this toast type
    fn get_style(&self, theme: &Theme) -> (Color, &str, Color) {
        match self.toast_type {
            ToastType::Success => (
                theme.get_color("success"),
                "✓",
                theme.get_color("toast_success_bg"),
            ),
            ToastType::Error => (
                theme.get_color("error"),
                "✗",
                theme.get_color("toast_error_bg"),
            ),
            ToastType::Warning => (
                theme.get_color("warning"),
                "⚠",
                theme.get_color("toast_warning_bg"),
            ),
            ToastType::Info => (
                theme.get_color("info"),
                "ℹ",
                theme.get_color("toast_info_bg"),
            ),
        }
    }
}

/// Toast manager to handle multiple notifications
#[derive(Debug, Clone)]
pub struct ToastManager {
    toasts: Vec<Toast>,
    max_toasts: usize,
}

impl ToastManager {
    /// Create a new toast manager
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            max_toasts: 5, // Show max 5 toasts at once
        }
    }

    /// Add a new toast
    pub fn add(&mut self, toast: Toast) {
        self.toasts.push(toast);

        // Keep only the most recent toasts
        if self.toasts.len() > self.max_toasts {
            self.toasts.drain(0..self.toasts.len() - self.max_toasts);
        }
    }

    /// Add a success toast
    pub fn success(&mut self, message: impl Into<String>) {
        self.add(Toast::success(message));
    }

    /// Add an error toast
    pub fn error(&mut self, message: impl Into<String>) {
        self.add(Toast::error(message));
    }

    /// Add a warning toast
    pub fn warning(&mut self, message: impl Into<String>) {
        self.add(Toast::warning(message));
    }

    /// Add an info toast
    pub fn info(&mut self, message: impl Into<String>) {
        self.add(Toast::info(message));
    }

    /// Remove expired toasts
    pub fn cleanup(&mut self) {
        self.toasts.retain(|toast| !toast.is_expired());
    }

    /// Check if there are any active toasts
    pub fn has_toasts(&self) -> bool {
        !self.toasts.is_empty()
    }

    /// Clear all toasts
    pub fn clear(&mut self) {
        self.toasts.clear();
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
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

    for (idx, toast) in manager.toasts.iter().enumerate() {
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
