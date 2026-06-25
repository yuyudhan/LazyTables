# Single keybinding table feeding handlers & help

## Why

`src/ui/help.rs` (926 LOC) hard-codes every key string across six `add_*_commands` functions
(`add_connections_commands` at line 180, `add_tables_commands` at 272, `add_details_commands` at
358, `add_tabular_commands` at 394, `add_sql_files_commands` at 500, `add_query_window_commands`
at 621) and two column builders (`create_left_column` lines 61–68, `create_right_column` lines
93–128). These string literals have no compile-time link to the actual `match` arms in
`src/app/handlers/global.rs`, `handlers/connections.rs`, `handlers/tables.rs`,
`handlers/query_results.rs`, `handlers/query_editor.rs`, `handlers/sql_files.rs`, and
`handlers/overlays.rs`.

CLAUDE.md (line 128) states: "Help system with '?' key" and under the per-pane reference (lines
183–254) lists the canonical bindings. Drift between this reference, `help.rs`, and the actual
handlers is already present: `create_right_column` lists `C-Enter` (line 125) and `C-S`/`C-O`/`C-N`
(lines 126–128) as "global" data-operation shortcuts even though they are pane-specific bindings in
`handlers/query_editor.rs`. There is no mechanism to catch when a handler key changes and help does
not.

The c0011 proposal adds `Ctrl+Y` (query history overlay) to `handlers/global.rs` and the c0012
proposal adds read-only mode gating — both require matching help entries that will only appear if an
author remembers to update `help.rs` manually.

## What Changes

- New `src/keybindings.rs`: a `const`/`static` table of
  `KeyBinding { context, key_display, action_id, description, condition }` entries, grouped by pane
  context (`Global`, `Connections`, `Tables`, `Details`, `TabularOutput`, `SqlFiles`,
  `QueryWindow`), covering every binding in `handlers/*` plus the c0011 `Ctrl+Y` and c0012
  read-only gate additions.
- `src/ui/help.rs` (`HelpSystem`): replace all literal `add_command(lines, "…", "…")` calls with
  an iterator over `keybindings::BINDINGS` filtered by context. Sections/headers are preserved by
  grouping on context; condition annotations are rendered as parenthetical notes. The `add_command`
  helper function stays (used by the iterator loop).
- `src/app/handlers/global.rs`: replace the three magic key chars (`'?'`, `'b'` for Ctrl+B, `'q'`)
  with constants re-exported from `keybindings.rs` (`KEY_HELP`, `KEY_DEBUG`, `KEY_QUIT`), and add
  the c0011 `Ctrl+Y` constant (`KEY_QUERY_HISTORY`). This satisfies the "at minimum the globals"
  requirement; full per-pane handler migration is optional.
- `src/ui/help.rs` section headers (emoji + labels) are preserved as static data within
  `keybindings.rs` per context group, so layout is unchanged.

## Impact

- Affected code:
  - `src/keybindings.rs` (new file)
  - `src/ui/help.rs` (refactor internals; public API `create_left_column`, `create_right_column`,
    `render_help` signatures unchanged)
  - `src/app/handlers/global.rs` (point key chars at constants)
  - `src/lib.rs` (add `pub mod keybindings;`)
- Affected specs: `help-system` (MODIFIED — help content now derived from the binding table)
- Risk: low-medium — `help.rs` is rendering-only; no runtime behavior changes to key dispatch.
  Main risk is a missing entry in the table causing a help section to go blank; mitigated by
  exhaustive entry list grounded in current handler source.
- Depends on: c0005 (AppState split — needed so `execute_query_at_cursor` and global handler
  paths are stable before adding constants to them)
