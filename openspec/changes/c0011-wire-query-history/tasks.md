# Tasks: Record queries and add a history overlay

## 1. Add `query_history` field to `AppState` and initialize it

- [ ] 1.1 In `src/app/state.rs` (post-c0005: `src/app/state/mod.rs`), add
      `query_history: Option<QueryHistoryManager>` to the `AppState` struct
      after the `app_state_db` field (line 50). Import
      `crate::persistence::QueryHistoryManager`.
- [ ] 1.2 In `AppState::new()` (line 87 block), add
      `query_history: QueryHistoryManager::new().ok()` to the struct literal.
      A construction failure (e.g. no home dir) yields `None` and is silently
      tolerated.
- [ ] 1.3 In `AppState::initialize_app_db()` (line 111), after the existing
      `AppStateDb` initialization block, add:
      ```rust
      if let Some(qh) = &mut self.query_history {
          if let Err(e) = qh.initialize().await {
              tracing::warn!("query history init failed: {}", e);
          }
      }
      ```
      A failure leaves `pool: None` inside the manager; subsequent `add_query`
      calls will return `Err` which the recording hook discards (see task 2.2).

## 2. Recording hook with execution timing

- [ ] 2.1 In `execute_query_at_cursor` (`src/app/state.rs:2013`; post-c0005:
      `src/app/state/query_editor.rs`), capture start time immediately before
      the `connection_manager.execute_raw_query()` call:
      ```rust
      let query_start = std::time::Instant::now();
      ```
- [ ] 2.2 After each branch (`Ok` and `Err`) of the `match` on
      `execute_raw_query`, compute elapsed time and record the entry. Extract
      `db_type` and `db_name` from the already-resolved `connection`
      (`ConnectionConfig`) in scope:
      ```rust
      let exec_ms = query_start.elapsed().as_millis()
          .min(i64::MAX as u128) as i64;
      if let Some(qh) = &self.query_history {
          if let Err(e) = qh.add_query(
              &query,
              connection.database_type.clone(),
              connection.database.as_deref(),
              Some(exec_ms),
              /* success */ true_or_false,
              /* error_message */ None_or_Some_str,
          ).await {
              tracing::warn!("query history recording failed: {}", e);
          }
      }
      ```
      In the `Ok` branch: `success = true`, `error_message = None`.
      In the `Err(ref e)` branch: `success = false`,
      `error_message = Some(e.to_string().as_str())`. The recording call is
      placed after the toast/debug-message calls so their text is unaffected.
      Recording error is logged and discarded; the branch's original
      `Ok(())`/`Err(e.to_string())` return is unchanged.

## 3. Add `OverlayView::QueryHistory` variant and predicate

- [ ] 3.1 In `src/state/view.rs`, add `QueryHistory` to the `OverlayView`
      enum after `Help` (line 29):
      ```rust
      /// Query history overlay
      QueryHistory,
      ```
- [ ] 3.2 Add `is_query_history()` predicate to `impl AppView`
      (after `is_help()` at line 98):
      ```rust
      /// Check if in query history overlay
      pub fn is_query_history(&self) -> bool {
          matches!(self, Self::Overlay(OverlayView::QueryHistory))
      }
      ```
- [ ] 3.3 Add arm to `OverlayView::display_name()` (line 110):
      ```rust
      Self::QueryHistory => "Query History",
      ```
      Verify no exhaustiveness warnings (`cargo check` in isolation if allowed;
      otherwise confirm by reading the match).

## 4. New overlay component `src/ui/components/query_history.rs`

- [ ] 4.1 Create `src/ui/components/query_history.rs`. Define
      `QueryHistoryView` struct:
      ```rust
      pub struct QueryHistoryView {
          /// Cached entries loaded when the overlay opens (newest first)
          pub entries: Vec<crate::persistence::QueryHistoryEntry>,
          /// Currently selected entry index (0 = topmost/newest)
          pub selected_index: usize,
      }
      ```
- [ ] 4.2 Implement `QueryHistoryView::new() -> Self` (empty entries,
      `selected_index: 0`) and `Default`.
- [ ] 4.3 Implement `QueryHistoryView::render(&self, frame, area, theme)`:
      - Call `frame.render_widget(Clear, area)` to dim the background.
      - Draw a bordered block titled `" Query History (Ctrl+Y to toggle) "`,
        matching the centering style of `DebugView` (`debug_view.rs:66–77`).
      - Split `inner_area` vertically:
        `[Constraint::Min(5), Constraint::Length(3)]` (entries list, help
        footer).
      - In the entries area: if `self.entries` is empty, render a `Paragraph`
        with text `"No query history yet. Execute queries with Ctrl+Enter."`.
        Otherwise render a `List` of `ListItem`s. Each item line:
        `[HH:MM:SS] [PG/MY/SQ] [OK/ERR] [N ms] query_text…` where
        `query_text` is truncated at 80 chars with `…`. Highlight the
        `selected_index` row with `theme.get_color("primary_highlight")`
        foreground and `theme.get_color("selection_bg")` background (matching
        existing list highlight patterns in `tables_pane.rs`). Use
        `List::new(items).highlight_style(…)` with a stateful `ListState`
        seeded from `self.selected_index`.
      - Help footer: `Paragraph` centered, text
        `"j/k: Scroll  Ctrl+d/u: Page  Enter: Load into editor  Esc/Ctrl+Y: Close"`.
- [ ] 4.4 Register in `src/ui/components/mod.rs`:
      ```rust
      pub mod query_history;
      pub use query_history::QueryHistoryView;
      ```
- [ ] 4.5 Add `pub query_history_view: QueryHistoryView` field to `AppState`
      (alongside `debug_view: DebugView`); construct with `QueryHistoryView::new()`.

## 5. Keybinding, scroll state, and Enter-to-load

- [ ] 5.1 Add `pub query_history_scroll_offset: usize` to `UIState`
      (`src/state/ui.rs`) adjacent to `debug_view_scroll_offset` (line 255).
      Initialize to `0` in `UIState` construction (adjacent to line 349).
      Post-c0006 this field lives in `src/state/ui/overlay.rs`.
- [ ] 5.2 Add the following methods to `impl UIState` following the
      `toggle_debug_view` / `debug_view_scroll_*` family pattern
      (`src/state/ui.rs:1376–1418`):
      ```rust
      pub fn toggle_query_history(&mut self) {
          if self.current_view.is_query_history() {
              self.return_to_main();
          } else {
              self.query_history_scroll_offset = 0;
              self.show_overlay(crate::state::view::OverlayView::QueryHistory);
          }
      }

      pub fn query_history_scroll_down(&mut self, max_entries: usize) {
          if max_entries > 0 && self.query_history_scroll_offset < max_entries.saturating_sub(1) {
              self.query_history_scroll_offset += 1;
          }
      }

      pub fn query_history_scroll_up(&mut self) {
          if self.query_history_scroll_offset > 0 {
              self.query_history_scroll_offset -= 1;
          }
      }

      pub fn query_history_page_down(&mut self, max_entries: usize, page_size: usize) {
          self.query_history_scroll_offset =
              (self.query_history_scroll_offset + page_size)
                  .min(max_entries.saturating_sub(1));
      }

      pub fn query_history_page_up(&mut self, page_size: usize) {
          self.query_history_scroll_offset =
              self.query_history_scroll_offset.saturating_sub(page_size);
      }
      ```
- [ ] 5.3 In `src/app/handlers/global.rs`, add a `Ctrl+Y` arm before the
      `_ => Ok(None)` catch-all (line 107):
      ```rust
      (KeyModifiers::CONTROL, KeyCode::Char('y')) => {
          let is_opening = !app.state.ui.current_view.is_query_history();
          app.state.ui.toggle_query_history();
          if is_opening {
              // Load entries; failure is non-fatal
              if let Some(qh) = &app.state.query_history {
                  match qh.get_history(None, Some(200)).await {
                      Ok(entries) => {
                          app.state.query_history_view.entries = entries;
                          app.state.query_history_view.selected_index = 0;
                      }
                      Err(e) => tracing::warn!("failed to load query history: {}", e),
                  }
              }
          } else {
              app.state.query_history_view.entries.clear();
          }
          Ok(Some(()))
      }
      ```
      Note: if `handle` in `global.rs` is not `async`, the `get_history` call
      must be refactored to be awaited in the caller (`app/mod.rs` event loop)
      via a pending-action flag or handled in the same async context already
      used for connect operations. Follow whichever async dispatch pattern the
      rest of the handler chain uses for async state mutations.
- [ ] 5.4 Add in-overlay key routing. When
      `app.state.ui.current_view.is_query_history()` is true, route before the
      general `global::handle` fall-through:
      - `j` / `KeyCode::Down` → `app.state.ui.query_history_scroll_down(app.state.query_history_view.entries.len())`; update `app.state.query_history_view.selected_index` to match.
      - `k` / `KeyCode::Up` → `query_history_scroll_up()`; update `selected_index`.
      - `Ctrl+d` → `query_history_page_down(len, 10)` + `selected_index` sync.
      - `Ctrl+u` → `query_history_page_up(10)` + `selected_index` sync.
      - `Enter` → if `selected_index < entries.len()`, call
        `app.state.query_editor.set_content(entry.query_text.clone())`,
        `app.state.ui.return_to_main()`, clear `entries`. Return `Ok(Some(()))`.
      - `Esc` / `Ctrl+Y` → `app.state.ui.return_to_main()`, clear `entries`.
- [ ] 5.5 Wire the overlay render call. In `src/ui/mod.rs` (or wherever
      `DebugView::render` is dispatched for `OverlayView::DebugView`), add a
      corresponding arm for `OverlayView::QueryHistory` that calls
      `app.state.query_history_view.render(frame, area, theme)`.

## 6. Help entry

- [ ] 6.1 In `src/ui/help.rs`, add to the globals section:
      `Ctrl+Y` → `"Toggle query history overlay"`. Position it adjacent to the
      `Ctrl+B` debug-view entry so global overlay keys are grouped. Verify the
      help overlay still scrolls cleanly with the additional entry.

## 7. Spec delta

- [ ] 7.1 Confirm `openspec/changes/c0011-wire-query-history/specs/query-history/spec.md`
      exists and contains the `query-history` ADD requirements with the
      scenarios defined in the proposal. (Authored alongside this tasks.md.)

## 8. Verify

- [ ] 8.1 `cargo build` clean
- [ ] 8.2 `cargo clippy --all-targets -- -D warnings` clean
- [ ] 8.3 `cargo fmt --check` clean
- [ ] 8.4 `cargo test` green (includes `database::query_history` tests, moved
      to `persistence::query_history` by c0010)
- [ ] 8.5 Run app against a SQLite file DB: execute `SELECT 1` (ok), `SELECT 2`
      (ok), and `SELECT * FROM nonexistent_table` (err) via `Ctrl+Enter`. Press
      `Ctrl+Y` — overlay opens showing all three entries newest-first; the
      failing query shows `ERR`. Navigate with `j`/`k` to the `SELECT 1`
      entry, press `Enter` — SQL editor contains `SELECT 1` and overlay is
      closed. Quit and reopen the app; press `Ctrl+Y` — all three entries
      persist. Confirm `Ctrl+B` debug view still opens and shows tracing output
      from the recording hook.
