# Design — DatabaseState becomes pure data

## Context

`src/state/database.rs` defines `DatabaseState`, a struct intended to hold database-specific
state separate from UI concerns (per its own doc comment, line 14). Despite this intent, it
imports `ui/components` types and passes `&mut TableViewerState` through its methods, creating
a bidirectional dependency:

```
state/database.rs  →  ui/components/table_viewer (CellUpdate, ColumnInfo,
                                                   DeleteConfirmation, SetNullConfirmation,
                                                   TableViewerState)
```

The `AppState` struct (app/state.rs) already legitimately bridges the state and UI layers: it
holds both `db: DatabaseState` and `table_viewer_state: TableViewerState` as fields and knows
about both. The thin wrappers it provides (e.g. `AppState::load_table_data` at line 1745) exist
precisely to mediate between layers. The issue is that these wrappers currently pass the UI
struct into `DatabaseState` instead of keeping the cross-cutting work at the `AppState` level.

After c0002 removes dead subsystems and after c0003 removes `list_tables` from all adapters and
c0004 replaces Postgres-only mutation methods with engine-agnostic `ConnectionManager`
passthroughs, the remaining violations in `DatabaseState` are:

1. `load_table_data` / `load_postgres_table_data` — fetch + write into `TableViewerState`.
2. `load_table_metadata` — fetch + write into `self.current_table_metadata`.
3. `try_connect_to_database` — divergent per-engine paths (PG via `ConnectionManager`; MySQL/
   SQLite via ad-hoc direct `list_tables()`).
4. The six mutation methods that survive until c0004 lands (they reference `CellUpdate` etc.).

This proposal resolves (1–3) and cleans up any remnant of (4).

## Goals / Non-Goals

**Goals:**
- `DatabaseState` has zero `ui::components` imports; its `impl` block contains only `new()`.
- All DB-execution logic is moved to `AppState`-layer methods (which legitimately straddle both
  layers) or a thin `src/database/service.rs` free-fn module.
- `try_connect_to_database` is unified: all three engines go through `ConnectionManager` after
  c0003 removes `list_tables`. Only the `database_objects`/`tables` field writes remain on
  `DatabaseState` data; the query logic lives in the `AppState` wrapper.
- `table_load_error` is still populated on failure (no regression in error surfacing).
- No user-visible behavior change.

**Non-Goals:**
- Changing the public fields of `DatabaseState` (they stay as-is).
- Changing `ConnectionManager` or the `ManagedConnection` trait (c0003's responsibility).
- Altering `TableViewerState` layout or rendering (c0008's responsibility).
- Rewriting the actual query logic — only the _location_ of the call changes.

## Decisions

### Decision 1: Introduce `LoadedTableData` return type instead of passing `&mut TableViewerState`

**What:** Define a plain-data struct `LoadedTableData` with fields `columns: Vec<TableColumn>`,
`rows: Vec<Vec<String>>`, `total_rows: usize`, `metadata: Option<TableMetadata>`. The service
fn returns this; the `AppState` wrapper unpacks it into `TableViewerState`.

**Why:** This is the minimal change that eliminates the UI import from `DatabaseState`. Returning
plain data from the adapter-layer function keeps it independently testable (no ratatui types
in scope). `TableColumn` and `TableMetadata` are already defined in `src/database/mod.rs` with
no UI dependencies.

**Alternatives rejected:**
- Keeping `&mut TableViewerState` but in a new location (`app/state/table_viewer.rs`): still
  puts a UI type dependency in a method called directly from the database layer — violates the
  same boundary, just in a different file.
- Using a generic callback / closure to populate the viewer: adds unnecessary abstraction and
  makes the call-site harder to read.

### Decision 2: Move execution logic to `AppState` wrappers, not a new `service` module

**What:** The bodies of `load_table_data`, `load_postgres_table_data`, and `load_table_metadata`
move directly into the `AppState::load_table_data` / `AppState::load_table_metadata` methods
(or into `app/state/table_viewer.rs` after c0005 splits `state.rs`). They call `ConnectionManager`
directly, which is already a field of `AppState`.

**Why:** `AppState` already owns `connection_manager` and `db: DatabaseState`. It is the natural
orchestration point. Creating a `src/database/service.rs` would be a second layer between
`AppState` and `ConnectionManager` that exists only to avoid a transient refactoring issue.
Inline in the wrapper is the boring, direct choice.

**Alternatives rejected:**
- `src/database/service.rs` free fns: adds a module with no clear long-term purpose; the code
  ends up in `app/state/table_viewer.rs` anyway after c0005.
- Keeping the logic on `DatabaseState` but returning `LoadedTableData`: the struct stays in
  `state/database.rs` without a UI import, which is acceptable, but `DatabaseState` becomes an
  execution service rather than a data bag — conflicts with the stated goal of making it pure
  data.

### Decision 3: Unify `try_connect_to_database` on `ConnectionManager.list_database_objects`

**What:** After c0003, `list_tables` is removed from all adapters. The MySQL and SQLite branches
of `try_connect_to_database` (state/database.rs lines 582–648) that directly instantiate
`MySqlConnection` / `SqliteConnection` and call `list_tables()` must be deleted. The unified
path is:
1. `connection_manager.connect(config)` — establishes a pooled connection for all engines.
2. `connection_manager.list_database_objects(config.id)` — returns `DatabaseObjectList`.
3. Write `database_objects` and `tables` fields on `DatabaseState`.

The unified method body moves to `AppState` (into `connection_lifecycle.rs` after c0005).

**Why:** `ConnectionManager.connect()` already handles all three engine types (lines 66–93
of `connection_manager.rs`). After c0003 removes `list_tables`, there is no remaining reason
for the per-engine split in `try_connect_to_database`. The unified path is shorter and easier
to maintain.

**Alternatives rejected:**
- Keep the per-engine split but call `list_database_objects` instead of `list_tables`: results
  in three identical code paths; dead generality.
- Move the method to `DatabaseState` but hide the engine dispatch inside `ConnectionManager`:
  `DatabaseState` would still be making remote calls — it remains an execution service, not
  pure data.

### Decision 4: `table_load_error` population responsibility stays at the `AppState` wrapper

**What:** `DatabaseState.table_load_error: Option<String>` is a data field. After this change,
errors from `load_table_data` propagate as `Err(String)` up to the `AppState` wrapper, which
sets `self.db.table_load_error = Some(e)` on failure — exactly as `AppState::load_table_data`
currently does via the `tab.error` field (app/state.rs line 1709).

**Why:** `table_load_error` is displayed in the details pane by the UI layer. It must remain on
`DatabaseState` so the renderer can read it. Populating it at the wrapper level is correct:
`AppState` has full access to `self.db` and to the `Err` value.

**Alternatives rejected:**
- Setting `table_load_error` inside the service fn: would require passing `&mut DatabaseState`
  into the service fn, reintroducing a dependency from the execution layer into the data struct.

## Edge Cases & Failure Modes

- **`table_load_error` not populated on failure:** `AppState::load_table_data` must explicitly
  write `self.db.table_load_error = Some(e.clone())` on the error branch, and clear it to `None`
  on the success branch. The verification step exercises this path.
- **MySQL/SQLite connect path post-c0003:** the `list_tables` calls on lines 591 and 626 will
  be compile errors once c0003 removes that method. This proposal's unified path through
  `ConnectionManager` resolves them. The implementer must verify `ManagedConnection::list_database_objects`
  is implemented for all three engines (confirmed in connection_manager.rs line 31 — it delegates
  to each adapter's `list_database_objects` implementation).
- **`execute_query` method:** `DatabaseState` also has `execute_query` / `execute_postgres_query`
  (lines 471–525) which have the same Postgres-only guard. These are NOT in scope for this
  proposal (they do not import UI types and are not part of the listed methods to move);
  they belong to a future DB-parity cleanup pass. Leave them in place.
- **Composite-PK / NULL handling in mutation methods:** fully handled by c0004's `dml.rs`.
  This proposal only ensures the remnants (import block, old method bodies) are absent from
  `DatabaseState` after c0004 lands.
- **`current_table_metadata` still a `DatabaseState` field:** it remains as a data field.
  `AppState::load_table_metadata` writes to it after fetching from `ConnectionManager`. No
  change to where the metadata is stored — only where the fetch call lives.
- **Thread safety:** `ConnectionManager` is `Arc<Mutex<…>>` internally and implements `Send +
  Sync` (connection_manager.rs lines 234–236). Moving calls up to `AppState` does not change
  the concurrency model.

## Migration / Cutover

### What to delete from `state/database.rs`

| Lines | Symbol | Action |
|-------|--------|--------|
| 8–11 | `use crate::ui::components::{…}` | Delete |
| 51–105 | `pub async fn load_table_data(…, table_viewer_state: &mut TableViewerState, …)` | Delete |
| 107–195 | `async fn load_postgres_table_data(…, table_viewer_state: &mut TableViewerState, …)` | Delete |
| 198–242 | `pub async fn load_table_metadata(…)` | Delete |
| 245–317 | `pub async fn update_table_cell(update: CellUpdate, …)` | Delete (replaced by c0004) |
| 279–317 | `async fn update_postgres_cell(…, update: CellUpdate, …)` | Delete (replaced by c0004) |
| 320–351 | `pub async fn delete_table_row(confirmation: DeleteConfirmation, …)` | Delete (replaced by c0004) |
| 354–391 | `pub async fn set_cell_to_null(confirmation: SetNullConfirmation, …)` | Delete (replaced by c0004) |
| 394–429 | `async fn delete_postgres_row(…, confirmation: DeleteConfirmation, …)` | Delete (replaced by c0004) |
| 432–468 | `async fn set_postgres_cell_to_null(…, confirmation: SetNullConfirmation, …)` | Delete (replaced by c0004) |
| 528–655 | `pub async fn try_connect_to_database(…, connection_manager: …)` | Delete (unified body moves to AppState) |

After deletions, `DatabaseState`'s `impl` block contains only `new()` (lines 35–48).

### What to add / rewrite in `app/state.rs` (or `app/state/table_viewer.rs` after c0005)

**`AppState::load_table_data`** (currently lines 1745–1754, thin wrapper):
Expand to contain the full logic previously in `load_postgres_table_data`, calling
`ConnectionManager` directly. All engines now share the same path:
```
connection_manager.connect(config).await
connection_manager.get_table_columns(id, table).await  → Vec<TableColumn>
connection_manager.execute_raw_query(id, count_sql).await  → total_rows
connection_manager.get_table_data(id, table, limit, offset).await  → rows
connection_manager.get_table_metadata(id, table).await  → Option<TableMetadata>
```
Then write results into `self.table_viewer_state.tabs[tab_idx]`, converting `TableColumn` to
`ColumnInfo` exactly as in the current `load_postgres_table_data` (lines 165–190).
On `Err`: set `self.db.table_load_error = Some(e.clone())` and `tab.error = Some(…)`.
On `Ok`: clear `self.db.table_load_error = None`.

**`AppState::load_table_metadata`** (currently lines 1757–1764, thin wrapper):
Expand to call `connection_manager.connect(config)` then `connection_manager.get_table_metadata(id, table)`,
then write `self.db.current_table_metadata = Some(metadata)`. All engines; drop the Postgres-only guard.

**`AppState::try_connect_to_database`** (currently lines 655–662, thin wrapper):
Expand to call `connection_manager.connect(config)` then `connection_manager.list_database_objects(id)`.
Write `self.db.database_objects = Some(objects.clone())` and build `self.db.tables` from the
object list using the qualified-name logic (currently in state/database.rs lines 551–578, all
engine-independent). Remove the per-engine `match` entirely.

**`AppState::update_table_cell`** / **`delete_table_row`** / **`set_cell_to_null`**
(lines 1802–1845): After c0004 these call `ConnectionManager` passthroughs directly. Ensure
they no longer delegate to `self.db.*`. The `AppState` wrappers extract primitives from
`CellUpdate`/`DeleteConfirmation`/`SetNullConfirmation` and pass them to the c0004-introduced
`connection_manager.update_cell()` / `delete_row()` methods.

### Callsites that need no change

- `AppState::open_table_for_viewing` (app/state.rs ~1706) — calls `self.load_table_data(tab_idx)`
  and `self.load_table_metadata(&table_name)`; these public `AppState` methods keep their
  signatures.
- Handlers that call `AppState::update_table_cell` / `delete_table_row` / `set_cell_to_null`
  — unchanged public signatures.
- UI rendering that reads `DatabaseState` fields (`tables`, `database_objects`, etc.) — struct
  fields unchanged.

## Verification

1. **Build clean:** `cargo build` produces no errors or warnings after the deletions.
2. **Clippy clean:** `cargo clippy --all-targets -- -D warnings` (no dead-code warnings from
   removed imports or unreferenced types on `DatabaseState`).
3. **Tests green:** existing database adapter tests and UIState tests all pass.
4. **Behavioral check (manual):**
   - Launch app → select a saved connection → connect: connecting animation shows; after
     completion the tables pane lists objects from `database_objects`.
   - Open a table: data rows and column headers render correctly; schema tab shows metadata.
   - Trigger a load error (open a non-existent table name via a forced call or a test connection
     whose table disappears): the details pane shows the error from `table_load_error` and a
     toast fires — confirming `table_load_error` is still populated on failure.
