# Tasks — Split connection_modal, table_viewer, query_editor

## 1. Split connection_modal

- [ ] 1.1 Create `src/ui/components/connection_modal/` directory.
- [ ] 1.2 Write `connection_modal/types.rs` — move `PasswordStorageType` (line 17), `TestConnectionStatus` (line 70), `ConnectionField` enum and its full `impl ConnectionField` (`next`, `prev`, `display_name`) from lines 17–190 of `connection_modal.rs`; add required `use` statements (`DatabaseType`, `SslMode` are referenced in field impls).
- [ ] 1.3 Write `connection_modal/state.rs` — move `ConnectionModalState` struct (line 25), `impl Default for ConnectionModalState` (line 192), `impl ConnectionModalState` (line 225) through end of impl at line 875, and the `#[cfg(test)] mod tests` block (lines 1678–2020); add `use super::*;` and any crate imports (`ConnectionConfig`, `DatabaseType`, `SslMode`, `PasswordSource`) referenced by the impl.
- [ ] 1.4 Write `connection_modal/render.rs` — move all free rendering functions (lines 877–1677): `render_modal_overlay`, `render_connection_modal`, and all `render_*` / `draw_*` / `centered_rect` helpers; add `use super::*;` plus ratatui and any other rendering imports.
- [ ] 1.5 Write `connection_modal/mod.rs`:
  ```rust
  pub mod types;
  pub mod state;
  pub mod render;

  pub use types::*;
  pub use state::*;
  pub use render::*;
  ```
- [ ] 1.6 Delete `src/ui/components/connection_modal.rs`.

## 2. Split table_viewer

- [ ] 2.1 Create `src/ui/components/table_viewer/` directory.
- [ ] 2.2 Write `table_viewer/types.rs` — move `TableViewMode` (line 14), `TableTab` struct (line 21) and its full `impl TableTab` (lines 56–517), `ColumnInfo` (line 47), `CellUpdate` (line 520), plus `DeleteConfirmation` (line 542) and `SetNullConfirmation` (line 550); add required `use` statements (`HashMap`, `crate::database::TableMetadata`, `arboard::Clipboard`).
- [ ] 2.3 Write `table_viewer/state.rs` — move `TableViewerState` struct (line 530), `impl TableViewerState` (lines 561–775), and `impl Default for TableViewerState` (lines 778–782); add `use super::*;` to bring in types from `types.rs`.
- [ ] 2.4 Write `table_viewer/render.rs` — move `render_table_viewer` (line 784) through end of file (line 1773), including all private render helpers and `render_empty_state`; add `use super::*;` and ratatui imports.
- [ ] 2.5 Write `table_viewer/mod.rs`:
  ```rust
  pub mod types;
  pub mod state;
  pub mod render;

  pub use types::*;
  pub use state::*;
  pub use render::*;
  ```
- [ ] 2.6 Delete `src/ui/components/table_viewer.rs`.

## 3. Split query_editor

- [ ] 3.1 Create `src/ui/components/query_editor/` directory.
- [ ] 3.2 Write `query_editor/state.rs` — move `QueryEditor` struct (lines 19–50), `impl Clone for QueryEditor` (lines 52–76), and `impl Default for QueryEditor` (lines 78–82); add `use` statements for `SqlSuggestionEngine`, `SuggestionPopup`, `DatabaseType`, `syntect` types.
- [ ] 3.3 Write `query_editor/input.rs` — move `impl QueryEditor` methods for cursor movement, vim keybinding handling, suggestion management, and `apply_syntax_highlighting_with_line_numbers` helper (lines 84–1101), plus `#[cfg(test)] mod tests` block (lines 1273–1512); add `use super::state::QueryEditor;` (or `use super::*;`) and all required crate imports.
- [ ] 3.4 Write `query_editor/render.rs` — move `pub fn render` (lines 1102–1271) and any render-only private helpers; add `use super::*;` plus ratatui and syntect imports.
- [ ] 3.5 Write `query_editor/mod.rs`:
  ```rust
  pub mod state;
  pub mod input;
  pub mod render;

  pub use state::*;
  pub use input::*;
  pub use render::*;
  ```
- [ ] 3.6 Delete `src/ui/components/query_editor.rs`.

## 4. Fix re-exports and callers

- [ ] 4.1 Verify `src/ui/components/mod.rs` — the three `pub mod` declarations (`connection_modal`, `query_editor`, `table_viewer`) will automatically resolve to the new `mod.rs` once the flat `.rs` files are removed; confirm no text change is required.
- [ ] 4.2 Verify `src/app/state.rs` glob import `crate::ui::components::*` still resolves `ConnectionModalState`, `TableViewerState`, `QueryEditor`; if any item is missing, add it explicitly to the inner `mod.rs` re-exports.
- [ ] 4.3 Verify `src/state/database.rs` explicit path `crate::ui::components::table_viewer::{CellUpdate, ColumnInfo, DeleteConfirmation, SetNullConfirmation}` resolves — these are re-exported from `table_viewer/mod.rs` via `pub use types::*`.
- [ ] 4.4 Verify `src/app/handlers/query_results.rs` explicit path `crate::ui::components::table_viewer::TableViewMode` resolves.
- [ ] 4.5 Verify `src/commands/connection.rs` explicit path `crate::ui::components::connection_modal::ConnectionModalState` resolves.
- [ ] 4.6 Verify `src/app/mod.rs` and `src/app/handlers/connections.rs` inline `use crate::ui::components::TestConnectionStatus` resolves.
- [ ] 4.7 Ensure `sql_suggestions.rs` and `suggestion_popup.rs` are unchanged; confirm their entries in `src/ui/components/mod.rs` are unmodified.

## 5. Verify

- [ ] 5.1 `cargo build` clean
- [ ] 5.2 `cargo clippy --all-targets -- -D warnings` clean
- [ ] 5.3 `cargo fmt --check` clean
- [ ] 5.4 `cargo test` green
- [ ] 5.5 Component tests pass post-move (`ui::components::connection_modal::state::tests` and `ui::components::query_editor::input::tests`); run app and confirm: connection modal opens with all fields/tabs rendering and input working, a table opens showing both data and schema views with edit/delete dialogs functional, the SQL editor responds to vim mode keybindings, shows the autocomplete popup, and applies syntax highlighting.
