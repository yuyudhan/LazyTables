# Tasks: c0010-persistence-module

## 1. Create the persistence module skeleton and move files

- [ ] 1.1 Create `src/persistence/app_state.rs` by moving the full contents of
      `src/database/app_state.rs` verbatim. Update `AppStateDb::database_path()`
      (currently app_state.rs:60–62, calling `Config::data_dir().join("app_state.db")`)
      to call `Config::persistence_db_path()` instead (added in task 2.3).
      Preserve all imports; no other logic changes.
- [ ] 1.2 Create `src/persistence/query_history.rs` by moving the full contents of
      `src/database/query_history.rs` verbatim. The `use crate::database::DatabaseType`
      import (line 4) is unchanged — `DatabaseType` remains in `crate::database`.
      Move the `#[cfg(test)]` block (lines 346–437) into this new file. Update any
      struct-literal test constructions that hard-wire `db_path` if needed by the
      single-pool refactor; under the fallback the tests are moved unchanged.
- [ ] 1.3 Create `src/persistence/mod.rs` with:
      - `pub mod app_state;` and `pub mod query_history;`
      - Re-exports: `pub use app_state::{ActiveConnectionState, AppStateDb,
        ConnectionSession, SqlFileActivity};` and `pub use query_history::{QueryHistoryEntry,
        QueryHistoryManager};`
      - The `pub async fn initialize() -> crate::core::error::Result<(AppStateDb, QueryHistoryManager)>`
        function that opens one `SqlitePool` to `Config::persistence_db_path()`
        (using `SqliteConnectOptions::new().filename(path).create_if_missing(true)`),
        calls `AppStateDb::from_pool(pool.clone())` and
        `QueryHistoryManager::from_pool(pool)` internal constructors (creating both
        managers), then runs `create_schema()` on each and returns the pair.
      - Under the **fallback**: omit `initialize()` and instead keep
        `AppStateDb::initialize()` and `QueryHistoryManager::initialize()` as-is;
        `mod.rs` re-exports only. Document which path was taken in a comment.

## 2. Repoint imports and update module declarations

- [ ] 2.1 Add `pub mod persistence;` to `src/lib.rs` after the existing
      `pub mod database;` line (lib.rs:9).
- [ ] 2.2 Remove from `src/database/mod.rs`:
      - `pub mod app_state;` (line 4)
      - `pub mod query_history;` (line 11)
      - `pub use query_history::{QueryHistoryEntry, QueryHistoryManager};` (line 32)
      - `pub use app_state::{ActiveConnectionState, AppStateDb, ConnectionSession, SqlFileActivity};` (line 35)
      Delete the now-orphaned source files `src/database/app_state.rs` and
      `src/database/query_history.rs`.
- [ ] 2.3 Add `Config::persistence_db_path()` to `src/config/mod.rs` after the
      existing `app_state_db_path()` helper (line 127–129):
      ```rust
      pub fn persistence_db_path() -> PathBuf {
          Self::data_dir().join("lazytables.db")
      }
      ```
      Under the single-pool plan, remove `Config::app_state_db_path()` (lines 126–129)
      since it has no external callers. Under the fallback, keep it and add a
      parallel `Config::query_history_db_path()` returning
      `Self::data_dir().join("query_history.db")`.
- [ ] 2.4 Update `src/app/state.rs` line 5 import: remove `AppStateDb` from the
      `crate::database` use statement; add `use crate::persistence::AppStateDb;`.
      Update `AppState::initialize_app_db()` (state.rs:111–117) to call
      `crate::persistence::initialize().await` and assign the `AppStateDb` result.
      Leave a `// TODO(c0011): assign QueryHistoryManager when that field is added`
      comment for the `_qhm` return value.

## 3. (Optional) Consolidate to one SQLite pool

- [ ] 3.1 Add `from_pool(pool: SqlitePool) -> Self` constructors to both `AppStateDb`
      and `QueryHistoryManager` (their existing `pool` fields are `Option<SqlitePool>`
      — these constructors set `pool: Some(pool)` and skip the `db_path` field where
      no longer needed). Keep the existing `initialize()` / `new()` methods so
      tests continue to work via struct literals or the old API.
- [ ] 3.2 Implement `persistence::initialize()` as specified in task 1.3: one pool
      open, two `from_pool` calls, two `create_schema()` calls.
- [ ] 3.3 Under the fallback (skip 3.1–3.2 if invasive): document in a comment block
      at the top of `src/persistence/mod.rs` that two pools are used and cite the
      design.md decision. Expose separate `async fn initialize_app_state() ->
      Result<AppStateDb>` and `async fn initialize_query_history() ->
      Result<QueryHistoryManager>` wrappers that call the individual managers'
      existing `initialize()` methods.

## 4. Verify

- [ ] 4.1 `cargo build` clean
- [ ] 4.2 `cargo clippy --all-targets -- -D warnings` clean
- [ ] 4.3 `cargo fmt --check` clean
- [ ] 4.4 `cargo test` green
- [ ] 4.5 Moved tests pass (`cargo test persistence::`) and active-connection +
      sql-file activity persist across app restart: launch the app, connect to a
      saved connection, open an SQL file, quit, relaunch — the previously active
      connection is pre-highlighted and the SQL file appears in the recent list.
      Confirm `~/.lazytables/lazytables.db` exists (single-pool path) or that
      `app_state.db` / `query_history.db` still appear under `~/.lazytables/`
      (fallback path) but are now opened through `src/persistence/`.
