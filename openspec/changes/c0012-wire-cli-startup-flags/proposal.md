# Honor --connection/--database/--table/--read-only

## Why

`src/cli.rs` defines four startup flags at lines 22–36:

```
pub connection: Option<String>,   // --connection
pub database:   Option<String>,   // --database / -d
pub table:      Option<String>,   // --table / -t
pub read_only:  bool,             // --read-only / -r
```

`src/main.rs:37` constructs the app with `App::new(config).await` — the parsed `Cli`
value is used only for `log_level` and theme subcommands; every other field is silently
discarded. `App::new` (src/app/mod.rs:71) accepts only a `Config`. The result is a
fully-wired CLI parser surface that produces no user-visible behaviour: users who type
`lazytables --connection prod --table users` land on the same blank UI as a bare
invocation.

## What Changes

- Thread `Cli` into the app entry-point: introduce a `StartupOptions` struct in
  `src/app/mod.rs`; build it from `Cli` in `src/main.rs`; change `App::new` signature
  to `pub async fn new(config: Config, options: StartupOptions) -> Result<Self>`; store
  `read_only: bool` and the pending startup intent on `App`.

- `--connection <name|conn-string>`: at the start of `App::run()`, after
  `initialize_app_db()`, resolve the value against `state.db.connections.connections`
  by `name` field. If found, select that index and spawn a background connection task
  using the same pattern as `handlers/connections.rs:64–98` (sends a `ConnectionEvent`
  back through `connection_events_tx`). If the value looks like a connection string
  (contains `://`), parse it via `AdapterFactory::create_connection_from_string`, add
  it to storage transiently, and connect. If neither resolves, emit a startup toast and
  continue to the normal UI. The connecting animation (`connecting_in_progress` +
  `tick()`) fires exactly as for a manual connection.

- `--database <name>`: stored on `App` as `startup_database: Option<String>`. Consumed
  in `tick()` when `ConnectionEvent::Success` fires: attempt to match the value against
  the loaded `database_objects` entries and set `state.db.selected_schema`. If not
  found/not applicable, emit a toast and clear the field.

- `--table <name>`: stored on `App` as `startup_table: Option<String>`. Consumed in
  `tick()` immediately after `--database` processing on the same `ConnectionEvent::Success`
  path: find the table name in `state.db.tables`, set `state.ui` selection to that
  index, then call `state.open_table_for_viewing()`. If not found, emit a toast.
  `--table` without `--connection` is a no-op with a startup toast.

- **BREAKING** `--read-only`: store `read_only: bool` on `App`. Gate all write paths:
  - `handlers/query_results.rs:322` — before `app.state.update_table_cell(update)`, if
    `app.read_only` emit a "read-only mode" toast and return early.
  - `handlers/overlays.rs:177` — before `app.state.delete_table_row(confirmation)`, same
    gate.
  - `handlers/overlays.rs:204` — before `app.state.set_cell_to_null(confirmation)`, same
    gate.
  - `handlers/query_editor.rs:24` and `:32` — before calling
    `app.state.execute_query_at_cursor()`, extract the query from
    `app.state.query_editor.get_statement_at_cursor()`, pass it to a new
    `is_write_sql(query: &str) -> bool` helper (leading-keyword check:
    `INSERT`, `UPDATE`, `DELETE`, `DROP`, `ALTER`, `TRUNCATE`, `CREATE`; case-insensitive,
    trimmed). If the flag is set and the query is a write, emit the toast and skip
    execution. Read queries execute normally even in read-only mode.

## Impact

- Affected code:
  - `src/main.rs` (App::new call)
  - `src/app/mod.rs` (App struct, App::new, App::run, tick)
  - `src/app/handlers/query_results.rs` (update_table_cell call)
  - `src/app/handlers/overlays.rs` (delete_table_row, set_cell_to_null calls)
  - `src/app/handlers/query_editor.rs` (execute_query_at_cursor calls)
  - `src/database/factory.rs` (AdapterFactory::create_connection_from_string, detect_database_type)
- Affected specs: `cli-startup` (new behavioral capability)
- Risk: medium — touches the central `App` struct and entry-point; startup flags are
  additive except for read-only gating which changes behaviour of existing mutation
  paths when the flag is supplied.
- Depends on: c0004 (mutation path rewrite; the gated methods exist post-c0004),
  c0005 (App struct and state module split; StartupOptions threads into the new layout)
