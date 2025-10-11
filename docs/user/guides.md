# User Guides & Workflows

This guide provides tips, tricks, and common workflows to help you get the most out of LazyTables.

## Table of Contents

- [Getting Started](#getting-started)
- [Common Workflows](#common-workflows)
- [Productivity Tips](#productivity-tips)
- [Advanced Features](#advanced-features)
- [Best Practices](#best-practices)

---

## Getting Started

### First Steps After Installation

1. **Launch LazyTables**
   ```bash
   lazytables
   ```

2. **Get familiar with the layout**
   - Press `?` to see the help overlay
   - Use numbers `1-6` to jump between panes
   - Try `Tab` and `Shift+Tab` to cycle through panes

3. **Add your first connection**
   - Press `1` to focus Connections pane
   - Press `a` to add a new connection
   - Fill in your database details
   - Press `Enter` to save and test

4. **Browse your database**
   - Press `Enter` on your connection to connect
   - Navigate to Tables pane with `2`
   - Use `j/k` to browse tables
   - Press `Enter` to open a table

### Learning Vim Navigation

If you're new to vim-style navigation:

1. **Start with basic movement**
   - `h` = left, `j` = down, `k` = up, `l` = right
   - Practice in any pane

2. **Learn jump commands**
   - `gg` = jump to top
   - `G` = jump to bottom
   - `0` = start of line
   - `$` = end of line

3. **Master search**
   - `/` = start search in any pane
   - Type to filter
   - `n` = next match, `N` = previous match

---

## Common Workflows

### Workflow 1: Database Exploration

**Goal**: Explore a new database and understand its structure.

1. **Connect to database**
   ```
   Press 1 → Navigate to connection → Press Enter
   ```

2. **Browse tables**
   ```
   Press 2 → Use j/k to navigate → Press Enter to open
   ```

3. **View schema details**
   ```
   Press 3 → Details pane shows columns, indexes, foreign keys
   Press t in Table Viewer to toggle between Data and Schema view
   ```

4. **Check relationships**
   - Schema view shows foreign keys
   - Format: `column → referenced_table(referenced_column)`
   - ON DELETE/UPDATE actions displayed

5. **Explore data**
   ```
   Press 4 → Navigate with h/j/k/l → View cell contents
   ```

### Workflow 2: Query Development

**Goal**: Write, test, and save SQL queries.

1. **Open Query Editor**
   ```
   Press 5 → Press i to enter edit mode
   ```

2. **Write your query**
   ```sql
   SELECT *
   FROM customers
   WHERE created_at > '2024-01-01'
   ORDER BY created_at DESC
   LIMIT 100;
   ```

3. **Execute query**
   ```
   Place cursor on query → Press Ctrl+Enter
   ```

4. **Review results**
   ```
   Press 4 → Navigate results with h/j/k/l
   ```

5. **Save query**
   ```
   Press 5 → Press Ctrl+S → Enter filename
   ```

### Workflow 3: Data Editing

**Goal**: Update data in a table.

1. **Open table**
   ```
   Press 2 → Select table → Press Enter
   ```

2. **Navigate to cell**
   ```
   Press 4 → Use h/j/k/l to find cell
   ```

3. **Edit cell**
   ```
   Press i or Enter → Type new value → Press Enter to save
   ```

4. **Verify changes**
   ```
   Press r to refresh table data
   ```

### Workflow 4: File Management

**Goal**: Organize and reuse saved SQL queries.

1. **View saved queries**
   ```
   Press 6 → Browse SQL files with j/k
   ```

2. **Load a query**
   ```
   Navigate to file → Press Enter
   Query loads in editor (Pane 5)
   ```

3. **Create new query file**
   ```
   Press Ctrl+N → New timestamped file created
   ```

4. **Rename file**
   ```
   Press 6 → Navigate to file → Press r → Enter new name
   ```

5. **Organize queries**
   - Files are automatically organized per connection
   - Use descriptive names for easy finding
   - Delete old queries with `d`

### Workflow 5: Multi-Table Analysis

**Goal**: Work with multiple tables simultaneously.

1. **Open first table**
   ```
   Press 2 → Select table → Press Enter
   ```

2. **Open more tables**
   ```
   Repeat for other tables → Each opens in new tab
   ```

3. **Switch between tables**
   ```
   Press S (previous tab) or D (next tab)
   ```

4. **Close unwanted tabs**
   ```
   Press x to close current tab
   ```

---

## Productivity Tips

### Navigation Tips

1. **Direct pane access**
   - Always use number keys `1-6` for instant pane switching
   - Faster than Tab cycling for specific panes

2. **Search everything**
   - Use `/` liberally to filter long lists
   - Works in Connections, Tables, and Files panes
   - Real-time filtering as you type

3. **Jump commands**
   - `gg` to jump to top of any list
   - `G` to jump to bottom
   - Saves time with long lists

4. **Keyboard focus**
   - Learn `Ctrl+h/j/k/l` for directional pane navigation
   - Feels natural after vim muscle memory kicks in

### Query Editor Tips

1. **Execute specific statements**
   - Place cursor on any SQL statement
   - Press `Ctrl+Enter` to execute just that statement
   - No need to select or highlight

2. **Query snippets**
   - Save commonly used queries as files
   - Load and modify as needed
   - Create templates for common patterns

3. **Multi-statement queries**
   ```sql
   -- Execute each statement separately by placing cursor on it
   SELECT COUNT(*) FROM users;
   SELECT AVG(age) FROM users;
   SELECT * FROM users ORDER BY created_at DESC LIMIT 10;
   ```

4. **Use auto-completion**
   - Start typing SQL keywords
   - Press `Tab` to accept suggestions
   - Speeds up query writing

### Table Viewer Tips

1. **Schema inspection**
   - Press `t` to toggle between Data and Schema view
   - Schema view shows:
     - Columns with types and constraints
     - Indexes with size and type
     - Foreign keys with ON DELETE/UPDATE rules
     - Constraints (CHECK, UNIQUE, etc.)
     - Table statistics (row count, size, last vacuum)

2. **Copy data**
   - Press `yy` to copy entire row as CSV
   - Paste in spreadsheets or other tools

3. **Quick search**
   - Press `/` to search within results
   - Press `n` to jump to next match
   - Press `N` for previous match

4. **Refresh data**
   - Press `r` to reload table data
   - Use after making changes in other tools

### Connection Tips

1. **Connection strings**
   - Use connection string method for quick setup
   - Format: `postgresql://user:pass@host:port/db`
   - Automatically parsed into fields

2. **Test before saving**
   - Connections are tested when you save
   - Fix any issues before saving

3. **Organize connections**
   - Use descriptive names
   - Include environment in name (e.g., "Production DB", "Dev DB")

4. **Quick connect**
   - Press `1` → Navigate → Press `Enter`
   - Connection remembered for session

### File Management Tips

1. **Timestamped files**
   - Use `Ctrl+N` for automatic timestamped files
   - Great for exploratory queries

2. **Descriptive names**
   - Rename important queries with `r`
   - Use names like `daily_report.sql`, `user_analysis.sql`

3. **Delete old files**
   - Keep files pane clean
   - Use `d` to delete unused queries

---

## Advanced Features

### Working with Large Result Sets

LazyTables handles large result sets efficiently:

1. **Virtual scrolling**
   - Smooth scrolling for thousands of rows
   - Only visible rows are rendered

2. **Pagination**
   - Results >10K rows are automatically paginated
   - Configure in `config.toml`:
     ```toml
     [query]
     max_rows = 10000
     ```

3. **Filtering**
   - Use WHERE clauses in queries
   - More efficient than loading all data

### Query History

LazyTables automatically tracks query history:

- History stored per connection
- Access via debug view (`Ctrl+B`)
- Useful for reviewing what queries were run

### Debug Mode

Enable debug mode for troubleshooting:

1. Press `Ctrl+B` to open debug view
2. View logs in real-time
3. See connection status and query execution
4. Press `Ctrl+B` again to close

### Keyboard Shortcuts for Speed

Create your own muscle memory patterns:

```
Common patterns:
1 → a → [fill form] → Enter     # Add connection
2 → / → [search] → Enter         # Find and open table
5 → i → [write query] → Ctrl+Enter → 4  # Write, execute, view results
6 → Enter → 5 → Ctrl+Enter       # Load saved query and execute
```

---

## Best Practices

### Security Best Practices

1. **Credential storage**
   - LazyTables encrypts credentials automatically
   - Never share `~/.lazytables/connections.json`

2. **File permissions**
   ```bash
   chmod 600 ~/.lazytables/connections.json
   chmod 700 ~/.lazytables/
   ```

3. **Production databases**
   - Use read-only accounts when possible
   - Test queries on non-production first
   - Be careful with DELETE/UPDATE statements

### Query Best Practices

1. **Use LIMIT**
   ```sql
   -- Always use LIMIT for exploratory queries
   SELECT * FROM large_table LIMIT 100;
   ```

2. **Use EXPLAIN**
   ```sql
   -- Check query performance
   EXPLAIN ANALYZE SELECT * FROM users WHERE email = 'test@example.com';
   ```

3. **Save useful queries**
   - Save frequently used queries
   - Create templates for common patterns
   - Document complex queries with comments

### Organization Best Practices

1. **Connection naming**
   - Use descriptive names: "Production - Main DB", "Staging - Analytics"
   - Include environment and purpose

2. **File organization**
   - Use descriptive filenames
   - Group related queries in same directory
   - Delete old/unused queries

3. **Workflow optimization**
   - Learn keyboard shortcuts for your most common tasks
   - Use search (`/`) instead of scrolling
   - Keep frequently used queries saved

### Performance Best Practices

1. **Query optimization**
   - Use indexes for filtered columns
   - Limit result sets with WHERE clauses
   - Avoid SELECT * for large tables

2. **Connection management**
   - Close connections when done
   - Use connection pooling (automatic)
   - Monitor connection status in Connections pane

3. **Result management**
   - Close unused tabs to free memory
   - Use appropriate `max_rows` setting
   - Refresh tables only when needed

---

## Tips for Specific Databases

### PostgreSQL

- Schema view shows comprehensive metadata
- Use `EXPLAIN ANALYZE` for query optimization
- Check `pg_stat_user_tables` for table statistics

### MySQL / MariaDB

- Schema view shows engine type (InnoDB, MyISAM)
- Check `SHOW TABLE STATUS` for detailed info
- Use `EXPLAIN` for query analysis

### SQLite

- Schema view shows indexes and constraints
- Use `EXPLAIN QUERY PLAN` for optimization
- Check `PRAGMA` commands for database info

---

## Getting Help

### Built-in Help

- Press `?` in any pane for context-aware help
- Help shows keybindings for current pane
- Updated automatically based on focus

### Documentation

- [Installation Guide](installation.md) - Setup and installation
- [Key Bindings](key-bindings.md) - Complete keyboard reference
- [Configuration](configuration.md) - Customize LazyTables
- [Troubleshooting](troubleshooting.md) - Fix common issues

### Community

- [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues) - Report bugs
- [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions) - Ask questions

---

## Next Steps

Now that you know the workflows and tips:

1. Practice the common workflows above
2. Create your own keyboard shortcut muscle memory
3. Customize configuration to your preferences
4. Share your tips with the community

**Pro tip**: The more you use keyboard shortcuts, the faster you'll become. Challenge yourself to avoid using the mouse!
