// FilePath: src/ui/components/toast.rs

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
    fn get_style(&self) -> (Color, &str, Color) {
        match self.toast_type {
            ToastType::Success => (Color::Green, "✓ SUCCESS", Color::Rgb(0, 50, 0)),
            ToastType::Error => (Color::Red, "✗ ERROR", Color::Rgb(50, 0, 0)),
            ToastType::Warning => (Color::Yellow, "⚠ WARNING", Color::Rgb(50, 50, 0)),
            ToastType::Info => (Color::Cyan, "ℹ INFO", Color::Rgb(0, 30, 50)),
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
pub fn render_toasts(f: &mut Frame, manager: &ToastManager, area: Rect) {
    if !manager.has_toasts() {
        return;
    }

    // Calculate toast area (top-right corner)
    let toast_width = 40;
    let toast_height = 3;
    let padding = 1;

    // Position toasts in the top-right corner
    let x = area.width.saturating_sub(toast_width + padding);

    for (idx, toast) in manager.toasts.iter().enumerate() {
        let y = padding + (idx as u16 * (toast_height + 1));

        // Don't render if we're out of vertical space
        if y + toast_height > area.height {
            break;
        }

        let toast_area = Rect {
            x,
            y,
            width: toast_width,
            height: toast_height,
        };

        render_single_toast(f, toast, toast_area);
    }
}

/// Render a single toast notification
fn render_single_toast(f: &mut Frame, toast: &Toast, area: Rect) {
    let (border_color, prefix, bg_color) = toast.get_style();

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

    // Format the message with prefix
    let content = vec![
        Line::from(vec![Span::styled(
            prefix,
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(toast.message.clone()),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(bg_color));

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
