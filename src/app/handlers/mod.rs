// FilePath: src/app/handlers/mod.rs
//
// Event handler modules for different panes and overlays.
// Each module contains handler functions that take `&mut App` and `KeyEvent`.

pub mod connections;
pub mod details;
pub mod global;
pub mod overlays;
pub mod query_editor;
pub mod query_results;
pub mod sql_files;
pub mod tables;
