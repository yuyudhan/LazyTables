# DatabaseState becomes pure data

## Why

`src/state/database.rs` imports `ui/components` types at lines 8–11:

```rust
use crate::ui::components::{
    table_viewer::{CellUpdate, ColumnInfo, DeleteConfirmation, SetNullConfirmation},
    TableViewerState,
};
```

`DatabaseState` then takes `&mut TableViewerState` as a parameter and mutates it directly inside
`load_table_data` (line 51) and `load_postgres_table_data` (line 109). This creates a
bidirectional `state ↔ ui` dependency: the state layer depends on a UI component type to do
its work, and the UI component type is mutated by a method that logically belongs to the data
layer. The result is that neither layer is independently testable or replaceable.

The struct `DatabaseState` itself is conceptually clean — it holds only pure data (connections,
tables, objects, schemas, metadata, errors). The problem is entirely in the six async execution
methods that live alongside it and reach into both layers simultaneously:

- `load_table_data` / `load_postgres_table_data` (lines 51–195): fetch rows + columns and write
  them directly into `TableViewerState`.
- `load_table_metadata` (lines 198–242): fetch metadata and store it on `DatabaseState.current_table_metadata`.
- `try_connect_to_database` (lines 528–655): connect via engine-specific divergent paths —
  PG uses `ConnectionManager`; MySQL and SQLite bypass it with direct `list_tables()` calls on
  freshly-created, non-pooled connections (lines 582–648).
- `update_table_cell` / `delete_table_row` / `set_cell_to_null` (lines 245–468): Postgres-only
  mutation methods that accept the UI carrier types (`CellUpdate`, `DeleteConfirmation`,
  `SetNullConfirmation`) directly and build raw SQL from their fields.

The `AppState` wrapper already exists as the legitimate mediator that knows both layers
(`app/state.rs` lines 1744–1854), but it currently just forwards all arguments unchanged into
`DatabaseState`, deferring the cross-cutting work.

After c0004, the three mutation methods will be replaced by `ConnectionManager` passthroughs that
take primitives; after c0003, `list_tables` will be removed from adapters. This proposal removes
the remaining layering violations that survive c0003/c0004.

## What Changes

- Remove the `use crate::ui::components::{…}` block (state/database.rs lines 8–11) from
  `DatabaseState`. The struct retains only pure data fields: `connections`, `tables`,
  `database_objects`, `schemas`, `selected_schema`, `table_load_error`,
  `current_table_metadata`, and `new()`.
- Delete `load_table_data` and `load_postgres_table_data` from `DatabaseState` (lines 51–195).
  Replace with a free `async fn load_table_data_for_tab(…) -> Result<LoadedTableData, String>`
  in `src/database/service.rs` (or inline into the c0005 `app/state/table_viewer.rs`), which
  takes only primitives (`connection_id`, `table_name`, `page`, `limit`) and returns
  `LoadedTableData { columns: Vec<TableColumn>, rows: Vec<Vec<String>>, total_rows: usize,
  metadata: Option<TableMetadata> }`. The `AppState::load_table_data` wrapper calls this and
  writes results into `TableViewerState` itself.
- Delete `load_table_metadata` from `DatabaseState` (lines 198–242). Replace with an inline call
  in the `AppState::load_table_metadata` wrapper that calls `ConnectionManager` directly and
  stores the result in `DatabaseState.current_table_metadata`.
- Delete `update_table_cell`, `delete_table_row`, `set_cell_to_null`, and their private
  Postgres-only helpers from `DatabaseState` (lines 245–468). After c0004 these are already
  replaced by `ConnectionManager` passthroughs — this proposal ensures no remnant in
  `DatabaseState` remains.
- Unify `try_connect_to_database` (lines 528–655): remove the MySQL/SQLite branches that
  directly instantiate adapter structs and call `list_tables()` (which no longer exists after
  c0003). All three engines use `connection_manager.connect(config)` followed by
  `connection_manager.list_database_objects(id)`. Move this unified method to
  `AppState`-layer directly (or thin service fn); `DatabaseState` keeps only the data fields
  it writes (`database_objects`, `tables`), not the query logic.
- `AppState::load_table_data` (app/state.rs line 1745) is rewritten to call the service fn and
  assemble `TableViewerState` from the returned `LoadedTableData`. Its public signature is
  unchanged.
- `CellUpdate`, `DeleteConfirmation`, `SetNullConfirmation` remain in
  `ui/components/table_viewer`. The `AppState` wrappers extract primitives from these structs
  before calling the c0004-introduced `ConnectionManager` passthroughs — `AppState` legitimately
  bridges both layers.

## Impact

- Affected code:
  - `src/state/database.rs` — primary change site
  - `src/app/state.rs` (lines 1706–1854) — `load_table_data`, `load_table_metadata`,
    `update_table_cell`, `delete_table_row`, `set_cell_to_null`, `try_connect_to_database`
    wrappers rewritten
  - `src/database/service.rs` — new thin module (if not inlined into c0005 `table_viewer.rs`)
  - `src/app/state/table_viewer.rs` — destination for execution logic after c0005 splits
    `state.rs`
  - `src/app/state/connection_lifecycle.rs` — destination for `try_connect_to_database` after
    c0005 splits `state.rs`
- Affected specs: none (internal refactor, behavior preserved)
- Risk: medium — touches the central state struct and all table-loading paths; mis-wiring
  `table_load_error` population is the primary risk
- Depends on: c0004 (mutation methods replaced), c0005 (AppState split into modules; provides
  `table_viewer.rs` and `connection_lifecycle.rs` as landing pads)
