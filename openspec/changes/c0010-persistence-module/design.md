# Design: Move app-state & query-history stores out of the adapter layer

## Context

LazyTables persists two orthogonal concerns in local SQLite files owned by the
`src/database/` adapter layer:

1. **AppStateDb** (`database/app_state.rs`) — tracks the active connection,
   connection sessions, and SQL file activity. Stored at
   `~/.lazytables/app_state.db` via `AppStateDb::database_path()` which calls
   `Config::data_dir().join("app_state.db")` (app_state.rs:60–62). The struct
   holds a single `Option<SqlitePool>`. Initialised via the static async
   `AppStateDb::initialize()` (app_state.rs:37–57) which creates the directory,
   opens a pool, and runs `create_schema()`. Called from
   `AppState::initialize_app_db()` (app/state.rs:111–117) on every startup. The
   result is stored on `AppState.app_state_db` (state.rs:50).

2. **QueryHistoryManager** (`database/query_history.rs`) — stores executed queries
   with timing, success flag, and error message. Stored at
   `~/.lazytables/query_history.db` via `dirs::home_dir().join(".lazytables")
   .join("query_history.db")` (query_history.rs:38–41). The struct holds
   `Option<SqlitePool>` and `db_path: PathBuf`. Initialised via the non-static
   `QueryHistoryManager::initialize(&mut self)` (query_history.rs:50–102).
   Currently has no external callers; c0011 will wire it.

Both modules are declared and re-exported from `database/mod.rs` (lines 4, 11, 32,
35) alongside `Connection`, `ConnectionManager`, `AdapterFactory`, and user-DB type
definitions — a conceptual mismatch.

The `Config` helper `app_state_db_path()` (config/mod.rs:127–129) returns the same
path as `AppStateDb::database_path()` but is never called by AppStateDb, creating
two independent path computations. `QueryHistoryManager::new()` hardcodes
`dirs::home_dir()` directly, bypassing `Config::data_dir()` entirely.

This change extracts both modules into a dedicated `src/persistence/` crate module,
consolidates them onto a single SQLite pool/file, and canonicalises the path through
`Config`.

## Goals / Non-Goals

**Goals:**
- Remove `database/app_state.rs` and `database/query_history.rs` from the adapter
  layer; they belong in a new `persistence/` module.
- Provide a single `persistence::initialize()` entry point that opens one
  `SqlitePool` and returns both managers fully initialised.
- Use one canonical path via `Config::persistence_db_path()`.
- Preserve all public method signatures on `AppStateDb` and `QueryHistoryManager`
  unchanged so callers (current and future, e.g. c0011) need only update import paths.
- Move the `#[cfg(test)]` block from `database/query_history.rs:346–437` to
  `persistence/query_history.rs`.

**Non-Goals:**
- Migrating existing user data from `app_state.db` / `query_history.db` to the new
  `lazytables.db` — the stance is fresh-DB-on-first-run (no migration SQL needed;
  connection metadata is re-established on next connect).
- Changing any public method signatures on `AppStateDb` or `QueryHistoryManager`.
- Adding new persistence functionality (that belongs in c0011).
- Touching the user-facing database adapters (`postgres.rs`, `mysql.rs`,
  `sqlite.rs`) — this is a module organisation change only.

## Decisions

### Decision 1: Create `src/persistence/` as a first-class top-level module

**What:** Add `src/persistence/mod.rs`, `src/persistence/app_state.rs`, and
`src/persistence/query_history.rs`. Declare with `pub mod persistence;` in
`src/lib.rs` after the existing `pub mod database;` line (lib.rs:9).

**Why:** A dedicated `persistence/` module at the crate root signals that these
stores are application-level concerns, parallel to `config/` and `state/`, not
subordinate to the user-DB adapter layer. It matches the naming convention of
adjacent top-level modules and will be the natural home for any future persistence
concerns (e.g., settings, cached schema).

**Alternatives rejected:**
- *Keep in `database/` but rename to `database/internal/`* — still wrong layer;
  the adapter layer would continue to own application-layer state.
- *Move to `state/persistence/`* — state/ holds volatile runtime state; durable
  SQLite stores are different enough in kind to merit their own root module.

### Decision 2: Consolidate to one SQLite pool and file (`lazytables.db`)

**What:** `persistence/mod.rs` exposes:
```rust
pub async fn initialize() -> Result<(AppStateDb, QueryHistoryManager)>
```
This function:
1. Resolves the path via `Config::persistence_db_path()` (new helper, see Decision 3).
2. Creates the `~/.lazytables/` directory with `fs::create_dir_all`.
3. Opens a single `SqlitePool` using `SqliteConnectOptions::new().filename(path).create_if_missing(true)`.
4. Constructs both managers via new internal constructors `AppStateDb::from_pool(pool.clone())` and `QueryHistoryManager::from_pool(pool)` (or equivalent).
5. Calls `create_schema()` on each manager (unchanged internal methods).
6. Returns the pair.

`AppState::initialize_app_db()` (app/state.rs:111–117) is updated to call
`crate::persistence::initialize().await` and assign both results.

**Why:** Two open SQLite pools for the same directory is unnecessary overhead.
SQLite handles multiple tables in one file without contention. A single initializer
call at startup is cleaner than two independent async init paths.

**Alternatives rejected:**
- *Keep two pools, just move the files* — this is the documented fallback (see
  below); preferred only if the pool-sharing refactor proves invasive.
- *Merge both managers into one struct* — would couple two independent concerns
  and change the public API, contradicting Non-Goals.

**Fallback:** If threading the shared pool through both managers requires too many
structural changes (e.g., the `db_path` field on `QueryHistoryManager` is used in
tests in a way that resists a `from_pool` constructor), keep two separate SQLite
files (`app_state.db` and `query_history.db`) but both opened and managed within
`src/persistence/`. The decoupling goal — removing these files from `database/` —
is still fully satisfied. Document the fallback in code comments. The
`Config::app_state_db_path()` helper stays in this case; add a parallel
`Config::query_history_db_path()` returning `Config::data_dir().join("query_history.db")`.

### Decision 3: Add `Config::persistence_db_path()` as the canonical path

**What:** Add to `config/mod.rs`:
```rust
pub fn persistence_db_path() -> PathBuf {
    Self::data_dir().join("lazytables.db")
}
```
Under the single-pool plan, remove `Config::app_state_db_path()` (config/mod.rs:127–129) since no code calls it from outside `AppStateDb::database_path()` (which itself moves to `persistence/`).

Under the fallback plan, keep `app_state_db_path()` and add `query_history_db_path()`.

**Why:** `QueryHistoryManager::new()` currently calls `dirs::home_dir()` directly
(query_history.rs:34), bypassing `Config::data_dir()` (which also calls
`dirs::home_dir()` internally, config/mod.rs:101–103). A canonical Config helper
eliminates the duplicate resolution and ensures all paths are consistent if
`data_dir()` is ever changed.

**Alternatives rejected:**
- *Use `AppStateDb::database_path()` as the shared source* — AppStateDb shouldn't
  be the authority on where QueryHistoryManager stores its data; Config is.
- *Hardcode `~/.lazytables/lazytables.db` in `persistence/mod.rs`* — bypasses
  Config, which already centralises path logic.

### Decision 4: No data migration from old files to new

**What:** On first run after the upgrade, the new `lazytables.db` will not exist;
`initialize()` creates it fresh via `create_if_missing(true)`. Old `app_state.db`
and `query_history.db` (if present) are silently ignored.

**Why:** `AppStateDb` already takes a fresh-DB-on-first-run stance — `AppStateDb::initialize()` (app_state.rs:37–57) creates the DB and schema if missing with no migration. The stored data (last active connection, SQL file activity) is ephemeral convenience — losing it on upgrade has no functional consequence. Query history is currently wired to nothing (c0011), so no user data exists there yet.

**Alternatives rejected:**
- *Copy old tables into the new file at first run* — significant complexity for
  ephemeral data; out of scope for a refactoring change.

## Edge Cases & Failure Modes

- **`AppStateDb::database_path()` used by existing callers:** only used internally in
  `AppStateDb::initialize()` (app_state.rs:38). After the move, `database_path()`
  is replaced by `Config::persistence_db_path()` called in `persistence::initialize()`.
  No external caller references `AppStateDb::database_path()` directly (confirmed by
  search — only app_state.rs:38 uses it).

- **`Config::app_state_db_path()` is currently unused by callers** (config/mod.rs:127–129
  — confirmed by search: no call site in `src/`). Safe to remove once consolidation
  lands.

- **Tests in `database/query_history.rs:346–437`** construct `QueryHistoryManager`
  via struct literal (`QueryHistoryManager { pool: None, db_path: ... }`, lines 356–359,
  389–392) rather than through `new()`. These tests survive the move unchanged —
  the struct literal bypasses path resolution and uses `tempdir()`. After moving, the
  only change needed is updating the `use crate::database::DatabaseType;` import
  (query_history.rs:4) to `crate::database::DatabaseType` (DatabaseType stays in
  `database/`; the persistence module imports it).

- **`AppState` holds two fields after c0011:** when c0011 adds `query_history:
  QueryHistoryManager` to `AppState`, both fields are initialised in one call to
  `persistence::initialize()`. The `initialize_app_db()` method (state.rs:111–117)
  must set both fields or be replaced by a unified init call.

- **Double-init guard:** `SqlitePool` creation is idempotent via `create_if_missing`.
  A second call to `persistence::initialize()` (if accidentally called twice) opens
  a second pool to the same file — harmless but wasteful. Callers must ensure
  `initialize()` is called exactly once at startup, as today.

- **`state_new/` (dead code, removed by c0002):** `state_new/mod.rs` also imports
  `AppStateDb` from `crate::database` (state_new/mod.rs:7). Since c0002 deletes
  `state_new/` entirely, the import repoint can ignore it.

## Migration / Cutover

### Files to create
- `src/persistence/mod.rs` — `pub mod app_state; pub mod query_history;`, re-exports
  `AppStateDb`, `ActiveConnectionState`, `SqlFileActivity`, `ConnectionSession`,
  `QueryHistoryManager`, `QueryHistoryEntry`; the `initialize()` async fn.
- `src/persistence/app_state.rs` — moved verbatim from `src/database/app_state.rs`;
  update `AppStateDb::database_path()` to delegate to `Config::persistence_db_path()`
  or remove it if `initialize()` owns path resolution.
- `src/persistence/query_history.rs` — moved verbatim from
  `src/database/query_history.rs`; update `use crate::database::DatabaseType` import
  to stay `crate::database::DatabaseType` (no change needed — the crate path is
  unchanged).

### Lines to change in existing files

**`src/lib.rs`** — add after line 9 (`pub mod database;`):
```rust
pub mod persistence;
```

**`src/database/mod.rs`** — remove:
- Line 4: `pub mod app_state;`
- Line 11: `pub mod query_history;`
- Line 32: `pub use query_history::{QueryHistoryEntry, QueryHistoryManager};`
- Line 35: `pub use app_state::{ActiveConnectionState, AppStateDb, ConnectionSession, SqlFileActivity};`

**`src/app/state.rs`** — update line 5:
```rust
// Before:
use crate::database::{AppStateDb, ConnectionConfig, ConnectionManager, ConnectionStatus};
// After:
use crate::database::{ConnectionConfig, ConnectionManager, ConnectionStatus};
use crate::persistence::AppStateDb;
```
Update `initialize_app_db()` (state.rs:111–117) to call `crate::persistence::initialize()`:
```rust
pub async fn initialize_app_db(&mut self) -> Result<(), String> {
    match crate::persistence::initialize().await {
        Ok((app_db, _qhm)) => {
            self.app_state_db = app_db;
            // _qhm stored when c0011 adds query_history field
            Ok(())
        }
        Err(e) => Err(format!("Failed to initialize application database: {}", e)),
    }
}
```

**`src/config/mod.rs`** — add after `app_state_db_path()` (line 129):
```rust
/// Get consolidated persistence database path
pub fn persistence_db_path() -> PathBuf {
    Self::data_dir().join("lazytables.db")
}
```
Remove `app_state_db_path()` (lines 126–129) once all callers are migrated.

### What to delete
- `src/database/app_state.rs` (the file; replaced by `src/persistence/app_state.rs`)
- `src/database/query_history.rs` (the file; replaced by `src/persistence/query_history.rs`)

## Verification

1. `cargo build` compiles cleanly with no unresolved imports in `app/state.rs`,
   `database/mod.rs`, or `lib.rs`.
2. `cargo test persistence::` — both moved tests (`test_query_history_creation`,
   `test_database_type_filtering`) pass in their new location.
3. Run the app: connect to a saved connection, open an SQL file, quit, restart —
   the previously active connection is pre-highlighted and the SQL file appears in
   the recent list (proves `AppStateDb` path and write/read are intact).
4. Verify `~/.lazytables/lazytables.db` is created on first run and `app_state.db`
   / `query_history.db` are no longer created (or, under the fallback, both files
   appear under `~/.lazytables/` as before but opened through `persistence/`).
