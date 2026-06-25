# c0009 Design: Pull Inline Pane Rendering Out of ui/mod.rs

## Context

`src/ui/mod.rs` is 1416 lines. The file already delegates table/viewer/query-editor rendering to
dedicated components in `src/ui/components/`. Three pane renderers remain inline: the connections
list, the table-details panel, and the SQL-files browser. Each is self-contained and reads only
from `AppState` and `Theme` — they have no caller other than `UI::draw` and no side effects
beyond `frame.render_*` calls.

Relevant existing component shapes:

```rust
// tables_pane.rs pattern (the target):
pub fn render_tables_pane(frame: &mut Frame, area: Rect, state: &mut AppState, theme: &Theme)

// table_viewer and query_editor delegate similarly from draw_tabular_output / draw_query_window
```

The three extracted panes must follow this same pattern.

## Goals / Non-Goals

**Goals:**
- Three new files under `src/ui/components/` — one per pane — each self-contained with its own
  imports.
- `UI::draw` reduced from ~90 lines in the `impl` block to delegation calls, matching
  `draw_tables_pane` / `draw_tabular_output`.
- All disabled/empty-state placeholder widgets remain inside the new component files (not
  re-lifted into `draw`).
- No logic change, no layout change, no theme key change, no visual change.

**Non-Goals:**
- Splitting components further (e.g. search sub-components within connections pane).
- Changing the `AppState` / `UIState` / `DatabaseState` field layout (that is c0005/c0006).
- Changing widget styling, color, or text (that is a future theming pass).

## Decisions

### Decision 1: Function signatures mirror `render_tables_pane` exactly

**What:** Each new `render_*` function takes `(frame: &mut Frame, area: Rect, state: &mut
AppState, theme: &Theme)` (or `&AppState` for sql_files_pane, matching the current `draw_sql_files_pane(&self, frame, area, state: &AppState)` immutable borrow).

**Why:** Matches the established pattern in `tables_pane.rs`. Consistent signatures mean callers
look the same in `UI::draw`. The `theme` parameter is passed explicitly so the component has no
dependency on the `UI` struct — it could be called from a test harness.

**Alternatives rejected:**
- Method on a new struct: adds indirection with no benefit for three stateless functions.
- Passing `&self` (the `UI` struct): couples the component to UI internals; the existing
  `tables_pane.rs` does not do this.

### Decision 2: Connecting-animation fields — parameter forwarding after c0005

**What:** `draw_connections_pane` currently reads three fields from `AppState` that c0005
relocates onto `App` (app/mod.rs):
- `state.connecting_in_progress: Option<usize>`
- `state.connecting_animation_frame: usize`
- `state.get_connection_elapsed_seconds() -> u64` (a method that reads `connection_start_time`)
- `state.connection_timeout_seconds: u64` (stays on AppState)

If c0009 is implemented **before** c0005: access all four via `state` unchanged.

If c0009 is implemented **after** c0005: `render_connections_pane` gains explicit params for the
three fields that moved to `App`:
```rust
pub fn render_connections_pane(
    frame: &mut Frame,
    area: Rect,
    state: &mut AppState,
    theme: &Theme,
    connecting_in_progress: Option<usize>,
    connecting_animation_frame: usize,
    connection_elapsed_secs: u64,
)
```
`UI::draw` extracts them from `app` and passes them in. Either path is correct; the implementer
chooses based on the actual execution order.

**Why:** The animation fields are display-only inputs; they need not live on AppState for the
component to render them. Making them explicit params avoids a circular borrow (App holds
AppState) and aligns with c0005's intent of reducing AppState's surface.

**Alternatives rejected:**
- Adding a thin `struct AnimationState` passed through — over-abstraction for three scalars.
- Keeping the fields on AppState permanently — contradicts c0005.

### Decision 3: `build_comprehensive_table_details` becomes module-private in details_pane.rs

**What:** Move `build_comprehensive_table_details` (currently a private method on `impl UI`,
ui/mod.rs:602–849) into `details_pane.rs` as `fn build_comprehensive_table_details(...)` with
`pub(super)` or crate-private visibility. It is called only by `render_details_pane` in the same
file.

**Why:** It is an implementation detail of the details pane, not a public API. Keeping it
non-public prevents accidental use elsewhere.

**Alternatives rejected:**
- Making it `pub` on the module: widens surface needlessly.
- Keeping it on `impl UI` and calling it via a parameter: couples the component to the `UI`
  struct (see Decision 1).

### Decision 4: `get_file_metadata` becomes module-private in sql_files_pane.rs

**What:** Move `get_file_metadata(path: &Path) -> (String, String)` (ui/mod.rs:1147–1185) into
`sql_files_pane.rs` as a free function, `fn get_file_metadata(path: &std::path::Path) ->
(String, String)`. Module-private (`fn`, not `pub`).

**Why:** Pure utility, used only by `render_sql_files_pane`. It takes no state, so a free
function is cleanest. The `std::fs` import moves with it.

**Alternatives rejected:**
- A separate `file_utils.rs` module: premature generalization; only one caller.

### Decision 5: Keep `pub use` glob re-exports in components/mod.rs

**What:** `src/ui/components/mod.rs` adds `pub mod connections_pane; pub use connections_pane::*;`
(and same for the other two) following the existing pattern for all 9 current modules.

**Why:** Existing callers such as `crate::ui::components::render_tables_pane` use the glob-reexport
pattern. New callers in `UI::draw` use `crate::ui::components::render_connections_pane` etc.
Consistency.

**Alternatives rejected:**
- Direct paths from `ui/mod.rs`: breaks the convention, makes the component harder to use from
  other call sites.

## Edge Cases & Failure Modes

- **Stateful widget borrow:** `draw_connections_pane` reads and writes `state.ui.connections_list_state`
  (clone → render → write back, lines 448–452). This pattern must be preserved verbatim in
  `render_connections_pane`. Missing the write-back silently breaks scroll state.
- **`state: &AppState` vs `&mut AppState`:** `draw_sql_files_pane` currently takes `&AppState`
  (immutable). The extracted function must keep the immutable borrow; accidentally upgrading to
  `&mut AppState` would cause a borrow conflict if `UI::draw` holds a mutable borrow for another
  pane render call on the same state.
- **`details_viewport_offset` scroll write:** `draw_details_pane` writes three fields on
  `state.ui` (`details_content_height`, `details_viewport_height`, `details_max_scroll_offset`,
  lines 562–564). These must be preserved in the extracted function to keep scroll-bounds
  checking working.
- **Missing `use` imports in new files:** Each new component file must carry its own `use` block.
  The implementer must grep `src/ui/mod.rs` for all types used in each method (e.g.
  `ConnectionStatus`, `DatabaseType`, `FocusedPane`, `crate::config::Config`,
  `crate::database::TableMetadata`) and add corresponding imports.
- **`self.theme` references:** All `self.theme.get_color(...)` calls become `theme.get_color(...)`
  with the passed `&Theme` param — same value, no behavior change.
- **The tautology at ui/mod.rs:228** (`is_connection_form() || is_connection_form()`) is dead
  code that c0002 removes. c0009 must not copy it into the new files; leave it in `UI::draw`
  unchanged (c0002 handles it).

## Migration / Cutover

**Files to create:**
- `src/ui/components/connections_pane.rs` — body from `draw_connections_pane` (ui/mod.rs:286–453)
- `src/ui/components/details_pane.rs` — body from `draw_details_pane` (ui/mod.rs:462–599) +
  `build_comprehensive_table_details` (ui/mod.rs:602–849)
- `src/ui/components/sql_files_pane.rs` — body from `draw_sql_files_pane` (ui/mod.rs:967–1144)
  + `get_file_metadata` (ui/mod.rs:1147–1185)

**`src/ui/components/mod.rs` — add three entries** (after existing ones, following the glob pattern):
```rust
pub mod connections_pane;
pub mod details_pane;
pub mod sql_files_pane;

pub use connections_pane::*;
pub use details_pane::*;
pub use sql_files_pane::*;
```

**`src/ui/mod.rs` — in `UI::draw` replace the three call-sites:**
```rust
// Before:
self.draw_connections_pane(frame, areas.connections, state);
self.draw_details_pane(frame, areas.details, state);
self.draw_sql_files_pane(frame, areas.sql_files, state);

// After:
components::render_connections_pane(frame, areas.connections, state, &self.theme);
components::render_details_pane(frame, areas.details, state, &self.theme);
components::render_sql_files_pane(frame, areas.sql_files, state, &self.theme);
// (Plus animation params if after c0005 — see Decision 2)
```

**Delete from `src/ui/mod.rs`:**
- `fn draw_connections_pane` (lines 286–453)
- `fn draw_details_pane` (lines 462–599)
- `fn build_comprehensive_table_details` (lines 602–849)
- `fn draw_sql_files_pane` (lines 967–1144)
- `fn get_file_metadata` (lines 1147–1185)

**Remove unused `use` imports from `src/ui/mod.rs`** (anything brought in only for the deleted
methods — e.g. `ratatui::widgets::List`, `ratatui::widgets::ListItem`, emoji icon matches).
Run `cargo clippy` after deletion; it will flag dead imports precisely.

## Verification

Build must be clean: `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo test`.

Behavioral check: launch the application and manually verify:
1. **Connections pane** — connecting animation (animated dots + elapsed/timeout counter) displays
   while a connection attempt is in progress; status icons (green ✓, yellow, red ✗) render
   correctly; search mode (`/` key) shows the search query in the pane title.
2. **Details pane** — selecting a table shows the metadata panel (row count, column count,
   storage sizes, primary keys, foreign keys, index count, comment if any); scroll works when
   content exceeds pane height; disabled state ("Connect to a database first") shows when no
   connection is active.
3. **SQL files pane** — file list renders with size/mtime metadata when focused; rename mode,
   create mode, and search mode each show their respective inline input prompt; disabled state
   ("Connect to database") shows when no connection is active.

All three panes must be visually **identical** to their pre-extraction appearance.
