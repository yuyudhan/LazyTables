# Tasks — c0013: Single keybinding table feeding handlers & help

## 1. Author `src/keybindings.rs`

- [ ] 1.1 Define `pub enum HelpContext` with variants: `Global`, `Connections`, `Tables`,
      `Details`, `TabularOutput`, `SqlFiles`, `QueryWindow`.
- [ ] 1.2 Define `pub struct KeyBinding` with fields: `context: HelpContext`,
      `key: &'static str`, `description: &'static str`,
      `is_section_header: bool`, `condition: Option<&'static str>`.
- [ ] 1.3 Populate `pub static BINDINGS: &[KeyBinding]` with all `Global` entries sourced
      from `src/app/handlers/global.rs` (lines 14–108) and CLAUDE.md lines 248–254:
      `q` (condition: "not in edit/search/insert mode"), `?`, `Ctrl+B`, `1`–`6`
      (condition: "main view only, not in table cell edit"), `Tab`
      (condition: "not in query editor insert mode"), `Shift+Tab` (same condition),
      `Ctrl+h/j/k/l` (pane nav); add placeholder entries for c0011 `Ctrl+Y`
      ("Toggle query history overlay") and c0012 read-only note ("Read-only mode:
      cell edits/row deletes are blocked").
- [ ] 1.4 Populate `Connections` entries from `handlers/connections.rs` and help.rs lines
      180–270 (CLAUDE.md lines 185–192): `j/k`, `Enter`/`Space` (connect), `x`
      (disconnect), `a` (add), `e` (edit), `d` (delete), `/` (search),
      `ESC` (exit search); add `Connection Modal` sub-section with `Type`, `Enter`,
      `←/→`, `Tab`/`S-Tab`, `ESC`, `Ctrl+T`, `c/b`.
- [ ] 1.5 Populate `Tables` entries from `handlers/tables.rs` and help.rs lines 272–356
      (CLAUDE.md lines 194–200): `j/k`, `gg/G`, `Ctrl+d/u`, `Enter`/`Space`
      (open table), `Tab` (toggle group), `r` (refresh), `/` (search), `ESC`, `↑/↓`.
- [ ] 1.6 Populate `Details` entries from `handlers/details.rs` and help.rs lines 358–392
      (CLAUDE.md lines 202–205): `j/k`, `↑/↓`, `Ctrl+d/u`, `gg`, `G` (navigation only
      — read-only pane).
- [ ] 1.7 Populate `TabularOutput` entries from `handlers/query_results.rs` and help.rs
      lines 394–498 (CLAUDE.md lines 208–221): `h/j/k/l`/arrows, `gg/G`, `0/$`,
      `Ctrl+d/u`, `i`/`Enter` (edit cell), `Enter` (save edit), `ESC` (cancel),
      `Ctrl+C` (cancel edit), `/` (search), `n/N` (next/prev match), `dd`
      (condition: "double-tap within 500ms"), `yy` (copy row), `yc` (copy cell),
      `dc` (set NULL), `t` (toggle view), `r` (refresh), `x` (close tab),
      `H/L` (prev/next tab).
- [ ] 1.8 Populate `SqlFiles` entries from `handlers/sql_files.rs` and help.rs lines
      500–619 (CLAUDE.md lines 240–246): `j/k`, `Enter`/`Space` (load file), `n`
      (create), `r` (rename), `d` (delete), `/` (search), `ESC`.
- [ ] 1.9 Populate `QueryWindow` entries from `handlers/query_editor.rs` lines 20–112 and
      help.rs lines 621–729 (CLAUDE.md lines 223–238): normal-mode `E`/`Ctrl+Enter`
      (execute), `i/a/o/O` (insert modes), `h/j/k/l`, `w/b/e`, `0/$`, `g`/`G`
      (gg/G for file start/end), `:` (command mode); insert-mode `ESC`, `Enter`
      (newline), typing, `Backspace`, `Tab` (accept suggestion), `↑/↓` (navigate
      suggestions), `Ctrl+p/n` (navigate suggestions vim-style); file management
      sub-section `Ctrl+S`, `Ctrl+O`, `Ctrl+N`.
- [ ] 1.10 Export key char constants: `pub const KEY_HELP: char = '?'`,
       `pub const KEY_DEBUG: char = 'b'`, `pub const KEY_QUIT: char = 'q'`,
       `pub const KEY_QUERY_HISTORY: char = 'y'`.
- [ ] 1.11 Add `pub mod keybindings;` to `src/lib.rs`.

## 2. Refactor `src/ui/help.rs` to render from the table

- [ ] 2.1 Add `use crate::keybindings::{BINDINGS, HelpContext, KeyBinding};` import.
- [ ] 2.2 Add a private `fn bindings_for(ctx: HelpContext) -> impl Iterator<Item = &'static KeyBinding>`
      that filters `BINDINGS` by `context == ctx`, preserving slice order.
- [ ] 2.3 Add a private `fn render_binding_list(lines: &mut Vec<Line<'static>>,
      ctx: HelpContext)` that iterates `bindings_for(ctx)`: emits a styled section-header
      `Line` for entries where `is_section_header` is true, otherwise calls
      `Self::add_command(lines, entry.key, full_desc)` where `full_desc` appends
      `" (condition)"` when `entry.condition` is `Some`.
- [ ] 2.4 Replace the body of `add_connections_commands` with
      `Self::render_binding_list(lines, HelpContext::Connections)`.
- [ ] 2.5 Replace the body of `add_tables_commands` with
      `Self::render_binding_list(lines, HelpContext::Tables)`.
- [ ] 2.6 Replace the body of `add_details_commands` with
      `Self::render_binding_list(lines, HelpContext::Details)`.
- [ ] 2.7 Replace the body of `add_tabular_commands` with
      `Self::render_binding_list(lines, HelpContext::TabularOutput)`.
- [ ] 2.8 Replace the body of `add_sql_files_commands` with
      `Self::render_binding_list(lines, HelpContext::SqlFiles)`.
- [ ] 2.9 Replace the body of `add_query_window_commands` with
      `Self::render_binding_list(lines, HelpContext::QueryWindow)`.
- [ ] 2.10 Replace the global `add_command` literal calls in `create_left_column` (lines
       62–68) and `create_right_column` (lines 93–128) with
       `Self::render_binding_list(lines, HelpContext::Global)`.
- [ ] 2.11 Keep `add_command` helper (line 164), `render_help` (line 732), all layout,
       scroll, and styling logic unchanged. The two-column layout, section headers,
       scroll offset, and pane-name rendering are not touched.

## 3. Point global handler keys at the exported constants

- [ ] 3.1 In `src/app/handlers/global.rs`, add
      `use crate::keybindings::{KEY_HELP, KEY_DEBUG, KEY_QUIT};`.
- [ ] 3.2 Replace `KeyCode::Char('?')` (global.rs line 16) with
      `KeyCode::Char(KEY_HELP)`.
- [ ] 3.3 Replace `KeyCode::Char('b')` (global.rs line 22) with
      `KeyCode::Char(KEY_DEBUG)`.
- [ ] 3.4 Replace `KeyCode::Char('q')` (global.rs line 27) with
      `KeyCode::Char(KEY_QUIT)`.
- [ ] 3.5 After c0011 lands: add `use crate::keybindings::KEY_QUERY_HISTORY` and wire the
       `Ctrl+Y` arm using that constant.

## 4. Author spec delta

- [ ] 4.1 Write `openspec/changes/c0013-keybindings-source-of-truth/specs/help-system/spec.md`
      with `## MODIFIED Requirements` as specified.

## 5. Verify

- [ ] 5.1 `cargo build` clean
- [ ] 5.2 `cargo clippy --all-targets -- -D warnings` clean
- [ ] 5.3 `cargo fmt --check` clean
- [ ] 5.4 `cargo test` green
- [ ] 5.5 Open the app, press `?`: every entry in the help overlay corresponds to a binding
      in `keybindings::BINDINGS` (including `Ctrl+Y` history entry from c0011 and any
      read-only note from c0012). Run:
      `grep -rn '"[A-Za-z?/]' src/ui/help.rs | grep add_command` — must match only the
      `add_command` helper definition (line 164) and zero literal call sites with
      hardcoded key strings, confirming the table is the sole source of truth.
