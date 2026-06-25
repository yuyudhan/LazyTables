# Design: Shared adapter layer & trait cleanup

## Context

LazyTables has three SQL adapters (PostgreSQL, MySQL, SQLite) implemented as concrete
structs in `src/database/{postgres,mysql,sqlite}.rs`. All three grow by copy-paste:
connection URL building, error classification, error formatting, per-row value extraction,
SQL type parsing, and keyword lists are near-identical across the files. The
`ConnectionManager` maintains a redundant `ManagedConnection` shadow trait
(`connection_manager.rs:18–33`) that duplicates the six methods already on `Connection`
(`connection.rs:338–396`), and its `connect()` method has a duplicate adapter-creation
`match` block that rivals `AdapterFactory::create_connection` (`factory.rs:15–32`).
The type structs that form the data contract (`TableMetadata`, `DataType`, etc.) live
inside the re-export hub `database/mod.rs`, making the file do too many jobs.

This change creates a `src/database/common/` shared module, collapses the shadow trait,
unifies the creation path, and separates the type structs — with zero change to runtime
behavior or public API paths.

## Goals / Non-Goals

**Goals:**
- Reduce the duplication that makes all three adapters a 4-site edit to extend.
- Make the `Connection` trait the single, complete contract for an adapter; eliminate
  the need for `ManagedConnection`.
- Give `AdapterFactory::create_connection` exclusive ownership of adapter instantiation.
- Move type structs to a dedicated `types.rs`; keep all external import paths stable.

**Non-Goals:**
- Changing any user-visible behavior (error messages, data displayed, query results).
- Adding new adapter features or fixing unrelated bugs.
- Changing the pool-options configuration (max connections, timeouts) on any adapter.
- Supporting new database engines.

## Decisions

### Decision 1 — `src/database/common/` module hierarchy

**What:** Create `src/database/common/mod.rs` exposing six sub-modules:
`connection_url`, `error_classifier`, `error_format`, `row_format`, `type_map`,
`keywords`. Declare `pub mod common;` in `database/mod.rs`.

**Why:** Grouping shared utilities under a single `common/` namespace makes it
immediately obvious to a future maintainer that these are not adapter-specific.
Separate files keep each concern independently legible and testable.

**Alternatives rejected:**
- *Inline as free functions in `database/mod.rs`* — mod.rs already doubles as a
  re-export hub; growing it further worsens navigation.
- *One big `shared.rs` file* — merges unrelated concerns (URL building and type mapping)
  into a single 600-line file with no structure.

---

### Decision 2 — `connection_url.rs`: shared URL builder

**What:**
```rust
pub fn build_standard_url(
    scheme: &str,
    config: &ConnectionConfig,
    default_db: &str,
) -> Result<String>
```
Extracts the password from `config.resolve_password(None)`, then produces
`scheme://user:pass@host:port/db` or `scheme://user@host:port/db` when the
password is empty.

PostgreSQL uses `scheme = "postgresql"`, `default_db = "postgres"`.
MySQL uses `scheme = "mysql"`, `default_db = "mysql"`.
SQLite keeps its own `build_connection_string` (file-path based, `sqlite://{path}`)
and does not use this function.

**Why:** The two URL builders at `postgres.rs:27–45` and `mysql.rs:25–43` are textually
identical aside from the scheme literal and the default database name.

**Alternatives rejected:**
- *Macro-based builder* — adds complexity with no advantage over a plain function.
- *Trait method with default impl* — the scheme and default_db are per-adapter
  constants, not instance state; a free function parameterised on those constants
  is simpler.

---

### Decision 3 — `error_classifier.rs`: table-driven connection error classification

**What:**
```rust
pub struct ErrorTableEntry {
    pub substrings: &'static [&'static str],
    pub code:       Option<&'static str>,
    pub error_type: ConnectionErrorType,
    pub user_msg:   &'static str,
    pub suggestion_factory: fn(&ConnectionConfig) -> Vec<String>,
}

pub fn classify_connection_error(
    error_lower:  &str,
    error_code:   Option<&str>,
    config:       &ConnectionConfig,
    table:        &[ErrorTableEntry],
) -> ConnectionError
```

Each adapter defines its own `&[ErrorTableEntry]` static table, porting the existing
`if … else if …` chains verbatim — same substring checks, same error codes, same
user messages and suggestions — and calls `classify_connection_error`.

**Why:** The three `parse_connection_error` methods share the same two-step pattern
(extract error code from sqlx::Error::Database; walk an if-chain matching substrings
and codes). Separating the table from the matching logic means new error conditions
need only an extra `ErrorTableEntry` row, not a new if-block in each adapter.

**Alternatives rejected:**
- *Trait method `parse_connection_error` with default impl* — requires the table to be
  an instance method return, making it harder to declare `const`/`static`.
- *Merging all three tables into one with a dialect discriminant* — increases coupling
  between adapters; dialect-specific strings (MySQL `can't connect`, PG
  `could not connect to server`, SQLite `unable to open database`) should remain
  separately owned.

**Port fidelity requirement:** The existing branches must be reproduced exactly.
PostgreSQL categories (in order): Network (`connection refused` / `could not connect`),
Authentication (`password authentication failed` / `authentication failed` / code
`28P01`), DatabaseNotFound (`database … does not exist` / code `3D000`), SslConfiguration
(`ssl` / `tls`), Network-timeout (`timeout` / `timed out`), ServerError-too-many
(`too many connections` / code `53300`), Unknown (fallback). MySQL: Network (`connection
refused` / `can't connect`), Authentication (`access denied` / `authentication` / code
`1045`), DatabaseNotFound (`unknown database` / code `1049`), SslConfiguration,
Network-timeout, ServerError-too-many (`too many connections` / code `1040`), Unknown.
SQLite (no code matching): Configuration (`unable to open database` / `no such file or
directory`), Configuration-readonly (`readonly` / `read-only`), ServerError-disk-full
(`disk … full`), ServerError-locked (`database is locked` / `locked`),
Configuration-malformed (`not a database` / `malformed`), Configuration-permission
(`permission denied`), Unknown (fallback).

---

### Decision 4 — `error_format.rs`: shared `format_error` bodies

**What:**
```rust
pub struct DialectErrorPatterns {
    pub permission_substrings:  &'static [&'static str],
    pub db_not_found_substrings: &'static [&'static str],
    pub network_substrings:     &'static [&'static str],
    pub syntax_substrings:      &'static [&'static str],
    pub table_not_found_substrings: &'static [&'static str],
    pub dialect_name:           &'static str,
    pub syntax_error_code:      Option<&'static str>,
}

pub fn format_error_generic(
    error: &str,
    patterns: &DialectErrorPatterns,
) -> FormattedError
```

Each adapter declares a file-level `static PATTERNS: DialectErrorPatterns = …` and
its `format_error` impl becomes a single call to `format_error_generic(error, &PATTERNS)`.

**Why:** The three `format_error` implementations at `postgres.rs:340–393`,
`mysql.rs:344–402`, and `sqlite.rs:314–354` are structurally identical: they match
error substrings, set three boolean flags (`is_connection_error`, `is_syntax_error`,
`is_permission_error`), accumulate suggestions, and fill a `FormattedError`. Only the
substrings and messages differ.

**Alternatives rejected:**
- *Trait default impl* — `format_error` takes `&str`, not `&sqlx::Error`; no instance
  state is required; a free function is cleaner.
- *Macro* — adds indirection without meaningful abstraction.

---

### Decision 5 — `row_format.rs`: unified per-column extractor

**What:**
```rust
pub fn get_column_as_string<R: sqlx::Row>(row: &R, idx: usize) -> String {
    row.try_get::<Option<String>, _>(idx)
        .ok()
        .flatten()
        .unwrap_or_else(|| "NULL".to_string())
}
```

Replaces six copy-paste loops in `get_table_data` and `execute_raw_query` across all
three adapters (postgres.rs:1154, 1279–1332; mysql.rs:851, 966; sqlite.rs:755, 793).

**Why:** Every adapter's inner row-extraction loop is:
`let value: Option<String> = row.try_get(idx).ok(); row_data.push(value.unwrap_or_else(|| "NULL".to_string()));`
Extracting it into one function removes the repetition and ensures the NULL sentinel
is spelled consistently everywhere.

**PG type-strictness note:** PostgreSQL's current `execute_raw_query` uses the
`extract_postgres_value` helper (postgres.rs:1261–1412) which dispatches on the
PostgreSQL type name (TEXT, INT2, INT4, etc.) before calling type-specific
`try_get::<Option<T>>` variants. `get_column_as_string` standardizes this to
`try_get::<Option<String>, _>`, which sqlx resolves via the database's implicit
text-cast for all PG types. Semantics are preserved: every value is represented as its
text form, identical to what `extract_postgres_value`'s text-type arms already produce,
and its final fallback (`try_get::<Option<String>, _>(col_ordinal)` at postgres.rs:1399)
already does for unrecognized types. **The function signature must use
`try_get::<Option<String>, _>(idx)` — not a bare `try_get::<String, _>` — to preserve
correct NULL → `"NULL"` conversion.**

**Alternatives rejected:**
- *Keeping `extract_postgres_value`* — it handles 15 type arms all producing `String`
  in the end; the final fallback already demonstrates that `try_get::<Option<String>, _>`
  is sufficient and that was the original intention before the explicit arms were added.
  Removing the explicit arms simplifies the code without changing output.

---

### Decision 6 — `type_map.rs`: consolidated type parsers

**What:** Move the three private functions `parse_postgres_type` (postgres.rs:1416),
`parse_mysql_type` (mysql.rs:997), `parse_sqlite_type` (sqlite.rs:825) to
`src/database/common/type_map.rs` as `pub fn`s. Each adapter removes its local copy
and imports from `crate::database::common::type_map`.

**Why:** The parsers are already self-contained pure functions. Co-locating them makes
it trivial to verify that a `DataType` variant is reached consistently across engines.

**Alternatives rejected:**
- *Leave them private in their adapters* — they are already implicitly shared knowledge;
  anyone reading the code must open three files to understand DataType mapping.

---

### Decision 7 — `keywords.rs`: common keyword constant

**What:**
```rust
pub const COMMON_SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE",
    "CREATE", "DROP", "ALTER", "TABLE", "INDEX", "VIEW", "DATABASE",
    "SCHEMA", "PROCEDURE", "FUNCTION", "TRIGGER", "JOIN", "INNER",
    "LEFT", "RIGHT", "OUTER", "ON", "AS", "DISTINCT", "GROUP",
    "ORDER", "BY", "HAVING", "LIMIT", "OFFSET", "UNION", "ALL",
    "AND", "OR", "NOT", "NULL", "IS", "IN", "LIKE", "BETWEEN",
    "CASE", "WHEN", "THEN", "ELSE", "END", "SET", "INTO", "VALUES",
];
```
Each adapter's `get_keywords` returns
`[COMMON_SQL_KEYWORDS, DIALECT_KEYWORDS].concat()`, where `DIALECT_KEYWORDS` holds
the adapter-specific terms (e.g., `"AUTO_INCREMENT"`, `"RETURNING"`, `"VACUUM"`, etc.)
that are not shared.

**Why:** Comparing the three keyword lists in `postgres.rs:395–425`, `mysql.rs:405–439`,
and `sqlite.rs:357–383` reveals a large common core. Pulling that core out removes the
risk of one adapter losing a common keyword during future maintenance.

**Alternatives rejected:**
- *Single merged list in one adapter* — creates a cross-module dependency in the wrong
  direction.

---

### Decision 8 — Per-adapter `pool()` helper

**What:** Add to each adapter a private method:
```rust
fn pool(&self) -> Result<&Pool> {
    self.pool.as_ref().ok_or_else(|| {
        LazyTablesError::Connection("No active connection".to_string())
    })
}
```
Replace all 27 inline `if let Some(pool) = &self.pool {…} else {Err(…)}` guards with
`let pool = self.pool()?;` at the top of each method body.

**Why:** 27 repetitions of the same five-line guard add noise and create divergence in
error message strings ("Not connected to database" vs. "No active connection" appear
in different methods). A single helper centralizes the string.

**Error message chosen:** `"No active connection"` — the most semantically precise and
the one already used in several MySQL methods. The `?` operator propagates it uniformly.

**Alternatives rejected:**
- *Macro* — unnecessary indirection for a 5-line helper.
- *Trait default on Connection* — `pool` is adapter-internal state with a concrete pool
  type per adapter (PgPool, MySqlPool, SqlitePool); cannot be expressed on the trait.

---

### Decision 9 — `Connection` trait: add `test_connection` default, remove `list_tables`

**What:**
```rust
// In connection.rs, inside the trait block:
async fn test_connection(&self) -> Result<()> {
    self.execute_raw_query("SELECT 1").await.map(|_| ())
}
```
Delete the three identical inherent `test_connection` methods from
`postgres.rs:461–470`, `mysql.rs:479–488`, `sqlite.rs:415–424`.

Remove `async fn list_tables(&self) -> Result<Vec<String>>;` from the trait and its
three adapter implementations. Update the two callers in `state/database.rs:591, 625`
to call `list_database_objects()` instead.

**Why:** `test_connection` is three identical five-line methods; a trait default
eliminates all three simultaneously. `list_tables` is a strict subset of
`list_database_objects`; removing it from the trait forces all callers to the richer
method, which already handles the use-cases at lines 591 and 625.

**Alternatives rejected:**
- *Keep `list_tables` as a deprecated default on the trait* — leaves dead surface area;
  the two callers are trivial to update.

---

### Decision 10 — Eliminate `ManagedConnection`; `ConnectionManager` stores `Box<dyn Connection>`

**What:**
1. Delete `pub trait ManagedConnection` from `connection_manager.rs:18–33`.
2. Delete the three `impl ManagedConnection for <Adapter>` blocks
   (postgres.rs:1220–1255, mysql.rs:1196–1228, sqlite.rs, end of file).
3. Change `ConnectionStorage` from `Arc<Mutex<Box<dyn ManagedConnection>>>` to
   `Arc<Mutex<Box<dyn Connection>>>`.
4. In `ConnectionManager::connect` (connection_manager.rs:65–93): replace the
   `match config.database_type {…}` block with:
   ```rust
   let mut connection = AdapterFactory::create_connection(config.clone())?;
   Connection::connect(&mut connection).await?;
   ```
   `connection` is now `Box<dyn Connection>` (the factory return type).

**Why:** All six methods in `ManagedConnection` (`execute_raw_query`, `get_table_data`,
`get_table_columns`, `get_table_metadata`, `list_database_objects`, `is_connected`) are
already present on the `Connection` trait (`connection.rs:345, 364, 358, 355, 352, 338`).
The shadow trait exists only because `Connection` was not object-safe when `ConnectionManager`
was written; with `list_tables` removed (Decision 9) and no other non-object-safe methods
remaining, `Connection` is object-safe and can be boxed directly.

**Pre-condition to verify before implementing:** All six `ManagedConnection` methods
(`execute_raw_query`, `get_table_data`, `get_table_columns`, `get_table_metadata`,
`list_database_objects`, `is_connected`) must be confirmed present on `Connection`
— they are, as verified at connection.rs:338–370.

**Alternatives rejected:**
- *Keep `ManagedConnection` and add `test_connection` to it* — extends the very shadow
  that should be removed.
- *Merge both traits* — the name `Connection` is the correct public-facing trait;
  `ManagedConnection` was always an internal workaround.

---

### Decision 11 — Extract type structs to `src/database/types.rs`

**What:** Move the following from `database/mod.rs` to `database/types.rs`:
`TableColumn` (mod.rs:41–47), `ColumnDefinition` (mod.rs:51–58), `DataType` (mod.rs:62–117),
`TableMetadata` (mod.rs:121–157), `ForeignKeyInfo` (mod.rs:161–168),
`IndexInfo` (mod.rs:172–179), `ConstraintInfo` (mod.rs:183–188),
`ColumnSummary` (mod.rs:192–199), `DatabaseSpecificMetadata` (mod.rs:203–229),
and `impl TableMetadata` (mod.rs:231–345).

Add `pub mod types;` to `database/mod.rs` and keep all existing `pub use …`
re-exports so no external import path changes (`crate::database::TableColumn` still
resolves correctly).

**Why:** `database/mod.rs` is a re-export hub; type definitions inside it conflate
two roles. Moving them to `types.rs` makes the module's responsibility obvious and
makes the type definitions grep-able in isolation.

**Alternatives rejected:**
- *Inline type definitions in each adapter* — the types are shared across all adapters
  and the UI layer.

## Edge Cases & Failure Modes

**Error-classifier table order:** The `if … else if` chain in each adapter evaluates
in top-to-bottom order. The static `ErrorTableEntry` table must preserve this order
exactly. The MySQL classifier has `authentication` before `unknown database`; reversing
rows would misclassify errors containing both substrings.

**PG `try_get::<Option<String>, _>` semantics:** PostgreSQL's sqlx driver can decode
most column types to `String` via text representation when `try_get::<Option<String>, _>`
is used. The one confirmed exception is OID-based types (e.g., `regclass`) where the
server returns a binary OID; these fall through to `"NULL"` in the current code's
catch-all too, so there is no regression.

**`list_tables` removal and `state/database.rs`:** The two call sites at lines 591 and
625 use `list_tables` only to populate the tables list after connecting to MySQL/SQLite.
`list_database_objects` returns a `DatabaseObjectList` which includes tables; the caller
must extract `Vec<String>` of table names by filtering `objects.objects` on type
`DatabaseObjectType::Table`. Update both sites accordingly.

**`ConnectionManager` box type change:** All methods on `ConnectionManager` that call
`connection.execute_raw_query(…)` etc. will call through `Box<dyn Connection>` instead
of `Box<dyn ManagedConnection>`. Because the method signatures are identical, no
call-site outside `connection_manager.rs` changes.

**`test_connection` trait default:** The default uses `execute_raw_query("SELECT 1")`
which returns `(Vec<String>, Vec<Vec<String>>)`. The `.map(|_| ())` discards the result.
For any engine that does not support `SELECT 1` (none of the three currently supported
engines), an adapter may override the default. The existing three inherent implementations
issue `sqlx::query("SELECT 1").fetch_one(pool)` directly on the pool; the default
implementation routes through `execute_raw_query` which also issues `SELECT 1` on the
pool — functionally identical.

**`impl ManagedConnection` block deletion:** The `postgres.rs` `ManagedConnection` impl
(lines 1220–1255) currently delegates to the concrete methods via `PostgresConnection::execute_raw_query(self, …)` etc. After deletion, `ConnectionManager` calls
through `Box<dyn Connection>` which delegates via the `Connection` trait impl (the
`impl Connection for PostgresConnection` block at lines 159–457). The same concrete
method is reached.

## Migration / Cutover

### Files to create

- `src/database/common/mod.rs`
- `src/database/common/connection_url.rs`
- `src/database/common/error_classifier.rs`
- `src/database/common/error_format.rs`
- `src/database/common/row_format.rs`
- `src/database/common/type_map.rs`
- `src/database/common/keywords.rs`
- `src/database/types.rs`

### Edits per file

**`database/mod.rs`:**
- Add `pub mod common;` and `pub mod types;`.
- Move the type-struct definitions and their impls to `types.rs`.
- Keep all existing `pub use …` lines; add `pub use types::*;` (or enumerate).

**`database/postgres.rs`:**
- Replace `build_connection_string` body with `common::connection_url::build_standard_url("postgresql", &self.config, "postgres")`.
- Replace `parse_connection_error` body: build `error_lower`/`error_code` as before, then delegate to `common::error_classifier::classify_connection_error`.
- Replace `format_error` body with `common::error_format::format_error_generic(error, &POSTGRES_ERROR_PATTERNS)` (define `static POSTGRES_ERROR_PATTERNS`).
- Replace `parse_postgres_type` usage with `common::type_map::parse_postgres_type`; delete local fn.
- Replace `get_keywords` body with `[common::keywords::COMMON_SQL_KEYWORDS, POSTGRES_KEYWORDS].concat()`.
- Replace `extract_postgres_value` and all six per-row loops with `common::row_format::get_column_as_string(row, idx)` (or `col.ordinal()`).
- Add private `fn pool(&self) -> Result<&PgPool>` method; replace all 7 `if let Some(pool) = &self.pool` guards.
- Delete inherent `test_connection` method (postgres.rs:461–470).
- Delete `impl ManagedConnection for PostgresConnection` block (postgres.rs:1220–1255).

**`database/mysql.rs`:**
- Same pattern as postgres: delegate to `build_standard_url`, `classify_connection_error`, `format_error_generic`, `parse_mysql_type`, `COMMON_SQL_KEYWORDS`.
- Replace all 14 pool guards with `self.pool()?`.
- Delete inherent `test_connection` (mysql.rs:479–488).
- Delete `impl ManagedConnection for MySqlConnection` block (mysql.rs:1196–1228).
- Keep the `#[cfg(test)]` block — it guards error-classifier parity and must continue to pass.

**`database/sqlite.rs`:**
- Delegate to `format_error_generic`, `parse_sqlite_type`, `COMMON_SQL_KEYWORDS`.
- SQLite keeps its own `build_connection_string` (path-based; does not call `build_standard_url`).
- Replace all 6 pool guards with `self.pool()?`.
- Delete inherent `test_connection` (sqlite.rs:415–424).
- Delete `impl ManagedConnection for SqliteConnection` block.

**`database/connection.rs`:**
- Add `async fn test_connection(&self) -> Result<()>` default impl inside the trait block.
- Remove `async fn list_tables(&self) -> Result<Vec<String>>;` from the trait.

**`database/connection_manager.rs`:**
- Delete `pub trait ManagedConnection` and `type ConnectionStorage` alias (lines 10, 18–33).
- Change stored type to `Arc<Mutex<HashMap<String, Arc<Mutex<Box<dyn Connection>>>>>>`.
- In `connect()`: replace lines 65–93 (match block) with factory + trait call.
- Update `get_connection` return type to `Arc<Mutex<Box<dyn Connection>>>`.
- All other `ConnectionManager` methods delegate the same way; only the stored type changes.

**`state/database.rs`:**
- Line 591: replace `.list_tables()` with `.list_database_objects()` and extract table names.
- Line 625: same.

### Code to delete

| File | What to delete |
|------|---------------|
| `postgres.rs` | `fn build_connection_string` body (keep signature, replace body); inherent `test_connection`; `impl ManagedConnection for PostgresConnection`; `fn extract_postgres_value`; `fn parse_postgres_type` |
| `mysql.rs` | Same pattern; `impl ManagedConnection for MySqlConnection`; `fn parse_mysql_type` |
| `sqlite.rs` | `fn parse_sqlite_type`; inherent `test_connection`; `impl ManagedConnection for SqliteConnection` |
| `connection_manager.rs` | `ManagedConnection` trait body (lines 18–33); adapter-creation `match` in `connect()` (lines 65–93) |
| `connection.rs` | `list_tables` declaration (line 349) |
| `database/mod.rs` | All type-struct definitions (lines 39–345) once moved to `types.rs` |

## Verification

1. `cargo build` clean with no warnings about unused code.
2. `cargo clippy --all-targets -- -D warnings` clean.
3. `cargo fmt --check` clean.
4. `cargo test` green, including:
   - `database::factory::tests::*` (connection creation per type)
   - `database::mysql::tests::*` (error formatting + parse_mysql_types + keyword/function lists)
5. If a live PostgreSQL server is accessible: connect, open a table, verify all column
   types render correctly (text, integers, booleans, UUIDs). If unavailable, rely on
   the adapter unit tests plus `cargo test database::` to cover row extraction logic.
   State any live-DB limitation in the Verify task note.
