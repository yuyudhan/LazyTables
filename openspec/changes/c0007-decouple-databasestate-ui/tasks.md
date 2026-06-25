# Tasks — DatabaseState becomes pure data

> **Prerequisites:** c0004 has landed (mutation methods replaced by ConnectionManager
> passthroughs), c0005 has landed (AppState split into `app/state/` modules including
> `table_viewer.rs` and `connection_lifecycle.rs`).
>
> All edits below are in `src/`. No spec delta — internal refactor, behavior preserved.

---

## 1. Remove execution methods from DatabaseState

- [ ] 1.1 Delete `pub async fn load_table_data(…, table_viewer_state: &mut TableViewerState, …)`
  from `src/state/database.rs` (lines 51–105).
- [ ] 1.2 Delete `async fn load_postgres_table_data(…, table_viewer_state: &mut TableViewerState, …)`
  from `src/state/database.rs` (lines 107–195). This is the sole caller of the removed UI
  imports for read paths.
- [ ] 1.3 Delete `pub async fn load_table_metadata(…)` from `src/state/database.rs` (lines 198–242).
- [ ] 1.4 Delete the three Postgres-only mutation methods that remain after c0004's migration:
  `update_postgres_cell`, `delete_postgres_row`, `set_postgres_cell_to_null` (lines 279–317,
  394–429, 432–468). Confirm c0004 has already deleted `update_table_cell`, `delete_table_row`,
  `set_cell_to_null` (lines 245–391); if not, delete those too.
- [ ] 1.5 Delete `pub async fn try_connect_to_database(…)` from `src/state/database.rs`
  (lines 528–655) — its body moves to `AppState`-layer in task 3.

---

## 2. Strip ui/components imports from DatabaseState

- [ ] 2.1 Delete the `use crate::ui::components::{…}` import block (state/database.rs lines 8–11)
  that brings in `CellUpdate`, `ColumnInfo`, `DeleteConfirmation`, `SetNullConfirmation`, and
  `TableViewerState`. After tasks 1.1–1.5 are done, none of these types should remain referenced
  in `state/database.rs`.
- [ ] 2.2 Verify via `cargo check` that `src/state/database.rs` compiles with only the remaining
  imports (`Connection`, `ConnectionStorage`, `ConnectionConfig`, `ConnectionStatus`,
  `DatabaseObjectList`, `DatabaseType`, `TableMetadata`). Fix any residual references.

---

## 3. Expand AppState wrappers with the moved execution logic

> Target file: `src/app/state/table_viewer.rs` (created by c0005); fallback `src/app/state.rs`
> if c0005 has not yet split the file.

- [ ] 3.1 Rewrite `AppState::load_table_data` to contain the full query logic previously in
  `load_postgres_table_data`. Call sequence (engine-agnostic via ConnectionManager):
  1. Retrieve `ConnectionConfig` for `self.ui.selected_connection`.
  2. Guard on `ConnectionStatus::Connected`; return descriptive `Err` strings otherwise.
  3. `self.connection_manager.connect(&config).await`
  4. `self.connection_manager.get_table_columns(&config.id, &table_name).await` → `Vec<TableColumn>`
  5. `self.connection_manager.execute_raw_query(&config.id, &count_sql).await` → parse `total_rows`
  6. `self.connection_manager.get_table_data(&config.id, &table_name, limit, offset).await` → rows
  7. `self.connection_manager.get_table_metadata(&config.id, &table_name).await.ok()` → metadata
  8. Write into `self.table_viewer_state.tabs[tab_idx]`: columns (converting `TableColumn` →
     `ColumnInfo`), `primary_key_columns`, `rows`, `total_rows`, `loading = false`, `error = None`,
     `table_metadata`.
  9. On any `Err`: set `self.db.table_load_error = Some(e.clone())` and set `tab.error`.
  10. On `Ok`: clear `self.db.table_load_error = None`.
- [ ] 3.2 Rewrite `AppState::load_table_metadata` to call `ConnectionManager` directly for all
  engines (drop the Postgres-only guard). Call sequence:
  1. Retrieve `ConnectionConfig`; guard on `Connected`.
  2. `self.connection_manager.connect(&config).await`
  3. `self.connection_manager.get_table_metadata(&config.id, table_name).await` → `TableMetadata`
  4. Write `self.db.current_table_metadata = Some(metadata)`.
- [ ] 3.3 Rewrite `AppState::update_table_cell` / `delete_table_row` / `set_cell_to_null`
  (app/state.rs lines 1802–1845 or c0005's `table_viewer.rs`) to call the c0004-introduced
  `ConnectionManager::update_cell` / `delete_row` passthroughs directly, extracting primitives
  from the UI carrier structs (`CellUpdate`, `DeleteConfirmation`, `SetNullConfirmation`). Remove
  the `self.db.update_table_cell(…)` delegation if it still exists.

---

## 4. Unify try_connect_to_database on the trait

> Target file: `src/app/state/connection_lifecycle.rs` (created by c0005); fallback `src/app/state.rs`.

- [ ] 4.1 Write a new `AppState::try_connect_to_database(connection: &ConnectionConfig) -> Result<DatabaseObjectList, String>`:
  1. `self.connection_manager.connect(connection).await` — unified for PG, MySQL, SQLite.
  2. `self.connection_manager.list_database_objects(&connection.id).await` → `DatabaseObjectList`.
  3. Write `self.db.database_objects = Some(objects.clone())`.
  4. Build `self.db.tables` from `objects` using the qualified-name logic (same logic as
     state/database.rs lines 551–578: if schema is `public` or `None` use bare name, else
     qualified name; also include views and materialized views).
  5. Return `Ok(objects)`.
- [ ] 4.2 Verify the call-site in `AppState` that previously delegated to `self.db.try_connect_to_database`
  (app/state.rs lines 655–662) now calls `self.try_connect_to_database` (or the new method body
  is in-lined there). No double-call.
- [ ] 4.3 Confirm no remaining references to `MySqlConnection::list_tables` or
  `SqliteConnection::list_tables` exist in `src/state/` (they are removed by c0003 from the
  adapters themselves).

---

## 5. Verify

- [ ] 5.1 `cargo build` clean
- [ ] 5.2 `cargo clippy --all-targets -- -D warnings` clean
- [ ] 5.3 `cargo fmt --check` clean
- [ ] 5.4 `cargo test` green
- [ ] 5.5 Run the app: select a connection → connect → open a table → confirm data rows and
  metadata (schema tab) load correctly for each engine (at minimum SQLite; MySQL/Postgres if
  a server is available). Then trigger a table-load error (e.g. rename/drop a table while
  connected, or force an invalid table name) and confirm the error surfaces in the details pane
  (`table_load_error`) and as a toast — verifying `table_load_error` is still populated on
  failure after the refactor.
