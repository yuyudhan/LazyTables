# Design — Split connection_modal, table_viewer, query_editor

## Context

LazyTables is a Rust TUI database client (ratatui 0.29, crossterm, tokio) whose UI layer lives
under `src/ui/`. Three component files have grown to 1500–2020 lines each by accumulating type
definitions, state logic, and rendering code without internal structure. The repo already has a
precedent — the abandoned `src/app/state_new/` directory uses `impl AppState` blocks spread across
sibling modules and compiles correctly (Rust allows `impl Foo` in any file as long as the type
is in scope). This change applies the same split to the three UI mega-components.

External callers import these types through two routes:
1. The flat glob `pub use connection_modal::*` in `src/ui/components/mod.rs` — yields bare paths
   like `crate::ui::components::ColumnInfo`.
2. Explicit submodule paths, already present for `CellUpdate`, `ColumnInfo`, `DeleteConfirmation`,
   `SetNullConfirmation`, `TableViewMode` in `src/state/database.rs` and
   `src/app/handlers/query_results.rs` using `crate::ui::components::table_viewer::*`.

Both routes must keep working after the split.

## Goals / Non-Goals

**Goals**
- Replace each flat `.rs` file with a same-named subdir module (`foo.rs` → `foo/mod.rs` +
  siblings), moving content at the verified seams with zero semantic change.
- Every existing `pub` type and function resolves at its current import path after the change.
- Tests move with the code they exercise; they must pass with no edits other than file location.
- `src/ui/components/mod.rs` module declarations update; the glob re-exports stay intact.

**Non-Goals**
- Changing rendering logic, input handling, or any observable behaviour.
- Merging or refactoring the split files further (e.g. extracting sub-renderers).
- Touching `src/app/state_new/` (dead code removed in c0002).
- Any change to `sql_suggestions.rs` or `suggestion_popup.rs`.

## Decisions

### Decision 1: subdir-module with `mod.rs` re-exporting everything

**What:** Each component becomes `foo/mod.rs` that declares the sibling modules and re-exports
all public items with `pub use types::*; pub use state::*; pub use render::*;` (and `pub use
input::*;` for `query_editor`). The outer `src/ui/components/mod.rs` glob (`pub use
connection_modal::*;`) then transitively re-exports everything to the crate-level path.

**Why:** Zero diff at every call site. Both routes (bare `crate::ui::components::Foo` and
subpath `crate::ui::components::table_viewer::Foo`) continue to resolve. The inner `mod.rs`
glob catches any `pub` item a future author adds to a sibling file automatically.

**Alternatives rejected:**
- *Inline modules*: would keep one file per component, defeating the readability goal.
- *Flat re-export in outer `mod.rs` only*: would break the explicit submodule paths
  (`crate::ui::components::table_viewer::CellUpdate`) already used in `state/database.rs` and
  `app/handlers/query_results.rs`.

### Decision 2: split boundaries follow struct/impl ownership

**What:** Each split boundary is placed between a struct definition (and its `impl`) and the
first free function or method that takes a `&mut Frame`. Specifically:
- `connection_modal`: types (17–190) / state impl (191–875) / render fns (877–2020 minus tests).
  Tests (1678–2020) go into `state.rs` because they exercise `ConnectionModalState` methods.
- `table_viewer`: types including `impl TableTab` (13–517) and remaining types + `impl
  TableViewerState` (519–782) are both in `types.rs`/`state.rs` respectively; render starts
  at line 784 (`render_table_viewer`). No tests exist in the file.
- `query_editor`: struct + Clone + Default (19–82) in `state.rs`; all `impl QueryEditor`
  non-render methods (84–1101) in `input.rs`; `render` and its private
  `apply_syntax_highlighting_with_line_numbers` (1102–1271) in `render.rs`. Tests (1273–1512)
  go into `input.rs` because they exercise input/cursor methods.

**Why:** Rust's `impl Foo` may span multiple files once `Foo` is in scope via `use super::*` or
`use super::QueryEditor`. Keeping each `impl` in one file avoids confusion about which file owns
the type.

**Alternatives rejected:**
- Splitting `impl TableTab` away from its struct definition: unnecessarily awkward; `TableTab` +
  its `impl` together are the natural unit.

### Decision 3: private helpers become `pub(super)` where shared

**What:** Any private function currently called only within one file stays `fn` in that file.
Any private function called from multiple sibling files within the same component module (e.g. a
render helper called by both `state.rs` and `render.rs`) becomes `pub(super)`.

**Why:** Minimal visibility escalation; prevents leaking implementation details outside the
component submodule. `pub(super)` is visible to all siblings and `mod.rs` but not outside the
component directory.

**Alternatives rejected:**
- Making everything `pub`: unnecessary coupling surface.

### Decision 4: `use super::*` at the top of each sibling

**What:** Each sibling file (`state.rs`, `render.rs`, `input.rs`) imports its required types
with a top-level `use super::*;` (or targeted `use super::types::*;` for cross-sibling access).

**Why:** Reduces boilerplate and ensures the sibling can reference the parent module's re-exported
items without repetition. The `mod.rs` re-exports guarantee that `super::*` covers everything in
scope at the component level.

**Alternatives rejected:**
- Fully qualified paths for every cross-sibling type: verbose, harder to maintain.

## Edge Cases & Failure Modes

**Missed re-export:** If a `pub` item in any sibling file is not reached by the `pub use *::*;`
chain in the inner `mod.rs`, callers that previously used the flat glob will get a "unresolved
import" compile error. Mitigation: after the split, run `cargo build` and treat any unresolved
path as a missed re-export, not a design issue.

**Explicit submodule paths still work:** `src/state/database.rs` imports
`crate::ui::components::table_viewer::{CellUpdate, ColumnInfo, DeleteConfirmation, SetNullConfirmation}`
and `src/app/handlers/query_results.rs` imports
`crate::ui::components::table_viewer::TableViewMode`. After the split, the Rust module path
`crate::ui::components::table_viewer` resolves to `src/ui/components/table_viewer/mod.rs`, which
re-exports all items from its siblings — so these paths continue to resolve without change.

**Circular use between siblings:** If a type in `types.rs` references a type in `state.rs` (or
vice versa), a direct `use super::state::*;` in `types.rs` would create a cycle. The current
files are clean: `TableTab` does not reference `TableViewerState`; `ConnectionField` does not
reference `ConnectionModalState`. Verify during implementation by confirming `types.rs` has no
upward dependency.

**`#[cfg(test)]` path references:** Test code in `connection_modal/state.rs` and
`query_editor/input.rs` uses `use super::*;` — after the move the `super` is the sibling module
scope, which reaches all re-exported items via `mod.rs`. No test path should require adjustment.

**`arboard` dependency in `table_viewer/state.rs`:** `copy_row_csv` and `copy_cell` (lines
639–692 of original) use `arboard::Clipboard`. This crate is already a `Cargo.toml` dependency;
no change needed. `arboard` is only used in `state.rs`, so it does not need to be in scope in
`render.rs`.

**`syntect` fields in `query_editor/state.rs`:** `QueryEditor` holds `syntax_set: SyntaxSet` and
`theme_set: ThemeSet`. The `Clone` impl (lines 52–76) re-creates them with `load_defaults_*()`.
The `render.rs` file will call `self.syntax_set` and `self.theme_set` — both are fields of
`QueryEditor`, so `render.rs` only needs `use super::state::QueryEditor;` (via `use super::*`)
and the `syntect` crates in scope (they are in `Cargo.toml`).

## Migration / Cutover

**What to create:**
```
src/ui/components/connection_modal/
  mod.rs          pub mod types; pub mod state; pub mod render;
                  pub use types::*; pub use state::*; pub use render::*;
  types.rs        lines 17–190 of connection_modal.rs (+ needed use statements)
  state.rs        lines 191–875 + 1678–2020 of connection_modal.rs
  render.rs       lines 877–1677 of connection_modal.rs

src/ui/components/table_viewer/
  mod.rs          pub mod types; pub mod state; pub mod render;
                  pub use types::*; pub use state::*; pub use render::*;
  types.rs        lines 13–527 of table_viewer.rs (+ needed use statements)
  state.rs        lines 529–782 of table_viewer.rs
  render.rs       lines 784–1773 of table_viewer.rs

src/ui/components/query_editor/
  mod.rs          pub mod state; pub mod input; pub mod render;
                  pub use state::*; pub use input::*; pub use render::*;
  state.rs        lines 19–82 of query_editor.rs (+ needed use statements)
  input.rs        lines 84–1101 + 1273–1512 of query_editor.rs
  render.rs       lines 1102–1271 of query_editor.rs
```

**What to delete:** The three flat source files
`src/ui/components/connection_modal.rs`,
`src/ui/components/table_viewer.rs`,
`src/ui/components/query_editor.rs`.

**What to update:**
- `src/ui/components/mod.rs` lines 3, 6, 9 (`pub mod connection_modal;`,
  `pub mod query_editor;`, `pub mod table_viewer;`) — Rust resolves these to the subdir
  `mod.rs` automatically when the `.rs` file is removed. No text change needed for the `pub
  mod` declarations; the glob `pub use` lines are already correct. Verify the file compiles.

**Call sites — no changes required (verified):**
- `src/app/state.rs` — imports via glob `crate::ui::components::*`; unchanged.
- `src/app/state_new/mod.rs` — same glob; unchanged (dead code, removed in c0002).
- `src/app/mod.rs` — uses `crate::ui::components::TestConnectionStatus` inline `use`; resolves
  via `connection_modal/mod.rs` re-export.
- `src/app/handlers/connections.rs` — uses `crate::ui::components::{ConnectionField,
  PasswordStorageType, TestConnectionStatus}`; resolves via re-exports.
- `src/app/handlers/query_results.rs` — uses
  `crate::ui::components::table_viewer::TableViewMode`; resolves because
  `table_viewer/mod.rs` re-exports `TableViewMode` from `types.rs`.
- `src/state/database.rs` — uses
  `crate::ui::components::table_viewer::{CellUpdate, ColumnInfo, DeleteConfirmation, SetNullConfirmation}`;
  resolves because `table_viewer/mod.rs` re-exports these from `types.rs`.
- `src/commands/connection.rs` — uses
  `crate::ui::components::connection_modal::ConnectionModalState`; resolves because
  `connection_modal/mod.rs` re-exports `ConnectionModalState` from `state.rs`.

## Verification

1. `cargo build` produces zero errors — proves all re-exports are in place and no circular deps.
2. `cargo clippy --all-targets -- -D warnings` clean.
3. `cargo fmt --check` clean.
4. `cargo test` green, specifically:
   - `ui::components::connection_modal::state::tests` (previously `connection_modal::tests`)
   - `ui::components::query_editor::input::tests` (previously `query_editor::tests`)
5. Run the app manually:
   - Press `c` to open the connection modal — all fields render, Tab cycles focus, test button works.
   - Connect to a DB and open a table — data and schema tabs render, cell edit and row delete
     confirmation dialogs appear.
   - Focus the SQL editor — Vim modes (normal/insert), autocomplete popup, syntax highlighting all
     function.
