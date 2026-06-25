# c0009 Tasks: Pull Inline Pane Rendering Out of ui/mod.rs

## 1. Extract connections_pane component

- [ ] 1.1 Create `src/ui/components/connections_pane.rs` with a `pub fn render_connections_pane`
  that contains the body of `draw_connections_pane` (src/ui/mod.rs:286–453). Signature:
  `render_connections_pane(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme)`.
  If implemented after c0005, add params for the animation fields that moved to `App`:
  `connecting_in_progress: Option<usize>`, `connecting_animation_frame: usize`,
  `connection_elapsed_secs: u64`. Replace all `self.theme.get_color(…)` with
  `theme.get_color(…)`. Add all required `use` imports (ratatui types, `AppState`,
  `FocusedPane`, `ConnectionStatus`, `DatabaseType`, `Theme`).
- [ ] 1.2 Preserve the `connections_list_state` clone → render → write-back pattern verbatim
  (currently at src/ui/mod.rs:448–452) to keep scroll/selection state working.
- [ ] 1.3 Register in `src/ui/components/mod.rs`: add `pub mod connections_pane;` and
  `pub use connections_pane::*;`.
- [ ] 1.4 In `UI::draw` (src/ui/mod.rs:190), replace `self.draw_connections_pane(frame,
  areas.connections, state);` with
  `components::render_connections_pane(frame, areas.connections, state, &self.theme);`
  (add animation params if after c0005).
- [ ] 1.5 Delete `fn draw_connections_pane` from `src/ui/mod.rs` (lines 286–453).

## 2. Extract details_pane component (+ details builder)

- [ ] 2.1 Create `src/ui/components/details_pane.rs` with:
  - `pub fn render_details_pane(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme)`
    containing the body of `draw_details_pane` (src/ui/mod.rs:462–599). Replace
    `self.build_comprehensive_table_details(…)` with a direct call to the local
    `build_comprehensive_table_details(…)` free function.
  - `fn build_comprehensive_table_details(table_name: String, db_state:
    &crate::state::DatabaseState, ui_state: &crate::state::UIState, is_focused: bool) ->
    Vec<Line<'static>>` — exact body from src/ui/mod.rs:608–848. Keep as module-private (`fn`,
    not `pub`).
- [ ] 2.2 Preserve writes to `state.ui.details_content_height`, `state.ui.details_viewport_height`,
  `state.ui.details_max_scroll_offset` (src/ui/mod.rs:562–564) to keep scroll-bounds tracking
  working.
- [ ] 2.3 Add all required `use` imports: ratatui types, `Alignment`, `TableMetadata` (for
  `TableMetadata::format_size`), `AppState`, `FocusedPane`, `Theme`, `DatabaseState`,
  `UIState`.
- [ ] 2.4 Register in `src/ui/components/mod.rs`: add `pub mod details_pane;` and
  `pub use details_pane::*;`.
- [ ] 2.5 In `UI::draw` (src/ui/mod.rs:196), replace `self.draw_details_pane(frame, areas.details,
  state);` with `components::render_details_pane(frame, areas.details, state, &self.theme);`.
- [ ] 2.6 Delete `fn draw_details_pane` (src/ui/mod.rs:462–599) and `fn
  build_comprehensive_table_details` (src/ui/mod.rs:602–849) from `src/ui/mod.rs`.

## 3. Extract sql_files_pane component (+ file-metadata helper)

- [ ] 3.1 Create `src/ui/components/sql_files_pane.rs` with:
  - `pub fn render_sql_files_pane(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme)`
    — note immutable borrow `&AppState` matching the current `draw_sql_files_pane` signature
    (src/ui/mod.rs:967). Contains the body from src/ui/mod.rs:967–1144.
  - `fn get_file_metadata(path: &std::path::Path) -> (String, String)` — module-private, exact
    body from src/ui/mod.rs:1147–1185. Add `use std::fs; use std::time::SystemTime;` locally.
- [ ] 3.2 Replace `self.get_file_metadata(&connection_path)` / `self.get_file_metadata(&root_path)`
  with bare `get_file_metadata(…)` calls (now a free function in the same file).
- [ ] 3.3 Replace `self.theme.get_color(…)` with `theme.get_color(…)` throughout.
- [ ] 3.4 Add all required `use` imports: ratatui types, `AppState`, `FocusedPane`, `Theme`,
  `crate::config::Config`.
- [ ] 3.5 Register in `src/ui/components/mod.rs`: add `pub mod sql_files_pane;` and
  `pub use sql_files_pane::*;`.
- [ ] 3.6 In `UI::draw` (src/ui/mod.rs:202), replace `self.draw_sql_files_pane(frame,
  areas.sql_files, state);` with
  `components::render_sql_files_pane(frame, areas.sql_files, state, &self.theme);`.
- [ ] 3.7 Delete `fn draw_sql_files_pane` (src/ui/mod.rs:967–1144) and `fn get_file_metadata`
  (src/ui/mod.rs:1147–1185) from `src/ui/mod.rs`.

## 4. Slim UI::draw to delegation only

- [ ] 4.1 After tasks 1–3, confirm `UI::draw` calls the three new `components::render_*`
  functions and contains no inline pane-rendering logic for the extracted panes.
- [ ] 4.2 Run `cargo clippy --all-targets -- -D warnings` locally (pre-verify) to identify any
  dead `use` items in `src/ui/mod.rs` that were only required by the deleted methods (e.g.
  `List`, `ListItem` if no longer used). Remove them.
- [ ] 4.3 Confirm that `draw_header`, `draw_tables_pane`, `draw_tabular_output`,
  `draw_query_window`, `draw_status_bar`, `render_modal_overlay`,
  `render_confirmation_modal`, and `center_modal` remain in `src/ui/mod.rs` unchanged —
  those are not in scope for this change.

## 5. Verify

- [ ] 5.1 `cargo build` clean
- [ ] 5.2 `cargo clippy --all-targets -- -D warnings` clean
- [ ] 5.3 `cargo fmt --check` clean
- [ ] 5.4 `cargo test` green
- [ ] 5.5 Launch the application and manually verify all three extracted panes are visually
  identical to before: (a) connections pane — status icons, connecting animation with animated
  dots and elapsed/timeout counter, search mode title; (b) details pane — metadata sections
  (rows, columns, storage, relationships, comment), scroll indicator in title, disabled-state
  placeholder; (c) SQL files pane — size/mtime metadata when focused, search/rename/create
  inline input prompts, disabled-state placeholder.
