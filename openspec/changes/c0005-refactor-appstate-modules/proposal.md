# Split AppState; relocate orchestration state; unify errors

## Why

`src/app/state.rs` (2195 LOC, ~55 methods, 0 tests) is the central God object of the
application. It mixes concerns in ways that make the file hard to navigate, test, and extend:

- **Mixed layers.** Pure state mutation (navigation predicates, selection updates at lines
  131–351) sits beside heavy async business logic: DB I/O in `connect_to_selected_database`
  (line 551), file I/O in `save_query_as` (line 806), and blocking `std::fs` calls in
  `load_query_file` (line 870).

- **Misplaced fields.** Six connection-progress/animation fields — `connecting_in_progress`,
  `connection_start_time`, `connecting_animation_frame`, `test_connection_in_progress`,
  `test_animation_frame`, `test_start_time` (state.rs:54–66) — are read and written
  exclusively by `src/app/mod.rs` `tick()` and `src/app/handlers/connections.rs`. They are
  orchestration state, not application state, and live on the wrong struct.

- **Private background protocol.** `ConnectionEvent` and `TestConnectionEvent` (app/mod.rs:22–39)
  are private enums carrying the background-task → event-loop channel protocol. Their
  privacy prevents testing and reuse by sibling modules.

- **Inconsistent error types.** Twelve methods return either `Result<_, Box<dyn std::error::Error>>`
  or `Result<_, String>`, diverging from the codebase-wide `crate::core::error::Result`
  (`LazyTablesError`). Every caller is forced to `map_err` and loses structured information.

- **Duplicate query state.** `query_content: String` (state.rs:36) is a redundant copy of
  `query_editor.get_content()`. Three sync methods — `handle_query_editor_input` (line 1967),
  `handle_query_editor_newline` (line 1974), `handle_query_editor_backspace` (line 1982) —
  exist only to keep this copy in sync after every keypress.

- **Duplicate spawn block.** The `tokio::spawn` connect task in `src/app/handlers/connections.rs`
  appears verbatim twice (lines 65–98 and 187–220), risking behavioral drift on any future edit.

## What Changes

- Convert `src/app/state.rs` into `src/app/state/mod.rs` plus sibling impl files, splitting
  methods by verified concern:
  - `mod.rs`: `AppState` struct def, fields, `new()`, `initialize_app_db()`, `Default`,
    `QueryEditorMovement` enum, and re-exports (pub use lines).
  - `navigation.rs`: focus cycling/moving and pane-enabled predicates
    (state.rs:131–351, 1876–1935).
  - `connections.rs`: connection selection, `get_selected_connection*`, modal open/close,
    `connection_up/down`, `clamp_connection_selection` (state.rs:352–497 except
    `save_connection_from_modal`).
  - `modals.rs`: `save_connection_from_modal` (state.rs:409).
  - `connection_lifecycle.rs`: async connect/disconnect/health —
    `disconnect_all_except`, `connect_to_selected_database`, `try_connect_to_database`,
    `disconnect_from_database`, `disconnect_from_database_sync`, `check_connection_health`
    (state.rs:540–709, 1768).
  - `sql_files.rs`: SQL file selection and I/O — `get_selected_sql_file`,
    `load_selected_sql_file`, `clamp_sql_file_selection`, `load_sql_files_for_connection`,
    `refresh_sql_files`, `save_query_as`, `save_query`, `load_query_file`,
    `save_sql_file_with_connection`, `delete_sql_file`, and remaining file helpers
    (state.rs:711–1018, 1300–1683).
  - `query_editor.rs`: editor delegation, cursor methods, and `execute_query_at_cursor`
    (state.rs:1020–1265, 1858–2152).
  - `table_viewer.rs`: table selection, `open_table_for_viewing`, `load_table_data`,
    `load_table_metadata`, mutation wrappers, `reload_current_table_tab`,
    `get_statement_under_cursor` (state.rs:499–531, 1685–1856).
  All implemented as `impl AppState` blocks; `mod.rs` re-exports every public type so
  external import paths are unchanged.

- Move the 6 orchestration-only fields from `AppState` to `App` (`src/app/mod.rs`):
  `connecting_in_progress`, `connection_start_time`, `connecting_animation_frame`,
  `test_connection_in_progress`, `test_animation_frame`, `test_start_time`.
  Update all read/write sites in `tick()` (app/mod.rs:295–463) and
  `handlers/connections.rs` (lines 38–48, 161–171) to reference `app.<field>`.

- Promote `ConnectionEvent` and `TestConnectionEvent` (app/mod.rs:22–39) into a new
  file `src/app/background.rs` as a single public `BackgroundEvent` enum with variants
  `Connection(ConnectionResult)` and `TestConnection(TestConnectionResult)`, or equivalent
  sub-enums. Update `app/mod.rs` channel types accordingly.

- Unify error types: change all 12 methods returning `Box<dyn std::error::Error>` or
  `String` errors to return `crate::core::error::Result<_>`. Add `LazyTablesError::SqlFile(String)`
  and `LazyTablesError::Editor(String)` variants to `src/core/error.rs`. Preserve all
  user-facing toast message strings. Port `load_query_file` (state.rs:869) from blocking
  `std::fs` reads to `crate::io::async_fs` (already used at state.rs:775, 842, 844),
  making `load_query_file` async.

- Remove `query_content: String` field and its 3 sync wrapper methods
  (`handle_query_editor_input`, `handle_query_editor_newline`, `handle_query_editor_backspace`).
  Make `query_editor` the single source of truth; update all read sites that used
  `self.query_content` to use `self.query_editor.get_content()` instead (e.g., `move_down`
  at state.rs:258, `move_right` at state.rs:339, `save_sql_file_with_connection` at 1304,
  `save_query_as` at 832).

- De-duplicate the connect `tokio::spawn` block in `src/app/handlers/connections.rs`
  (verbatim copies at lines 65–98 and 187–220) by extracting it to a single
  `async fn connect_to_selected(app: &mut App, selected_index: usize)` helper function,
  called from both the search-mode Enter branch and the normal-mode Enter/Space branch.

## Impact

- Affected code: `src/app/state.rs` (split into `src/app/state/`),
  `src/app/mod.rs` (fields added, enum moved), `src/app/handlers/connections.rs`
  (field accesses updated, spawn de-duplicated), `src/app/background.rs` (new file),
  `src/core/error.rs` (new variants), `src/app/handlers/query_editor.rs` (callers at
  lines 24 and 32 unchanged — call signature preserved).
- Affected specs: none (internal refactor, behavior preserved).
- Risk: medium-high — touches the central struct; every handler and the render path
  borrows from `AppState`.
- Depends on: c0002 (which deletes `commands/`, `query_content` callers via commands,
  and `events.rs::AppEvent`; c0005 must land after c0002 so dead code is already gone).
