# Shared adapter layer & trait cleanup

## Why

`src/database/postgres.rs` (1 439 LOC), `src/database/mysql.rs` (1 231 LOC), and
`src/database/sqlite.rs` (894 LOC) are ~70 % structurally identical. Every time a
new adapter is added — or an existing one corrected — the same logic must be replicated
in three places:

| Pattern | Locations |
|---------|-----------|
| Connection-URL builder (scheme + password guard) | postgres.rs:27–45, mysql.rs:25–43 |
| SQLx error → `ConnectionError` classifier | postgres.rs:49–156, mysql.rs:46–150, sqlite.rs:41–125 |
| `format_error` (string → `FormattedError`) | postgres.rs:340–393, mysql.rs:344–402, sqlite.rs:314–354 |
| Per-row `try_get` string extraction | postgres.rs:1154, 1279–1332; mysql.rs:851, 966; sqlite.rs:755, 793 |
| Type-string → `DataType` parser | postgres.rs:1416–1439, mysql.rs:997–1017, sqlite.rs:825–854 |
| `get_keywords()` (shared core + dialect tail) | postgres.rs:395–425, mysql.rs:405–439, sqlite.rs:357–383 |
| Pool-present guard (`if let Some(pool) = &self.pool {…} else {Err(…)}`) | pg ×7, mysql ×14, sqlite ×6 = 27 sites |

Two shadow traits add unnecessary indirection: `ManagedConnection`
(`src/database/connection_manager.rs:18–33`) duplicates all six methods that already
live on the `Connection` trait (`src/database/connection.rs:338–396`). `ConnectionManager::connect`
(`connection_manager.rs:65–93`) repeats the adapter-creation `match database_type {…}`
block that `AdapterFactory::create_connection` (`factory.rs:15–32`) already owns.
The result is a four-site edit to add any new engine variant.

`test_connection` is an identical three-line `SELECT 1` inherent method on each adapter
(`postgres.rs:461–470`, `mysql.rs:479–488`, `sqlite.rs:415–424`) but is absent from
the `Connection` trait, forcing callers (`handlers/connections.rs:557, 580, 603`) to
match on concrete types instead of working through the trait.

`list_tables` on the `Connection` trait (`connection.rs:349`) is a strict subset of
`list_database_objects` (`connection.rs:352`); keeping both means every caller choosing
between them must understand the redundancy. The two remaining callers
(`state/database.rs:591, 625`) use it only as a fallback where `list_database_objects`
would serve identically.

~250 LOC of type structs (`TableColumn`, `ColumnDefinition`, `DataType`,
`TableMetadata`, `ForeignKeyInfo`, `IndexInfo`, `ConstraintInfo`, `ColumnSummary`,
`DatabaseSpecificMetadata`) are defined in `src/database/mod.rs`, bloating what should
be a thin re-export hub.

## What Changes

- Create `src/database/common/` with six focused modules and a `mod.rs` that exposes
  them as `pub(crate)` or `pub` as needed:
  - `connection_url.rs`: `pub fn build_standard_url(scheme: &str, config: &ConnectionConfig, default_db: &str) -> Result<String>`. Used by PostgreSQL and MySQL; SQLite keeps its path-based builder.
  - `error_classifier.rs`: `pub fn classify_connection_error(error_lower: &str, error_code: Option<&str>, table: &[ErrorTableEntry]) -> ConnectionError` plus `ErrorTableEntry` type. Each adapter constructs its static table and delegates.
  - `error_format.rs`: `pub struct DialectErrorPatterns` + `pub fn format_error_generic(error: &str, patterns: &DialectErrorPatterns) -> FormattedError`. Replaces three identical `format_error` bodies.
  - `row_format.rs`: `pub fn get_column_as_string<R: sqlx::Row>(row: &R, idx: usize) -> String` using `try_get::<Option<String>, _>(idx)`. Replaces six copy-paste per-row loops.
  - `type_map.rs`: `pub fn parse_postgres_type`, `pub fn parse_mysql_type`, `pub fn parse_sqlite_type` — moved verbatim from their adapters.
  - `keywords.rs`: `pub const COMMON_SQL_KEYWORDS: &[&str]`; each adapter's `get_keywords` returns `[COMMON_SQL_KEYWORDS, DIALECT_KEYWORDS].concat()`.
- Add private `fn pool(&self) -> Result<&Pool>` to each adapter, returning the pool or
  `LazyTablesError::Connection("No active connection")`. Replace all 27 inline pool guards
  with `let pool = self.pool()?;`.
- `Connection` trait (`connection.rs:327`): add a default async `test_connection` that
  calls `execute_raw_query("SELECT 1").map(|_| ())`. Delete the three identical inherent
  `test_connection` methods. Remove `list_tables` from the trait; update
  `state/database.rs:591, 625` to call `list_database_objects` instead.
- Eliminate `ManagedConnection` (`connection_manager.rs:18–33`). Change the stored type
  to `Box<dyn Connection>`. Delete the duplicate `match database_type` creation block
  in `ConnectionManager::connect` (`connection_manager.rs:65–93`); replace it with
  `AdapterFactory::create_connection(config)` + `Connection::connect`. One creation site.
- Extract type structs out of `src/database/mod.rs` into `src/database/types.rs`
  (`TableColumn`, `ColumnDefinition`, `DataType`, `TableMetadata`, `ForeignKeyInfo`,
  `IndexInfo`, `ConstraintInfo`, `ColumnSummary`, `DatabaseSpecificMetadata` — ~250 LOC).
  Re-export from `mod.rs` so all external import paths (`crate::database::TableColumn`, etc.)
  are unchanged.

## Impact

- Affected code:
  - `src/database/postgres.rs`, `mysql.rs`, `sqlite.rs`
  - `src/database/connection.rs` (trait changes)
  - `src/database/connection_manager.rs` (ManagedConnection removal + box type change)
  - `src/database/mod.rs` (add `pub mod common; pub mod types;`, keep re-exports)
  - `src/database/factory.rs` (no logic change; now the sole creation site)
  - `src/app/handlers/connections.rs` (calls `conn.test_connection()` via trait — stays, now dispatches through default impl)
  - `src/state/database.rs` (`list_tables` → `list_database_objects` at lines 591, 625)
- Affected specs: none (internal refactor, behavior preserved)
- Risk: medium — touches all three adapters; error-classifier table ports must exactly
  reproduce existing category assignments; `ManagedConnection` removal changes the
  stored box type throughout `ConnectionManager`.
- Depends on: none
