# Delete dead code & scaffolding

## Why

Three whole subsystems and assorted scaffolding are unreachable, inflating the codebase by roughly 3 000 LOC and hiding the real architecture from anyone reading the code.

**Confirmed dead (no live reference outside their own files or each other):**

- `src/app/state_new/` (7 files, ~460 LOC) — `app/mod.rs` declares only `pub mod handlers` and `pub mod state`; `state_new` is never declared and therefore never compiled into the module tree.
- `src/ui/components/connection_mode.rs` (826 LOC) — the `AppState.connection_mode` field (`state.rs:48`) is `None` at both construction sites (`state.rs:97` and `state.rs:2182`) and is never set to `Some`. The render guard at `ui/mod.rs:245–256` (`if let Some(connection_mode) = &state.connection_mode`) therefore never fires. Simultaneously, `ui/mod.rs:228` contains a literal copy-paste tautology: `state.ui.current_view.is_connection_form() || state.ui.current_view.is_connection_form()`.
- `src/commands/` (6 files — `mod.rs`, `basic.rs`, `connection.rs`, `navigation.rs`, `query.rs`, `editing.rs`, ~1 700 LOC) — only two `CommandId` variants are ever dispatched: `ToggleHelp` (handlers/global.rs:17) and `Help` (app/mod.rs:183 via `handle_command_action`). Both perform straightforward `UIState` mutations that can be inlined directly; the entire registry, trait tower, and remaining commands are never reached.
- `src/app/events.rs` (`AppEvent` enum, 110 LOC) — zero references outside the file itself. The live inter-task protocol (`ConnectionEvent`, `TestConnectionEvent`) stays as private enums in `app/mod.rs`.
- `src/themes/mod.rs` (1 line: `pub use crate::ui::theme::*`) — a deprecated shim; `pub mod themes` in `lib.rs:16` is its only entry point. No file in `src/` contains a `crate::themes::` import.
- Dead functions: `EventHandler::start()` (`event/mod.rs:90`, body is just `Ok(())`), `terminal::clear_screen()` (`terminal.rs:57`, never called), `constants::version_string()` (`constants.rs:10`, its one call-site at `ui/mod.rs:273` can be replaced with an inline `format!`).
- No-op logging macros `log_debug!` / `log_info!` / `log_warn!` / `log_error!` / `log_span!` (`logging.rs:272–309`) — each is a thin alias for the corresponding `tracing::` macro. Every call-site should call `tracing::{debug,info,warn,error}!` directly.

## What Changes

- Delete `src/app/state_new/` (all 7 files). No module declaration exists in `app/mod.rs`, so no source file need be edited for this step.
- Delete `src/ui/components/connection_mode.rs`. Remove `connection_mode: Option<ConnectionMode>` from `AppState` (`state.rs:48`, init at `:97` and `:2182`). Remove the dead render block `ui/mod.rs:245–256`. Fix the tautology at `ui/mod.rs:228` to a single `is_connection_form()` call. Remove the `connection_mode_scroll_offset` field from `UIState` (`state/ui.rs:257`, init at `:350`), its two scroll methods (`connection_mode_scroll_down` at `:1443`, `connection_mode_scroll_up` at `:1450`), and all three reset assignments inside `enter_add_connection_mode` (`:1422`), `enter_edit_connection_mode` (`:1430`), and the connection-mode close method (`:1439`).
- Delete `src/commands/` (entire directory). Inline `ToggleHelp` behaviour directly in `handlers/global.rs:17`: check `app.state.ui.help_mode != HelpMode::None`; if so, reset `help_mode = HelpMode::None` and call `app.state.ui.return_to_main()`; otherwise set `help_mode` by focused pane and call `app.state.ui.show_overlay(OverlayView::Help)` + `app.state.ui.reset_help_modal_state()` (exact state mutations from `commands/basic.rs:60–82` and `:106–119`). Remove from `app/mod.rs`: the `command_registry: CommandRegistry` field, the `CommandRegistry::new()` call in `App::new()`, `execute_command()`, `handle_command_action()`, and the `crate::commands::*` import at line 4. Remove `pub mod commands;` from `src/lib.rs:5`.
- Delete `src/app/events.rs`. Remove `pub mod events;` from `src/app/mod.rs` (no such declaration currently exists in `app/mod.rs`, but confirm no stray `use crate::app::events` import exists).
- Delete `src/themes/mod.rs`. Remove `pub mod themes;` from `src/lib.rs:16`. No callers to repoint (grep confirms zero `crate::themes::` references).
- Delete `EventHandler::start()` from `src/event/mod.rs:90–92`. Remove the `self.event_handler.start()?` call at `app/mod.rs:108`. Delete `terminal::clear_screen()` from `src/terminal.rs:57–63`. Replace `constants::version_string()` at `ui/mod.rs:273` with `format!("{} v{}", constants::APP_NAME, constants::VERSION)`, then delete `version_string()` from `src/constants.rs:10–12`.
- Delete macros `log_debug!` / `log_info!` / `log_warn!` / `log_error!` / `log_span!` from `src/logging.rs:272–309`. Replace every `crate::log_debug!` / `crate::log_info!` / etc. call-site with the corresponding `tracing::debug!` / `tracing::info!` / etc. call. Call-sites confirmed in: `src/app/state.rs`, `src/database/postgres.rs`, `src/io/async_fs.rs`, `src/state/database.rs`, `src/state/ui.rs`, `src/ui/components/table_viewer.rs`. **Keep** `add_debug_message()`, `MemoryLogLayer`, and `DebugLogStorage` — these feed the live debug overlay.

## Impact

- Affected code:
  - `src/app/state_new/` (deleted)
  - `src/ui/components/connection_mode.rs` (deleted)
  - `src/commands/` (deleted)
  - `src/app/events.rs` (deleted)
  - `src/themes/mod.rs` (deleted)
  - `src/lib.rs` — remove `pub mod commands;` (line 5) and `pub mod themes;` (line 16)
  - `src/app/mod.rs` — strip command plumbing; remove `start()` call
  - `src/app/state.rs` — remove `connection_mode` field + inits; migrate `log_debug!` macros
  - `src/app/handlers/global.rs` — inline help-toggle; remove `CommandId` import
  - `src/ui/mod.rs` — remove dead render block; fix tautology; replace `version_string()` call
  - `src/state/ui.rs` — remove `connection_mode_scroll_offset` field + scroll methods; migrate `log_debug!`
  - `src/event/mod.rs` — remove `start()` fn
  - `src/terminal.rs` — remove `clear_screen()` fn
  - `src/constants.rs` — remove `version_string()` fn
  - `src/logging.rs` — remove five macro definitions
  - `src/database/postgres.rs`, `src/io/async_fs.rs`, `src/state/database.rs`, `src/ui/components/table_viewer.rs` — migrate `crate::log_*!` call-sites to `tracing::`
- Affected specs: none (internal refactor, behavior preserved)
- Risk: low — all deleted code is confirmed unreachable; the only user-visible behavior touched is `?` toggling the help overlay, which is preserved by the inline
- Depends on: none
