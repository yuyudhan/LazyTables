# Design — c0004 Database Write Parity

## Context

`src/state/database.rs` exposes three mutation methods that reach through to
`ConnectionManager::execute_raw_query`. All three already build ordinary
`UPDATE`/`DELETE` SQL; the Postgres-only gate (`match connection.database_type
{ DatabaseType::PostgreSQL => …, _ => Err(…) }`) was never generalised when
MySQL and SQLite adapters were added. The underlying `execute_raw_query` call
is identical across all three adapters — the guards are purely artificial.

The helpers (`update_postgres_cell` at line 280, `delete_postgres_row` at 394,
`set_postgres_cell_to_null` at 432 of `src/state/database.rs`) also build SQL
with naked string interpolation: identifiers are not quoted and literal values
use a single `str::replace("'", "''")` at the top level but nowhere in the
WHERE clause PK values. Both are bugs waiting to surface on any table whose
name or column contains a SQL keyword or a single quote.

After c0003 lands:
- `src/database/common/` exists as a shared-utilities module.
- `ConnectionManager` stores `Box<dyn Connection>` (the `ManagedConnection`
  shadow trait is gone), so trait methods on `Connection` are directly callable
  through the manager's lock guard.

## Goals / Non-Goals

**Goals**
- Cell edit, row delete, and set-NULL work identically on PostgreSQL, MySQL,
  MariaDB, and SQLite.
- Identifier quoting is dialect-correct (no injection via reserved-word names
  or names with special characters).
- Single-quote characters in literal values are escaped in both the SET clause
  and the WHERE clause.
- Empty primary-key slices are rejected early with a clear error message.
- `set_cell_to_null` is unambiguously distinct from setting a cell to an empty
  string.
- All existing behavior on PostgreSQL is preserved.

**Non-Goals**
- Switching to bind parameters (see Decision 1).
- Supporting Oracle, Redis, or MongoDB (those engine types remain unsupported).
- Auto-detecting whether a table has a primary key at the schema level;
  the caller (the table-viewer UI component) is responsible for providing
  `primary_key_values`.

## Decisions

### Decision 1 — Keep value-as-text-literal semantics; do NOT use bind parameters

**What:** SQL UPDATE statements are built as a single string with the value
embedded as a quoted literal (`'escaped_value'`) rather than using sqlx bind
parameters (`$1` / `?`).

**Why:** PostgreSQL relies on implicit text-to-column-type coercion inside an
`UPDATE SET col = 'text'` expression. The existing Postgres path (database.rs
line 302–308) already relies on this. Switching to bind parameters would
require knowing the target column's SQL type at call time and choosing the
correct `sqlx::types` variant — information that is not available in the
current `CellUpdate` / `SetNullConfirmation` structs without an additional
metadata round-trip. The risk/reward is unfavourable for this change.

**Alternatives rejected:**
- `query_with` + typed bind params: requires schema metadata per column type;
  substantially more invasive and out of scope.
- Keeping the raw-string approach without quoting: preserves the injection
  risk and reserved-word breakage.

### Decision 2 — Dialect-aware quoting in `dml.rs`

**What:** A `quote_ident(dt: DatabaseType, name: &str) -> String` helper
returns:
- PostgreSQL / SQLite: `"name"` — any embedded `"` doubled to `""`
- MySQL / MariaDB: `` `name` `` — any embedded `` ` `` doubled to `` `` ``

A `quote_literal(value: &str) -> String` helper returns `'value'` with any `'`
doubled to `''`, independent of dialect.

**Why:** Standard SQL identifier quoting prevents reserved-word collisions and
special-character breakage. Literal quoting must cover the WHERE clause PK
values as well — the current helpers quote the SET value but not the PK values
in the WHERE clause.

**Alternatives rejected:**
- A single double-quote for all dialects: MySQL rejects `"name"` in its
  default mode (requires `ANSI_QUOTES` SQL mode, which users may not have set).

### Decision 3 — Default impls on `Connection` trait; no per-adapter override

**What:** `update_cell` and `delete_row` are added to `Connection` as `async
fn` with default implementations. No concrete adapter (`PostgresConnection`,
`MySqlConnection`, `SqliteConnection`) needs to override them.

**Why:** The logic is identical for all engines: build SQL via `dml.rs` using
`self.config().database_type`, then call `self.execute_raw_query(&sql)`. A
default impl on the trait centralises the behavior and avoids three copies.
Any future adapter automatically inherits the behavior.

**Alternatives rejected:**
- A free function called from each adapter: more boilerplate, no benefit.
- Adding methods only to `ConnectionManager` without touching `Connection`:
  requires `ConnectionManager` to know the `DatabaseType` of each stored
  connection (it currently does not; only the config does). With c0003's
  change (`ConnectionManager` stores `Box<dyn Connection>`), calling
  trait methods via the lock guard is the cleanest path.

### Decision 4 — `ConnectionManager` passthroughs call trait default impls

**What:** `ConnectionManager::update_cell(connection_id, table, column, value,
pk)` and `::delete_row(connection_id, table, pk)` lock the stored
`Arc<Mutex<Box<dyn Connection>>>`, call the trait method on the guard, and
return.

**Why:** Mirrors the existing `execute_raw_query`, `get_table_data`, and other
passthroughs in `connection_manager.rs` (lines 163–217). Consistent pattern.

**Alternatives rejected:**
- Building the SQL in `ConnectionManager` and calling `execute_raw_query`:
  `ConnectionManager` would need to import `dml.rs` helpers and the connection
  config's `database_type`; cleaner to let the trait method own it.

## Edge Cases & Failure Modes

| Scenario | Handling |
|---|---|
| Empty `pk` slice | `build_update_cell_sql` / `build_delete_row_sql` returns `Err("Cannot update/delete row without primary key")` before any SQL is produced |
| Composite primary key | All (column, value) pairs joined with `AND`: `WHERE "pk1" = 'v1' AND "pk2" = 'v2'` |
| Identifier is a SQL reserved word (`order`, `group`, `select`) | `quote_ident` wraps it: `"order"` (PG/SQLite) or `` `order` `` (MySQL) |
| Identifier contains a double-quote (`my"col`) | Doubled inside quotes: `"my""col"` |
| Value contains a single quote (`O'Brien`) | `quote_literal` doubles it: `'O''Brien'` |
| PK value contains a single quote | Same `quote_literal` applied to all PK values in WHERE clause (the current helpers do NOT do this — this is a bug fix) |
| `value = None` (set-NULL) | `SET "col" = NULL` — no `quote_literal` applied; `NULL` is unquoted keyword |
| `value = Some("")` (set empty string) | `SET "col" = ''` — distinct from NULL |
| MySQL table opened as `MariaDB` type | `DatabaseType::MariaDB` treated identically to `MySQL` in `quote_ident` |
| Connection not in `ConnectionManager` | `get_connection` returns `LazyTablesError::Connection`; propagates to caller as `String`-error via `map_err` |
| `ConnectionStatus` not `Connected` | Existing guard in `DatabaseState` methods returns `Err("No active database connection")` before reaching `ConnectionManager` |

## Migration / Cutover

### New file

`src/database/common/dml.rs` — add the module to `src/database/common/mod.rs`
(which c0003 creates). Expose `pub fn build_update_cell_sql`, `pub fn
build_delete_row_sql`, `pub fn quote_ident`, `pub fn quote_literal`.

### `src/database/connection.rs`

Add two methods to the `Connection` trait (after the `get_functions` method,
before the closing `}`):

```rust
/// Update a single cell. Default impl builds dialect-correct SQL via
/// `database::common::dml` and executes it through `execute_raw_query`.
async fn update_cell(
    &self,
    table: &str,
    column: &str,
    value: Option<&str>,
    pk: &[(String, String)],
) -> Result<()> {
    let sql = crate::database::common::dml::build_update_cell_sql(
        self.config().database_type.clone(),
        table, column, value, pk,
    )?;
    self.execute_raw_query(&sql).await.map(|_| ())
}

/// Delete a row. Default impl builds dialect-correct SQL and executes it.
async fn delete_row(
    &self,
    table: &str,
    pk: &[(String, String)],
) -> Result<()> {
    let sql = crate::database::common::dml::build_delete_row_sql(
        self.config().database_type.clone(),
        table, pk,
    )?;
    self.execute_raw_query(&sql).await.map(|_| ())
}
```

No concrete adapter needs an explicit `impl` for these methods.

### `src/database/connection_manager.rs`

Add two passthroughs after `health_check` (line 220), following the same
pattern as `execute_raw_query` (line 164):

```rust
pub async fn update_cell(
    &self,
    connection_id: &str,
    table: &str,
    column: &str,
    value: Option<&str>,
    pk: &[(String, String)],
) -> Result<()> {
    let connection_ref = self.get_connection(connection_id).await?;
    let connection = connection_ref.lock().await;
    connection.update_cell(table, column, value, pk).await
}

pub async fn delete_row(
    &self,
    connection_id: &str,
    table: &str,
    pk: &[(String, String)],
) -> Result<()> {
    let connection_ref = self.get_connection(connection_id).await?;
    let connection = connection_ref.lock().await;
    connection.delete_row(table, pk).await
}
```

These require that `ConnectionManager` stores `Box<dyn Connection>` (guaranteed
after c0003) so that `connection.update_cell(…)` dispatches to the trait method.

### `src/state/database.rs`

**Rewrite** `update_table_cell` (lines 245–277): replace the `match
connection.database_type` guard with a single call to
`connection_manager.update_cell(&connection.id, &update.table_name,
&update.column_name, Some(&update.new_value), &update.primary_key_values)`.
Map `LazyTablesError` to `String` via `.map_err(|e| format!("…{e}"))`.

**Rewrite** `delete_table_row` (lines 320–352): replace the guard with
`connection_manager.delete_row(&connection.id, &confirmation.table_name,
&confirmation.primary_key_values)`.

**Rewrite** `set_cell_to_null` (lines 355–391): replace the guard with
`connection_manager.update_cell(&connection.id, &confirmation.table_name,
&confirmation.column_name, None, &confirmation.primary_key_values)`.

**Delete** `update_postgres_cell` (lines 280–317), `delete_postgres_row`
(lines 394–429), and `set_postgres_cell_to_null` (lines 432–468).

## Verification

1. **Unit tests in `dml.rs`** (new `#[cfg(test)]` block):
   - `quote_ident` for PG, SQLite, MySQL/MariaDB with a plain name, a reserved
     word, and a name containing the quoting character.
   - `quote_literal` with a plain value, a value with one single quote, a value
     with multiple single quotes, an empty string.
   - `build_update_cell_sql` with a single-column PK, a composite PK, `None`
     value (NULL), and empty PK (expect `Err`).
   - `build_delete_row_sql` with a single PK, composite PK, and empty PK
     (expect `Err`).
2. **SQLite end-to-end** via the docker/sqlite fixture (or the file-based SQLite
   DB, which requires no server): connect, open a table that has a primary key,
   edit a cell value, delete a row, set a cell to NULL — confirm each operation
   persists on reload.
3. **MySQL** (if the docker/mysql fixture is available): same three operations.
4. **PostgreSQL** (if docker/postgres is available): verify existing behavior is
   unchanged — all three operations still work identically to before this change.
