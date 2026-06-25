# Design: Record queries and add a history overlay

## Context

`QueryHistoryManager` in `src/database/query_history.rs` (relocated to
`src/persistence/query_history.rs` by c0010) is already fully implemented:

- `new()` computes `~/.lazytables/query_history.db` and returns a manager
  with `pool: None`.
- `initialize(&mut self) -> Result<()>` connects via `SqliteConnectOptions`,
  creates the `query_history` table with columns `id`, `query_text`,
  `database_type`, `database_name`, `executed_at`, `execution_time_ms`,
  `success`, `error_message`, and a covering index on `(database_type,
  executed_at DESC)`.
- `add_query(query_text, database_type, database_name, execution_time_ms,
  success, error_message) -> Result<i64>` inserts one row.
- `get_history(database_type_filter, limit) -> Result<Vec<QueryHistoryEntry>>`
  returns entries newest first.
- `deduplicate_queries() -> Result<usize>` removes older duplicates (not
  auto-invoked).
- `#[cfg(test)]` coverage exists in `src/database/query_history.rs:346–437`
  and moves with the file in c0010.

`execute_query_at_cursor` (`src/app/state.rs:2013`) is the single query
execution path; handlers call it from `src/app/handlers/query_editor.rs:24,32`
on `Ctrl+Enter`. It already records timing context via `debug_messages`. The
overlay system uses `AppView::Overlay(OverlayView)` with existing variants
`ConnectionForm`, `DebugView`, and `Help` (`src/state/view.rs:22–30`).
`DebugView` (`src/ui/components/debug_view.rs`) is the template for a
full-screen, scrollable, cached overlay.

## Goals / Non-Goals

**Goals:**
- Record every query execution (success and failure) with text, db type,
  db name, timestamp, execution time, and error message.
- Display history in a full-screen overlay accessible via `Ctrl+Y`.
- Allow the user to load any historical query into the SQL editor with `Enter`.
- Persist history across restarts (SQLite-backed, existing manager behavior).

**Non-goals:**
- Automatic deduplication on record (manager supports it; invoked by user
  action only — not in scope here).
- In-overlay search (not required; `search_history()` exists for a future
  change).
- Pagination beyond the initial 200-entry cap (the manager supports arbitrary
  limits; this change uses `Some(200)` — sufficient for normal use).
- Changing the storage format or file path (managed entirely by
  `QueryHistoryManager`; this change only wires callers).

## Decisions

### Decision 1 — Recording failure is non-fatal

**What:** If `add_query()` returns `Err`, the error is logged with
`tracing::warn!("query history recording failed: {}", e)` and the `Result`
from `execute_query_at_cursor` is unaffected (the query's own Ok/Err
is returned as before).

**Why:** History is a convenience feature. A transient SQLite write error
(disk full, locked file, corrupted DB) must never surface to the user as a
query failure. The existing `AppStateDb` uses the same silent-on-error pattern.

**Alternatives rejected:** Propagating the recording error upward would break
every query when the history DB is unavailable — unacceptable.

---

### Decision 2 — Load entries on overlay open, cache for the session

**What:** When `toggle_query_history()` opens the overlay, call
`query_history.get_history(None, Some(200)).await` once and store the
`Vec<QueryHistoryEntry>` on `QueryHistoryView` (the new component struct).
Re-use the cached slice on every render frame until the overlay is closed;
clear the cache on close.

**Why:** `get_history` is async and must not be called from the synchronous
render path. `DebugView` solves the analogous problem by caching computed
statistics (`CachedStatistics` at `src/ui/components/debug_view.rs:27–31`).
Loading 200 entries on open is fast (SQLite local read) and the list does not
need real-time refresh inside a single overlay session.

**Alternatives rejected:** Storing the entries on `AppState` and refreshing
every tick would couple the render cycle to async I/O. Per-frame DB reads are
unjustifiable for a static history view.

---

### Decision 3 — `Ctrl+Y` keybinding (mnemonic: histor**Y**)

**What:** `(KeyModifiers::CONTROL, KeyCode::Char('y'))` in
`src/app/handlers/global.rs` toggles the query history overlay, following the
exact same pattern as `Ctrl+B` at `src/app/handlers/global.rs:21–24`.

**Why:** All six numeric pane keys (`1`–`6`), `Tab`, `Shift+Tab`, `?`, `q`,
`Ctrl+B`, and `Ctrl+h/j/k/l` are taken (verified against
`src/app/handlers/global.rs:14–109`). `Ctrl+Y` is free and carries an obvious
mnemonic.

**Alternatives rejected:** `Ctrl+H` (taken), `Ctrl+R` (common terminal
reverse-search; avoid confusion), `h` (used by vim navigation).

---

### Decision 4 — Query truncation in list view; full text on `Enter`

**What:** The overlay list displays each entry's `query_text` truncated to 80
characters with `…` appended. When the user presses `Enter` on a selected
entry, the **full** `query_text` (from the cached `QueryHistoryEntry`) is
loaded into the SQL editor via `query_editor.set_content(entry.query_text)`.

**Why:** Consistent with how `execute_query_at_cursor` already truncates
queries in toast messages (`src/app/state.rs:2050–2054`). The full text is
preserved in the cached struct so `Enter` can load it without a second DB call.

---

### Decision 5 — No structural coupling to `DatabaseType` at the AppState level

**What:** The `db_type` passed to `add_query()` is extracted from the active
connection config (`connection.database_type`) already available at the point
of `execute_query_at_cursor`. The `db_name` is the connection's `database`
field (`Option<String>`), passed as `.as_deref()`.

**Why:** `execute_query_at_cursor` already holds `connection` (the
`ConnectionConfig`) in scope. No new traversal is needed.

---

### Decision 6 — `query_history` field initialized in `initialize_app_db`

**What:** `AppState::new()` constructs `QueryHistoryManager::new()` (fallible
— propagates as a `String` error to the caller the same way `AppStateDb` does).
`AppState::initialize_app_db()` calls `query_history.initialize().await` after
the existing `AppStateDb::initialize().await`. A failure logs a warning but
does not prevent startup (the `pool` field stays `None`; `add_query` will
return `Err` which is non-fatal per Decision 1).

**Why:** Matching the existing two-step pattern (construct / async-init) used
by `AppStateDb` keeps the initialization shape uniform. A broken history DB
must not prevent the app from opening.

## Edge Cases & Failure Modes

- **No DB connected when overlay opens:** `get_history(None, Some(200))` has
  no connection filter and operates on the local SQLite pool — independent of
  whether a remote DB connection exists. The overlay opens normally.

- **Empty history:** The overlay renders an explanatory `Paragraph` ("No query
  history yet. Execute queries with Ctrl+Enter.") instead of the list. The
  `Enter` key is a no-op when the list is empty.

- **`query_text` contains a single-quote or special chars:** `add_query` uses
  sqlx bind parameters (see `src/database/query_history.rs:113–136`);
  the text is stored and retrieved verbatim.

- **`execution_time_ms` overflow:** `Instant::elapsed().as_millis()` returns
  `u128`; cast to `i64` with `.min(i64::MAX as u128) as i64` before passing as
  `Option<i64>`. Queries running longer than ~292 million years are saturated.

- **`QueryHistoryManager::new()` fails** (home directory unavailable): treat
  as a warn-and-continue at `AppState::new()` time; store a sentinel that
  makes every subsequent `add_query` / `get_history` a silent no-op. The
  simplest implementation: keep `Option<QueryHistoryManager>` on `AppState`
  and `None`-check before every call.

- **Concurrent access:** `SqlitePool` handles connection pooling internally;
  concurrent writes from the same process are safe. The overlay loads entries
  once on open — no concurrent overlay + recording conflict.

- **Overlay opened during an in-progress query:** The cached entry list
  reflects state at open time; the in-flight query's entry appears after the
  next open. Acceptable — history is not a live feed.

## Migration / Cutover

### Files to add
- `src/ui/components/query_history.rs` — new overlay component.

### Files to modify

**`src/app/state.rs`** (becomes `src/app/state/mod.rs` + siblings after c0005):
- Add `query_history: Option<QueryHistoryManager>` to `AppState` struct
  (after the `app_state_db` field at line 50).
- In `AppState::new()` (line 87 block): add `query_history: QueryHistoryManager::new().ok()`.
- In `initialize_app_db()` (line 111): after the existing `AppStateDb` init,
  call `if let Some(qh) = &mut self.query_history { if let Err(e) =
  qh.initialize().await { tracing::warn!("query history init failed: {}", e); } }`.
- In `execute_query_at_cursor()` (line 2013): wrap the
  `connection_manager.execute_raw_query()` call with a
  `std::time::Instant::now()` before and recording after each branch.

**`src/state/view.rs`**:
- Add `QueryHistory` to `OverlayView` enum (line 29, after `Help`).
- Add `is_query_history()` predicate to `impl AppView` (after `is_help()` at
  line 98).
- Add `Self::QueryHistory => "Query History"` arm to `display_name()` (line
  110).

**`src/state/ui.rs`** (becomes `src/state/ui/overlay.rs` after c0006):
- Add `pub query_history_scroll_offset: usize` field adjacent to
  `debug_view_scroll_offset` (line 255).
- Initialize to `0` in `UIState::new()` (adjacent to line 349).
- Add `toggle_query_history()`, `query_history_scroll_down(max)`,
  `query_history_scroll_up()`, `query_history_page_down(max, page)`,
  `query_history_page_up(page)` methods following the
  `toggle_debug_view` / `debug_view_scroll_*` pattern (lines 1376–1418).

**`src/app/handlers/global.rs`**:
- Add `(KeyModifiers::CONTROL, KeyCode::Char('y'))` arm before the `_ =>
  Ok(None)` catch-all (line 107), calling
  `app.state.ui.toggle_query_history()` and, on open, awaiting
  `app.state.query_history.get_history(None, Some(200))` to populate the
  cached entry list on the component.
- Add in-overlay routing: when `app.state.ui.current_view.is_query_history()`
  intercept `j`/`k`/`Ctrl+d`/`Ctrl+u`/`Enter`/`Esc`/`Ctrl+Y`.

**`src/ui/components/mod.rs`**:
- Add `pub mod query_history;` and re-export `QueryHistoryView`.

**`src/ui/help.rs`**:
- Add `Ctrl+Y` → "Toggle query history overlay" entry to the global section.

### What to delete
Nothing — this change is purely additive.

## Verification

Run the application against a SQLite file DB (no server required):
1. Execute three queries: two valid (`SELECT 1`, `SELECT 2`), one invalid
   (`SELECT * FROM nonexistent_table`).
2. Press `Ctrl+Y` — the history overlay opens showing all three entries,
   newest first, with the failing query marked `ERR`.
3. Navigate to the `SELECT 1` entry with `j`/`k`, press `Enter` — the SQL
   editor is populated with `SELECT 1` and the overlay closes.
4. Quit and reopen the application; press `Ctrl+Y` — all three entries are
   still present (SQLite persistence confirmed).
5. Confirm that `Ctrl+B` (debug view) still works and its entries show
   tracing output from the recording hook.
