# Tasks: Delete dead code & scaffolding

## 1. Delete `state_new/` and confirm no module declaration

- [ ] 1.1 Delete all 7 files under `src/app/state_new/` (`mod.rs`, `connections.rs`, `modals.rs`, `navigation.rs`, `query.rs`, `sql_files.rs`, `tables.rs`) and the directory itself.
- [ ] 1.2 Confirm `src/app/mod.rs` contains no `pub mod state_new;` declaration (grep; expected zero — this step is verification only, no edit required).

## 2. Delete `connection_mode.rs`, remove its field, fix the tautology, remove scroll offset

- [ ] 2.1 Delete `src/ui/components/connection_mode.rs`.
- [ ] 2.2 In `src/app/state.rs`: remove the field `pub connection_mode: Option<ConnectionMode>` (line 48) and its `ConnectionMode` import.
- [ ] 2.3 In `src/app/state.rs`: remove `connection_mode: None` from `AppState::new()` (line 97) and from `AppState::default()` (line 2182).
- [ ] 2.4 In `src/state/ui.rs`: remove the field `pub connection_mode_scroll_offset: usize` (line 257) and its initialiser `connection_mode_scroll_offset: 0` in `UIState::new()` (line 350).
- [ ] 2.5 In `src/state/ui.rs`: remove the `connection_mode_scroll_offset = 0` reset assignments from `enter_add_connection_mode` (line 1422), `enter_edit_connection_mode` (line 1430), and the connection-mode close method (line 1439).
- [ ] 2.6 In `src/state/ui.rs`: delete the `connection_mode_scroll_down()` method (lines 1442–1447) and the `connection_mode_scroll_up()` method (lines 1449–1454).
- [ ] 2.7 In `src/ui/mod.rs`: delete the dead render block at lines 245–256 (the `// Draw connection mode if active` comment through the closing `}`).
- [ ] 2.8 In `src/ui/mod.rs:228`: fix the tautology — replace `state.ui.current_view.is_connection_form() || state.ui.current_view.is_connection_form()` with a single `state.ui.current_view.is_connection_form()`.

## 3. Delete `commands/`, inline help toggle, strip command plumbing from `app/mod.rs`

- [ ] 3.1 In `src/app/handlers/global.rs`: inline the `ToggleHelp` logic to replace `app.execute_command(CommandId::ToggleHelp)?;` (line 17). The inline must: if `app.state.ui.help_mode != HelpMode::None` then set `help_mode = HelpMode::None` and call `return_to_main()`; else set `help_mode` from `focused_pane` (matching the pane-to-`HelpMode` mapping in `commands/basic.rs:65–72`), call `show_overlay(OverlayView::Help)`, and call `reset_help_modal_state()`.
- [ ] 3.2 In `src/app/handlers/global.rs`: remove the `use crate::commands::CommandId;` import (line 7); add any new imports required by the inlined code (`HelpMode`, `OverlayView`).
- [ ] 3.3 In `src/app/mod.rs`: remove the `crate::commands::{CommandAction, CommandContext, CommandId, CommandRegistry, CommandResult}` import (line 4).
- [ ] 3.4 In `src/app/mod.rs`: remove the `command_registry: CommandRegistry` field from the `App` struct (line 52) and the `command_registry: CommandRegistry::new()` initialiser inside `App::new()` (line 89).
- [ ] 3.5 In `src/app/mod.rs`: delete the `execute_command()` method (lines 147–171) and the `handle_command_action()` method (lines 173–244).
- [ ] 3.6 In `src/lib.rs`: remove `pub mod commands;` (line 5).
- [ ] 3.7 Delete the entire `src/commands/` directory and all 6 files within it.

## 4. Delete `events.rs` / `AppEvent`

- [ ] 4.1 Delete `src/app/events.rs`.
- [ ] 4.2 Grep for `pub mod events` and `use crate::app::events` across `src/` — expect zero matches; no edit needed if confirmed.

## 5. Delete `themes` shim + confirm no callers

- [ ] 5.1 Delete `src/themes/mod.rs`.
- [ ] 5.2 In `src/lib.rs`: remove `pub mod themes;` (line 16).
- [ ] 5.3 Grep `crate::themes::` across `src/` — expect zero matches.

## 6. Delete dead functions

- [ ] 6.1 In `src/event/mod.rs`: delete the `start()` method (lines 89–92, `/// Start the event handler` doc comment through closing `}`).
- [ ] 6.2 In `src/app/mod.rs`: remove `self.event_handler.start()?;` (line 108).
- [ ] 6.3 In `src/terminal.rs`: delete the `clear_screen()` function (lines 56–63, `/// Clear the entire terminal screen` doc comment through closing `}`).
- [ ] 6.4 In `src/ui/mod.rs:273`: replace `constants::version_string()` with `format!("{} v{}", constants::APP_NAME, constants::VERSION)`.
- [ ] 6.5 In `src/constants.rs`: delete the `version_string()` function (lines 9–12, `/// Full version string` doc comment through closing `}`).

## 7. Delete logging macros + migrate all call-sites to `tracing::`

- [ ] 7.1 In `src/logging.rs`: delete the five macro definitions at lines 271–309 (`/// Convenience macros…` comment through the closing `}` of `log_span!`).
- [ ] 7.2 In `src/app/state.rs`: replace every `crate::log_debug!` with `tracing::debug!` (confirmed call-sites in `move_left`, `move_right`, `table_down`, `table_up` and others throughout the file).
- [ ] 7.3 In `src/database/postgres.rs`: replace `crate::log_debug!` with `tracing::debug!` and `crate::log_warn!` with `tracing::warn!` at all call-sites.
- [ ] 7.4 In `src/io/async_fs.rs`: replace `crate::log_debug!` with `tracing::debug!` and `crate::log_error!` with `tracing::error!` at all call-sites.
- [ ] 7.5 In `src/state/database.rs`: replace `crate::log_debug!` with `tracing::debug!` at all call-sites.
- [ ] 7.6 In `src/state/ui.rs`: replace `crate::log_debug!` with `tracing::debug!` at all call-sites.
- [ ] 7.7 In `src/ui/components/table_viewer.rs`: replace `crate::log_debug!` with `tracing::debug!` at all call-sites.
- [ ] 7.8 Grep `log_debug!\|log_info!\|log_warn!\|log_error!\|log_span!` across `src/` — expect zero remaining matches.

## 8. Verify

- [ ] 8.1 `cargo build` clean
- [ ] 8.2 `cargo clippy --all-targets -- -D warnings` clean
- [ ] 8.3 `cargo fmt --check` clean
- [ ] 8.4 `cargo test` green (including `test_connection_modal_state_new` in `src/ui/components/connection_modal.rs`)
- [ ] 8.5 Run the application: press `?` → help overlay opens; press `?` again → it closes. Press `Ctrl+B` → debug overlay opens and shows live log messages (confirms tracing migration intact); press `Ctrl+B` again → it closes.
