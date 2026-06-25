# Record queries and add a history overlay

## Why

`QueryHistoryManager` (`src/database/query_history.rs`, relocated to
`src/persistence/query_history.rs` by c0010) is a complete, tested subsystem:
it opens its own SQLite pool at `~/.lazytables/query_history.db`, creates the
`query_history` table + index on startup, and exposes `add_query()`,
`get_history()`, `search_history()`, `deduplicate_queries()`, and
`clear_old_history()`. Nothing in the application ever calls it. No recording
hook exists in `execute_query_at_cursor` (`src/app/state.rs:2013`), no
`OverlayView::QueryHistory` variant exists in `src/state/view.rs`, and no
`Ctrl+Y` binding exists in `src/app/handlers/global.rs`. The feature is
implemented below the surface but completely invisible to users.

## What Changes

- Add `query_history: QueryHistoryManager` field to `AppState`
  (`src/app/state.rs:28–67`). Construct it in `AppState::new()`
  (`src/app/state.rs:71`) alongside `app_state_db: AppStateDb::new()`. Call
  `query_history.initialize().await` inside `initialize_app_db()`
  (`src/app/state.rs:111`) so both persistence managers are ready before the
  first query runs. After c0005 the split lives in
  `src/app/state/mod.rs` + `src/app/state/query_editor.rs`.

- Record every execution in `execute_query_at_cursor`
  (`src/app/state.rs:2013`; `src/app/state/query_editor.rs` after c0005).
  Capture `std::time::Instant::now()` before the
  `connection_manager.execute_raw_query()` call at `src/app/state.rs:2064`;
  compute `exec_time_ms` from the elapsed duration on both `Ok` and `Err`
  branches. Call `query_history.add_query(query, db_type, db_name,
  Some(exec_time_ms), success, error_message)`. Any recording error is logged
  via `tracing::warn!` and discarded — it MUST NOT fail the query result.

- Add `OverlayView::QueryHistory` to the `OverlayView` enum
  (`src/state/view.rs:22–30`) and an `is_query_history()` predicate on
  `AppView` (`src/state/view.rs:68–101`), matching the pattern of the existing
  `is_debug_view()` predicate at `src/state/view.rs:93`. Extend
  `OverlayView::display_name()` (`src/state/view.rs:105`) with the new arm.

- Add `query_history_scroll_offset: usize` to `UIState`
  (`src/state/ui.rs:254–257` adjacent to `debug_view_scroll_offset`).
  Add `toggle_query_history()`, `query_history_scroll_down()`,
  `query_history_scroll_up()`, `query_history_page_down()`,
  `query_history_page_up()` methods on `UIState`, following the exact pattern
  of the `toggle_debug_view()` / `debug_view_scroll_*` family
  (`src/state/ui.rs:1376–1418`). After c0006 these live in
  `src/state/ui/overlay.rs`.

- New `src/ui/components/query_history.rs` overlay component modeled on
  `src/ui/components/debug_view.rs`. Renders full-screen with `Clear` widget
  for background dimming, a bordered block titled
  `" Query History (Ctrl+Y to toggle) "`, a scrollable `List` of entries
  (newest first) showing `[timestamp] [db_type] [OK/ERR] [exec_ms ms] query…`
  (query truncated to 80 chars), an empty-state paragraph when the list is
  empty, and a help-text footer. Cache the loaded entries on overlay open
  (assigned to a `Vec<QueryHistoryEntry>` field on the component) to avoid
  per-frame `async` calls, matching the caching pattern in `DebugView`.
  Register the new module in `src/ui/components/mod.rs`.

- Global keybinding `Ctrl+Y` in `src/app/handlers/global.rs` toggles
  `OverlayView::QueryHistory` (calls `app.state.ui.toggle_query_history()`)
  and loads the 200 most recent entries via
  `get_history(None, Some(200)).await` on open, storing them on the component.
  `Ctrl+Y` is verified free: existing globals are `?`, `Ctrl+B`, `q`,
  `1`–`6`, `Tab`, `Shift+Tab`, `Ctrl+h/j/k/l` (see
  `src/app/handlers/global.rs:14–109`). In-overlay routing (separate handler
  invoked when `is_query_history()` is true): `j`/`k` scroll one line,
  `Ctrl+d`/`Ctrl+u` page down/up, `Enter` copies the selected entry's full
  `query_text` into the SQL editor via `query_editor.set_content()` and closes
  the overlay, `Esc`/`Ctrl+Y` close.

- Add a help entry for `Ctrl+Y` in `src/ui/help.rs` (the `HelpSystem` that
  builds the help overlay content).

## Impact

- Affected code:
  - `src/app/state.rs` (field, construction, init, recording; `src/app/state/mod.rs` + `src/app/state/query_editor.rs` after c0005)
  - `src/state/view.rs` (new enum variant + predicate)
  - `src/state/ui.rs` (new scroll-offset field + toggle/scroll methods; `src/state/ui/overlay.rs` after c0006)
  - `src/app/handlers/global.rs` (new `Ctrl+Y` arm)
  - `src/ui/components/mod.rs` (register new module)
  - `src/ui/components/query_history.rs` (new file)
  - `src/ui/help.rs` (new help entry)
  - `src/database/query_history.rs` → `src/persistence/query_history.rs` (path rename handled by c0010)

- Affected specs: `query-history` (behavioral ADD — new user-visible feature)

- Risk: medium — new async I/O path at query execution; overlay render path;
  main risk is recording failure leaking into the query result (mitigated by
  explicit `warn!` + discard pattern). No existing behavior changes.

- Depends on: c0010 (moves `QueryHistoryManager` to `src/persistence/`; this
  proposal references the post-c0010 import path
  `crate::persistence::QueryHistoryManager`)
