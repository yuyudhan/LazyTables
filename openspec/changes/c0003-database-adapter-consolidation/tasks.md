# Tasks: Shared adapter layer & trait cleanup

## 1. Create `src/database/common/` modules

- [ ] 1.1 Create `src/database/common/mod.rs` declaring `pub mod connection_url; pub mod error_classifier; pub mod error_format; pub mod row_format; pub mod type_map; pub mod keywords;`
- [ ] 1.2 Create `src/database/common/connection_url.rs` with `pub fn build_standard_url(scheme: &str, config: &ConnectionConfig, default_db: &str) -> Result<String>` — password-conditional URL following the pattern in `postgres.rs:27–45` and `mysql.rs:25–43`
- [ ] 1.3 Create `src/database/common/error_classifier.rs` with `pub struct ErrorTableEntry { pub substrings: &'static [&'static str], pub code: Option<&'static str>, pub error_type: ConnectionErrorType, pub user_msg: &'static str, pub suggestion_factory: fn(&ConnectionConfig) -> Vec<String> }` and `pub fn classify_connection_error(error_lower: &str, error_code: Option<&str>, config: &ConnectionConfig, table: &[ErrorTableEntry]) -> ConnectionError`
- [ ] 1.4 Create `src/database/common/error_format.rs` with `pub struct DialectErrorPatterns` (permission, db-not-found, network, syntax, table-not-found substrings; dialect name; optional syntax error code) and `pub fn format_error_generic(error: &str, patterns: &DialectErrorPatterns) -> FormattedError`
- [ ] 1.5 Create `src/database/common/row_format.rs` with `pub fn get_column_as_string<R: sqlx::Row>(row: &R, idx: usize) -> String` using `row.try_get::<Option<String>, _>(idx).ok().flatten().unwrap_or_else(|| "NULL".to_string())`
- [ ] 1.6 Create `src/database/common/type_map.rs` with placeholder `pub fn parse_postgres_type`, `pub fn parse_mysql_type`, `pub fn parse_sqlite_type` (bodies filled in tasks 2–4)
- [ ] 1.7 Create `src/database/common/keywords.rs` with `pub const COMMON_SQL_KEYWORDS: &[&str]` containing the shared keyword set visible in all three adapters (SELECT, FROM, WHERE, INSERT, UPDATE, DELETE, CREATE, DROP, ALTER, TABLE, INDEX, VIEW, DATABASE, SCHEMA, PROCEDURE, FUNCTION, TRIGGER, JOIN, INNER, LEFT, RIGHT, OUTER, ON, AS, DISTINCT, GROUP, ORDER, BY, HAVING, LIMIT, OFFSET, UNION, ALL, AND, OR, NOT, NULL, IS, IN, LIKE, BETWEEN, CASE, WHEN, THEN, ELSE, END, SET, INTO, VALUES and other universally shared terms)
- [ ] 1.8 Add `pub mod common;` to `src/database/mod.rs`

## 2. Refactor `src/database/postgres.rs` onto the common modules

- [ ] 2.1 Replace `build_connection_string` body with a call to `common::connection_url::build_standard_url("postgresql", &self.config, "postgres")`
- [ ] 2.2 Define `static POSTGRES_ERROR_TABLE: &[ErrorTableEntry]` porting the five classification branches from `parse_connection_error` (lines 66–155) verbatim — same substring checks (connection refused, password authentication failed, database does not exist, ssl/tls, timeout, too many connections / code 53300) and same error codes (28P01, 3D000, 53300)
- [ ] 2.3 Replace `parse_connection_error` body with `common::error_classifier::classify_connection_error(&error_lower, error_code.as_deref(), &self.config, POSTGRES_ERROR_TABLE)`
- [ ] 2.4 Define `static POSTGRES_FORMAT_PATTERNS: DialectErrorPatterns` porting the substrings from `format_error` (lines 347–381): permission (`permission denied`, `authentication failed`), db-not-found (`database … does not exist`), network (`connection refused`, `could not connect`), syntax (`syntax error`), table-not-found (`relation … does not exist`); dialect name `"PostgreSQL"`; no syntax error code
- [ ] 2.5 Replace `format_error` body with `common::error_format::format_error_generic(error, &POSTGRES_FORMAT_PATTERNS)`
- [ ] 2.6 Move `parse_postgres_type` body (lines 1417–1439) into `common::type_map::parse_postgres_type`; delete the local function; import and use `crate::database::common::type_map::parse_postgres_type` at every call site in `postgres.rs`
- [ ] 2.7 Add `POSTGRES_DIALECT_KEYWORDS: &[&str]` for PG-specific terms not in COMMON_SQL_KEYWORDS; replace `get_keywords` body with `[common::keywords::COMMON_SQL_KEYWORDS, POSTGRES_DIALECT_KEYWORDS].concat()`
- [ ] 2.8 Add private `fn pool(&self) -> crate::core::error::Result<&sqlx::postgres::PgPool>` returning `self.pool.as_ref().ok_or_else(|| LazyTablesError::Connection("No active connection".to_string()))`; replace all 7 `if let Some(pool) = &self.pool` guards with `let pool = self.pool()?;`
- [ ] 2.9 Replace the per-row extraction loop in `get_table_data` (around line 1154) and in `execute_raw_query` (around lines 1196–1201) with `common::row_format::get_column_as_string(row, idx)`; delete `fn extract_postgres_value`
- [ ] 2.10 Delete inherent `test_connection` method (lines 461–470)
- [ ] 2.11 Delete `impl ManagedConnection for PostgresConnection` block (lines 1220–1255)

## 3. Refactor `src/database/mysql.rs` onto the common modules

- [ ] 3.1 Replace `build_connection_string` body with `common::connection_url::build_standard_url("mysql", &self.config, "mysql")`
- [ ] 3.2 Define `static MYSQL_ERROR_TABLE: &[ErrorTableEntry]` porting the six branches from `parse_connection_error` (lines 64–150): connection refused / can't connect, access denied / authentication / code 1045, unknown database / code 1049, ssl/tls, timeout, too many connections / code 1040
- [ ] 3.3 Replace `parse_connection_error` body with `common::error_classifier::classify_connection_error`
- [ ] 3.4 Define `static MYSQL_FORMAT_PATTERNS: DialectErrorPatterns` porting `format_error` substrings (lines 352–392): permission (`access denied`), db-not-found (`unknown database`), network (`can't connect`, `connection refused`), syntax (`syntax error`, `you have an error in your sql syntax`), table-not-found (`table … doesn't exist`); dialect name `"MySQL"`; syntax error code `"1064"`
- [ ] 3.5 Replace `format_error` body with `common::error_format::format_error_generic(error, &MYSQL_FORMAT_PATTERNS)`
- [ ] 3.6 Move `parse_mysql_type` body (lines 998–1017) into `common::type_map::parse_mysql_type`; delete the local function; use `crate::database::common::type_map::parse_mysql_type` at every call site
- [ ] 3.7 Add `MYSQL_DIALECT_KEYWORDS: &[&str]` for MySQL-specific terms (`AUTO_INCREMENT`, `ENGINE`, `CHARSET`, `COLLATE`, `SHOW`, `DESCRIBE`, `EXPLAIN`, `PROCESSLIST`, etc.); replace `get_keywords` body with `[common::keywords::COMMON_SQL_KEYWORDS, MYSQL_DIALECT_KEYWORDS].concat()`
- [ ] 3.8 Add private `fn pool(&self) -> crate::core::error::Result<&sqlx::mysql::MySqlPool>`; replace all 14 `if let Some(pool) = &self.pool` guards with `let pool = self.pool()?;`
- [ ] 3.9 Replace per-row extraction loops in `get_table_data` (~line 851) and `execute_raw_query` (~line 966) with `common::row_format::get_column_as_string(row, col.ordinal())`
- [ ] 3.10 Delete inherent `test_connection` (lines 479–488)
- [ ] 3.11 Delete `impl ManagedConnection for MySqlConnection` block (lines 1196–1228)
- [ ] 3.12 Keep the `#[cfg(test)]` block unchanged (tests guard error-format parity)

## 4. Refactor `src/database/sqlite.rs` onto the common modules

- [ ] 4.1 SQLite keeps `build_connection_string` as-is (path-based builder; does not use `build_standard_url`)
- [ ] 4.2 Define `static SQLITE_ERROR_TABLE: &[ErrorTableEntry]` porting the six branches from `parse_connection_error` (lines 52–124): unable-to-open / no-such-file, readonly / read-only, disk … full, database is locked / locked, not a database / malformed, permission denied; no error codes (SQLite does not return sqlx database error codes)
- [ ] 4.3 Replace `parse_connection_error` body with `common::error_classifier::classify_connection_error`
- [ ] 4.4 Define `static SQLITE_FORMAT_PATTERNS: DialectErrorPatterns` porting `format_error` substrings (lines 321–343): table-not-found (`no such table`), syntax (`syntax error`), db-locked (`database is locked`), file-not-found (`no such file`, `unable to open`); dialect name `"SQLite"`; no permission substring (sets `is_permission_error = false` always)
- [ ] 4.5 Replace `format_error` body with `common::error_format::format_error_generic(error, &SQLITE_FORMAT_PATTERNS)`
- [ ] 4.6 Move `parse_sqlite_type` body (lines 826–854) into `common::type_map::parse_sqlite_type`; delete the local function; use `crate::database::common::type_map::parse_sqlite_type` at every call site
- [ ] 4.7 Add `SQLITE_DIALECT_KEYWORDS: &[&str]` for SQLite-specific terms (`AUTOINCREMENT`, `PRAGMA`, `ATTACH`, `DETACH`, `VACUUM`, `ROWID`, `WITHOUT`, `STRICT`, `RETURNING`, etc.); replace `get_keywords` body with `[common::keywords::COMMON_SQL_KEYWORDS, SQLITE_DIALECT_KEYWORDS].concat()`
- [ ] 4.8 Add private `fn pool(&self) -> crate::core::error::Result<&sqlx::sqlite::SqlitePool>`; replace all 6 `if let Some(pool) = &self.pool` guards with `let pool = self.pool()?;`
- [ ] 4.9 Replace per-row extraction loops in `get_table_data` (~line 756) and `execute_raw_query` (~line 793) with `common::row_format::get_column_as_string(row, col.ordinal())`
- [ ] 4.10 Delete inherent `test_connection` (lines 415–424)
- [ ] 4.11 Delete `impl ManagedConnection for SqliteConnection` block

## 5. Trait default `test_connection` + remove `list_tables` + update callers

- [ ] 5.1 In `src/database/connection.rs`, inside the `Connection` trait block, add a default async fn: `async fn test_connection(&self) -> Result<()> { self.execute_raw_query("SELECT 1").await.map(|_| ()) }`
- [ ] 5.2 Remove `async fn list_tables(&self) -> Result<Vec<String>>;` from the `Connection` trait declaration (line 349) and all three adapter implementations (`postgres.rs:202–204`, `mysql.rs:195–197`, `sqlite.rs:177–179`) plus each adapter's inherent `list_tables` method body (`postgres.rs:499–536`, `mysql.rs:514–548`, `sqlite.rs:449–476`)
- [ ] 5.3 In `src/state/database.rs:591`, replace `.list_tables().await` with `.list_database_objects().await` and extract table names by filtering on `DatabaseObjectType::Table`
- [ ] 5.4 In `src/state/database.rs:625`, same replacement as 5.3
- [ ] 5.5 Verify no remaining references to `list_tables` exist in `src/` (grep check)

## 6. Collapse factory/manager creation + delete `ManagedConnection`

- [ ] 6.1 Delete `pub trait ManagedConnection` and its `#[async_trait]` attribute (connection_manager.rs:17–33)
- [ ] 6.2 Update `type ConnectionStorage` (connection_manager.rs:10) to use `Box<dyn Connection>` instead of `Box<dyn ManagedConnection>`; add the `Connection` import from `crate::database::connection`
- [ ] 6.3 In `ConnectionManager::connect` (connection_manager.rs:51–103), replace the `match config.database_type {…}` block (lines 65–93) with: `let mut connection = crate::database::factory::AdapterFactory::create_connection(config.clone())?; Connection::connect(&mut connection).await?;` then store `Box::new(connection)` — but note `create_connection` already returns `Box<dyn Connection>`, so store it directly after connecting
- [ ] 6.4 Update `ConnectionManager::get_connection` return type to `Arc<Mutex<Box<dyn Connection>>>` and remove the `ManagedConnection` import
- [ ] 6.5 Confirm all `ConnectionManager` public methods (`execute_raw_query`, `get_table_data`, `get_table_columns`, `get_table_metadata`, `list_database_objects`, `is_connected`) delegate correctly through the new `Box<dyn Connection>` — method names and signatures on `Connection` are identical, so no other changes required

## 7. Extract `src/database/types.rs`

- [ ] 7.1 Create `src/database/types.rs` and move the following from `database/mod.rs` into it: `TableColumn` (mod.rs:41–47), `ColumnDefinition` (mod.rs:51–58), `DataType` (mod.rs:62–117), `TableMetadata` (mod.rs:121–157), `ForeignKeyInfo` (mod.rs:161–168), `IndexInfo` (mod.rs:172–179), `ConstraintInfo` (mod.rs:183–188), `ColumnSummary` (mod.rs:192–199), `DatabaseSpecificMetadata` (mod.rs:203–229), and `impl TableMetadata` (mod.rs:231–345)
- [ ] 7.2 Add `pub mod types;` to `database/mod.rs` and replace the inline definitions with `pub use types::*;` (or enumerate each type explicitly to preserve the public API)
- [ ] 7.3 Verify that all crate-internal import paths (`crate::database::TableColumn`, `crate::database::DataType`, etc.) still resolve without change

## 8. Verify

- [ ] 8.1 `cargo build` clean
- [ ] 8.2 `cargo clippy --all-targets -- -D warnings` clean
- [ ] 8.3 `cargo fmt --check` clean
- [ ] 8.4 `cargo test` green
- [ ] 8.5 `cargo test database::` green, including `database::factory::tests::*` and `database::mysql::tests::*` (which guard error-format and parse-type correctness). If a live PostgreSQL server is available, additionally connect interactively, open a table with mixed column types (integer, text, boolean, UUID), and confirm all values render correctly. If no live server is available, note this limitation and rely on the unit tests and SQLite end-to-end path.
