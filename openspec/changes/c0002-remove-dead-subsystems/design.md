# Design: Delete dead code & scaffolding

## Context

LazyTables carries three entire subsystems that are never reachable at runtime:

1. **`src/app/state_new/`** — an abandoned attempt at splitting `AppState` across modules. `app/mod.rs` declares only `pub mod handlers` and `pub mod state`; `state_new` is never declared and therefore never compiled. All 7 files exist only on disk.

2. **`src/commands/`** — a full command-registry architecture (~1 700 LOC) with trait objects, a registry, and 30+ command types. In practice the runtime dispatches exactly two `CommandId` variants: `ToggleHelp` (from `handlers/global.rs:17`) and `Help` (from `app/mod.rs:183`). Both perform trivial `UIState` mutations (set `help_mode`, call `show_overlay` / `return_to_main` / `reset_help_modal_state`). All other commands exist only in the registry with no dispatch path.

3. **`src/ui/components/connection_mode.rs`** — a 826-LOC full-screen connection-management overlay. `AppState.connection_mode` is initialised to `None` in both `AppState::new()` (`state.rs:97`) and `AppState::default()` (`state.rs:2182`) and is never set to `Some`. The render guard at `ui/mod.rs:247` (`if let Some(connection_mode) = &state.connection_mode`) therefore never fires. Additionally, the outer condition at `ui/mod.rs:228` duplicates `is_connection_form()` in both arms of an `||`, a clear copy-paste artefact.

Beyond these three subsystems, several smaller pieces are also dead:

- `src/app/events.rs` — `AppEvent` enum with no references outside its own file. The live inter-task channels use private enums defined directly in `app/mod.rs`.
- `src/themes/mod.rs` — a one-line shim (`pub use crate::ui::theme::*`) with no callers.
- `EventHandler::start()` — body is `Ok(())`. Called once at `app/mod.rs:108` only to satisfy an old interface; the call can be removed.
- `terminal::clear_screen()` — no call-sites anywhere in `src/`.
- `constants::version_string()` — called once at `ui/mod.rs:273`; replaceable with an inline `format!` using the `APP_NAME` and `VERSION` constants.
- Five `log_*!` macros in `src/logging.rs:272–309` — each is a thin wrapper for the identical `tracing::` macro. Every call-site gains nothing from the alias.

## Goals / Non-Goals

**Goals:**
- Remove all confirmed-dead code so the compiler boundary matches the runtime boundary.
- Fix the `is_connection_form() || is_connection_form()` tautology as part of the `connection_mode` cleanup.
- Preserve `add_debug_message()`, `MemoryLogLayer`, and `DebugLogStorage` (they feed the live debug overlay behind `Ctrl+B`).
- Preserve all live behaviour: `?` toggles the help overlay identically after inlining; the debug overlay shows `tracing`-level logs identically after macro removal.
- Leave the build, clippy, fmt, and tests fully green.

**Non-goals:**
- Refactoring the live command-dispatch path beyond what is needed to inline the two remaining commands.
- Splitting `state.rs` or `ui.rs` (see c0005, c0006).
- Changing any user-visible key binding, layout, or toast text.

## Decisions

### Decision 1: Inline ToggleHelp rather than keeping a minimal commands module

**What:** Delete the entire `src/commands/` directory. Inline the `ToggleHelp` state mutation directly in `handlers/global.rs` where `?` is pressed.

**Why:** Keeping even a stripped-down `commands/` module for two commands adds indirection with no benefit. The inlined code is six lines copied verbatim from `commands/basic.rs:60–119` — readable and testable at the handler level.

**Alternatives rejected:**
- *Keep `commands/` but delete unused commands* — leaves a dead module pattern (registry, trait) that the next reader must evaluate all over again; doesn't fix the layering.
- *Move the two commands into a `src/app/commands.rs`* — unnecessary; the handler already owns the key-binding and the state; one extra hop adds nothing.

### Decision 2: Remove `connection_mode_scroll_offset` entirely

**What:** Delete the `connection_mode_scroll_offset` field from `UIState`, its initialisation, its three reset assignments, and its two scroll methods (`connection_mode_scroll_down`, `connection_mode_scroll_up`).

**Why:** The field exists exclusively to feed `connection_mode.render()` inside the dead render block. The two scroll methods are never called from any handler (verified by grep — they appear only in `state/ui.rs`). Retaining an unused field and two dead methods with no callers would mislead future readers into thinking they matter.

**Alternatives rejected:**
- *Keep the field as a placeholder for future re-use* — the "new" connection mode (`connection_mode.rs`) is being deleted in the same change; there is no planned future use. Dead fields are pure noise.

### Decision 3: Replace `version_string()` call-site and delete the function

**What:** Replace `constants::version_string()` at `ui/mod.rs:273` with `format!("{} v{}", constants::APP_NAME, constants::VERSION)`, then delete the `version_string()` function from `constants.rs`.

**Why:** The assignment marks this function as dead and requires deletion. The single call-site is trivially replaced with the inline form. The constants `APP_NAME` and `VERSION` remain; no information is lost.

**Alternatives rejected:**
- *Keep the function* — contradicts the assignment; also keeps a trivial helper with one caller that doesn't need the indirection.

### Decision 4: Replace `log_*!` macros with direct `tracing::` calls at all call-sites

**What:** Delete the five macro definitions from `logging.rs:272–309` and update every `crate::log_debug!` / `crate::log_warn!` / `crate::log_error!` call-site across `src/app/state.rs`, `src/database/postgres.rs`, `src/io/async_fs.rs`, `src/state/database.rs`, `src/state/ui.rs`, `src/ui/components/table_viewer.rs` to call `tracing::debug!` / `tracing::warn!` / `tracing::error!` directly.

**Why:** The macros are transparent aliases — they expand to the same `tracing::` call. Replacing them removes one indirection level, makes the `tracing` dependency explicit at the call-site, and allows clippy to lint the macro arguments against the `tracing` API directly.

**Alternatives rejected:**
- *Keep the macros as a portability shim* — there is no portability concern; `tracing` is already a direct dependency and is used directly in `logging.rs` itself.

### Decision 5: Delete `EventHandler::start()` along with its sole call-site

**What:** Remove the `start()` method body from `src/event/mod.rs:90–92` and remove `self.event_handler.start()?;` from `app/mod.rs:108`.

**Why:** The method is a no-op (`Ok(())`). The call-site gains nothing. Removing both makes `EventHandler::new()` the sole initialisation step, which is already where the background thread is spawned.

**Alternatives rejected:**
- *Keep the method as an extension point* — premature; if a future change needs a start hook it can add one then.

## Edge Cases & Failure Modes

**Help toggle regression:** The only user-visible behavior change is in `handlers/global.rs`. After inlining, pressing `?` must:
1. If `help_mode != HelpMode::None` → set `help_mode = HelpMode::None`, call `return_to_main()`.
2. Otherwise → set `help_mode` to the pane-appropriate variant, call `show_overlay(OverlayView::Help)` and `reset_help_modal_state()`.
This mirrors `commands/basic.rs:60–119` exactly. A missed branch would leave the overlay permanently open or permanently closed.

**`connection_modal_state` vs `connection_mode`:** The connection _modal_ (add/edit connection form, rendered at `ui/mod.rs:228–243`) is NOT being removed — only the full-screen `connection_mode` overlay component. The tautology fix at `ui/mod.rs:228` must not accidentally remove the live `render_connection_modal` call block.

**`connection_mode_scroll_offset` resets in live methods:** `enter_add_connection_mode` (state/ui.rs:1421), `enter_edit_connection_mode` (state/ui.rs:1429), and the connection-mode close method (state/ui.rs:1437) each do `self.connection_mode_scroll_offset = 0`. These reset lines must be removed when the field is removed; they are inside live methods, so forgetting them will cause a compile error (field does not exist) — which is actually a safety net.

**Logging migration completeness:** If any `crate::log_debug!` call-site is missed, the build fails (undefined macro). The grep output above lists all files; implementing this task should grep after migration to confirm zero remaining `log_debug!` / `log_info!` / `log_warn!` / `log_error!` / `log_span!` occurrences.

**`pub mod events;` in `app/mod.rs`:** The current `app/mod.rs` does NOT contain a `pub mod events;` declaration (verified: the file only declares `pub mod handlers;` and `pub mod state;`). Deleting `src/app/events.rs` requires no module declaration edit; it simply removes the file from disk. Confirm with a grep for `events` in `app/mod.rs` before closing this task.

**`test_connection_modal_state_new` test name:** `src/ui/components/connection_modal.rs` contains a `#[cfg(test)]` block with a test named `test_connection_modal_state_new`. The `_new` suffix is unrelated to `src/app/state_new/`; it refers to the modal's own `new()` constructor. This test must not be deleted.

## Migration / Cutover

Ordered to keep the tree green at each step:

**Step 1 — Delete `state_new/`:**
- Delete all 7 files under `src/app/state_new/`.
- No module declaration exists in `app/mod.rs`, so nothing else needs editing.

**Step 2 — Delete `connection_mode` + fix tautology:**
- Delete `src/ui/components/connection_mode.rs`.
- In `src/app/state.rs:48` remove the field `pub connection_mode: Option<ConnectionMode>`.
- Remove the `connection_mode: None` initialisers at `state.rs:97` and `state.rs:2182`.
- Remove any `ConnectionMode` import in `state.rs`.
- In `src/state/ui.rs`: remove field `pub connection_mode_scroll_offset: usize` (line 257), its init `connection_mode_scroll_offset: 0` (line 350), the three reset assignments in `enter_add_connection_mode` (line 1422), `enter_edit_connection_mode` (line 1430), and the close method (line 1439), and both scroll methods `connection_mode_scroll_down` (lines 1442–1447) and `connection_mode_scroll_up` (lines 1449–1454).
- In `src/ui/mod.rs`: delete the dead render block lines 245–256. Fix the tautology at line 228: change `state.ui.current_view.is_connection_form() || state.ui.current_view.is_connection_form()` to `state.ui.current_view.is_connection_form()`.
- Remove any `use` import for `ConnectionMode` or `connection_mode` in `ui/mod.rs` if one exists.

**Step 3 — Delete `commands/` + inline help toggle:**
- Inline the `ToggleHelp` logic in `handlers/global.rs:16–19`. Replace `app.execute_command(CommandId::ToggleHelp)?;` with:
  ```rust
  use crate::{app::state::HelpMode, state::view::OverlayView};
  if app.state.ui.help_mode != HelpMode::None {
      app.state.ui.help_mode = HelpMode::None;
      app.state.ui.return_to_main();
  } else {
      app.state.ui.help_mode = match app.state.ui.focused_pane {
          FocusedPane::Connections   => HelpMode::Connections,
          FocusedPane::Tables        => HelpMode::Tables,
          FocusedPane::Details       => HelpMode::Details,
          FocusedPane::TabularOutput => HelpMode::TabularOutput,
          FocusedPane::QueryWindow   => HelpMode::QueryWindow,
          FocusedPane::SqlFiles      => HelpMode::SqlFiles,
      };
      app.state.ui.show_overlay(OverlayView::Help);
      app.state.ui.reset_help_modal_state();
  }
  ```
- Remove the `use crate::commands::CommandId;` import from `handlers/global.rs:7`.
- In `src/app/mod.rs`:
  - Remove `use crate::commands::{CommandAction, CommandContext, CommandId, CommandRegistry, CommandResult};` (line 4).
  - Remove the `command_registry: CommandRegistry` field declaration (line 52) and its struct literal initialiser `command_registry: CommandRegistry::new()` (line 89, inside `App::new()`).
  - Delete the `execute_command()` method (lines 147–171).
  - Delete the `handle_command_action()` method (lines 173–244).
- In `src/lib.rs`: remove `pub mod commands;` (line 5).
- Delete the entire `src/commands/` directory.

**Step 4 — Delete `events.rs` / `AppEvent`:**
- Delete `src/app/events.rs`.
- Confirm no `pub mod events;` or `use crate::app::events` appears in any file (grep; expected zero).

**Step 5 — Delete `themes` shim:**
- Delete `src/themes/mod.rs`.
- In `src/lib.rs`: remove `pub mod themes;` (line 16).
- Grep `crate::themes::` across `src/` — expect zero matches.

**Step 6 — Delete dead functions:**
- In `src/event/mod.rs`: delete the `start()` method (lines 89–92).
- In `src/app/mod.rs`: remove `self.event_handler.start()?;` (line 108).
- In `src/terminal.rs`: delete `clear_screen()` (lines 56–63).
- In `src/ui/mod.rs:273`: replace `constants::version_string()` with `format!("{} v{}", constants::APP_NAME, constants::VERSION)`.
- In `src/constants.rs`: delete `version_string()` (lines 9–12).

**Step 7 — Delete logging macros + migrate call-sites:**
- In `src/logging.rs`: delete the five macro definitions at lines 271–309 (from the `log_debug!` doc comment through the closing `}` of `log_span!`).
- In each call-site file, replace `crate::log_debug!` → `tracing::debug!`, `crate::log_info!` → `tracing::info!`, `crate::log_warn!` → `tracing::warn!`, `crate::log_error!` → `tracing::error!`. Files: `src/app/state.rs`, `src/database/postgres.rs`, `src/io/async_fs.rs`, `src/state/database.rs`, `src/state/ui.rs`, `src/ui/components/table_viewer.rs`.
- After migration: grep `log_debug!\|log_info!\|log_warn!\|log_error!\|log_span!` across `src/` — expect zero matches.

## Verification

1. `cargo build` completes without errors.
2. `cargo clippy --all-targets -- -D warnings` is clean.
3. `cargo fmt --check` is clean.
4. `cargo test` is green, including the `test_connection_modal_state_new` test in `src/ui/components/connection_modal.rs`.
5. Run the application. Press `?` — the help overlay opens. Press `?` again — it closes. Press `Ctrl+B` — the debug overlay opens and shows live `tracing`-level log messages (confirming the macro migration left the debug layer intact). Press `Ctrl+B` again — it closes.
