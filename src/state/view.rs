// FilePath: src/state/view.rs

#![forbid(unsafe_code)]

use crate::database::ConnectionConfig;
use serde::{Deserialize, Serialize};

/// Main application view hierarchy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppView {
    /// Main six-pane database browsing interface
    Main,
    /// Full-screen overlay view
    Overlay(OverlayView),
}

impl Default for AppView {
    fn default() -> Self {
        Self::Main
    }
}

/// Types of full-screen overlays
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlayView {
    /// Connection form (Add or Edit)
    ConnectionForm(ConnectionFormMode),
    /// Debug view (logs and diagnostics)
    DebugView,
    /// Help overlay
    Help,
}

/// Connection form mode (Add new or Edit existing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionFormMode {
    /// Adding a new connection
    Add,
    /// Editing an existing connection
    #[serde(skip)]
    Edit(Box<ConnectionConfig>),
}

impl PartialEq for ConnectionFormMode {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Add, Self::Add) | (Self::Edit(_), Self::Edit(_))
        )
    }
}

impl Eq for ConnectionFormMode {}

/// Text input mode for all text fields (vim-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextInputMode {
    /// Normal navigation mode
    Normal,
    /// Insert/edit mode for typing
    Insert,
}

impl Default for TextInputMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl AppView {
    /// Check if currently in main view
    pub fn is_main(&self) -> bool {
        matches!(self, Self::Main)
    }

    /// Check if currently in an overlay
    pub fn is_overlay(&self) -> bool {
        matches!(self, Self::Overlay(_))
    }

    /// Get the overlay type if in overlay view
    pub fn overlay(&self) -> Option<&OverlayView> {
        match self {
            Self::Overlay(overlay) => Some(overlay),
            _ => None,
        }
    }

    /// Check if in connection form overlay
    pub fn is_connection_form(&self) -> bool {
        matches!(self, Self::Overlay(OverlayView::ConnectionForm(_)))
    }

    /// Check if in debug view overlay
    pub fn is_debug_view(&self) -> bool {
        matches!(self, Self::Overlay(OverlayView::DebugView))
    }

    /// Check if in help overlay
    pub fn is_help(&self) -> bool {
        matches!(self, Self::Overlay(OverlayView::Help))
    }
}

impl OverlayView {
    /// Get the display name of the overlay
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ConnectionForm(ConnectionFormMode::Add) => "Add Connection",
            Self::ConnectionForm(ConnectionFormMode::Edit(_)) => "Edit Connection",
            Self::DebugView => "Debug View",
            Self::Help => "Help",
        }
    }
}
