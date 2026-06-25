# query-history spec delta

## ADDED Requirements

### Requirement: Executed queries are recorded

The system SHALL record every SQL query execution — successful or failed — in
persistent local storage, including the query text, database type, database
name, timestamp, execution time in milliseconds, success flag, and error
message where applicable.

#### Scenario: Successful query is recorded with full metadata

- **WHEN** the user executes a SQL query via `Ctrl+Enter` and the query
  succeeds
- **THEN** a history entry is stored containing the exact query text, the
  active connection's database type (Postgres/MySQL/SQLite), the database
  name, the UTC timestamp of execution, the execution duration in
  milliseconds, `success = true`, and `error_message = NULL`

#### Scenario: Failed query is recorded with error message

- **WHEN** the user executes a SQL query via `Ctrl+Enter` and the query
  returns an error from the database engine
- **THEN** a history entry is stored containing the exact query text, database
  type, database name, timestamp, execution duration, `success = false`, and
  the error message string from the engine

#### Scenario: Recording failure does not affect the query result

- **WHEN** a query executes successfully or fails, AND the history recording
  operation itself fails (e.g. the local SQLite history file is unavailable,
  the pool is uninitialized, or a write error occurs)
- **THEN** the query's own result (the rows on success, or the error toast on
  failure) is delivered to the user unchanged, the recording error is emitted
  as a `WARN`-level trace log entry, and no error toast or modal is shown for
  the recording failure

#### Scenario: History persists across application restarts

- **WHEN** the user quits the application after executing queries and
  relaunches it
- **THEN** previously recorded history entries are still present in the
  overlay and retrievable via `Ctrl+Y`

---

### Requirement: Browse and reuse history

The system SHALL provide a full-screen scrollable overlay that displays the
most recent 200 executed queries, newest first, and SHALL allow the user to
load any historical query into the SQL editor.

#### Scenario: History overlay opens with entries newest-first

- **WHEN** the user presses `Ctrl+Y` while no other overlay is active
- **THEN** the query history overlay opens as a full-screen panel dimming the
  background, displaying recorded entries in reverse chronological order
  (newest at the top), each showing the timestamp, database type, `OK` or
  `ERR` status, execution time in milliseconds, and a truncated (≤ 80
  characters) preview of the query text

#### Scenario: History overlay is accessible without an active DB connection

- **WHEN** the user presses `Ctrl+Y` with no database connection established
- **THEN** the query history overlay opens and displays any previously recorded
  entries; the overlay does not require a live connection because history is
  stored locally

#### Scenario: Enter loads the selected query into the SQL editor

- **WHEN** the query history overlay is open and the user navigates to an
  entry with `j`/`k` (or `Ctrl+d`/`Ctrl+u` for paging) and presses `Enter`
- **THEN** the full, untruncated query text of the selected entry is loaded
  into the SQL editor, the history overlay closes, and the application returns
  to the main view with the SQL editor focused or unchanged-focused

#### Scenario: Escape or Ctrl+Y closes the overlay without loading

- **WHEN** the query history overlay is open and the user presses `Esc` or
  `Ctrl+Y`
- **THEN** the overlay closes and the application returns to the main view;
  the SQL editor content is unchanged

#### Scenario: Empty history state is explained to the user

- **WHEN** the user opens the query history overlay and no entries have been
  recorded yet
- **THEN** the overlay displays an explanatory message (e.g. "No query history
  yet. Execute queries with Ctrl+Enter.") instead of an empty list, and the
  `Enter` key has no effect

#### Scenario: Long queries are truncated in the list but loaded in full

- **WHEN** a history entry's query text exceeds 80 characters and the overlay
  is displaying it in the list
- **THEN** the displayed preview is truncated to 80 characters with a `…`
  suffix; **AND WHEN** the user loads that entry with `Enter`, the full
  untruncated query text is placed in the SQL editor
