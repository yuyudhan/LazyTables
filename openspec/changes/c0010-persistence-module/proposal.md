# Move app-state & query-history stores out of the adapter layer

## Why

`src/database/app_state.rs` (`AppStateDb`) and `src/database/query_history.rs`
(`QueryHistoryManager`) persist application and session state in two separate local
SQLite files, but both live inside the user-database adapter layer
(`src/database/mod.rs` lines 4, 11). This is a conceptual leak: the adapter layer
should mediate access to external *user* databases; it has no business owning the
internal housekeeping stores that record active connections, SQL-file activity, and
query history.

Concretely:
- `database/mod.rs` re-exports all four `AppStateDb` types (line 35) and both
  `QueryHistoryManager` types (line 32) alongside adapter abstractions like
  `Connection`, `ConnectionManager`, and `AdapterFactory`.
- `AppStateDb::database_path()` (app_state.rs:60–62) calls `Config::data_dir()`
  directly, duplicating `Config::app_state_db_path()` (config/mod.rs:127–129).
- `QueryHistoryManager::new()` (query_history.rs:33–47) uses `dirs::home_dir()`
  directly instead of the canonical `Config::data_dir()`, creating a second path
  resolution strategy.
- Both managers open an independent `SqlitePool`, giving the process two open
  handles to local SQLite files that could share one.

`AppStateDb` is wired to `app/state.rs` (imported at line 5, field at line 50,
initialised at line 112, called at lines 635, 669, 984). `QueryHistoryManager` has
no external callers yet outside `database/mod.rs` (wired in c0011).

## What Changes

- Create `src/persistence/` with `mod.rs`, `app_state.rs` (receives `AppStateDb`
  and its supporting types `ActiveConnectionState`, `SqlFileActivity`,
  `ConnectionSession`), and `query_history.rs` (receives `QueryHistoryManager` and
  `QueryHistoryEntry`).
- Add `pub mod persistence;` to `src/lib.rs` (after the existing `pub mod database;`
  at line 9).
- Remove `pub mod app_state;` (line 4), `pub mod query_history;` (line 11), the
  app-state re-export (line 35), and the query-history re-export (line 32) from
  `src/database/mod.rs`.
- Repoint imports in `src/app/state.rs` (line 5) from `crate::database::AppStateDb`
  to `crate::persistence::AppStateDb`. (`QueryHistoryManager` has no other callers;
  c0011 will import from `crate::persistence`.)
- Consolidate to a single SQLite pool/file `~/.lazytables/lazytables.db` with both
  table sets, opened once through a new `persistence::initialize()` async function;
  both managers' public method signatures are unchanged. Add
  `Config::persistence_db_path()` returning `Config::data_dir().join("lazytables.db")`
  as the canonical path; retire `Config::app_state_db_path()` (config/mod.rs:127–129)
  if consolidation lands.
- Move the `#[cfg(test)]` block from `database/query_history.rs` (lines 346–437)
  into `persistence/query_history.rs`.

**FALLBACK** (see design.md): if single-pool consolidation is invasive, keep two
separate SQLite files but both under `src/persistence/`; the decoupling goal is
still fully satisfied.

## Impact

- Affected code:
  - `src/database/app_state.rs` → moved to `src/persistence/app_state.rs`
  - `src/database/query_history.rs` → moved to `src/persistence/query_history.rs`
  - `src/database/mod.rs` — two `pub mod` declarations and two re-export lines removed
  - `src/lib.rs` — one `pub mod persistence;` line added
  - `src/app/state.rs` — import path updated (line 5)
  - `src/config/mod.rs` — `Config::persistence_db_path()` added; `Config::app_state_db_path()` removed if consolidation lands
- Affected specs: none (internal refactor, behavior preserved)
- Risk: low-medium — touches the AppStateDb init path called on every app startup;
  the main risk is the DB file path changing (data loss for existing users unless
  migration is handled; design.md documents the fresh-DB-on-first-run stance)
- Depends on: c0003
