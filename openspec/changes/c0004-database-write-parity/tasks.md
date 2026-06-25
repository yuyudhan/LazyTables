# Tasks — c0004 Database Write Parity

## 1. Implement `dml.rs` — SQL builders, quoting helpers, and unit tests

- [ ] 1.1 Create `src/database/common/dml.rs` with `pub fn quote_ident(dt: DatabaseType, name: &str) -> String` — PG/SQLite: `"name"` with `"` doubled; MySQL/MariaDB: `` `name` `` with `` ` `` doubled.
- [ ] 1.2 Add `pub fn quote_literal(value: &str) -> String` — wraps in `'…'` with internal `'` doubled to `''`.
- [ ] 1.3 Add `pub fn build_update_cell_sql(dt: DatabaseType, table: &str, column: &str, value: Option<&str>, pk: &[(String,String)]) -> Result<String>` — returns `Err` when `pk` is empty; `value = None` emits `SET col = NULL`; PK values pass through `quote_literal` in the WHERE clause.
- [ ] 1.4 Add `pub fn build_delete_row_sql(dt: DatabaseType, table: &str, pk: &[(String,String)]) -> Result<String>` — returns `Err` when `pk` is empty; PK values pass through `quote_literal` in the WHERE clause.
- [ ] 1.5 Register `pub mod dml;` in `src/database/common/mod.rs` (created by c0003).
- [ ] 1.6 Add `#[cfg(test)]` block to `dml.rs`:
  - [ ] 1.6.1 `quote_ident` — plain name (PG, MySQL, SQLite), reserved word (`order`), name containing `"` (PG/SQLite), name containing `` ` `` (MySQL).
  - [ ] 1.6.2 `quote_literal` — plain value, value with one `'`, value with multiple `'`, empty string.
  - [ ] 1.6.3 `build_update_cell_sql` — single-column PK (each of PG, MySQL, SQLite), composite PK, `None` value (asserts `= NULL`), `Some("")` (asserts `= ''`), empty PK (`Err`).
  - [ ] 1.6.4 `build_delete_row_sql` — single PK, composite PK, empty PK (`Err`).

## 2. Add default `update_cell` / `delete_row` to the `Connection` trait

- [ ] 2.1 In `src/database/connection.rs`, add `async fn update_cell(&self, table: &str, column: &str, value: Option<&str>, pk: &[(String,String)]) -> Result<()>` with a default impl that calls `crate::database::common::dml::build_update_cell_sql(self.config().database_type.clone(), table, column, value, pk)?` then `self.execute_raw_query(&sql).await.map(|_| ())`.
- [ ] 2.2 Add `async fn delete_row(&self, table: &str, pk: &[(String,String)]) -> Result<()>` with a default impl calling `build_delete_row_sql` then `execute_raw_query` analogously.
- [ ] 2.3 Confirm no concrete adapter (`postgres.rs`, `mysql.rs`, `sqlite.rs`) defines `update_cell` or `delete_row` that would shadow the defaults — if any exists, remove it.

## 3. Add `update_cell` / `delete_row` passthroughs to `ConnectionManager`

- [ ] 3.1 In `src/database/connection_manager.rs`, add `pub async fn update_cell(&self, connection_id: &str, table: &str, column: &str, value: Option<&str>, pk: &[(String,String)]) -> Result<()>` — lock the stored `Box<dyn Connection>` via `self.get_connection(connection_id).await?` and call `connection.update_cell(table, column, value, pk).await`.
- [ ] 3.2 Add `pub async fn delete_row(&self, connection_id: &str, table: &str, pk: &[(String,String)]) -> Result<()>` analogously.
- [ ] 3.3 Verify these compile: they require that `ConnectionManager` stores `Box<dyn Connection>` (guaranteed by c0003); if c0003 is not yet applied, note the hard dependency here.

## 4. Rewrite the three `DatabaseState` mutation methods; delete PG-only helpers

- [ ] 4.1 In `src/state/database.rs`, rewrite `update_table_cell` (lines 245–277): remove the `match connection.database_type` guard; call `connection_manager.update_cell(&connection.id, &update.table_name, &update.column_name, Some(&update.new_value), &update.primary_key_values).await` inside the `ConnectionStatus::Connected` arm; map `LazyTablesError` to `String`.
- [ ] 4.2 Rewrite `delete_table_row` (lines 320–352): call `connection_manager.delete_row(&connection.id, &confirmation.table_name, &confirmation.primary_key_values).await` in the `Connected` arm.
- [ ] 4.3 Rewrite `set_cell_to_null` (lines 355–391): call `connection_manager.update_cell(&connection.id, &confirmation.table_name, &confirmation.column_name, None, &confirmation.primary_key_values).await`.
- [ ] 4.4 Delete private helper `update_postgres_cell` (lines 280–317).
- [ ] 4.5 Delete private helper `delete_postgres_row` (lines 394–429).
- [ ] 4.6 Delete private helper `set_postgres_cell_to_null` (lines 432–468).
- [ ] 4.7 Remove any now-unused imports from `src/state/database.rs` introduced solely by the deleted helpers (check `DatabaseType` variant paths if they are no longer referenced).

## 5. Spec delta

- [ ] 5.1 Ensure `openspec/changes/c0004-database-write-parity/specs/table-data-editing/spec.md` is present (authored as part of this change set).

## 6. Verify

- [ ] 6.1 `cargo build` clean
- [ ] 6.2 `cargo clippy --all-targets -- -D warnings` clean
- [ ] 6.3 `cargo fmt --check` clean
- [ ] 6.4 `cargo test` green
- [ ] 6.5 Unit tests in `src/database/common/dml.rs` all pass (`cargo test database::common::dml`); SQLite end-to-end via the docker/sqlite fixture (or the repo's file-based SQLite DB): connect, edit a cell, delete a row, set a cell to NULL — confirm each persists on table reload. MySQL and PostgreSQL if their docker fixtures are available. Verify PostgreSQL operations produce identical results to before this change.
