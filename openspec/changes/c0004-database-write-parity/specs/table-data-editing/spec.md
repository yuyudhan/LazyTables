# table-data-editing spec delta

## ADDED Requirements

### Requirement: Edit a cell on any supported engine

The system SHALL execute a dialect-correct `UPDATE` statement for any connected
engine (PostgreSQL, MySQL, MariaDB, SQLite) when the user saves an edited cell
value on a row that has a primary key, and SHALL reject the edit with an error
message when the row has no primary key without executing any SQL.

#### Scenario: Cell edit persists on all three engines

- **WHEN** the user is connected to PostgreSQL, MySQL, or SQLite and saves an
  edited cell for a row that has a primary key
- **THEN** the system executes
  `UPDATE <quoted-table> SET <quoted-column> = '<escaped-value>' WHERE <quoted-pk-col> = '<escaped-pk-val>'`
  using the correct identifier-quoting style for the engine (double-quotes for
  PG/SQLite, backticks for MySQL), and the table reloads showing the updated value

#### Scenario: Value containing a single quote is stored literally

- **WHEN** the user saves a cell value that contains a single-quote character
  (for example `O'Brien`)
- **THEN** the single quote is escaped to `''` in the generated SQL
  (`SET col = 'O''Brien'`) and the stored value is the literal string `O'Brien`,
  not a syntax error or a truncated value

#### Scenario: Edit rejected when no primary key exists

- **WHEN** the user attempts to save an edited cell on a row for which
  `primary_key_values` is empty
- **THEN** no SQL is executed, and the operation returns an error message
  (`"Cannot update row without primary key"`)

---

### Requirement: Delete a row on any supported engine

The system SHALL execute a dialect-correct `DELETE` statement for any connected
engine when the user confirms row deletion on a row that has a primary key, and
SHALL reject the deletion with an error message when the row has no primary key
without executing any SQL.

#### Scenario: Row deletion with a composite primary key

- **WHEN** the user confirms deletion of a row whose primary key spans two or
  more columns
- **THEN** the system executes
  `DELETE FROM <quoted-table> WHERE <quoted-pk1> = '<val1>' AND <quoted-pk2> = '<val2>'`
  and the row is absent from the table on the next reload

#### Scenario: Deletion rejected when no primary key exists

- **WHEN** the user confirms deletion of a row for which `primary_key_values`
  is empty
- **THEN** no SQL is executed, and the operation returns an error message
  (`"Cannot delete row without primary key"`)

---

### Requirement: Set a cell to NULL on any supported engine

The system SHALL execute a dialect-correct `UPDATE … SET col = NULL` statement
(without quotes around `NULL`) for any connected engine when the user confirms
the set-NULL action, and SHALL distinguish this operation from setting the cell
to an empty string.

#### Scenario: Set-NULL produces a NULL value, not an empty string

- **WHEN** the user confirms the set-NULL action on a cell
- **THEN** the system executes
  `UPDATE <quoted-table> SET <quoted-column> = NULL WHERE <pk-clause>`
  (the keyword `NULL` is unquoted in the SQL), and after reload the cell
  displays as `NULL`, not as an empty string

#### Scenario: Set-NULL rejected when no primary key exists

- **WHEN** the user confirms set-NULL on a row for which `primary_key_values`
  is empty
- **THEN** no SQL is executed, and the operation returns an error message
  (`"Cannot update cell without primary key"`)
