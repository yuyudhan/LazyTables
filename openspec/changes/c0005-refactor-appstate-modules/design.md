# Design: Split AppState; relocate orchestration state; unify errors

## Context

`src/app/state.rs` is 2195 lines and holds a single `AppState` struct with 17 fields and
roughly 55 methods spanning 8 distinct concerns. It was the founding file of the app and
accumulated all state-adjacent logic without ever being split. The abandoned
`src/app/state_new/` directory (7 files, ~460 LOC) proves a previous attempt failed; c0002
deleted that dead tree. The `impl AppState` across multiple files pattern (what state_new
tried) is valid Rust and is the chosen approach.

`src/app/mod.rs` holds two private enums (`ConnectionEvent`, `TestConnectionEvent`) that
carry the only inter-task protocol in the application. They live in the orchestrator file
because there was nowhere better to put them; their privacy prevents unit testing.

`src/app/handlers/connections.rs` has two verbatim copies of the `tokio::spawn` connect
block (lines 65–98 and 187–220), one for the search-mode Enter path and one for the
normal-mode Enter/Space path. They were written separately and never reconciled.

## Goals / Non-Goals

**Goals:**
- Split `state.rs` into coherent siblings without changing any method signature, toast
  text, or user-observable behavior.
- Remove the 6 App-only fields from `AppState`; keep `AppState` pure application state.
- Give the background task protocol a stable public home in `background.rs`.
- Make all methods return `crate::core::error::Result` so error handling is uniform.
- Eliminate the `query_content` duplicate field to have one source of truth.
- De-duplicate the spawn block.

**Non-Goals:**
- Changing any rendering logic or key-handler routing.
- Adding tests in this proposal (no existing tests exist for state.rs; future proposals
  can add them once the surface is smaller).
- Splitting `UIState` or `DatabaseState` (separate proposals: c0006, c0007).
- Changing the `ConnectionManager` or adapter layer.

## Decisions

### Decision 1: `impl AppState` blocks in sibling files, not new structs

**What:** Keep the single `AppState` struct in `mod.rs`. Move method groups into sibling
`*.rs` files as additional `impl AppState { … }` blocks. Rust allows this freely within
the same crate.

**Why:** Creating new structs (e.g., `NavigationState`, `SqlFileState`) would require
threading those through all 30+ call sites in handlers, the render path, and `UI::draw`.
Split-by-impl is a zero-churn refactor — external paths (`app.state.open_add_connection_modal()`)
do not change.

**Alternatives rejected:**
- *New sub-structs with delegation:* Too many callsite changes; risks introducing borrowing
  issues when two sub-structs borrow `self` fields simultaneously.
- *Trait per concern:* Adds trait-object overhead and complicates future method addition;
  the split-by-impl gives the same organization for free.

### Decision 2: Move 6 orchestration fields to `App`, not to a new sub-struct

**What:** Add `connecting_in_progress`, `connection_start_time`, `connecting_animation_frame`,
`test_connection_in_progress`, `test_animation_frame`, `test_start_time` as top-level
fields on `App` (app/mod.rs). `connection_timeout_seconds` stays on `AppState` because it
is a user-configured value, not transient orchestration state.

**Why:** These six fields are only ever read or written in `App::tick()` (app/mod.rs:290–469)
and `handlers::connections::handle` (handlers/connections.rs:38–48, 161–171). They do not
express properties of the application's persistent data model — they express the
orchestrator's in-flight work. The `App` struct is the orchestrator.

**Alternatives rejected:**
- *A new `ConnectionProgress` sub-struct on `AppState`:* Still leaks orchestration into
  the data layer; the 6 fields would still be bundled with app state.
- *Leave them on `AppState`:* Status quo — the exact problem being fixed.

### Decision 3: `BackgroundEvent` in `src/app/background.rs`

**What:** Create `src/app/background.rs` with a public `BackgroundEvent` enum whose
variants carry the success/failure outcomes for both regular connections and test connections:

```
pub enum BackgroundEvent {
    Connection(ConnectionOutcome),
    TestConnection(TestConnectionOutcome),
}
pub enum ConnectionOutcome { Success { … }, Failed { … } }
pub enum TestConnectionOutcome { Success(String), Failed(String) }
```

(Exact naming may stay close to the originals — `ConnectionEvent`/`TestConnectionEvent` —
as nested enums or as variants with inner enums; implementer's choice as long as both
channel types collapse to one `BackgroundEvent`.)

Update `App`'s channel pair: `connection_events_tx/rx` and `test_connection_events_tx/rx`
collapse to a single `UnboundedSender<BackgroundEvent>` / `UnboundedReceiver<BackgroundEvent>`.
The `tick()` method demultiplexes on the variant.

**Why:** One channel pair is simpler and the channel type now has a stable public home
that tests can import.

**Alternatives rejected:**
- *Keep two channels:* Preserves the current split but doesn't solve the privacy/testability
  issue and leaves two `try_recv` blocks in `tick()`.
- *Put variants directly in `app/mod.rs` as `pub`:* Leaks the module's internal protocol
  into the crate root for callers that shouldn't care.

### Decision 4: Two new `LazyTablesError` variants; preserve toast text

**What:** Add to `src/core/error.rs`:
```rust
#[error("SQL file error: {0}")]
SqlFile(String),
#[error("Editor error: {0}")]
Editor(String),
```

Change all 12 identified method return types:
- `Result<_, Box<dyn std::error::Error>>` → `crate::core::error::Result<_>`
  (use `?` after converting string-literal errors: `"…".into()` → `LazyTablesError::SqlFile("…".into())`)
- `Result<_, String>` → `crate::core::error::Result<_>`
  (map callers that matched on the String error to match on `LazyTablesError::Other(s)`)

Toast messages in `execute_query_at_cursor` (state.rs:2019, 2025, 2034, 2107, 2132)
and in sql-file methods are preserved verbatim — the toast text is in the toast call,
not in the error value.

**Why:** Uniform error type removes 30+ `map_err` calls at callers. The two new variants
give structured information without forcing every case into `Other(String)`.

**Alternatives rejected:**
- *Leave `Box<dyn std::error::Error>` in place:* Perpetuates the inconsistency; conflicts
  with the codebase convention seen in handlers and the database layer.
- *Use only `LazyTablesError::Other(String)`:* Possible but loses category information;
  `SqlFile` and `Editor` are distinct enough to warrant named variants.

### Decision 5: Make `load_query_file` async; port to `io::async_fs`

**What:** Change `load_query_file` (state.rs:869) from a sync `fn` returning
`Result<_, Box<dyn std::error::Error>>` to `async fn` returning
`crate::core::error::Result<()>`. Replace `std::fs::read_to_string` with
`crate::io::async_fs::read_to_string` (the module is already in scope and used at
state.rs:775, 842, 844).

Update `load_selected_sql_file` and `load_sql_file_into_editor` — both currently call
`load_query_file` synchronously — to `await` the call and be marked `async` themselves.

**Why:** A blocking `std::fs` call inside an async tokio task stalls the thread. The
`io::async_fs` module exists precisely to provide async file I/O. The change is mechanical
and non-behavioral.

**Alternatives rejected:**
- *`tokio::task::spawn_blocking`:* Would work but adds a task allocation; using the
  existing async abstraction is cleaner.

### Decision 6: Remove `query_content` duplicate; `query_editor` is the single source

**What:** Remove the `query_content: String` field from `AppState`. Remove the three sync
wrapper methods that maintained it:
- `handle_query_editor_input` (state.rs:1967)
- `handle_query_editor_newline` (state.rs:1974)
- `handle_query_editor_backspace` (state.rs:1982)

Update all sites that read `self.query_content` to call `self.query_editor.get_content()`
instead:
- `move_down` (state.rs:258): `query_content.lines().count()` → `query_editor.get_content().lines().count()`
- `move_right` (state.rs:339): `query_content.lines().nth(…)` → `query_editor.get_content().lines().nth(…)`
- `save_sql_file_with_connection` (state.rs:1304, 1365): sync step replaced with direct
  `query_editor.get_content()` call.
- `save_query_as` (state.rs:832–837): comparison/sync step removed; use `query_editor.get_content()` directly.
- `reset_query_editor` (state.rs:1937): remove `self.query_content.clear()`.
- `Default` impl (state.rs:2176): remove `query_content: String::new()`.
- `new()` (state.rs:91): remove `query_content: String::new()`.
- `set_query_content` (state.rs:1863): method now only calls `query_editor.set_content(content)`.

The `commands/` module (which was the only external caller of the wrapper methods) is
deleted in c0002, so no external callers exist after c0002 lands.

**Why:** Two sources of truth for the same data require constant synchronization and create
subtle bugs (e.g., `save_query_as` had to compare the two at line 832 to pick the "more
up-to-date" one). The `QueryEditor` component owns the authoritative content; `AppState`
should read from it, not mirror it.

**Alternatives rejected:**
- *Keep `query_content` as a cache:* The sync methods already demonstrate the cost:
  every insert must call two update paths. With `query_editor.get_content()` being a cheap
  `&str` borrow, caching adds complexity with no benefit.

### Decision 7: Extract `connect_to_selected` helper in `handlers/connections.rs`

**What:** Extract a private `async fn connect_to_selected(app: &mut App, selected_index: usize)`
helper in `src/app/handlers/connections.rs` that contains the full setup sequence (mark
in-progress, set status, clone config, spawn background task). Both the search-mode Enter
handler (lines 37–100) and the normal-mode Enter/Space handler (lines 160–221) call this
helper instead of inlining the logic.

**Why:** The two blocks (lines 65–98 and 187–220) are already textually identical. Any
future change to the connect flow must currently be made in two places.

**Alternatives rejected:**
- *Deduplicate via `App` method:* Would work, but `handlers/connections.rs` already owns
  this concern; a file-local helper avoids creating a public method on `App` for internal
  handler logic.

## Edge Cases & Failure Modes

- **`execute_query_at_cursor` toast text must be preserved verbatim.** The callers in
  `handlers/query_editor.rs` (lines 24 and 32) surface the error string to the user again
  on top of what `execute_query_at_cursor` itself toasts — identical wording matters for
  UX consistency. The method's return type changes to `crate::core::error::Result<()>` but
  callers already call `.to_string()` on the error, so no visible change.

- **Moving orchestration fields to `App`.** The `tick()` loop reads these every 250 ms
  tick. After the move, `tick()` uses `self.connecting_in_progress` (not
  `self.state.connecting_in_progress`). The `handlers/connections.rs` handler receives
  `app: &mut App`, so it already has access via `app.connecting_in_progress`. No logic
  change — only field-path change.

- **`connection_timeout_seconds`.** This field (state.rs:60) is NOT moved to `App` — it
  is a user-configurable constant, not transient orchestration state. It stays on
  `AppState`. The `tick()` method reads it as `self.state.connection_timeout_seconds`.

- **`load_query_file` becoming async.** Its callers (`load_selected_sql_file`,
  `load_sql_file_into_editor`) become async. These are called from key handlers in
  `handlers/sql_files.rs` which are already `async fn` — no propagation issue.

- **`BackgroundEvent` channel collapse.** `App::new()` creates two channel pairs today.
  After the collapse it creates one. The `tick()` demultiplexes on the variant. Care
  must be taken that the `test_connection_task_handle` abort path still works (it
  currently aborts a `JoinHandle<()>` stored on `App`; this is unchanged by the channel
  merge).

- **Module visibility.** Private helpers shared across the new sibling files (e.g.,
  `insert_newline_at_cursor` at state.rs:1053 which is `fn`, not `pub fn`) must be
  declared `pub(super)` in the sibling file so `mod.rs` and other siblings can call them.

- **Borrow checker.** The split into multiple `impl AppState` blocks in sibling files is
  legal Rust. However, methods that borrow two sub-fields simultaneously (e.g.,
  `self.ui` and `self.db`) will still work because the split is at the file level, not
  at the struct level.

## Migration / Cutover

### Files to create
- `src/app/state/mod.rs` (contains struct, fields, `new()`, `initialize_app_db()`,
  `Default`, `QueryEditorMovement` enum, all `pub use` re-exports)
- `src/app/state/navigation.rs`
- `src/app/state/connections.rs`
- `src/app/state/modals.rs`
- `src/app/state/connection_lifecycle.rs`
- `src/app/state/sql_files.rs`
- `src/app/state/query_editor.rs`
- `src/app/state/table_viewer.rs`
- `src/app/background.rs`

### Files to delete
- `src/app/state.rs` (replaced by `src/app/state/` directory)

### Callsites to update in `src/app/mod.rs`
- Remove `ConnectionEvent` and `TestConnectionEvent` enum definitions (lines 22–39);
  `use crate::app::background::BackgroundEvent;` instead.
- Replace `connection_events_tx/rx` and `test_connection_events_tx/rx` fields with a
  single `background_events_tx: UnboundedSender<BackgroundEvent>` and
  `background_events_rx: UnboundedReceiver<BackgroundEvent>`.
- Add 6 fields to the `App` struct:
  `connecting_in_progress: Option<usize>`, `connection_start_time: Option<Instant>`,
  `connecting_animation_frame: u8`, `test_connection_in_progress: bool`,
  `test_animation_frame: u8`, `test_start_time: Option<Instant>`.
- In `App::new()`: remove channel-pair declarations; add single-pair declaration; initialise
  the 6 new fields to their zero values.
- In `tick()` (lines 295–469): rewrite `self.state.connecting_in_progress` →
  `self.connecting_in_progress`, and likewise for the other 5 fields.

### Callsites to update in `src/app/handlers/connections.rs`
- Lines 38, 46, 47, 48: `app.state.connecting_in_progress` → `app.connecting_in_progress`;
  `app.state.connecting_animation_frame` → `app.connecting_animation_frame`;
  `app.state.connection_start_time` → `app.connection_start_time`.
- Lines 161, 169, 170, 171: same updates for the Enter/Space path.
- Lines 65–98 and 187–220: replace with a call to `connect_to_selected(app, selected_index).await`.
- Update `use` line: `ConnectionEvent` → `crate::app::background::BackgroundEvent` (or
  the relevant nested variant names).
- The `tx.send(BackgroundEvent::Connection(…))` replaces `tx.send(ConnectionEvent::…)`.

### Callsites to update in `src/core/error.rs`
- Add two variants: `SqlFile(String)` and `Editor(String)`.

### Internal state.rs sites to update (become internal after the split)
- `move_down` (line 258): `self.query_content.lines().count()` → `self.query_editor.get_content().lines().count()`
- `move_right` (line 339): `self.query_content.lines().nth(…)` → `self.query_editor.get_content().lines().nth(…)`
- `save_sql_file_with_connection` (lines 1304, 1365): remove `query_content` sync steps;
  call `self.query_editor.get_content()` directly.
- `save_query_as` (lines 832–838): remove comparison/sync; `content_to_save = self.query_editor.get_content().to_string()`.
- `reset_query_editor` (line 1938): remove `self.query_content.clear()`.
- `set_query_content` (line 1863): remove `self.query_content = content.clone()`.
- `new()` and `Default`: remove `query_content: String::new()`.

## Verification

Run the application:
1. **Spinner animation + timeout:** Start a connection attempt to an unreachable host. The
   loading-dot animation increments every tick (250 ms); after 30 seconds the timeout toast
   fires and `connecting_in_progress` clears. Proves the 6 fields moved correctly.
2. **Query execution (Ctrl+Enter):** With an active connection, open a SQL file, type a
   query, press Ctrl+Enter. A "Query Result (HH:MM:SS)" tab opens in the tabular output
   pane with correct rows. Proves `execute_query_at_cursor` path intact after the module
   split.
3. **SQL file save/load:** Use `:w <name>` to save the query, then navigate to the SQL
   files pane and load it. The content round-trips without corruption. Proves
   `save_sql_file_with_connection` and `load_query_file` (now async) work correctly.
4. **Toast messages:** All three operations above produce the same toast text as before
   the refactor (no `map_err`-introduced formatting changes).
