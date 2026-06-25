# Tasks: c0005 — Split AppState; relocate orchestration state; unify errors

## 1. Carve `state.rs` into `state/` module siblings

- [ ] 1.1 Add `src/core/error.rs` variants `SqlFile(String)` and `Editor(String)` (needed
      for subsequent conversion steps).
- [ ] 1.2 Create `src/app/state/mod.rs` containing: the `AppState` struct definition with
      all remaining fields (state.rs:28–67), `new()` (state.rs:71–108),
      `initialize_app_db()` (state.rs:111–119 — return type already `Result<_, String>`;
      convert to `crate::core::error::Result<()>` using `LazyTablesError::Other`),
      `Default` impl (state.rs:2155–2194), the `QueryEditorMovement` enum (state.rs:18–24),
      all existing `pub use` re-export lines (state.rs:14–15), and `mod` declarations for
      each sibling file.
- [ ] 1.3 Create `src/app/state/navigation.rs` as `impl AppState` with the focus-cycling /
      directional-move / pane-enabled-predicate methods (state.rs:131–351, 1876–1935):
      `cycle_focus_forward`, `cycle_focus_backward`, `move_focus_left`, `move_focus_down`,
      `move_focus_up`, `move_focus_right`, `move_up`, `move_down`, `move_left`, `move_right`,
      `are_sql_panes_enabled`, `is_query_editor_enabled`, `is_tables_pane_enabled`,
      `is_details_pane_enabled`, `is_query_results_pane_enabled`.
      Update `move_down` (state.rs:258) and `move_right` (state.rs:339) to use
      `self.query_editor.get_content()` instead of `self.query_content`.
- [ ] 1.4 Create `src/app/state/connections.rs` as `impl AppState` with connection-selection
      methods (state.rs:352–497 excluding `save_connection_from_modal`):
      `get_selected_connection`, `get_selected_connection_mut`, `open_add_connection_modal`,
      `close_add_connection_modal`, `open_edit_connection_modal`, `close_edit_connection_modal`,
      `clamp_connection_selection`, `connection_down`, `connection_up`.
- [ ] 1.5 Create `src/app/state/modals.rs` as `impl AppState` with `save_connection_from_modal`
      (state.rs:409–453). Change return type from `Result<(), String>` to
      `crate::core::error::Result<()>` (use `LazyTablesError::Other` to wrap string errors).
- [ ] 1.6 Create `src/app/state/table_viewer.rs` as `impl AppState` with table-viewer methods
      (state.rs:499–531, 1685–1856): `table_down`, `table_up`, `update_table_selection`,
      `open_table_for_viewing`, `load_table_data`, `load_table_metadata`, `check_connection_health`
      (state.rs:1768), `update_table_cell`, `delete_table_row`, `set_cell_to_null`,
      `reload_current_table_tab`, `get_statement_under_cursor`.
- [ ] 1.7 Create `src/app/state/connection_lifecycle.rs` as `impl AppState` with async
      connect/disconnect methods (state.rs:540–709): `disconnect_all_except`,
      `connect_to_selected_database`, `try_connect_to_database`, `disconnect_from_database`,
      `disconnect_from_database_sync`. Change `try_connect_to_database` return type from
      `Result<_, String>` to `crate::core::error::Result<_>`.
- [ ] 1.8 Create `src/app/state/sql_files.rs` as `impl AppState` with SQL file methods
      (state.rs:711–1018, 1300–1683): `get_selected_sql_file`, `load_selected_sql_file`,
      `clamp_sql_file_selection`, `load_sql_files_for_connection`, `refresh_sql_files`,
      `save_query_as`, `save_query`, `load_query_file` (made async — see task group 4),
      `save_sql_file_with_connection`, `delete_sql_file`, and any remaining helper methods
      in those ranges. Change all `Box<dyn std::error::Error>` return types to
      `crate::core::error::Result<_>`.
- [ ] 1.9 Create `src/app/state/query_editor.rs` as `impl AppState` with query editor
      delegation and execution methods (state.rs:1020–1265, 1858–2152):
      `insert_char_at_cursor`, `insert_newline_at_cursor` (mark `pub(super)` or `pub`
      as appropriate), `delete_char_at_cursor`, `reset_query_editor`,
      `update_query_editor_context`, `set_query_editor_focus`, `toggle_query_editor_insert_mode`,
      `handle_query_editor_input`, `handle_query_editor_newline`, `handle_query_editor_backspace`,
      `handle_query_editor_movement`, `load_sql_file_into_editor`, `set_query_content`,
      `get_query_content`, `execute_query_at_cursor`, and any remaining methods in those ranges.
      Change `execute_query_at_cursor` return from `Result<(), String>` to
      `crate::core::error::Result<()>`.
- [ ] 1.10 Delete `src/app/state.rs` once the directory-module form compiles cleanly.
- [ ] 1.11 Confirm `src/app/mod.rs` `pub mod state;` resolves to the new directory; add
       `pub use state::{AppState, AppView, ConnectionFormMode, FocusedPane, HelpMode,
       OverlayView, TextInputMode};` if not already present.

## 2. Move 6 orchestration fields from `AppState` to `App`

- [ ] 2.1 Remove from `AppState` struct (state/mod.rs) the fields `connecting_in_progress`,
      `connection_start_time`, `connecting_animation_frame`, `test_connection_in_progress`,
      `test_animation_frame`, `test_start_time`. Remove their initialisation from `new()`
      and `Default`.
- [ ] 2.2 Add the 6 fields to the `App` struct in `src/app/mod.rs` (after the existing
      `tick_counter` field). Initialise them in `App::new()`:
      `connecting_in_progress: None`, `connection_start_time: None`,
      `connecting_animation_frame: 0`, `test_connection_in_progress: false`,
      `test_animation_frame: 0`, `test_start_time: None`.
- [ ] 2.3 Update `App::tick()` (app/mod.rs:290–469): replace every
      `self.state.connecting_in_progress` with `self.connecting_in_progress`,
      `self.state.connecting_animation_frame` with `self.connecting_animation_frame`,
      `self.state.connection_start_time` with `self.connection_start_time`,
      `self.state.test_connection_in_progress` with `self.test_connection_in_progress`,
      `self.state.test_animation_frame` with `self.test_animation_frame`,
      `self.state.test_start_time` with `self.test_start_time`.
- [ ] 2.4 Update `src/app/handlers/connections.rs`: replace
      `app.state.connecting_in_progress` → `app.connecting_in_progress`,
      `app.state.connecting_animation_frame` → `app.connecting_animation_frame`,
      `app.state.connection_start_time` → `app.connection_start_time`
      at all access sites (search-mode Enter path ~lines 38–48; normal-mode Enter/Space path
      ~lines 161–171).

## 3. Promote background protocol to `src/app/background.rs`

- [ ] 3.1 Create `src/app/background.rs` with a public `BackgroundEvent` enum that covers
      both the regular connection outcome and the test connection outcome. Model the inner
      data after the current `ConnectionEvent` (Success `{ connection_index, objects }`,
      Failed `{ connection_index, error }`) and `TestConnectionEvent` (Success(String),
      Failed(String)).
- [ ] 3.2 Add `pub mod background;` to `src/app/mod.rs`. Remove the private
      `ConnectionEvent` and `TestConnectionEvent` enum definitions (app/mod.rs:22–39).
- [ ] 3.3 Replace the two channel pairs in `App` (`connection_events_tx/rx` and
      `test_connection_events_tx/rx`) with a single
      `background_events_tx: tokio::sync::mpsc::UnboundedSender<BackgroundEvent>` /
      `background_events_rx: tokio::sync::mpsc::UnboundedReceiver<BackgroundEvent>`.
      Update `App::new()` to create one channel pair.
- [ ] 3.4 Rewrite `App::tick()` to `try_recv()` from `self.background_events_rx` and
      `match` on `BackgroundEvent` variants instead of the two separate `try_recv()` blocks.
      Preserve all state mutations and toast messages verbatim.
- [ ] 3.5 Update `src/app/handlers/connections.rs` `use` imports: replace
      `ConnectionEvent, TestConnectionEvent` with `crate::app::background::BackgroundEvent`.
      Update `tx.send(…)` calls to send `BackgroundEvent` variants.
- [ ] 3.6 Update the test connection handler path (handlers that send to
      `test_connection_events_tx`, typically in `handlers/overlays.rs`) to send
      `BackgroundEvent::TestConnection(…)` on the single sender.
      Verify `test_connection_task_handle` abort still compiles (it stores a `JoinHandle<()>`;
      the join handle is unaffected by the channel merge).

## 4. Error unification and `load_query_file` async port

- [ ] 4.1 In `src/app/state/sql_files.rs`: make `load_query_file` an `async fn`. Replace
      `use std::fs;` and `fs::read_to_string(…)` with
      `crate::io::async_fs::read_to_string(…).await`. Change return type from
      `Result<(), Box<dyn std::error::Error>>` to `crate::core::error::Result<()>`;
      convert string-literal errors to `LazyTablesError::SqlFile("…".into())`.
- [ ] 4.2 Make `load_selected_sql_file` and `load_sql_file_into_editor` `async fn` and
      `.await` the now-async `load_query_file`. Change both return types to
      `crate::core::error::Result<()>`.
- [ ] 4.3 Convert all remaining `Box<dyn std::error::Error>` returns in `sql_files.rs`
      (`save_query_as`, `save_query`, `delete_sql_file`) to `crate::core::error::Result<_>`.
      Use `LazyTablesError::SqlFile(…)` for file-operation errors and the existing `?`
      operator for `io::async_fs` errors (which already implement `From<std::io::Error>`).
- [ ] 4.4 Convert `execute_query_at_cursor` in `query_editor.rs` from `Result<(), String>`
      to `crate::core::error::Result<()>`. Map `String` errors from `connection_manager`
      to `LazyTablesError::Other(s)`. Toast messages in the method body are unchanged.
      Callers in `handlers/query_editor.rs` (lines 24 and 32) call `.to_string()` on the
      error — this still compiles because `LazyTablesError` implements `Display`.
- [ ] 4.5 Convert `save_connection_from_modal` (modals.rs) and `initialize_app_db` (mod.rs)
      from `Result<(), String>` to `crate::core::error::Result<()>`. Update callers in
      `app/mod.rs` that `match`/`if let Err(e)` on the String error to match on
      `LazyTablesError` (or use `e.to_string()` for the existing toast/eprintln call,
      which is fine).
- [ ] 4.6 Verify no `Box<dyn std::error::Error>` or bare `String` errors remain in any
      `src/app/state/` file. Run `grep -r "Box<dyn" src/app/state/` and
      `grep -r "Result<(), String>" src/app/state/` — both should produce no output.

## 5. Remove `query_content` duplicate; de-duplicate connect spawn

- [ ] 5.1 Remove the `query_content: String` field from `AppState` struct in
      `src/app/state/mod.rs`. Remove its initialisation from `new()` and `Default`.
- [ ] 5.2 Delete the three sync wrapper methods that synced to `query_content`:
      `handle_query_editor_input` (state.rs:1967–1972), `handle_query_editor_newline`
      (state.rs:1974–1980), `handle_query_editor_backspace` (state.rs:1982–1988).
      Any caller still referencing these after c0002 must be updated to call
      `query_editor.insert_char(ch)`, `query_editor.insert_newline()`, or
      `query_editor.backspace()` directly and then set `ui.query_modified = true`.
- [ ] 5.3 Update `move_down` in `navigation.rs`: `self.query_content.lines().count()` →
      `self.query_editor.get_content().lines().count()`.
- [ ] 5.4 Update `move_right` in `navigation.rs`: `self.query_content.lines().nth(…)` →
      `self.query_editor.get_content().lines().nth(…)`.
- [ ] 5.5 Update `save_sql_file_with_connection` in `sql_files.rs`: remove the
      `self.query_content = self.query_editor.get_content().to_string()` sync line
      (state.rs:1304); replace `self.query_content.clone()` (line 1365) with
      `self.query_editor.get_content().to_string()`.
- [ ] 5.6 Update `save_query_as` in `sql_files.rs`: remove the comparison/sync block
      (state.rs:832–837); replace `content_to_save` assignment with
      `let content_to_save = self.query_editor.get_content().to_string()`.
- [ ] 5.7 Update `reset_query_editor` in `query_editor.rs`: remove the
      `self.query_content.clear()` line (state.rs:1938). Keep the other resets
      (`ui.current_sql_file`, `ui.query_modified`, cursor fields).
- [ ] 5.8 Update `set_query_content` in `query_editor.rs`: remove
      `self.query_content = content.clone()`; the method body becomes only
      `self.query_editor.set_content(content); self.ui.query_modified = true;`.
- [ ] 5.9 In `src/app/handlers/connections.rs`: extract a private
      `async fn connect_to_selected(app: &mut App, selected_index: usize)` helper
      containing the full setup sequence (set in-progress, set animation frame, set start
      time, set connecting status, emit toast, clone config, clone connection_manager,
      clone tx, spawn background task). Replace the verbatim blocks at lines 65–98 and
      187–220 with `connect_to_selected(app, selected_index).await`.

## 6. Verify

- [ ] 6.1 `cargo build` clean
- [ ] 6.2 `cargo clippy --all-targets -- -D warnings` clean
- [ ] 6.3 `cargo fmt --check` clean
- [ ] 6.4 `cargo test` green
- [ ] 6.5 Run the application manually: (a) attempt a connection to an unreachable host
      and confirm the loading-dot animation ticks and a "Connection timeout" toast fires
      after 30 s; (b) with an active connection open a SQL file, type a query, press
      Ctrl+Enter — a "Query Result" tab opens with rows; (c) use `:w <name>` to save the
      file and then load it from the SQL files pane — content round-trips intact; (d)
      confirm all toast messages read identically to pre-refactor.
