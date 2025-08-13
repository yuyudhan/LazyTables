# 005 - Querying Data

Master SQL querying in LazyTables with the built-in query editor, result navigation, and data exploration features.

## Query Editor Overview

LazyTables provides a powerful query editor integrated into the main content area, designed for efficient SQL development with vim-style editing.

### Opening the Query Editor

```bash
# Focus main content area
m                   # Jump to main area
q                   # Enter query mode

# Alternative: Direct query mode
:query              # Command mode query access
```

### Query Editor Interface

```
┌─── Query Editor ────────────────────────────────────────────┐
│ SELECT u.username, u.email, COUNT(o.id) as order_count     │
│ FROM users u                                               │
│ LEFT JOIN orders o ON u.id = o.user_id                    │
│ WHERE u.created_at > '2024-01-01'                         │
│ GROUP BY u.id, u.username, u.email                        │
│ ORDER BY order_count DESC                                  │
│ LIMIT 10;                                                  │
├─── Results ─────────────────────────────────────────────────┤
│ username    │ email              │ order_count             │
│ alice_dev   │ alice@example.com  │ 15                      │
│ bob_user    │ bob@example.com    │ 12                      │
│ charlie_admin│ charlie@example.com│ 8                       │
└─────────────────────────────────────────────────────────────┘
```

## Writing Queries

### Basic Query Editing

**Insert Mode Navigation**:
```bash
i                   # Insert at cursor
I                   # Insert at line beginning
a                   # Append after cursor
A                   # Append at line end
o                   # New line below
O                   # New line above
Esc                 # Return to normal mode
```

**Normal Mode Editing**:
```bash
# Movement
h, j, k, l          # Character and line movement
w, b, e             # Word movement
gg, G               # Start/end of query
0, $                # Start/end of line

# Editing
x                   # Delete character
dd                  # Delete line
yy                  # Copy line
p                   # Paste
u                   # Undo
Ctrl+r              # Redo

# Search and replace
/pattern            # Search forward
?pattern            # Search backward
:s/old/new/g        # Replace in line
:%s/old/new/g       # Replace in all lines
```

### SQL Syntax Features

**Syntax Highlighting**:
- **Keywords**: `SELECT`, `FROM`, `WHERE` in blue
- **Strings**: Text literals in green  
- **Comments**: `--` and `/* */` in gray
- **Numbers**: Numeric literals highlighted
- **Functions**: Built-in functions in purple

**Auto-Indentation**:
```sql
SELECT 
    u.username,
    u.email,
    COUNT(o.id) as order_count
FROM users u
LEFT JOIN orders o 
    ON u.id = o.user_id
WHERE 
    u.created_at > '2024-01-01'
    AND u.status = 'active'
GROUP BY 
    u.id, 
    u.username, 
    u.email
ORDER BY order_count DESC
LIMIT 10;
```

## Executing Queries

### Basic Execution

```bash
Ctrl+Enter          # Execute current query
Ctrl+Shift+Enter    # Execute selected text
:exec               # Command mode execution
```

### Query Execution Modes

**Full Query Execution**:
- Executes entire query in editor
- Results replace previous output
- Execution time displayed

**Partial Query Execution**:
- Select text in visual mode (`v`)
- Execute only selected SQL
- Useful for testing portions of complex queries

**Multiple Query Execution**:
- Separate queries with semicolons
- Execute all queries sequentially
- Results displayed for each query

### Example Execution Flow

```sql
-- Query 1: Basic selection
SELECT COUNT(*) FROM users;

-- Query 2: Detailed analysis
SELECT 
    DATE_TRUNC('month', created_at) as month,
    COUNT(*) as new_users
FROM users 
WHERE created_at >= '2024-01-01'
GROUP BY month
ORDER BY month;

-- Query 3: Join analysis
SELECT 
    u.username,
    COUNT(DISTINCT o.id) as orders,
    SUM(o.total) as revenue
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.username
HAVING COUNT(o.id) > 0
ORDER BY revenue DESC
LIMIT 20;
```

## Navigating Results

### Result Display Features

**Column Information**:
- Column names with data types
- Sortable headers (future feature)
- Resizable columns based on content
- Null value indicators

**Row Navigation**:
```bash
# Basic movement
j, k                # Move up/down rows
h, l                # Move left/right columns
gg, G               # First/last row
0, $                # First/last column

# Page navigation  
Ctrl+f              # Page down
Ctrl+b              # Page up
Page Down/Up        # Alternative page navigation

# Jump navigation
:{number}           # Jump to specific row
/pattern            # Search within results
n, N                # Next/previous search match
```

### Large Dataset Handling

**Lazy Loading**:
- Results load incrementally
- Scroll to load more data
- Configurable page size
- Memory-efficient for large datasets

**Virtual Scrolling**:
```bash
# For datasets with 1M+ rows
:set page-size 1000     # Increase page size
:set virtual-scroll on  # Enable virtual scrolling
:limit 10000           # Limit results for performance
```

**Pagination Controls**:
```bash
]                   # Next page of results
[                   # Previous page of results
:page 5             # Jump to specific page
:first              # First page
:last               # Last page
```

## Query Management

### Saving and Loading Queries

**Save Query**:
```bash
Ctrl+S              # Save current query
:write filename.sql # Save with specific name
:w                  # Quick save
```

**Load Query**:
```bash
Ctrl+O              # Open file browser
:read filename.sql  # Load specific file
:r                  # Quick open recent
```

**Query History**:
```bash
:history            # Show query history
:h                  # Quick history access
Ctrl+Up/Down        # Navigate history in editor
```

### Query Templates

**Common Templates**:
```sql
-- Basic SELECT template
SELECT column1, column2
FROM table_name
WHERE condition
ORDER BY column1
LIMIT 100;

-- JOIN template
SELECT t1.col1, t2.col2
FROM table1 t1
JOIN table2 t2 ON t1.id = t2.foreign_key
WHERE t1.status = 'active';

-- Aggregation template
SELECT 
    category,
    COUNT(*) as count,
    AVG(value) as avg_value
FROM table_name
GROUP BY category
ORDER BY count DESC;
```

**Template Usage**:
```bash
:template select    # Load SELECT template
:template join      # Load JOIN template
:template agg       # Load aggregation template
```

## Advanced Query Features

### Query Building Assistance

**Schema Introspection**:
- Auto-complete table names (future)
- Column name suggestions (future)
- Function name completion (future)

**Query Validation**:
- Syntax checking before execution
- Warning for potentially expensive queries
- Estimation of result size

### Database-Specific Features

**PostgreSQL-Specific**:
```sql
-- EXPLAIN ANALYZE
EXPLAIN ANALYZE SELECT * FROM large_table;

-- JSON operations
SELECT data->>'name' FROM json_table;

-- Array operations
SELECT ARRAY_AGG(id) FROM users;

-- Window functions
SELECT 
    username,
    ROW_NUMBER() OVER (ORDER BY created_at) as user_number
FROM users;
```

**MySQL-Specific**:
```sql
-- SHOW commands
SHOW TABLES;
SHOW COLUMNS FROM table_name;
SHOW INDEXES FROM table_name;

-- MySQL functions
SELECT DATE_FORMAT(created_at, '%Y-%m') FROM table_name;
```

**SQLite-Specific**:
```sql
-- Pragmas
PRAGMA table_info(table_name);
PRAGMA foreign_key_list(table_name);

-- SQLite functions
SELECT datetime(created_at, 'localtime') FROM table_name;
```

## Query Performance

### Performance Monitoring

**Execution Metrics**:
- Query execution time
- Rows returned
- Rows examined (when available)
- Memory usage

**Performance Display**:
```
Query executed in 0.234s
Returned 1,245 rows (examined 10,000 rows)
Memory usage: 2.1 MB
```

### Optimization Tips

**Query Optimization**:
```sql
-- Use LIMIT for exploratory queries
SELECT * FROM large_table LIMIT 100;

-- Use indexes effectively
SELECT * FROM users WHERE email = 'user@example.com'; -- indexed column

-- Avoid SELECT *
SELECT id, username, email FROM users; -- specific columns

-- Use appropriate JOINs
SELECT u.username, p.title
FROM users u
INNER JOIN posts p ON u.id = p.user_id; -- INNER vs LEFT JOIN
```

**Performance Settings**:
```bash
:set query-timeout 30       # 30 second timeout
:set max-rows 10000        # Limit result rows
:set explain-mode on       # Auto-explain queries
```

## Data Export

### Export Formats

**CSV Export**:
```bash
:export csv filename.csv    # Export results to CSV
:export csv                 # Export with auto-generated name
```

**JSON Export**:
```bash
:export json data.json      # Export as JSON array
:export json-lines data.jsonl # Export as JSON lines
```

**SQL Export**:
```bash
:export sql insert.sql      # Export as INSERT statements
:export sql update.sql      # Export as UPDATE statements
```

### Export Options

```toml
[export]
include_headers = true      # Include column headers
null_value = "NULL"        # How to represent NULL values
date_format = "iso8601"    # Date formatting
escape_quotes = true       # Escape quotes in strings
```

## Query Debugging

### Error Handling

**SQL Errors**:
- Syntax error highlighting
- Detailed error messages
- Line number indication
- Suggested corrections (when possible)

**Connection Errors**:
- Connection timeout handling
- Automatic reconnection attempts
- Error recovery suggestions

### Debug Mode

```bash
:debug on               # Enable debug mode
:debug query           # Debug current query
:debug connection      # Debug connection issues
:debug off             # Disable debug mode
```

**Debug Information**:
- Query execution plan
- Connection status
- Server version information
- Performance statistics

## Best Practices

### Query Writing

1. **Start Simple**: Begin with basic SELECT, add complexity gradually
2. **Use LIMIT**: Always limit results during development
3. **Comment Complex Logic**: Use `--` for inline comments
4. **Test Incrementally**: Build queries step by step
5. **Check Performance**: Monitor execution time for slow queries

### Result Analysis

1. **Verify Data Types**: Check column types in results
2. **Handle NULLs**: Be aware of NULL values in calculations
3. **Check Row Counts**: Verify expected number of results
4. **Sample Large Results**: Use LIMIT for initial exploration
5. **Export for Analysis**: Save results for external tools

### Security Considerations

1. **Parameterize Values**: Use proper escaping for user input
2. **Limit Permissions**: Use read-only accounts when possible
3. **Avoid Sensitive Data**: Don't query production secrets
4. **Log Monitoring**: Be aware that queries may be logged
5. **Connection Security**: Use SSL for remote connections

## Troubleshooting Queries

### Common Issues

**"Query timeout"**:
- Increase timeout: `:set query-timeout 60`
- Optimize query with indexes
- Add LIMIT clause
- Check for table locks

**"Too many rows returned"**:
- Add LIMIT clause
- Increase max-rows: `:set max-rows 50000`
- Use pagination
- Export large results instead of viewing

**"Syntax error"**:
- Check SQL syntax for your database type
- Verify table and column names
- Check for missing quotes or semicolons
- Use database-specific syntax

**"Connection lost"**:
- Check network connectivity
- Verify database server status
- Reconnect: `:reconnect`
- Check connection timeout settings

## Next Steps

Now that you can query data effectively:
- Explore [006 - Configuration](006-configuration.md) to customize your experience
- Learn [007 - Keyboard Shortcuts](007-keyboard-shortcuts.md) for maximum efficiency
- Check [008 - Tips and Tricks](008-tips-and-tricks.md) for advanced techniques

Happy querying! 🔍