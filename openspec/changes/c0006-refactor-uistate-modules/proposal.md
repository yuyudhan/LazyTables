# Split UIState into Cohesive Modules

## Why

`src/state/ui.rs` is 2018 LOC and plays seven distinct roles in a single file:
persistence (save/load), pane-focus navigation, hierarchical table item selection and
vim-style navigation, sequence-search across three panes (tables, connections, SQL
files), SQL-file rename/create input modes, overlay management (debug view, connection
form, help modal), and the explicit ratatui `ListState` bridge.

All seven concerns share the one `UIState` struct. There is no module boundary
separating them, which makes it hard to locate a feature, hard to reason about which
fields a given section touches, and hard to add tests at the concern level.

The existing file comment at `src/state/ui.rs:200` describes `UIState` as "All UI-related
state that can be saved/restored" — a scope that admits everything and bounds nothing.

The `state_new/` experiment deleted in c0002 proved the compiler accepts split
`impl` blocks across sibling modules; this proposal applies that pattern correctly.

## What Changes

- Convert `src/state/ui.rs` into `src/state/ui/mod.rs` + seven sibling modules:
  - `mod.rs` — `UIState` struct with **all fields**, `new()`, `Default`, `reset()`, enum types `HelpMode`/`HelpPaneFocus`, and re-exports of types defined in siblings.
  - `persistence.rs` — `save`, `load`, `state_file_path` (lines 379–424 in the original).
  - `focus.rs` — `cycle_focus_forward`, `cycle_focus_backward`, `move_focus_left`, `move_focus_down`, `move_focus_up`, `move_focus_right`, and the private `update_focus` helper (lines 426–672).
  - `selection.rs` — `SelectableTableItem` struct + impl, `FocusedPane` enum + impl, all selection-index navigation (`connection_down/up`, `table_down/up`, etc.), `build_selectable_table_items`, vim navigation (`table_go_to_first`, `table_go_to_last`, `handle_g_key_press`, `cancel_pending_gg`), and view-transition helpers (`return_to_main`, `show_overlay`, `is_in_overlay`, `is_in_main`) (lines 32–167, 674–1088, 1228–1265).
  - `search.rs` — `matches_sequence` private fn, all search entry/exit/add/backspace methods and filtered-list updaters for tables, connections, and SQL files (lines 8–29, 1089–1174, 1456–1604 connections search; SQL-files search 1269–1309).
  - `sql_files.rs` — rename/create input mode methods and `clear_sql_files_input_modes` (lines 1311–1371).
  - `overlay.rs` — debug-view scroll, connection-form scroll (field `connection_mode_scroll_offset` is present; note c0002 may later remove it), help-modal navigation (lines 1373–1454, 1606–1681).
  - `list_states.rs` — `update_connection_selection`, `update_table_selection`, and `update_tables_list_state_selection` (private), the three methods that synchronise application-level indices with ratatui `ListState` (lines 674–702, 1049–1059).
- Move the five `#[cfg(test)]` tests at lines 1690–2017 into the files that hold the code they exercise: the four search tests (`test_matches_sequence`, `test_tables_search_functionality`, `test_tables_search_with_j_k_characters`, `test_connections_search_functionality`) move to `search.rs`; the vim-navigation test (`test_vim_navigation_commands`) moves to `selection.rs`.
- Keep `src/state/view.rs` name unchanged (no churn; it is already clean and separate).
- Keep `src/state/mod.rs` unchanged (228 bytes; only re-exports).

No runtime or public API change. All existing external callers import `UIState` and its public types from `crate::state::ui`; the `mod.rs` re-exports ensure these paths resolve identically after the split.

## Impact

- Affected code:
  - `src/state/ui.rs` → replaced by `src/state/ui/` module tree (8 files)
  - `src/state/mod.rs` — unchanged (its `pub mod ui;` declaration remains valid for a directory module)
  - No call-sites outside `state/` need updating; `UIState`, `FocusedPane`, `SelectableTableItem`, `HelpMode`, `HelpPaneFocus` are all re-exported from `src/state/ui/mod.rs`
- Affected specs: none (internal refactor, behavior preserved)
- Risk: medium — the central UI state struct is touched; an incorrect move of a private helper or a missing `pub(super)` breaks the build immediately, which makes errors obvious and caught before the Verify step
- Depends on: c0002 (the `state_new/` experiment, `connection_mode` dead code, and logging macro migration in c0002 clear several call-sites that would otherwise tangle this split; the `connection_mode_scroll_offset` field and scroll methods in `overlay.rs` are retained here and removed by c0002 if that proposal deletes the field — this proposal documents the conditional in overlay.rs)
