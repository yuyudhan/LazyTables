// FilePath: src/lib.rs

pub mod app;
pub mod cli;
pub mod config;
pub mod constants;
pub mod core;
pub mod database;
pub mod event;
pub mod logging;
pub mod terminal;
pub mod themes;
pub mod ui;

pub use app::App;
pub use cli::Cli;
pub use config::Config;

// Re-export commonly used types
pub mod prelude {
    pub use crate::app::{App, AppState, Mode};
    pub use crate::config::Config;
    pub use crate::core::error::{Error, Result};
    pub use crate::event::{Event, EventHandler};
}

