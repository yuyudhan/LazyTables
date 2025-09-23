# Testing SQL Files Connection-Specific Behavior

## Test Scenario

This document describes how to test the connection-specific SQL files functionality.

## Expected Behavior

1. **Initial State (No Connection)**:
   - SQL Files pane should be empty
   - No SQL files should be visible
   - Status should indicate no files

2. **After Connecting to a Database**:
   - SQL Files pane should show only files specific to that connection
   - Files are stored in `~/.lazytables/sql_files/{connection_name}/`
   - Each connection has its own isolated set of SQL files

3. **When Disconnecting**:
   - SQL Files pane should immediately become empty
   - No SQL files should be visible
   - Should return to initial state

4. **Switching Between Connections**:
   - Each connection should show only its own SQL files
   - Files from other connections should not be visible
   - File lists should update immediately when connections change

## Test Steps

1. Start LazyTables: `./target/release/lazytables`
2. Verify SQL Files pane is empty initially
3. Create a database connection (add connection 'test1')
4. Connect to 'test1' - verify SQL Files pane becomes available
5. Create a SQL file (Ctrl+N) and save it (Ctrl+S)
6. Verify the file appears in SQL Files pane
7. Disconnect (press 'x' in connections pane)
8. Verify SQL Files pane becomes empty again
9. Create another connection 'test2'
10. Connect to 'test2' - verify SQL Files pane is empty (different connection)
11. Create a SQL file for 'test2'
12. Switch back to 'test1' connection
13. Verify only 'test1' files are visible

## File Structure

Each connection should create files in:
```
~/.lazytables/sql_files/
├── test1/
│   ├── query1.sql
│   └── query2.sql
└── test2/
    ├── another_query.sql
    └── test_query.sql
```

## Implementation Details

- SQL files are now strictly connection-specific
- No shared SQL files across connections
- Files only show when connection is actively connected
- Automatic refresh when connection state changes
- App state database tracks active connections