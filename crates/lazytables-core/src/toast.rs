// FilePath: crates/lazytables-core/src/toast.rs

// Toast notification data types and queue management.
// Rendering logic lives in lazytables-ui.

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
            duration: Duration::from_secs(3),
        }
    }

    /// Create a success toast
    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Success)
    }

    /// Create an error toast
    pub fn error(message: impl Into<String>) -> Self {
        let mut toast = Self::new(message, ToastType::Error);
        toast.duration = Duration::from_secs(5);
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
            max_toasts: 5,
        }
    }

    /// Add a new toast
    pub fn add(&mut self, toast: Toast) {
        self.toasts.push(toast);

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

    /// Get active toasts for rendering
    pub fn toasts(&self) -> &[Toast] {
        &self.toasts
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
