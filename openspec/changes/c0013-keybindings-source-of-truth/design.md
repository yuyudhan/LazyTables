# Design — c0013: Single keybinding table feeding handlers & help

## Context

`src/ui/help.rs` contains 926 lines building key/description pairs as raw string literals. Six
private methods (`add_connections_commands`, `add_tables_commands`, `add_details_commands`,
`add_tabular_commands`, `add_sql_files_commands`, `add_query_window_commands`) collectively hold
~70 `add_command` calls plus two column builders that duplicate the global entries. The actual key
dispatch lives across seven handler files under `src/app/handlers/`. No compile-time link exists
between these two locations.

`HelpMode` (imported at `help.rs:11` from `crate::app::state::HelpMode`) has six variants
(`Connections`, `Tables`, `Details`, `TabularOutput`, `SqlFiles`, `QueryWindow`, `None`) which
map 1:1 to the six handler files. The match at `create_left_column` line 41 dispatches to the
corresponding `add_*_commands` function. This enum is the natural grouping key for the new table.

The c0005 proposal stabilises `AppState` split paths and the global handler layout. c0011 adds
`Ctrl+Y` to `handlers/global.rs`. c0012 adds read-only gating to mutation paths. Both require
help entries; c0013 must incorporate them so they appear automatically once the table is authored.

## Goals / Non-Goals

**Goals:**
- Single authoritative source: every key shown in the help overlay is defined in `keybindings.rs`.
- Zero orphan entries: removing a binding from the table removes it from help automatically.
- Handler constants: at minimum, the global keys (`?`, `Ctrl+B`, `q`, `Ctrl+Y` from c0011) are
  named constants so a typo is a compile error, not a silent mismatch.
- Preserve current help layout: same sections, same emoji headers, same two-column rendering,
  same `render_help` public signature.

**Non-Goals:**
- Migrating every handler arm to reference constants (only globals required; pane handlers are
  `match key.code` arms and the marginal safety gain does not justify the mechanical churn).
- Keybinding runtime configurability (not in scope).
- Changing the `HelpMode` enum shape or adding new modes.
- Altering `render_help` rendering logic beyond wiring the data source.

## Decisions

### Decision 1: Static `&[KeyBinding]` slice, not a `HashMap`

**What:** `pub static BINDINGS: &[KeyBinding]` — a flat array of structs defined with `const`
initialisers. `KeyBinding` is a plain struct: `context: HelpContext`, `key: &'static str`,
`description: &'static str`, `condition: Option<&'static str>`.

**Why:** Help rendering iterates in insertion order (section order matches the existing layout);
a slice preserves that order naturally. `const` initialisers guarantee zero runtime allocation.
`HashMap` would require runtime initialisation (e.g. `once_cell`) for no navigational benefit —
the only access pattern is linear iteration filtered by `context`.

**Alternatives rejected:**
- `HashMap<HelpContext, Vec<KeyBinding>>`: allocation at startup, insertion-order not guaranteed
  before Rust 1.36 BTreeMap semantics.
- `phf` crate for perfect hash: adds a build-time dependency for no gain (the table is iterated,
  not point-looked-up).

### Decision 2: `HelpContext` mirrors `HelpMode` exactly

**What:** A new `pub enum HelpContext` in `keybindings.rs`:
`Global | Connections | Tables | Details | TabularOutput | SqlFiles | QueryWindow`.

**Why:** `HelpMode` (in `src/app/state.rs`) is an app-state type; `HelpContext` is a
keybindings-layer type. They have identical variants today, but keeping them separate avoids a
circular dependency (`keybindings.rs` must not import from `app::state`). `help.rs` already knows
both and does the trivial mapping.

**Alternatives rejected:**
- Reuse `HelpMode` directly: `keybindings.rs` would depend on `app::state`, pulling in the full
  `AppState` dependency graph; violates the layering goal of c0005/c0007.

### Decision 3: Section headers live in `keybindings.rs` alongside entries

**What:** Each context group may have sub-section headers (e.g. `"🔧 Connection Management"` in
`add_connections_commands` at help.rs line 188). These are represented as a `KeyBindingGroup`
wrapper or inline as entries with `key: ""` and a `section_header: Some(&'static str)` variant,
or — simplest — as a companion `SECTION_HEADERS` map from `(HelpContext, insertion_index)` to
header string.

**Simplest workable approach:** `KeyBinding` gains a boolean `is_section_header: bool` and uses
`description` for the header text when true. The rendering loop in `help.rs` emits a styled header
`Line` when it encounters a header entry, otherwise calls `add_command`. This keeps everything in
one flat slice with well-defined order.

**Why:** Preserves the current rich section structure without a separate data structure. The
`add_command` helper is not changed; `help.rs` just drives it from the table.

**Alternatives rejected:**
- Separate `SECTIONS` array with index ranges: fragile — inserting an entry shifts indices.
- `Vec<Either<Header, Binding>>` enum: expressive but requires a generic enum type and more code
  in the renderer.

### Decision 4: `condition` field for context-conditional bindings

**What:** `condition: Option<&'static str>` holds a short human-readable guard string (e.g.
`"not in insert/edit mode"`, `"insert mode only"`, `"after 'd' press"`). The renderer appends
it parenthetically: `"(not in insert/edit mode)"`.

**Why:** Bindings like `q` (guarded by `can_quit` at handlers/global.rs line 112), `Tab` (skipped
when query editor is in insert mode at global.rs line 72–75), `dd`/`yy` (double-tap within 500ms
per query_results.rs lines 47–76), and `i`/`a`/`o`/`O` (only meaningful in normal mode) carry
implicit conditions that users need to understand. A `condition` field makes them explicit in the
table and surfaced in help, without changing dispatch logic.

**Alternatives rejected:**
- Omitting conditions: help entries for `q` and `Tab` are currently shown without caveats, causing
  user confusion when they don't work — documenting the condition is strictly better.
- Encoding conditions as `bool` predicates: would require runtime evaluation and break the `const`
  nature of the table.

### Decision 5: Global handler key constants only (not full per-pane migration)

**What:** Export `pub const KEY_HELP: char = '?'`, `pub const KEY_DEBUG: char = 'b'`,
`pub const KEY_QUIT: char = 'q'`, `pub const KEY_QUERY_HISTORY: char = 'y'` from `keybindings.rs`.
`handlers/global.rs` references these in its `match` arms.

**Why:** The global handler (`global.rs` lines 14–108) has the highest reuse surface — it runs on
every keypress before pane dispatch. Pane handlers are already scoped to a single pane and the
match arms are self-documenting. The marginal drift risk for pane handlers is low compared to the
cost of touching every handler file.

**Alternatives rejected:**
- Full per-pane constant migration: ~50 match arms across 7 files, high churn, low payoff.
- No constants at all: valid for the first pass, but leaves the global handler, which sees every
  key, entirely unchecked.

## Edge Cases & Failure Modes

- **Missing entry:** If a binding exists in a handler but not in `BINDINGS`, the help overlay
  simply omits it. No panic, no blank page. The Verify task (grep) catches this at review time.
- **Extra entry:** A binding in `BINDINGS` with no corresponding handler (stale from a deleted
  binding) appears in help as an orphan. The grep check in the Verify task catches this.
- **c0011/c0012 not yet landed:** `BINDINGS` includes `Ctrl+Y` and read-only notes provisionally.
  If those proposals have not yet applied, the help entries for those features are present but the
  keys do nothing — harmless (help is informational, not dispatch).
- **`HelpContext` vs `HelpMode` mapping:** `help.rs` does `match help_mode { HelpMode::X => HelpContext::X }`. This is a trivial mapping; a compile error fires if `HelpMode` gains a variant without a corresponding `HelpContext`.
- **Section header ordering:** Entries are in a `static` slice; insertion order is author-defined.
  The rendering loop must not sort or group — it must iterate the slice in declaration order.
- **`render_help` scroll state:** The current scroll offset (`ui_state.help_scroll_offset`) counts
  rendered `Line` rows, not `KeyBinding` entries. Header entries each emit one `Line`; binding
  entries emit one `Line` each (same as today). The total line count is unchanged, so scroll
  logic is unaffected.

## Migration / Cutover

### Files to create
- `src/keybindings.rs` — new; add `pub mod keybindings;` to `src/lib.rs`.

### Files to modify
- `src/ui/help.rs`:
  - Replace bodies of `add_connections_commands` (line 180), `add_tables_commands` (272),
    `add_details_commands` (358), `add_tabular_commands` (394), `add_sql_files_commands` (500),
    `add_query_window_commands` (621) with iterator loops over `keybindings::BINDINGS`.
  - Replace global `add_command` calls in `create_left_column` (lines 62–68) and
    `create_right_column` (lines 93–128) with iterator loops over `keybindings::BINDINGS`
    filtered by `HelpContext::Global`.
  - Keep `add_command` helper (line 164), `render_help` (line 732), and all layout/styling logic
    unchanged.
- `src/app/handlers/global.rs`:
  - Import `crate::keybindings::{KEY_HELP, KEY_DEBUG, KEY_QUIT, KEY_QUERY_HISTORY}`.
  - Replace `KeyCode::Char('?')` at line 16 with `KeyCode::Char(KEY_HELP)`.
  - Replace `KeyCode::Char('b')` at line 22 with `KeyCode::Char(KEY_DEBUG)`.
  - Replace `KeyCode::Char('q')` at line 27 with `KeyCode::Char(KEY_QUIT)`.
  - After c0011 lands: add `Ctrl+Y` arm using `KEY_QUERY_HISTORY`.

### Nothing to delete
The `add_*_commands` methods become thin wrappers around the iterator; they may be inlined into
`create_left_column`'s match arms at the implementer's discretion. The private method boundary is
not load-bearing and can be collapsed.

## Verification

1. `cargo build` — checks that `HelpContext` variants map exhaustively from `HelpMode` variants
   and that the global handler constants have matching types.
2. `cargo test` — existing component tests in `ui/components/` are unaffected; `help.rs` has no
   existing unit tests (rendering-only), so no tests break.
3. Manual: open the app, press `?` — verify the help overlay displays all sections with the correct
   keys for the focused pane; scroll with `j/k`/`h/l` — verify section navigation works.
4. Grep check: `grep -rn 'add_command' src/ui/help.rs` — should show only the helper definition
   at line 164 and zero literal `add_command("key_string", …)` call sites (all calls go through
   the table iterator).
5. Grep check: `grep -rn 'Ctrl+Y\|C-Y\|query.history' src/ui/help.rs` — should find the entry
   rendered by the table iterator, confirming c0011's binding is present without a manual edit.
