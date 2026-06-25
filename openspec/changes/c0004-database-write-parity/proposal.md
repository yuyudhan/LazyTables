# Cell edit / row delete / set-NULL for all engines

## Why

Editing a cell, deleting a row, and setting a cell to NULL are gated on
`DatabaseType::PostgreSQL` only. In `src/state/database.rs`, three public
methods each contain a `match connection.database_type` that falls through to an
error arm for every non-Postgres engine:

- `update_table_cell` — line 266: `Err(format!("Database type {} not yet
  supported for cell updates", …))`
- `delete_table_row` — line 341: `Err(format!("Database type {} not yet
  supported for row deletion", …))`
- `set_cell_to_null` — line 380: `Err(format!("Database type {} not yet
  supported for setting NULL", …))`

The actual work is delegated to three private helpers (`update_postgres_cell`
at line 280, `delete_postgres_row` at 394, `set_postgres_cell_to_null` at 432)
that build standard `UPDATE`/`DELETE` SQL and execute it through the
engine-agnostic `connection_manager.execute_raw_query(&connection.id, &sql)`.
Nothing about those helpers is Postgres-specific; they exist only because a
`match` guard was never removed.

Additionally, all three helpers build SQL with raw string interpolation and no
identifier quoting — identifiers like `order`, `group`, or names containing
spaces will produce invalid SQL, and a table or column name with a double-quote
can escape the quoting context.

## What Changes

- New `src/database/common/dml.rs` (inside the `database/common/` module
  introduced by c0003). Provides `build_update_cell_sql`, `build_delete_row_sql`,
  `quote_ident` (dialect-aware: PG/SQLite use `"…"`, MySQL uses `` `…` ``), and
  `quote_literal` (`'…'` with `'` doubled). Both builders reject an empty PK
  slice with `Err`.
- Two new default methods on the `Connection` trait (`src/database/connection.rs`):
  `update_cell` and `delete_row`. Default impls call `build_*_sql` with
  `self.config().database_type` and then `self.execute_raw_query`. All three
  concrete adapters inherit the behavior without additional code.
- `ConnectionManager` (`src/database/connection_manager.rs`): two new
  passthroughs `update_cell` and `delete_row` that lock the stored
  `Box<dyn Connection>` (after c0003 eliminates `ManagedConnection`) and call
  the trait default methods.
- `DatabaseState` (`src/state/database.rs`): rewrite `update_table_cell`,
  `delete_table_row`, and `set_cell_to_null` to call
  `connection_manager.update_cell` / `connection_manager.delete_row` for **any**
  `ConnectionStatus::Connected` engine. Delete the three Postgres-only private
  helpers (`update_postgres_cell`, `delete_postgres_row`,
  `set_postgres_cell_to_null`). Keep the `ConnectionStatus::Connected`
  precondition. `set_cell_to_null` passes `None` as the value to
  `connection_manager.update_cell`, which maps to `SET col = NULL` in the
  generated SQL.

## Impact

- Affected code:
  - `src/database/common/dml.rs` (new file)
  - `src/database/connection.rs` (trait gains two default methods)
  - `src/database/connection_manager.rs` (two new passthroughs)
  - `src/state/database.rs` (three methods rewritten; three helpers deleted)
- Affected specs: `table-data-editing` (behavioral ADD — new capability, all
  engines now supported)
- Risk: medium — new DB behavior; correctness of quoting and escaping must be
  verified against each engine; existing Postgres behavior must be preserved
  exactly
- Depends on: c0003 (introduces `src/database/common/`, eliminates
  `ManagedConnection` so `ConnectionManager` stores `Box<dyn Connection>`)
