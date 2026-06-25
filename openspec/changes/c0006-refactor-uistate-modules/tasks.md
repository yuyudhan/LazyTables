# Tasks: Split UIState into Cohesive Modules

## 1. Create `src/state/ui/` skeleton — struct, fields, `new()`, `reset()`, enums

- [ ] 1.1 Create directory `src/state/ui/` and the file `src/state/ui/mod.rs`
- [ ] 1.2 Move `HelpMode` enum (original `ui.rs` lines 169–186) and `HelpPaneFocus`
      enum (lines 188–195) into `mod.rs`
- [ ] 1.3 Move the `UIState` struct definition with all 37 fields (lines 200–318) into
      `mod.rs`; confirm all six `#[serde(skip)]` attributes (`confirmation_modal`,
      `selectable_table_items`, `filtered_table_items`, `pending_gg_command`,
      `connections_list_state`, `tables_list_state`) are present and correct
- [ ] 1.4 Move `new()` (lines 321–377), `reset()` (lines 757–760), and `Default` impl
      (lines 1684–1688) into `mod.rs`
- [ ] 1.5 Add `pub mod` declarations for all seven sibling modules in `mod.rs`:
      `persistence`, `focus`, `selection`, `search`, `sql_files`, `overlay`,
      `list_states`
- [ ] 1.6 Add re-exports in `mod.rs`: `pub use selection::{FocusedPane,
      SelectableTableItem};` so external paths (`crate::state::ui::FocusedPane`, etc.)
      resolve identically to before
- [ ] 1.7 Confirm `src/state/mod.rs` (228 bytes) is unchanged — its `pub mod ui;`
      declaration is already valid for the directory module

## 2. Move focus methods into `src/state/ui/focus.rs`

- [ ] 2.1 Create `src/state/ui/focus.rs` with `use super::*;` (or targeted use of
      `UIState`, `FocusedPane`)
- [ ] 2.2 Move the following `impl UIState` methods into `focus.rs`:
      `cycle_focus_forward` (lines 426–469), `cycle_focus_backward` (lines 471–513),
      `move_focus_left` (lines 516–559), `move_focus_down` (lines 561–594),
      `move_focus_up` (lines 597–615), `move_focus_right` (lines 617–659), and the
      private `update_focus` helper (lines 661–672)
- [ ] 2.3 Verify `update_focus` is declared `fn` (private within the crate) — it is
      only called from within `focus.rs` itself, so no `pub(super)` needed

## 3. Move selection / type-def methods into `src/state/ui/selection.rs`

- [ ] 3.1 Create `src/state/ui/selection.rs` with needed imports
- [ ] 3.2 Move `SelectableTableItem` struct + impl (lines 31–87) and `FocusedPane` enum
      + impl (lines 89–167) into `selection.rs`
- [ ] 3.3 Move `impl UIState` methods: `update_sql_file_selection` (lines 703–711),
      `connection_down/up` (lines 713–732), `table_down/up` (lines 734–754),
      `return_to_main` (lines 762–764), `show_overlay` (lines 767–769),
      `is_in_overlay` (lines 772–774), `is_in_main` (lines 777–779),
      `toggle_schema_expansion` / `is_schema_expanded` (lines 782–793),
      `toggle_object_group_expansion` / `is_object_group_expanded` (lines 796–807),
      `build_selectable_table_items` (lines 810–954), `find_first_selectable_index`
      (lines 957–964, private), `table_selection_down` (lines 967–999),
      `table_selection_up` (lines 1002–1046), `get_selected_table_item` (lines
      1061–1070), `get_selected_item_raw` (lines 1073–1080), `get_selected_table_name`
      (lines 1083–1087), `table_go_to_first` (lines 1228–1232), `table_go_to_last`
      (lines 1235–1247), `handle_g_key_press` (lines 1250–1258), `cancel_pending_gg`
      (lines 1261–1263)
- [ ] 3.4 Declare `find_first_selectable_index` as `fn` (private); it is called only
      within `selection.rs`. Declare `update_tables_list_state_selection` in
      `list_states.rs` (step 5) as `pub(super)` because `selection.rs` calls it

## 4. Move search methods and SQL-files input methods

- [ ] 4.1 Create `src/state/ui/search.rs` with `use super::*;` and the `fn
      matches_sequence` private free function (lines 8–29) declared `pub(super)` within
      `search.rs` (or kept private since all callers in `search.rs` are in the same file)
- [ ] 4.2 Move tables-search `impl UIState` methods into `search.rs`:
      `enter_tables_search` (lines 1089–1094), `exit_tables_search` (lines 1097–1108),
      `add_to_tables_search` (lines 1111–1121), `backspace_tables_search` (lines
      1124–1133), `update_filtered_table_items` (lines 1136–1164, private),
      `get_display_table_items` (lines 1167–1173), `table_search_selection_down` (lines
      1176–1197), `table_search_selection_up` (lines 1200–1225)
- [ ] 4.3 Move SQL-files search methods into `search.rs`:
      `enter_sql_files_search` (lines 1268–1271), `exit_sql_files_search` (lines
      1274–1277), `add_to_sql_files_search` (lines 1280–1283), `backspace_sql_files_search`
      (lines 1287–1290), `filter_sql_files` (lines 1294–1308)
- [ ] 4.4 Move connections-search methods into `search.rs`:
      `enter_connections_search` (lines 1458–1465), `exit_connections_search` (lines
      1468–1472), `add_to_connections_search` (lines 1475–1479),
      `backspace_connections_search` (lines 1482–1486), `update_filtered_connections`
      (lines 1489–1517), `get_display_connections` (lines 1520–1534),
      `connections_selection_down` (lines 1537–1558), `connections_selection_up` (lines
      1561–1583), `get_selected_connection_index` (lines 1586–1603)
- [ ] 4.5 Create `src/state/ui/sql_files.rs` and move the rename/create input mode
      methods: `enter_sql_files_rename`, `exit_sql_files_rename`,
      `add_to_sql_files_rename`, `backspace_sql_files_rename` (lines 1311–1334),
      `enter_sql_files_create`, `exit_sql_files_create`, `add_to_sql_files_create`,
      `backspace_sql_files_create` (lines 1337–1363), `clear_sql_files_input_modes`
      (lines 1366–1370)
- [ ] 4.6 Confirm `clear_sql_files_input_modes` calls `exit_sql_files_search`,
      `exit_sql_files_rename`, `exit_sql_files_create` — all defined within the same
      `impl UIState` context; call paths are unchanged (`self.exit_sql_files_search()`
      resolves across impl blocks in the same crate)

## 5. Move overlay and list-state bridge methods

- [ ] 5.1 Create `src/state/ui/overlay.rs` and move: `toggle_debug_view` (lines
      1375–1382), `debug_view_scroll_down` (lines 1385–1389), `debug_view_scroll_up`
      (lines 1392–1396), `debug_view_page_down` (lines 1399–1402),
      `debug_view_page_up` (lines 1405–1407), `debug_view_go_to_top` (lines
      1410–1412), `debug_view_go_to_bottom` (lines 1415–1417),
      `enter_add_connection_mode` (lines 1420–1425), `enter_edit_connection_mode`
      (lines 1428–1433), `exit_connection_mode` (lines 1436–1439),
      `connection_mode_scroll_down` (lines 1442–1445), `connection_mode_scroll_up`
      (lines 1448–1453), `toggle_help_pane_focus` (lines 1608–1613),
      `reset_help_modal_state` (lines 1616–1620), `help_scroll_down` (lines 1623–1636),
      `help_scroll_up` (lines 1639–1652), `help_page_down` (lines 1655–1666),
      `help_page_up` (lines 1669–1680)
- [ ] 5.2 Note: if c0002 has already removed `connection_mode_scroll_offset` from the
      struct, omit `connection_mode_scroll_down/up` and `enter/exit_connection_mode`'s
      reset of that field; otherwise include them as-is
- [ ] 5.3 Create `src/state/ui/list_states.rs` and move: `update_connection_selection`
      (lines 674–686), `update_table_selection` (lines 689–701), and
      `update_tables_list_state_selection` (lines 1049–1058, private). Add `use
      ratatui::widgets::ListState;` import if not already in scope via `super::*`
- [ ] 5.4 Declare `update_tables_list_state_selection` as `pub(super)` — it is called
      from `selection.rs` and `search.rs` (both siblings), so `pub(super)` grants
      access within `state::ui` without leaking to the crate

## 6. Relocate tests and delete original file

- [ ] 6.1 Move the four search-related tests from `ui.rs` lines 1694–1945 into a
      `#[cfg(test)] mod tests { use super::*; … }` block at the bottom of
      `search.rs`: `test_matches_sequence`, `test_tables_search_functionality`,
      `test_tables_search_with_j_k_characters`, `test_connections_search_functionality`
- [ ] 6.2 Move the vim-navigation test from `ui.rs` lines 1948–2016 into a
      `#[cfg(test)] mod tests { use super::*; … }` block at the bottom of
      `selection.rs`: `test_vim_navigation_commands`
- [ ] 6.3 Confirm each `use super::*;` in the relocated test blocks resolves correctly:
      `super` in a file-level `mod tests` refers to the enclosing module, which in turn
      has `use super::*;` from `mod.rs` — all `UIState`, `FocusedPane`,
      `SelectableTableItem`, `DatabaseObjectType`, `ConnectionConfig` etc. must be in
      scope. Add targeted `use` statements where the glob is insufficient
- [ ] 6.4 Delete `src/state/ui.rs` (the original flat file is fully replaced by the
      directory module)

## 7. Verify

- [ ] 7.1 `cargo build` clean
- [ ] 7.2 `cargo clippy --all-targets -- -D warnings` clean
- [ ] 7.3 `cargo fmt --check` clean
- [ ] 7.4 `cargo test` green — confirm the five UIState tests pass in their new files
      (`search.rs` × 4, `selection.rs` × 1) by checking output for
      `state::ui::search::tests::*` and `state::ui::selection::tests::*`
- [ ] 7.5 Behavioral smoke-test: run the application and verify: (a) Tab cycles pane
      focus through Connections → Tables → Details → TabularOutput → QueryWindow →
      SqlFiles and back, skipping disabled panes; (b) `/` in the tables pane opens
      search, typing characters narrows the list via sequence matching, `j`/`k`
      navigate, Esc exits; (c) `gg` goes to the first table, `G` goes to the last;
      (d) quit and reopen — `focused_pane`, `selected_connection`, and expanded-schema
      state are restored from `~/.config/lazytables/ui_state.json` (persistence path
      confirmed in `state_file_path()` at original line 417)
