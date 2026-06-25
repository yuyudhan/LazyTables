# help-system spec delta

## MODIFIED Requirements

### Requirement: Help overlay reflects real bindings

The system SHALL derive every key/description entry displayed in the help overlay from a single
`keybindings::BINDINGS` static table (`src/keybindings.rs`). No key string or description that
appears in the help overlay may be defined only in `src/ui/help.rs` (i.e., as a literal argument
to `add_command` or equivalent).

#### Scenario: Help content matches the binding table on open

- **WHEN** the user presses `?` to open the help overlay from any pane
- **THEN** every key/description row shown in the overlay corresponds to an entry in
  `keybindings::BINDINGS` for the matching `HelpContext`, and no entry is shown that does not
  exist in the table

#### Scenario: New binding appears automatically

- **WHEN** a developer adds a new `KeyBinding` entry to `keybindings::BINDINGS`
- **THEN** that entry appears in the help overlay under the appropriate pane section on the next
  build, without any additional edit to `src/ui/help.rs`

### Requirement: Conditional bindings are annotated

The system SHALL annotate context-conditional bindings in the help overlay with a short condition
note so users understand when the key is active.

#### Scenario: Quit key shows condition

- **WHEN** the user views the Global section of the help overlay
- **THEN** the `q` (Quit) entry displays a condition note indicating it is not active in
  edit/search/insert modes (derived from `can_quit` logic in `handlers/global.rs`)

#### Scenario: Tab key shows insert-mode exception

- **WHEN** the user views the Global section of the help overlay
- **THEN** the `Tab` (Next pane) entry displays a condition note indicating it is inactive when
  the Query Editor is in insert mode

#### Scenario: Double-tap bindings show their condition

- **WHEN** the user views the Table Viewer section of the help overlay
- **THEN** the `dd` (Delete row) and `yy` (Copy row) entries display a condition note indicating
  they require a double-tap within 500 ms

### Requirement: Help layout and navigation are preserved

The system SHALL preserve the current two-column layout, section emoji headers, scroll offset
behaviour, and pane-switching navigation of the help overlay after the `keybindings.rs` refactor.

#### Scenario: Section headers remain in help

- **WHEN** the help overlay opens on the Connections pane
- **THEN** the sub-section headers (e.g., "Connection Management", "Search & Filter",
  "Connection Modal") are still present and visually styled as before

#### Scenario: Scroll and pane navigation still work

- **WHEN** the help overlay is open
- **THEN** pressing `j/k` scrolls the content and pressing `h/l` (or arrow keys) switches
  between the left (pane-specific) and right (global) columns, identical to pre-refactor behaviour
