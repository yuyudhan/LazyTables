# cli-startup spec delta

## ADDED Requirements

### Requirement: Auto-connect to a named connection on startup

The system SHALL accept a `--connection <name>` flag at launch and automatically
initiate a connection to the saved connection whose `name` field matches the supplied
value, surfacing the standard connecting animation and producing the same success/failure
toasts as a manual connection.

#### Scenario: Known connection name

- **WHEN** the user launches `lazytables --connection <name>` and `<name>` matches the
  `name` field of a saved connection
- **THEN** the app starts, displays the connecting animation, and on success transitions
  to the connected state (tables loaded, connection marked Connected) without any user
  interaction

#### Scenario: Unknown connection name

- **WHEN** the user launches `lazytables --connection <name>` and no saved connection
  has that name and `<name>` does not contain `://`
- **THEN** a startup toast is shown ("--connection '<name>': connection not found") and
  the app opens normally on the connections pane with no connection attempted

#### Scenario: Connection string auto-connect

- **WHEN** the user launches `lazytables --connection <uri>` and `<uri>` contains `://`
  (e.g. `postgres://user:pass@host/db`)
- **THEN** the app parses the URI via `AdapterFactory::create_connection_from_string`,
  connects to it as a transient (unsaved) connection, and on success is in the connected
  state; if parsing fails a toast describes the error and the app opens normally

#### Scenario: Connection fails

- **WHEN** auto-connect is attempted and the engine rejects the connection
- **THEN** the failure toast appears (identical to a manually initiated failed
  connection), any pending `--table` or `--database` intent is discarded, and the app
  opens normally on the connections pane

---

### Requirement: Auto-open a table after startup auto-connect

The system SHALL accept a `--table <name>` flag (in combination with `--connection`)
and automatically open the named table in the table viewer once the startup connection
succeeds.

#### Scenario: Table found after connect

- **WHEN** the user launches `lazytables --connection <c> --table <t>` and the
  connection succeeds and `<t>` is present in the database's table list
- **THEN** after the connection success event is processed, the table `<t>` is selected
  in the UI and `open_table_for_viewing` is called, opening the table in the viewer and
  switching focus to the tabular output pane

#### Scenario: Table not found after connect

- **WHEN** the connection succeeds but `<t>` does not appear in `state.db.tables`
- **THEN** a toast "--table '<t>': not found in database" is shown and the app remains
  on the normal post-connect state (tables pane, no auto-opened tab)

#### Scenario: `--table` without `--connection`

- **WHEN** the user launches `lazytables --table <t>` with no `--connection` flag
- **THEN** a startup toast "--table/--database requires --connection; ignored" is shown
  and the flag has no effect; the app opens normally

---

### Requirement: Scope to a database or schema on startup

The system SHALL accept a `--database <name>` flag (in combination with `--connection`)
and, after a successful startup auto-connect, scope the table list to the named
database or schema by setting `selected_schema`.

#### Scenario: Database found after connect

- **WHEN** the user launches `lazytables --connection <c> --database <d>` and the
  connection succeeds and `<d>` matches an entry in the loaded `database_objects`
- **THEN** `selected_schema` is set to `<d>` and the table list is filtered
  accordingly

#### Scenario: Database not found or not applicable

- **WHEN** `<d>` is not present in `database_objects` or the engine does not support
  named schemas
- **THEN** a toast "--database '<d>': not found in this connection" is shown; no schema
  filter is applied; the full object list is shown

---

### Requirement: Read-only mode blocks all data mutations

The system SHALL accept a `--read-only` flag at launch and, when it is present, reject
every operation that would modify database data (cell edit, row delete, set-NULL) or
execute a write SQL statement, surfacing a clear toast instead and leaving all data
unchanged.

#### Scenario: Cell edit blocked in read-only mode

- **WHEN** the app is started with `--read-only` and the user completes a cell edit in
  the table viewer (enters edit mode, modifies the buffer, presses Enter/Esc to commit)
- **THEN** the `update_table_cell` call is skipped, a toast
  "Read-only mode: cell edits are disabled" is shown, and the cell value in the database
  is unchanged

#### Scenario: Row deletion blocked in read-only mode

- **WHEN** the app is started with `--read-only` and the user triggers row deletion
  (confirmation modal would normally appear)
- **THEN** the deletion is blocked before or at the confirmation step, a toast
  "Read-only mode: row deletion is disabled" is shown, and no row is deleted

#### Scenario: Set-NULL blocked in read-only mode

- **WHEN** the app is started with `--read-only` and the user triggers set-NULL on a cell
- **THEN** the operation is blocked, a toast "Read-only mode: set-NULL is disabled"
  is shown, and the cell value is unchanged

#### Scenario: Write SQL blocked in read-only mode

- **WHEN** the app is started with `--read-only` and the user executes a query whose
  first keyword is one of INSERT, UPDATE, DELETE, DROP, ALTER, TRUNCATE, or CREATE
- **THEN** execution is skipped, a toast "Read-only mode: write queries are disabled"
  is shown, and no SQL is sent to the database engine

#### Scenario: Read SQL executes normally in read-only mode

- **WHEN** the app is started with `--read-only` and the user executes a SELECT (or
  any query not matching the write-keyword list)
- **THEN** the query executes normally and results are displayed; the read-only flag
  does not restrict read operations

---

### Requirement: No-flags launch is unchanged

The system SHALL behave identically to the pre-change binary when launched with no
startup flags.

#### Scenario: Bare invocation

- **WHEN** `lazytables` is launched with no `--connection`, `--database`, `--table`,
  or `--read-only` flags
- **THEN** the app opens on the connections pane in the normal disconnected state, with
  no auto-connect attempt, no table pre-opened, and no mutation restrictions applied
