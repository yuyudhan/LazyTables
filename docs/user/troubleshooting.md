# Troubleshooting Guide

This guide helps you diagnose and fix common issues with LazyTables.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Connection Issues](#connection-issues)
- [Performance Issues](#performance-issues)
- [UI and Display Issues](#ui-and-display-issues)
- [Query Execution Issues](#query-execution-issues)
- [File and Configuration Issues](#file-and-configuration-issues)
- [Keyboard Navigation Issues](#keyboard-navigation-issues)
- [Getting More Help](#getting-more-help)

---

## Installation Issues

### "cargo: command not found"

**Problem**: Rust is not installed or not in your PATH.

**Solution**:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart terminal or reload shell
source $HOME/.cargo/env

# Verify installation
cargo --version
```

### "lazytables: command not found" after installation

**Problem**: Cargo bin directory is not in your PATH.

**Solution**:
```bash
# Add cargo bin to PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# For zsh users
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# Verify
lazytables --version
```

### Compilation errors during installation

**Problem**: Missing system dependencies or outdated Rust version.

**Solution**:
```bash
# Update Rust to latest version
rustup update

# Install system dependencies
# macOS:
brew install postgresql mysql-client sqlite

# Linux (Ubuntu/Debian):
sudo apt-get update
sudo apt-get install libpq-dev libmysqlclient-dev libsqlite3-dev pkg-config build-essential

# Try installation again
cargo install lazytables --force
```

### "error: linker 'cc' not found"

**Problem**: C compiler not installed.

**Solution**:
```bash
# macOS:
xcode-select --install

# Linux (Ubuntu/Debian):
sudo apt-get install build-essential

# Linux (Fedora):
sudo dnf install gcc
```

---

## Connection Issues

### "Connection failed" or "Unable to connect to database"

**Problem**: Database not accessible or incorrect credentials.

**Solutions**:

1. **Verify database is running**:
   ```bash
   # PostgreSQL
   pg_isready -h localhost -p 5432

   # MySQL
   mysqladmin ping -h localhost -P 3306
   ```

2. **Check credentials**:
   - Verify username and password
   - Try connecting with `psql`, `mysql`, or `sqlite3` CLI
   - Ensure user has necessary permissions

3. **Check host and port**:
   - Localhost: Use `localhost` or `127.0.0.1`
   - Remote: Ensure firewall allows connection
   - Verify correct port (PostgreSQL: 5432, MySQL: 3306)

4. **Test connection string**:
   ```bash
   # PostgreSQL
   psql "postgresql://user:pass@localhost:5432/dbname"

   # MySQL
   mysql -h localhost -P 3306 -u user -p dbname
   ```

### "SSL connection error"

**Problem**: Database requires SSL but LazyTables is not configured for it.

**Solution**:
- Use connection string with SSL parameters:
  ```
  postgresql://user:pass@host:5432/db?sslmode=require
  ```

### "Too many connections"

**Problem**: Database connection limit reached.

**Solution**:
1. Close unused connections in LazyTables
2. Check database connection limits:
   ```sql
   -- PostgreSQL
   SHOW max_connections;
   SELECT count(*) FROM pg_stat_activity;

   -- MySQL
   SHOW VARIABLES LIKE 'max_connections';
   SHOW STATUS LIKE 'Threads_connected';
   ```

3. Disconnect from unused databases in LazyTables:
   - Press `1` → Select connection → Press `x` to disconnect

### "Authentication failed"

**Problem**: Incorrect credentials or authentication method not supported.

**Solutions**:
1. **PostgreSQL**: Check `pg_hba.conf` authentication method
   ```bash
   # Check pg_hba.conf location
   psql -c "SHOW hba_file;"
   ```

2. **MySQL**: Verify user exists and has proper host permissions
   ```sql
   SELECT User, Host FROM mysql.user WHERE User='your_user';
   ```

3. **Re-create connection** with correct credentials

### SQLite file not found

**Problem**: SQLite database file path is incorrect.

**Solution**:
- Use absolute path: `/Users/user/data/mydb.sqlite3`
- Or relative path from home: `~/Documents/mydb.sqlite3`
- Ensure file exists and is readable:
  ```bash
  ls -l /path/to/database.sqlite3
  ```

---

## Performance Issues

### Slow query execution

**Problem**: Query takes too long to execute.

**Solutions**:

1. **Add LIMIT to queries**:
   ```sql
   -- Instead of:
   SELECT * FROM large_table;

   -- Use:
   SELECT * FROM large_table LIMIT 1000;
   ```

2. **Use indexes**:
   ```sql
   -- Check if query uses indexes
   EXPLAIN ANALYZE SELECT * FROM users WHERE email = 'test@example.com';
   ```

3. **Optimize query**:
   - Avoid SELECT *
   - Use WHERE clauses to filter
   - Add appropriate indexes

4. **Increase query timeout** in config:
   ```toml
   [query]
   query_timeout = 60  # Increase from default 30 seconds
   ```

### UI is slow or laggy

**Problem**: Terminal rendering is slow.

**Solutions**:

1. **Reduce max_rows**:
   ```toml
   [query]
   max_rows = 5000  # Reduce from default 10000
   ```

2. **Close unused tabs**:
   - Press `x` to close tabs you're not using
   - Each open table uses memory

3. **Check terminal performance**:
   - Some terminals render slower than others
   - Try iTerm2 (macOS), Alacritty, or kitty for better performance

4. **Reduce result columns**:
   - Query only needed columns instead of SELECT *

### High memory usage

**Problem**: LazyTables using too much memory.

**Solutions**:

1. **Close unused tabs and connections**
2. **Reduce max_rows** in configuration
3. **Restart LazyTables** to clear memory
4. **Query specific columns** instead of entire tables

---

## UI and Display Issues

### Garbled or corrupted display

**Problem**: Terminal encoding or size issues.

**Solutions**:

1. **Ensure 256 color support**:
   ```bash
   echo $TERM
   # Should show xterm-256color or similar

   # Set if needed
   export TERM=xterm-256color
   ```

2. **Check terminal size**:
   - LazyTables requires minimum 80x24 terminal size
   - Resize terminal window

3. **Restart LazyTables**:
   ```bash
   # Quit (q) and restart
   lazytables
   ```

### Colors not displaying correctly

**Problem**: Terminal doesn't support colors properly.

**Solution**:
```bash
# Set TERM environment variable
export TERM=xterm-256color

# Make permanent
echo 'export TERM=xterm-256color' >> ~/.bashrc
source ~/.bashrc
```

### Help overlay not showing

**Problem**: Pressing `?` doesn't show help.

**Solutions**:
1. Ensure you're not in insert mode (press `ESC` first)
2. Try pressing `Shift+?` (might be terminal interpretation issue)
3. Check if `?` key is mapped in your terminal

### Pane borders missing or incorrect

**Problem**: Pane borders not rendering correctly.

**Solution**:
- Check terminal Unicode support
- Try a different terminal emulator
- Enable UTF-8 in terminal settings

---

## Query Execution Issues

### "Query timeout"

**Problem**: Query takes longer than configured timeout.

**Solution**:
```toml
# Edit ~/.config/lazytables/config.toml
[query]
query_timeout = 120  # Increase timeout to 120 seconds
```

### Query executes but no results shown

**Problem**: Query returns empty result set.

**Solutions**:
1. Verify query syntax:
   ```sql
   -- Check if table has data
   SELECT COUNT(*) FROM table_name;
   ```

2. Check WHERE clause filters
3. View query in debug mode (`Ctrl+B`)

### "Syntax error" when executing query

**Problem**: SQL syntax is incorrect for database type.

**Solutions**:
1. Check database type (PostgreSQL, MySQL, SQLite have different syntax)
2. Verify query in database CLI first
3. Common differences:
   - PostgreSQL: `LIMIT` syntax
   - MySQL: Backticks for identifiers
   - SQLite: Limited ALTER TABLE support

### Auto-completion not working

**Problem**: Tab completion doesn't show suggestions.

**Solutions**:
1. Ensure you're in insert mode (press `i` in Query Editor)
2. Type at least 2 characters before expecting suggestions
3. Currently only SQL keywords are completed (table names coming soon)

---

## File and Configuration Issues

### Configuration not loading

**Problem**: Changes to config.toml not taking effect.

**Solutions**:

1. **Verify file location**:
   ```bash
   cat ~/.config/lazytables/config.toml
   ```

2. **Check TOML syntax**:
   ```bash
   # Invalid TOML will be ignored
   # Look for missing quotes, brackets, etc.
   ```

3. **Reset to defaults**:
   ```bash
   mv ~/.config/lazytables/config.toml ~/.config/lazytables/config.toml.bak
   # LazyTables will create new default config
   ```

4. **Restart LazyTables** after config changes

### SQL files not saving

**Problem**: Saved queries not appearing in Files pane.

**Solutions**:

1. **Check save location**:
   ```bash
   ls -la ~/.lazytables/sql_files/
   ```

2. **Verify file permissions**:
   ```bash
   chmod -R 755 ~/.lazytables/sql_files/
   ```

3. **Ensure connection is active**:
   - Files are saved per connection
   - Connect to database first

### Cannot load saved SQL file

**Problem**: SQL file exists but won't load.

**Solutions**:
1. Check file permissions:
   ```bash
   chmod 644 ~/.lazytables/sql_files/connection_name/query.sql
   ```

2. Verify file encoding (should be UTF-8)
3. Try opening file in text editor to check for corruption

### Lost connections after update

**Problem**: Connections disappeared after updating LazyTables.

**Solutions**:

1. **Check backup files**:
   ```bash
   ls -la ~/.lazytables/backups/
   ```

2. **Restore from backup**:
   ```bash
   cp ~/.lazytables/backups/connections-backup-YYYYMMDD.json ~/.lazytables/connections.json
   ```

3. **Re-create connections** if no backup available

---

## Keyboard Navigation Issues

### Keys not responding

**Problem**: Keyboard shortcuts not working.

**Solutions**:

1. **Check current mode**:
   - If in insert mode, press `ESC` first
   - Status bar shows current mode

2. **Terminal key capture**:
   - Some terminals intercept certain keys
   - Try in a different terminal

3. **Verify key bindings**:
   - Press `?` to see active key bindings
   - Check [Key Bindings](key-bindings.md) documentation

### Cannot type in text fields

**Problem**: Text input not working in forms or query editor.

**Solutions**:

1. **Enter insert mode first**:
   - Query Editor: Press `i` to enter insert mode
   - Text fields: Press `i` to start typing

2. **Check if in correct pane**:
   - Press `5` to focus Query Editor
   - Ensure status bar shows correct pane

### Vim motions not working

**Problem**: h/j/k/l movement not functioning.

**Solutions**:

1. **Ensure not in insert mode**:
   - Press `ESC` to exit insert mode
   - Vim motions only work in normal mode

2. **Check pane support**:
   - Not all panes support all vim motions
   - Press `?` to see pane-specific keys

### Tab key not working

**Problem**: Tab doesn't cycle through panes or complete.

**Solutions**:

1. **Context matters**:
   - In normal mode: Cycles panes
   - In insert mode (Query Editor): Accepts completion
   - In forms: Moves to next field

2. **Try Shift+Tab** for reverse cycling

---

## Debug Mode

For most issues, debug mode provides valuable information:

1. **Enable debug mode**: Press `Ctrl+B`
2. **View logs in real-time**
3. **Check for error messages**
4. **Close with** `Ctrl+B` again

Or view log file directly:
```bash
tail -f ~/.lazytables/logs/lazytables.log
```

---

## Getting More Help

### Before Reporting Issues

1. **Check existing issues**: [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
2. **Try debug mode**: `Ctrl+B` to view logs
3. **Check version**: `lazytables --version`
4. **Review documentation**:
   - [Installation Guide](installation.md)
   - [Key Bindings](key-bindings.md)
   - [Configuration](configuration.md)
   - [User Guides](guides.md)

### Reporting Bugs

When reporting issues, include:

1. **LazyTables version**: `lazytables --version`
2. **Operating system**: macOS/Linux version
3. **Terminal emulator**: iTerm2, Alacritty, etc.
4. **Database type and version**
5. **Steps to reproduce**
6. **Error messages** from debug mode (`Ctrl+B`)
7. **Configuration** (if relevant)

### Community Support

- **GitHub Issues**: [Report bugs](https://github.com/yuyudhan/LazyTables/issues)
- **GitHub Discussions**: [Ask questions](https://github.com/yuyudhan/LazyTables/discussions)
- **Documentation**: [User guides](guides.md)

---

## Common Error Messages

### "Failed to initialize terminal"

**Cause**: LazyTables can't initialize TUI (might be running in non-interactive mode)

**Solution**: Run in a proper terminal, not piped or in background

### "Failed to read configuration"

**Cause**: Configuration file corrupted or invalid TOML

**Solution**: Reset config (see [Configuration not loading](#configuration-not-loading))

### "Database pool error"

**Cause**: Connection pool exhausted or connection lost

**Solution**: Restart LazyTables, check database status

### "Permission denied"

**Cause**: File/directory permission issues

**Solution**:
```bash
chmod -R 755 ~/.config/lazytables
chmod -R 755 ~/.lazytables
chmod 600 ~/.lazytables/connections.json
```

---

## Performance Benchmarks

Expected performance on modern hardware:

- **Startup time**: < 100ms
- **Query execution**: First results in < 50ms
- **Scrolling**: 60 FPS smooth scrolling
- **Memory usage**: < 50MB base, +5-10MB per open table
- **Connection time**: < 500ms to local database

If your performance is significantly worse, check:
1. Database server performance
2. Network latency (for remote databases)
3. Terminal emulator performance
4. System resource availability

---

**Still having issues?** [Open an issue](https://github.com/yuyudhan/LazyTables/issues) with details about your problem.
