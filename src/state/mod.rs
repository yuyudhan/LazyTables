// FilePath: src/state/mod.rs

// Application state module. Re-exports DatabaseState, UIState, and view types
// used throughout the app to separate database concerns from UI concerns.

pub mod database;
pub mod ui;
pub mod view;

pub use database::DatabaseState;
pub use ui::{FocusedPane, HelpMode, UIState};
pub use view::{AppView, ConnectionFormMode, OverlayView, TextInputMode};
