# 002 - First Steps

This guide will help you get started with LazyTables after installation, from launching the application to connecting to your first database.

## Launching LazyTables

### Basic Launch

```bash
# Launch LazyTables
lazytables
```

On first launch, you'll see the four-pane interface with empty connections.

### Launch Options

```bash
# Show help
lazytables --help

# Show version
lazytables --version

# Launch with debug logging
RUST_LOG=debug lazytables

# Launch with specific config file
lazytables --config /path/to/config.toml
```

## Understanding the Interface

LazyTables uses a **four-pane layout** designed for efficient database navigation:

```
┌─────────────┬─────────────────────────────┐
│ Connections │                             │
│             │                             │
├─────────────┤        Main Content         │
│ Tables/     │                             │
│ Views       │                             │
├─────────────┤                             │
│ Table       │                             │
│ Details     │                             │
└─────────────┴─────────────────────────────┘
```

### Pane Overview

1. **Connections Pane** (Top Left)
   - Lists all configured database connections
   - Shows connection status (connected/disconnected)
   - Access with focus shortcut: `c`

2. **Tables/Views Pane** (Middle Left)  
   - Shows tables and views for selected database
   - Displays table types and schemas
   - Access with focus shortcut: `t`

3. **Table Details Pane** (Bottom Left)
   - Shows metadata for selected table
   - Column information, indexes, constraints
   - Access with focus shortcut: `d`

4. **Main Content Area** (Right)
   - Query editor and results display
   - Primary workspace for data interaction
   - Access with focus shortcut: `m`

## Basic Navigation

### Moving Around

LazyTables uses **vim-style navigation** throughout:

```bash
# Navigate within active pane
h, j, k, l          # Left, Down, Up, Right
gg                  # Go to top
G                   # Go to bottom
0                   # Go to beginning of line
$                   # Go to end of line

# Switch between panes
Ctrl+h              # Move to left pane
Ctrl+j              # Move to pane below
Ctrl+k              # Move to pane above  
Ctrl+l              # Move to right pane
Tab                 # Cycle to next pane
Shift+Tab           # Cycle to previous pane
```

### Focus Shortcuts

Quickly jump to specific panes:

```bash
c                   # Focus Connections pane
t                   # Focus Tables pane
d                   # Focus Details pane
m                   # Focus Main content area
```

### Essential Commands

```bash
:q                  # Quit LazyTables (requires command mode)
?                   # Show help overlay
:                   # Enter command mode
Esc                 # Return to normal mode
```

## Your First Connection

### Adding a Connection

1. **Focus the Connections pane**: Press `c` or navigate with `Ctrl+h`
2. **Add new connection**: Press `a`
3. **Fill in connection details** in the dialog:
   - Connection name (for display)
   - Database type (PostgreSQL, MySQL, etc.)
   - Host and port
   - Username and password
   - Database name (optional)
4. **Test connection**: The dialog will test connectivity
5. **Save connection**: Press `Enter` to save, `Esc` to cancel

### Example Connection Setup

**PostgreSQL Connection**:
```
Name: Local Development
Type: PostgreSQL
Host: localhost
Port: 5432
Username: your_username
Password: your_password
Database: your_database
SSL: false
```

**MySQL Connection**:
```
Name: Production MySQL
Type: MySQL
Host: mysql.example.com
Port: 3306
Username: app_user
Password: secure_password
Database: production_db
SSL: true
```

### Connecting to a Database

1. **Select connection**: Navigate to the connection in the list
2. **Connect**: Press `Enter` or `Space`
3. **View status**: Connected databases show a green indicator
4. **Disconnect**: Press `d` to disconnect

## Exploring Your Database

### Browsing Tables

1. **Select a connected database** in the Connections pane
2. **Navigate to Tables pane**: Press `t` or `Ctrl+j`
3. **Browse tables**: Use `j/k` to move up/down the list
4. **Select a table**: Press `Enter` to select
5. **View table data**: Data appears in the Main Content area

### Viewing Table Details

1. **Select a table** in the Tables pane
2. **Check Table Details pane**: Automatically shows:
   - Column names and types
   - Primary keys and indexes
   - Foreign key relationships
   - Table statistics

### Basic Data Viewing

When you select a table:
- **First 100 rows** load automatically
- **Column headers** show data types
- **Navigation**: Use arrow keys to scroll
- **Pagination**: Use `Page Up/Page Down` for more data

## Running Your First Query

### Opening the Query Editor

1. **Focus Main Content area**: Press `m`
2. **Enter Query mode**: Press `q` (switches to query editor)
3. **Type your query**:
   ```sql
   SELECT * FROM users LIMIT 10;
   ```
4. **Execute query**: Press `Ctrl+Enter`
5. **View results**: Results display below the query editor

### Query Editor Features

```bash
# Query editing
Ctrl+Enter          # Execute query
Ctrl+S              # Save query to file
Ctrl+O              # Open query from file
Ctrl+Z              # Undo
Ctrl+Y              # Redo

# Result navigation
j, k                # Scroll results up/down
h, l                # Scroll results left/right
gg, G               # Jump to top/bottom of results
```

## Common First-Time Tasks

### 1. Test Connection

```sql
-- PostgreSQL
SELECT version();

-- MySQL
SELECT VERSION();

-- SQLite
SELECT sqlite_version();
```

### 2. List All Tables

```sql
-- PostgreSQL
SELECT tablename FROM pg_tables WHERE schemaname = 'public';

-- MySQL
SHOW TABLES;

-- SQLite
SELECT name FROM sqlite_master WHERE type='table';
```

### 3. Explore Table Structure

```sql
-- PostgreSQL
\d table_name

-- MySQL  
DESCRIBE table_name;

-- Or universal
SELECT column_name, data_type, is_nullable 
FROM information_schema.columns 
WHERE table_name = 'your_table';
```

### 4. View Sample Data

```sql
SELECT * FROM your_table LIMIT 5;
```

## Customizing Your Experience

### Basic Configuration

LazyTables creates a config file at:
- **macOS**: `~/Library/Application Support/LazyTables/config.toml`
- **Linux**: `~/.config/lazytables/config.toml`

### Common Settings

```toml
[display]
theme = "default"           # Color theme
page_size = 100            # Rows per page
show_line_numbers = true   # Show line numbers in query editor

[behavior]
auto_connect = true        # Auto-connect to last used database
confirm_exit = false       # Ask before quitting
save_query_history = true  # Save executed queries

[keys]
quit = ":q"               # Command to quit (requires command mode)
help = "?"                # Key to show help
```

### Keyboard Shortcuts Reference

Press `?` anytime to see a quick reference of all keyboard shortcuts.

## Getting Help

### Built-in Help

```bash
?                   # Show keyboard shortcuts
:help               # Detailed help in command mode
:help connections   # Help for specific topics
```

### External Resources

- **Documentation**: [docs/help/](../help/) directory
- **GitHub Issues**: Report bugs and request features
- **Discussions**: Get community support
- **Examples**: Check `examples/` directory for sample queries

## Troubleshooting First Steps

### Connection Issues

**"Connection refused"**:
- Check database server is running
- Verify host and port are correct
- Check firewall settings

**"Authentication failed"**:
- Verify username and password
- Check database user permissions
- Confirm database exists

**"SSL/TLS errors"**:
- Try disabling SSL for local connections
- Check SSL certificate validity
- Verify SSL configuration

### Interface Issues

**"Terminal too small"**:
- Resize terminal to at least 80x24
- Recommended: 120x40 for best experience

**"Garbled text"**:
- Ensure terminal supports UTF-8
- Check terminal color support (256 colors)
- Try different terminal emulator

**"Keyboard shortcuts not working"**:
- Ensure terminal is in focus
- Check for conflicting terminal shortcuts
- Try alternative key combinations

## Next Steps

Now that you're familiar with the basics:

1. **Learn advanced navigation**: [003 - Navigation](003-navigation.md)
2. **Master connection management**: [004 - Managing Connections](004-managing-connections.md)  
3. **Explore querying features**: [005 - Querying Data](005-querying-data.md)
4. **Customize your setup**: [006 - Configuration](006-configuration.md)

## Practice Exercise

Try this simple workflow to get comfortable:

1. **Add a connection** to your local database
2. **Connect and browse** the available tables
3. **Select a table** and view its structure
4. **Run a simple query**: `SELECT COUNT(*) FROM table_name;`
5. **Navigate results** using keyboard shortcuts
6. **Disconnect** and try connecting to another database

Congratulations! You're now ready to use LazyTables effectively. 🎉