# Split connection_modal, table_viewer, query_editor

## Why

Three UI components mix type definitions, state logic, and rendering code in single flat files,
each exceeding 1500 lines:

- `src/ui/components/connection_modal.rs` — 2020 lines. Houses three concerns:
  - Type defs (`PasswordStorageType`, `TestConnectionStatus`, `ConnectionField` with `next()`/`prev()`/`display_name()` impls) at lines 17–190.
  - `ConnectionModalState` struct, `Default`, and all parsing/validation/population methods (`get_smart_next_field`, `save_connection_from_modal`, `populate_from_connection`, etc.) at lines 191–875.
  - All `render_*` functions (modal overlay, form fields, tabs, test status) at lines 877–2020, plus its `#[cfg(test)]` block at lines 1678–2020.
- `src/ui/components/table_viewer.rs` — 1773 lines. Houses:
  - `TableViewMode`, `TableTab`, `ColumnInfo` structs at lines 13–55, and `impl TableTab` (creation, edit, search, pagination) at lines 56–517.
  - Remaining data-carrier types (`CellUpdate`, `DeleteConfirmation`, `SetNullConfirmation`, `TableViewerState`) and `impl TableViewerState` at lines 519–782.
  - `Default` impl for `TableViewerState` and all `render_*` functions at lines 778–1773. No `#[cfg(test)]` block.
- `src/ui/components/query_editor.rs` — 1512 lines. Houses:
  - `QueryEditor` struct, `Clone`, and `Default` impls at lines 19–82.
  - All cursor/vim/suggestion/highlighting logic (`impl QueryEditor` non-render methods) at lines 84–1101.
  - `render` and syntax-highlighting helpers at lines 1102–1512, plus its `#[cfg(test)]` block at lines 1273–1512.

Finding boundary concepts, understanding where state ends and rendering begins, and adding new
behaviour all require navigating one monolithic file. The abandoned `src/app/state_new/` directory
demonstrates that Rust `impl` blocks split across sibling modules compiles cleanly — the same
technique applies here.

## What Changes

- `src/ui/components/connection_modal.rs` is replaced by a `src/ui/components/connection_modal/`
  submodule directory containing:
  - `types.rs` — `PasswordStorageType`, `TestConnectionStatus`, `ConnectionField` (lines 17–190 of original).
  - `state.rs` — `ConnectionModalState` struct + `Default` + `impl` + `#[cfg(test)]` (lines 191–875 and 1678–2020 of original).
  - `render.rs` — all `render_*` functions and private helpers (lines 877–1677 of original).
  - `mod.rs` — re-exports every `pub` item so that `crate::ui::components::ConnectionModalState`,
    `::ConnectionField`, `::PasswordStorageType`, `::TestConnectionStatus`, and
    `::render_connection_modal` resolve unchanged.
- `src/ui/components/table_viewer.rs` is replaced by a `src/ui/components/table_viewer/` submodule directory containing:
  - `types.rs` — `TableViewMode`, `TableTab` (struct + `impl`), `ColumnInfo`, `CellUpdate`,
    `DeleteConfirmation`, `SetNullConfirmation` (lines 13–527 of original, covering all non-state types).
  - `state.rs` — `TableViewerState` struct, `impl TableViewerState`, and `Default` (lines 529–782 of original). No existing tests to move.
  - `render.rs` — all `render_*` functions and private helpers (lines 784–1773 of original).
  - `mod.rs` — re-exports every `pub` item so that `crate::ui::components::TableViewerState`,
    `::ColumnInfo`, `::CellUpdate`, `::DeleteConfirmation`, `::SetNullConfirmation`,
    `::TableViewMode`, `::TableTab`, and `::render_table_viewer` resolve unchanged.
- `src/ui/components/query_editor.rs` is replaced by a `src/ui/components/query_editor/` submodule directory containing:
  - `state.rs` — `QueryEditor` struct, `Clone`, `Default` impls (lines 19–82 of original).
  - `input.rs` — `impl QueryEditor` non-render methods: all cursor movement, vim keybindings,
    suggestion engine interaction, and syntax-highlighting helpers (lines 84–1101 of original),
    plus the `#[cfg(test)]` block (lines 1273–1512 of original).
  - `render.rs` — `render` method and `apply_syntax_highlighting_with_line_numbers` (lines 1102–1271 of original).
  - `mod.rs` — re-exports every `pub` item so that `crate::ui::components::QueryEditor` resolves unchanged.
- `src/ui/components/mod.rs` — replace `pub mod connection_modal;`, `pub mod table_viewer;`,
  `pub mod query_editor;` with the path-based module declarations pointing at each new `mod.rs`.
  The glob re-exports (`pub use connection_modal::*;` etc.) remain and propagate all items.
- `src/ui/components/sql_suggestions.rs` and `src/ui/components/suggestion_popup.rs` are **unchanged**
  (confirmed complementary; no duplication).

No rendering or input behaviour changes in this proposal.

## Impact

- Affected code:
  - `src/ui/components/connection_modal.rs` (replaced by `connection_modal/`)
  - `src/ui/components/table_viewer.rs` (replaced by `table_viewer/`)
  - `src/ui/components/query_editor.rs` (replaced by `query_editor/`)
  - `src/ui/components/mod.rs` (module declaration lines)
- Affected specs: none (internal refactor, behavior preserved)
- Risk: medium — the type splits are at clean seams but a missed re-export breaks a caller silently
  until the build; the full `pub use *::*` glob in `mod.rs` mitigates this if any `pub` item is
  omitted from an inner `mod.rs`.
- Depends on: none (ordered after c0002 to avoid rebase friction on `mod.rs` if c0002 removes
  `connection_mode.rs` from the same file; no hard compile dependency)
