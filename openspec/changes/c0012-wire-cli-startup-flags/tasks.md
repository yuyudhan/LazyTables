# Tasks: Honor --connection/--database/--table/--read-only

## 1. Thread Cli/StartupOptions into App

- [ ] 1.1 In `src/app/mod.rs`, add `pub struct StartupOptions` with fields
      `connection: Option<String>`, `database: Option<String>`, `table: Option<String>`,
      `read_only: bool`.
- [ ] 1.2 Add fields to the `App` struct: `read_only: bool`,
      `startup_database: Option<String>`, `startup_table: Option<String>`.
- [ ] 1.3 Change `App::new` signature from `pub async fn new(config: Config) -> Result<Self>`
      to `pub async fn new(config: Config, options: StartupOptions) -> Result<Self>`.
      Populate the new fields from `options` in the constructor body.
- [ ] 1.4 In `src/main.rs`, build `StartupOptions` from `cli.connection`, `cli.database`,
      `cli.table`, `cli.read_only` and pass it as the second argument to `App::new`.
- [ ] 1.5 If `startup_table` or `startup_database` is set without a `startup_connection`
      present on the new `App` fields, emit a warning toast at the start of `App::run()`
      ("--table/--database requires --connection; ignored") and clear those fields.

## 2. Auto-connect on --connection

- [ ] 2.1 In `App::run()` (`src/app/mod.rs`), after the `initialize_app_db()` call and
      before the event loop, add a startup auto-connect block. When
      `self.startup_options.connection` (or a stored `startup_connection` field) is
      `Some(ref val)`:
      - If `val` contains `://`: call
        `AdapterFactory::create_connection_from_string(val, "startup".into())` to get a
        transient `ConnectionConfig`; push it to `state.db.connections.connections` but
        do **not** save to disk. If parsing fails, emit a toast and skip.
      - Otherwise: find the first entry in `state.db.connections.connections` where
        `conn.name == val` (case-sensitive). If not found, emit a startup toast
        "--connection '<val>': connection not found" and skip.
- [ ] 2.2 When a resolved connection index is available, set
      `state.connecting_in_progress = Some(index)`, `state.connecting_animation_frame = 0`,
      `state.connection_start_time = Some(Instant::now())`, and set the connection's
      status to `ConnectionStatus::Connecting`.
- [ ] 2.3 Spawn the background connection task (same pattern as
      `src/app/handlers/connections.rs:64–98`): call
      `connection_manager.connect(&config)`, then `list_database_objects`, then send
      `ConnectionEvent::Success` or `ConnectionEvent::Failed` on `connection_events_tx`.
- [ ] 2.4 In the `ConnectionEvent::Failed` arm of `tick()` (`src/app/mod.rs`), clear
      `self.startup_database` and `self.startup_table` so they do not dangle after a
      failed auto-connect.

## 3. Auto-select database and auto-open table after connect

- [ ] 3.1 In the `ConnectionEvent::Success` arm of `tick()`, after all existing success
      processing, check `self.startup_database`. If `Some(ref db_name)`:
      - Search `state.db.database_objects` for a matching schema or database name.
      - If found, set `state.db.selected_schema = Some(db_name.clone())` and rebuild
        the table items via `state.ui.build_selectable_table_items(…)`.
      - If not found, emit a toast "--database '<name>': not found in this connection".
      - Set `self.startup_database = None`.
- [ ] 3.2 Immediately after step 3.1, check `self.startup_table`. If `Some(ref tbl)`:
      - Search `state.db.tables` for an entry matching `tbl` (exact match, then case-
        insensitive fallback).
      - If found, set `state.ui.selected_connection` to the current connection index
        (already set) and use the table-selection helpers to move the UI selection to
        that entry; then call `state.open_table_for_viewing().await`.
      - If not found, emit a toast "--table '<name>': not found in database".
      - Set `self.startup_table = None`.

## 4. Read-only flag and mutation/query gates

- [ ] 4.1 Add `pub(crate) fn is_write_sql(query: &str) -> bool` in `src/app/mod.rs`.
      Trim, take the first whitespace-delimited token, uppercase it, check against
      `["INSERT", "UPDATE", "DELETE", "DROP", "ALTER", "TRUNCATE", "CREATE"]`.
      Document the CTE limitation in a comment.
- [ ] 4.2 In `src/app/handlers/query_results.rs:316` (`handle_edit_mode`), before
      `app.state.update_table_cell(update).await`, check `app.read_only`: if true, emit
      `app.state.toast_manager.warning("Read-only mode: cell edits are disabled")` and
      return `Ok(())`.
- [ ] 4.3 In `src/app/handlers/overlays.rs:172` (`handle_table_delete_confirmation`),
      at the top of the function, check `app.read_only`: if true, emit
      `app.state.toast_manager.warning("Read-only mode: row deletion is disabled")`,
      clear `table_viewer_state.delete_confirmation`, and return `Ok(())`. This prevents
      both modal display and execution.
- [ ] 4.4 In `src/app/handlers/overlays.rs:199` (`handle_set_null_confirmation`), same
      pattern: guard at top, emit warning, clear `set_null_confirmation`, return early.
- [ ] 4.5 In `src/app/handlers/query_editor.rs`, in both the `KeyCode::Char('E')` arm
      (line 23) and the `KeyCode::Enter + CONTROL` arm (line 31): after extracting the
      query text via `app.state.query_editor.get_statement_at_cursor()`, if `app.read_only`
      and `is_write_sql(&query)`, emit
      `app.state.toast_manager.warning("Read-only mode: write queries are disabled")`
      and return `Ok(())`. Let read queries execute normally.

## 5. Spec delta

- [ ] 5.1 Create `openspec/changes/c0012-wire-cli-startup-flags/specs/cli-startup/spec.md`
      with ADDED Requirements for the four startup-flag scenarios (auto-connect by name,
      auto-connect by string, auto-open table, read-only gates, no-flags regression).

## 6. Verify

- [ ] 6.1 `cargo build` clean
- [ ] 6.2 `cargo clippy --all-targets -- -D warnings` clean
- [ ] 6.3 `cargo fmt --check` clean
- [ ] 6.4 `cargo test` green
- [ ] 6.5 Behavioral check:
      - `lazytables --connection <saved-name> --table <table-name>` — app starts,
        connecting animation fires, then `<table-name>` opens in the table viewer
        automatically.
      - `lazytables --read-only` — connect normally; attempt cell edit (press `e`,
        commit) → toast "Read-only mode: cell edits are disabled"; data unchanged.
        Run `DELETE FROM nonexistent` in query editor → toast; no query sent. Run
        `SELECT 1` → executes normally.
      - `lazytables` with no flags — behaviour identical to pre-change binary.
      - `lazytables --connection unknown-name` — toast "--connection 'unknown-name':
        connection not found"; app opens normally on the connections pane.
