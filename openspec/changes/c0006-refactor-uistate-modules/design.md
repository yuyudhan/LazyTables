# Design: Split UIState into Cohesive Modules

## Context

`src/state/ui.rs` (2018 LOC) defines `UIState`, the central UI state object for
LazyTables. It is the second-largest God file in the codebase after `src/app/state.rs`.
It contains:

1. A private `matches_sequence` fn (lines 8–29) used only by table, connection, and
   SQL-file search.
2. Two public type definitions used across the codebase — `SelectableTableItem`
   (lines 32–87) and `FocusedPane` (lines 90–167).
3. Two small enums scoped to overlay navigation — `HelpMode` (lines 170–186) and
   `HelpPaneFocus` (lines 189–195).
4. The `UIState` struct with 37 fields (lines 202–318), six of which carry
   `#[serde(skip)]` — `confirmation_modal`, `selectable_table_items`,
   `filtered_table_items`, `pending_gg_command`, `connections_list_state`,
   `tables_list_state` — and must survive any split with those attrs intact.
5. A single `impl UIState` block spanning lines 320–1682 covering all seven concerns.
6. A `Default` impl (lines 1684–1688) delegating to `new()`.
7. Five unit tests in a `mod tests` block (lines 1690–2017).

`src/state/mod.rs` is 228 bytes and declares `pub mod ui;` (and `pub mod view;`, `pub
mod database;`). Rust accepts a directory module with the same `pub mod ui;` declaration
pointing at `src/state/ui/mod.rs` — no change to `mod.rs` is required.

`src/state/view.rs` (2.8 KB) is a clean, focused file and is not touched.

## Goals / Non-Goals

**Goals:**
- Every method lives in the module whose name describes its concern.
- No public path changes: `crate::state::ui::UIState`, `crate::state::ui::FocusedPane`,
  `crate::state::ui::SelectableTableItem`, `crate::state::ui::HelpMode`,
  `crate::state::ui::HelpPaneFocus` resolve identically after the split.
- `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and
  `cargo test` all pass after the change.
- The five moved tests still pass and remain in files adjacent to the code they exercise.

**Non-goals:**
- No field changes to `UIState` — all 37 fields remain on the struct, `#[serde(skip)]`
  attrs intact.
- No behavior changes of any kind.
- No changes to `src/state/view.rs`, `src/state/mod.rs`, or any file outside
  `src/state/ui/`.
- No new public API surface.

## Decisions

### Decision 1: Convert `ui.rs` to a directory module (`ui/mod.rs`)

**What:** Rename `src/state/ui.rs` → `src/state/ui/mod.rs` and add sibling files in
`src/state/ui/`.

**Why:** Rust's directory-module convention (`mod.rs` inside a directory) is the only
mechanism that allows sibling file modules under `state/ui/`. The parent `state/mod.rs`
declaration `pub mod ui;` needs no change — Rust's module resolver accepts both
`state/ui.rs` and `state/ui/mod.rs` for the same path.

**Alternatives rejected:**
- Keep `ui.rs` and use `#[path]` attributes for inline submodules: works but the
  non-standard `#[path]` usage confuses tooling and rustfmt.
- Use `ui.rs` + a separate `ui/` directory with `mod ui;` inside: the compiler does not
  allow both `ui.rs` and `ui/` to coexist.

---

### Decision 2: All 37 UIState fields stay on the struct in `mod.rs`

**What:** Only `impl UIState` methods move across files. The struct definition,
including all `#[serde(skip)]` attributes, `#[derive(Debug, Clone, Serialize,
Deserialize)]`, and every field declaration (lines 202–318), lives in `mod.rs`.

**Why:** Moving fields to separate structs would break serialization (the `serde` derive
sees the whole struct), require field-access changes at every call-site, and add runtime
indirection. The goal is a pure method split.

**Alternatives rejected:**
- Sub-structs (e.g., `FocusState`, `SearchState` embedded in `UIState`): changes the
  field access syntax at every call-site and cannot be done as a pure rename in a single
  PR without cascading edits. Out of scope.

---

### Decision 3: Module → method assignment

| File | Methods / items |
|---|---|
| `mod.rs` | `UIState` struct + all fields, `new()`, `reset()`, `Default`, `HelpMode`, `HelpPaneFocus` |
| `persistence.rs` | `save`, `load`, `state_file_path` (private) |
| `focus.rs` | `cycle_focus_forward/backward`, `move_focus_left/down/up/right`, `update_focus` (private) |
| `selection.rs` | `SelectableTableItem`, `FocusedPane`, `update_sql_file_selection`, `connection_down/up`, `table_down/up`, `return_to_main`, `show_overlay`, `is_in_overlay`, `is_in_main`, `toggle_schema_expansion`, `is_schema_expanded`, `toggle_object_group_expansion`, `is_object_group_expanded`, `build_selectable_table_items`, `find_first_selectable_index` (private), `table_selection_down/up`, `get_selected_table_item`, `get_selected_item_raw`, `get_selected_table_name`, `table_go_to_first`, `table_go_to_last`, `handle_g_key_press`, `cancel_pending_gg` |
| `search.rs` | `matches_sequence` (private fn), tables search (enter/exit/add/backspace/update/get_display), SQL-files search (enter/exit/add/backspace/filter), connections search (enter/exit/add/backspace/update/get_display/down/up/get_selected_index) |
| `sql_files.rs` | `enter/exit_sql_files_rename`, `add/backspace_sql_files_rename`, `enter/exit_sql_files_create`, `add/backspace_sql_files_create`, `clear_sql_files_input_modes` |
| `overlay.rs` | `toggle_debug_view`, `debug_view_scroll_down/up/page_down/page_up/go_to_top/go_to_bottom`, `enter_add/edit_connection_mode`, `exit_connection_mode`, `connection_mode_scroll_down/up`, `toggle_help_pane_focus`, `reset_help_modal_state`, `help_scroll_down/up/page_down/page_up` |
| `list_states.rs` | `update_connection_selection`, `update_table_selection`, `update_tables_list_state_selection` (private) |

**Why this grouping:** Each module name is a single, unambiguous concern. `list_states.rs`
isolates the ratatui-specific bridge (the only methods that directly call
`ListState::select`), making the ratatui coupling explicit and easy to swap. `search.rs`
keeps `matches_sequence` private within the module rather than promoting it — the three
pane-search modules all live there, so the helper is naturally co-located.

**Alternatives rejected:**
- Merging `sql_files.rs` into `search.rs`: sql-files search is in `search.rs` but
  rename/create modes are orthogonal to search — they deserve their own file.
- `panes.rs` for `FocusedPane` instead of `selection.rs`: the enum is only about pane
  focus for selection purposes; keeping it in `selection.rs` avoids a 4th small file
  that provides no conceptual separation.
- Keeping `return_to_main/show_overlay/is_in_overlay/is_in_main` in `overlay.rs`:
  these are view-transition helpers and would fit there, but the line-range convention
  in the spec (638–1265) groups them with selection; consistent range mapping is more
  predictable for the implementer.

---

### Decision 4: Private helpers use `pub(super)`

**What:** Private methods that move into a sibling file and are called by methods in
a different sibling (e.g., `update_tables_list_state_selection` called from
`selection.rs` and `search.rs`) are declared `pub(super)` instead of `pub` to remain
accessible within `src/state/ui/` without leaking to the crate.

**Why:** `pub(super)` scopes visibility to the parent module (`state::ui`) — callers
outside `ui/` cannot see these helpers. `pub(crate)` would leak them further than
needed. Making them fully `pub` violates encapsulation.

**Alternatives rejected:**
- Consolidating all callers of a helper into the same file: would force unnatural
  groupings that defeat the purpose of the split.

---

### Decision 5: `connection_mode_scroll_offset` field and its methods

**What:** The field `connection_mode_scroll_offset: usize` (line 257) and its two
methods `connection_mode_scroll_down/up` (lines 1443–1454) are included in the split:
the field stays on the struct, the methods go to `overlay.rs`.

**Why:** c0002 may delete this field if it removes the dead `connection_mode`
component. If c0002 lands first (as the dependency chain requires), the implementer of
c0006 will find the field and methods already gone and simply omit them. If c0006 lands
in an environment where c0002 has not yet landed, the field and methods are still live
and must be included. Either way, the split is consistent.

**Alternatives rejected:**
- Conditionally skip the field in c0006: unnecessary complexity — compile errors
  immediately surface any mismatch.

## Edge Cases & Failure Modes

1. **Missing `pub(super)` on a private helper.** `find_first_selectable_index` (called
   from `selection.rs` itself) and `update_tables_list_state_selection` (called from
   both `selection.rs` and `search.rs`) need `pub(super)` in `list_states.rs` or their
   hosting file. A missing annotation is a compile error, not a silent bug.

2. **`#[serde(skip)]` attribute loss.** If any of the six `#[serde(skip)]` fields lose
   their annotation during the struct copy, `UIState::load()` will fail to deserialize
   saved state (ratatui `ListState` is not `Deserialize`). The `cargo test` step
   exercises serialization indirectly; the behavioral check (save → reopen) confirms it.

3. **Duplicate `impl UIState` blocks compiling in different files.** Rust allows
   multiple `impl` blocks for the same type in the same crate, including across
   modules that declare the type, as long as they are in the same crate. The compiler
   will catch any duplicate method name with a clear error.

4. **Import loops within `ui/`.** `selection.rs` uses `FocusedPane` (defined in
   `selection.rs` itself) and `SelectableTableItem` (also in `selection.rs`). `focus.rs`
   uses `FocusedPane` — it must `use super::FocusedPane` (or `use super::selection::FocusedPane`
   if that type is declared there). `mod.rs` must re-export `FocusedPane` and
   `SelectableTableItem` so external callers see them at `crate::state::ui::FocusedPane`.

5. **`crate::log_debug!` macro calls in moved methods.** Several methods in
   `selection.rs` (e.g., `table_selection_down`, `table_selection_up`) call
   `crate::log_debug!`. If c0002 has already migrated these to `tracing::debug!`, the
   implementer uses `tracing::debug!` instead. If c0002 has not landed, `crate::log_debug!`
   remains valid — the file just uses the macro by crate path, which works from any
   module.

6. **Test relocations breaking `super::*`.** The test modules use `use super::*`. When
   moved to `search.rs` and `selection.rs`, `super` refers to the `ui` module
   (via `mod.rs`), which re-exports all needed types — the glob import remains valid.

## Migration / Cutover

**Step 1:** Create `src/state/ui/` directory (Rust needs the directory to exist before
the files).

**Step 2:** Create `src/state/ui/mod.rs` with: the imports currently at `ui.rs:1–6`
(adjusted — remove `use std::fs;` and `use std::path::PathBuf;` if `persistence.rs`
owns those exclusively; add `use ratatui::widgets::ListState;` only if `list_states.rs`
needs it re-exported), the `HelpMode`/`HelpPaneFocus` enums (lines 170–195), the
`UIState` struct (lines 202–318), `new()` (lines 322–377), `reset()` (lines 757–760),
and `Default` impl (lines 1684–1688). Add `pub mod persistence; pub mod focus; pub mod
selection; pub mod search; pub mod sql_files; pub mod overlay; pub mod list_states;`
and re-export `pub use selection::{FocusedPane, SelectableTableItem};`.

**Step 3:** Create each sibling file, moving the `impl UIState` blocks identified in
Decision 3. Each file needs `use super::*;` (or targeted `use super::{UIState, …};`) to
access the struct and its fields.

**Step 4:** Delete the original `src/state/ui.rs` file.

**Step 5:** Verify `src/state/mod.rs` still contains `pub mod ui;` — no edit needed.

**What to delete:** `src/state/ui.rs` in its entirety, replaced by the directory module.

**Callers to verify (grep `use crate::state::ui` across `src/`):**
- `src/app/state.rs` (or `src/app/state/mod.rs` after c0005) imports `UIState`,
  `FocusedPane`, `HelpMode`, `SelectableTableItem`.
- `src/ui/mod.rs`, `src/ui/components/` — import `FocusedPane`, `SelectableTableItem`,
  `UIState` fields via the state ref.
- `src/app/handlers/` — import `FocusedPane`.

All of these are re-exported from `crate::state::ui` (the `ui/mod.rs`), so no import
path changes are needed at call-sites.

## Verification

1. `cargo build` clean with no errors or warnings.
2. `cargo clippy --all-targets -- -D warnings` clean.
3. `cargo fmt --check` clean (no renames, no structural changes, only file splits).
4. `cargo test` green — all five relocated tests pass in their new files.
5. Behavioral smoke-test: run the application and confirm:
   - Pane focus cycling (Tab) works correctly through all six panes, including
     disabled-pane skipping.
   - Table search (`/` in tables pane): sequence matching finds tables correctly; `j`/`k`
     navigate filtered results; Escape exits search.
   - Connection search: same sequence-match behavior in connections pane.
   - SQL-files search: filter_sql_files returns correct subset.
   - `gg` navigates to first selectable table; `G` navigates to last.
   - `Ctrl+B` saves UI state to disk; reopening the app restores `focused_pane`,
     `selected_connection`, `expanded_schemas`, `expanded_object_groups`.
