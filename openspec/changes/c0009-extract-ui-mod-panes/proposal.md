# Pull Inline Pane Rendering Out of ui/mod.rs

## Why

`src/ui/mod.rs` (1416 lines) hosts five pane-rendering methods inline alongside layout, modal,
and orchestration code, violating the same delegation principle the rest of the file already
follows: `draw_tables_pane` (line 455) delegates to `components::render_tables_pane`;
`draw_tabular_output` (line 852) delegates to `components::render_table_viewer`;
`draw_query_window` (line 1187) delegates to the `QueryEditor` component. The remaining three
panes are God methods:

- `draw_connections_pane` (lines 286–453, 168 LOC): builds list items, status icons, the
  connecting animation, and the search-mode header, all inline.
- `draw_details_pane` + `build_comprehensive_table_details` (lines 462–849, 388 LOC): the
  details pane renderer and a 240-line metadata-builder helper live together.
- `draw_sql_files_pane` + `get_file_metadata` (lines 967–1185, 219 LOC): the SQL-files list,
  three input-mode overlays (search/rename/create), and a file-metadata formatter helper.

Adding or debugging any pane requires navigating a 1400-line file. Extracting them aligns with
the existing pattern and makes each pane independently maintainable.

## What Changes

- Add `src/ui/components/connections_pane.rs` containing a `pub fn render_connections_pane`
  extracted from `draw_connections_pane` (ui/mod.rs:286–453). The function signature mirrors
  existing component signatures: `(frame: &mut Frame, area: Rect, state: &mut AppState, theme:
  &Theme)`, plus animation params when executed after c0005 (see design.md).
- Add `src/ui/components/details_pane.rs` containing `pub fn render_details_pane` extracted
  from `draw_details_pane` (ui/mod.rs:462–599) and a `pub(super) fn
  build_comprehensive_table_details` extracted from the same-named method (ui/mod.rs:602–849).
  Signature: `render_details_pane(frame: &mut Frame, area: Rect, state: &mut AppState, theme:
  &Theme)`.
- Add `src/ui/components/sql_files_pane.rs` containing `pub fn render_sql_files_pane` extracted
  from `draw_sql_files_pane` (ui/mod.rs:967–1144) and a `fn get_file_metadata(path:
  &std::path::Path) -> (String, String)` module-private helper extracted from
  `get_file_metadata` (ui/mod.rs:1147–1185). Signature: `render_sql_files_pane(frame: &mut
  Frame, area: Rect, state: &AppState, theme: &Theme)`.
- `UI::draw` (ui/mod.rs:180–269) calls the new `render_*` functions instead of the removed
  inline methods. The disabled/empty-state placeholder widgets stay co-located inside each new
  component file.
- Register the three new modules in `src/ui/components/mod.rs` with `pub mod` + `pub use`.
- Delete the now-empty `draw_connections_pane`, `build_comprehensive_table_details`,
  `draw_details_pane`, `draw_sql_files_pane`, and `get_file_metadata` inherent methods from
  `src/ui/mod.rs`.

No rendering logic, layout constraints, theme key names, or widget parameters change. **No
visual change.**

## Impact

- Affected code: `src/ui/mod.rs`, `src/ui/components/mod.rs`, (new)
  `src/ui/components/connections_pane.rs`, `src/ui/components/details_pane.rs`,
  `src/ui/components/sql_files_pane.rs`.
- Affected specs: none (internal refactor, behavior preserved).
- Risk: medium — touching the main render path; a missed `use` or wrong mutability breaks
  compilation, but the refactor is mechanical with no logic changes.
- Depends on: c0008 (serializes `src/ui/components/mod.rs` edits; must land first to avoid
  collision on that file).
