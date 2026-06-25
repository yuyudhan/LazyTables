# Design: Honor --connection/--database/--table/--read-only

## Context

`src/cli.rs` (lines 22–36) declares four pub fields on `Cli` that clap parses from
argv: `connection: Option<String>`, `database: Option<String>`, `table: Option<String>`,
`read_only: bool`. These are silently discarded: `src/main.rs:37` calls
`App::new(config).await`, which accepts only `Config` (src/app/mod.rs:71). The four
flags have never been wired to any behaviour.

The app's connection system is event-driven: handler code spawns a tokio task that
calls `ConnectionManager::connect`, then sends a `ConnectionEvent::Success|Failed` on
an unbounded mpsc channel. `App::tick()` (src/app/mod.rs:290) drains that channel and
updates state — including the connecting animation fields
(`connecting_in_progress`, `connecting_animation_frame`, `connection_start_time`) that
are read every tick to render the spinner. Any auto-connect implementation must route
through this same channel so the animation and timeout logic fires identically.

`AdapterFactory::create_connection_from_string` (src/database/factory.rs:65) parses a
connection URI, detects the database type, and returns a `(DatabaseType, Box<dyn Connection>)`.
`detect_database_type` (factory.rs:36) uses a scheme-prefix check. This is the hook for
the "looks like a connection string" branch of `--connection`.

`state.db.connections.connections` is a `Vec<ConnectionConfig>`; each entry carries a
`name: String` field (connection.rs:78). Name-based lookup is a linear scan.

`state.db.tables` is a `Vec<String>` built in `tick()` from `ConnectionEvent::Success`
objects (app/mod.rs:344–354). `open_table_for_viewing` (state.rs:1685) reads
`ui.get_selected_table_name()`, so the table must be selected in `state.ui` before
the call.

`state.db.selected_schema` is `Option<String>` (state/database.rs:26); it is set
elsewhere to scope table lists to one schema.

## Goals / Non-Goals

**Goals:**
- `--connection <name>` auto-connects to a saved connection by name at startup.
- `--connection <conn-string>` parses and connects to an ad-hoc URL at startup.
- `--database <name>` scopes to a database/schema after successful auto-connect.
- `--table <name>` opens a table in the viewer after successful auto-connect.
- `--read-only` gates all data-mutation paths (cell edit, row delete, set-NULL, write
  SQL) with an informative toast; read queries and navigation are unaffected.
- No flags → behaviour identical to today.

**Non-Goals:**
- Persisting startup flags or adding a config-file equivalent.
- Automatically reconnecting on disconnect when `--connection` was used.
- Validating flags before the terminal initialises (errors surface as toasts, not
  early exits, to avoid raw-terminal teardown complications).
- Fully parsing connection strings into `ConnectionConfig` with password storage;
  transient (in-memory, not saved) entries are sufficient for a URI-supplied connection.

## Decisions

### Decision 1 — `StartupOptions` struct in `src/app/mod.rs`

**What:** Introduce a plain struct:
```rust
pub struct StartupOptions {
    pub connection: Option<String>,
    pub database:   Option<String>,
    pub table:      Option<String>,
    pub read_only:  bool,
}
```
Built in `src/main.rs` from `Cli` fields before calling `App::new`. `App::new`
signature becomes `pub async fn new(config: Config, options: StartupOptions) -> Result<Self>`.

**Why:** Decouples the CLI parsing type (`Cli`, a clap derive) from the application
constructor; keeps clap out of the `app` module's import surface. A bare struct is more
testable and easier for c0005's restructure to absorb than a direct `Cli` reference.
If future flags are added, only `main.rs` and `StartupOptions` change.

**Alternatives rejected:**
- Pass `Cli` directly into `App::new` — imports clap into the app module, couples the
  constructor to the CLI surface.
- Store flags as individual fields on `App` without a struct — no grouping, harder to
  evolve.

### Decision 2 — Auto-connect runs at `App::run()` start, after `initialize_app_db()`

**What:** In `App::run()`, immediately after the `initialize_app_db()` call and before
entering the event loop, check `self.startup_options.connection`. If set, perform
startup auto-connect.

**Why:** `initialize_app_db()` makes `AppStateDb` ready, which is needed to record the
active connection. Running before the event loop means the first `draw()` already shows
the "Connecting…" animation. The terminal is already initialised at this point (it is
passed into `run()`) so there is no raw-mode concern.

**Alternatives rejected:**
- Auto-connect in `App::new()` — the terminal is not yet set up; the event channel
  exists but the event loop hasn't started, so animation ticks never fire.
- Auto-connect on the first `Event::Tick` — introduces one-tick latency and requires a
  "pending startup action" flag, more state to manage.

### Decision 3 — Auto-connect reuses the existing `ConnectionEvent` channel

**What:** Startup auto-connect spawns a tokio task that calls
`connection_manager.connect(&connection_config)` then
`connection_manager.list_database_objects(…)` and sends `ConnectionEvent::Success` or
`ConnectionEvent::Failed` on `connection_events_tx`. `tick()` processes these exactly
as it does for manually initiated connections (app/mod.rs:325–410).

**Why:** Zero duplication: the connecting animation, timeout, success/failure toasts,
table-list build, and `app_state_db.set_active_connection()` recording all happen
without any new code paths. The `--database` / `--table` follow-up is handled as a
post-success hook inside the same `tick()` branch.

**Alternatives rejected:**
- Await the connection inline in `run()` — blocks the draw loop; no animation.
- A dedicated "startup event" variant — adds a new enum arm to process for no gain.

### Decision 4 — `--database` and `--table` consumed in `tick()` on `ConnectionEvent::Success`

**What:** `App` stores `startup_database: Option<String>` and `startup_table: Option<String>`
(set from `StartupOptions` in `new()`). In the `ConnectionEvent::Success` arm in `tick()`,
after the normal success processing, consume both if they are `Some`:

1. `startup_database`: scan `state.db.database_objects` for a matching schema/database
   name and call `state.db.selected_schema = Some(name)`. If not found, toast and clear.
2. `startup_table`: scan `state.db.tables` for the name, set `state.ui` selection to
   that index (using the same selection helpers used by the connections pane), then call
   `state.open_table_for_viewing().await`. If not found, toast and clear.

Both fields are set to `None` after consumption so they do not re-fire on any subsequent
`ConnectionEvent` (e.g., reconnection).

**Why:** The table list only exists after `ConnectionEvent::Success` populates it.
Consuming at that point avoids any ordering dependency or extra synchronisation.

**Alternatives rejected:**
- Consuming immediately after the `spawn` call — tables are not yet loaded.
- A separate `StartupEvent` variant — extra complexity, no benefit.

### Decision 5 — Connection string detection via `detect_database_type`

**What:** If `startup_options.connection` contains `://`, treat it as a connection
string. Call `AdapterFactory::create_connection_from_string(value, "startup".into())`
to produce a `ConnectionConfig`. Add it to `state.db.connections.connections` as a
transient entry (not saved to disk). Proceed with normal spawn path. If parsing fails,
toast "Invalid connection string: …" and continue.

**Why:** `AdapterFactory::detect_database_type` (factory.rs:36) already uses a
scheme-prefix check; `create_connection_from_string` (factory.rs:65) wraps both
detection and parsing. No new parsing logic required. Not saving the entry avoids
polluting the user's saved connection list with ad-hoc URLs.

**Alternatives rejected:**
- Always attempt name lookup, fall back to string parse — ambiguous if a saved
  connection name happens to contain `://`; explicit sentinel is cleaner.

### Decision 6 — `is_write_sql` leading-keyword heuristic, inline in `src/app/mod.rs`

**What:** A small `pub(crate) fn is_write_sql(query: &str) -> bool` that trims the
query, uppercases the first word, and checks membership in
`["INSERT","UPDATE","DELETE","DROP","ALTER","TRUNCATE","CREATE"]`. Place it in
`src/app/mod.rs` (or a sibling utility, but colocated is fine at this size). Read
queries (SELECT, EXPLAIN, SHOW, WITH, …) pass through.

**Why:** A leading-keyword heuristic is documented as intentionally imperfect (a
`WITH … DELETE` CTE would bypass it) but covers the realistic common cases without
requiring a SQL parser dependency. The limitation is noted in a code comment. This
mirrors the approach used by many embedded read-only modes (Beekeeper Studio,
TablePlus).

**Alternatives rejected:**
- Full SQL parse (sqlparser-rs) — heavyweight dependency for a guard that the user opted
  into explicitly with `--read-only`.
- Blocking all SQL execution in read-only mode — prevents SELECT, making the editor
  useless.

### Decision 7 — `read_only` stored on `App`, gates placed in handlers

**What:** Add `pub read_only: bool` to `App`, set from `StartupOptions.read_only` in
`new()`. The three table-viewer mutation call-sites in handlers check `app.read_only`
before calling `app.state.update_table_cell` / `delete_table_row` / `set_cell_to_null`.
The two query-execution call-sites in `handlers/query_editor.rs` check `app.read_only`
and call `is_write_sql` before calling `execute_query_at_cursor`.

**Why:** The flag is an app-lifecycle concern, not a per-request state concern. Placing
gates in the handlers (the outermost layer before state mutation) keeps the `AppState`
methods clean and un-guarded; any future caller (tests, macros) that bypasses handlers
would not be inadvertently blocked. The handler layer is already the canonical place for
access control (e.g., the "no connection selected" guard lives there too).

**Alternatives rejected:**
- Store `read_only` on `AppState` and guard inside state methods — couples state
  methods to runtime policy; harder to test mutations independently.
- Re-check argv in the handler — argv is not available at handler call time.

## Edge Cases & Failure Modes

- **`--table` without `--connection`**: `startup_table` is non-None but
  `startup_connection` is None. No auto-connect fires. Emit a startup toast
  "–-table requires --connection; ignored" once before entering the event loop. Same
  for `--database` without `--connection`.

- **`--connection` resolves but connect fails**: `ConnectionEvent::Failed` fires.
  Normal failure toast shown. `startup_table` and `startup_database` are cleared (set to
  None) in the `ConnectionEvent::Failed` arm so they do not dangle.

- **`--table` not found after connect**: `state.db.tables` is searched linearly; if
  absent, toast "Table '<name>' not found; use the tables pane to select one." and
  clear.

- **`--database` not applicable to the engine**: not all engines support named schemas.
  If `database_objects` is None or the name is absent, toast and clear.

- **Multiple `ConnectionEvent::Success` events**: `startup_table` and
  `startup_database` are consumed (set to None) on first success; subsequent events
  (e.g., reconnect) do not re-open the table.

- **Read-only + `WITH … DELETE`**: The leading-keyword heuristic mis-classifies a
  `WITH cte AS (…) DELETE …` as a read query. Documented limitation; acceptable for
  the explicit opt-in use case.

- **Read-only mode and the confirmation modal**: The delete-row and set-NULL gates in
  `overlays.rs` short-circuit before the confirmation modal is confirmed. The modal
  should ideally never open. Post-c0004 the triggering key in `query_results.rs` also
  sets `table_viewer_state.delete_confirmation`; that path must also be gated to prevent
  the modal from appearing. Add an early-return guard at the confirmation-trigger site
  in addition to the confirmation-execution site.

- **`connection` CLI flag doc string** says "Connection string to connect immediately"
  (cli.rs:22–23); this change aligns implementation with documentation. Name-based
  lookup is the primary path; connection-string is the secondary path distinguished by
  the `://` sentinel.

## Migration / Cutover

**Callsites to update:**

1. `src/main.rs:37` — change `App::new(config)` to `App::new(config, StartupOptions { … }).await`.
   Build `StartupOptions` from `cli.connection.clone()`, `cli.database.clone()`,
   `cli.table.clone()`, `cli.read_only`.

2. `src/app/mod.rs:71` — change `pub async fn new(config: Config) -> Result<Self>` to
   `pub async fn new(config: Config, options: StartupOptions) -> Result<Self>`. Add
   `startup_options`, `startup_table`, `startup_database`, `read_only` fields to `App`
   struct. Populate them in `new()`.

3. `src/app/mod.rs:100` (`App::run`) — add startup auto-connect block after
   `initialize_app_db()`.

4. `src/app/mod.rs:290` (`App::tick`) — add `--database` / `--table` follow-up in the
   `ConnectionEvent::Success` arm. Clear `startup_database`/`startup_table` in both
   `Success` and `Failed` arms.

5. `src/app/handlers/query_results.rs:322` — add read-only guard before
   `update_table_cell`.

6. `src/app/handlers/overlays.rs:172` (`handle_table_delete_confirmation`) — add
   read-only guard at entry (before modal confirmation AND before execution).

7. `src/app/handlers/overlays.rs:199` (`handle_set_null_confirmation`) — add read-only
   guard at entry.

8. `src/app/handlers/query_editor.rs:23` (`KeyCode::Char('E')` arm) — add read-only +
   write-SQL guard.

9. `src/app/handlers/query_editor.rs:31` (`KeyCode::Enter + CTRL` arm) — same guard.

**Nothing to delete**: The existing `Cli` fields `connection`, `database`, `table`,
`read_only` stay; they are now consumed rather than ignored.

## Verification

Run with a saved connection named `<name>` and a known table `<t>`:

```
lazytables --connection <name> --table <t>
```
→ The app starts, displays the connecting animation, then immediately opens table `<t>`
in the table viewer without user interaction.

```
lazytables --read-only
```
→ Connect normally, navigate to a table, press `e` to enter edit mode, commit (Enter
in edit mode) → toast "read-only mode: mutations are disabled"; cell unchanged.
→ Navigate to query editor, type `DELETE FROM foo`, press `E` → toast; no query runs.
→ Type `SELECT 1`, press `E` → executes normally.

```
lazytables
```
→ Identical behaviour to the pre-change binary (no regressions).
