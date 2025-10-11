# Configuration Guide

LazyTables uses a minimal configuration approach, storing settings and data in standard user directories.

## Configuration Locations

### Configuration Directory

```
~/.config/lazytables/
└── config.toml       # Main configuration file
```

### Data Directory

```
~/.lazytables/
├── README.md         # Data directory documentation
├── connections.json  # Database connection definitions (encrypted)
├── connections/      # Individual connection files
├── sql_files/        # Saved SQL query files
│   └── connection_name/   # Per-connection SQL files
│       └── query.sql
├── logs/             # Application log files
│   └── lazytables.log
└── backups/          # Backup files
```

## Configuration File (config.toml)

The main configuration file is located at `~/.config/lazytables/config.toml`.

### Default Configuration

```toml
[general]
theme = "default"
startup_connection = ""  # Optional: auto-connect on startup

[editor]
tab_size = 4
line_numbers = true
syntax_highlighting = true
auto_indent = true

[ui]
show_line_numbers = true
show_status_bar = true
pane_borders = true

[query]
max_rows = 10000        # Maximum rows to fetch per query
query_timeout = 30      # Query timeout in seconds
auto_save_history = true

[logging]
level = "info"          # Options: trace, debug, info, warn, error
file_logging = true
console_logging = false
```

### Customizing Configuration

Edit `~/.config/lazytables/config.toml` to customize LazyTables:

```bash
# Open in your editor
vim ~/.config/lazytables/config.toml

# Or use any editor
nano ~/.config/lazytables/config.toml
```

After editing, restart LazyTables for changes to take effect.

## Connection Storage

### Connection Files

Connections are stored in two locations for backward compatibility:

1. **Primary**: `~/.lazytables/connections.json` (encrypted)
2. **Individual**: `~/.lazytables/connections/` (one file per connection)

### Security

Connection credentials are encrypted using:
- **AES-GCM encryption** for passwords
- **Argon2 key derivation** for secure key generation
- Credentials never stored in plain text

### Connection File Format

Individual connection files (in `~/.lazytables/connections/`) use this structure:

```json
{
  "id": "uuid-v4",
  "name": "My Database",
  "database_type": "PostgreSQL",
  "host": "localhost",
  "port": 5432,
  "database": "mydb",
  "username": "user",
  "password": "encrypted_base64",
  "created_at": "2025-10-12T00:00:00Z",
  "modified_at": "2025-10-12T00:00:00Z"
}
```

**Warning**: Do not manually edit connection files. Always use the UI to manage connections.

## SQL Files

### Directory Structure

SQL files are organized per connection:

```
~/.lazytables/sql_files/
├── my_postgres_db/
│   ├── query_2025-10-12_10-30-15.sql
│   └── customer_analysis.sql
└── my_mysql_db/
    └── inventory_report.sql
```

### File Naming

- **Timestamped files**: Created with `Ctrl+N`, named `query_YYYY-MM-DD_HH-MM-SS.sql`
- **Custom names**: Saved with `Ctrl+S`, you choose the filename

### File Management

SQL files can be managed from the SQL Files Browser (Pane 6):
- Load files with `Enter`
- Rename with `r`
- Delete with `d`
- Copy/duplicate with `c`

## Logs

### Log Files

Application logs are stored at:

```
~/.lazytables/logs/lazytables.log
```

### Log Levels

Configure logging in `config.toml`:

```toml
[logging]
level = "info"  # Options: trace, debug, info, warn, error
```

- **trace**: Most verbose, includes all operations
- **debug**: Detailed debugging information
- **info**: General information (default)
- **warn**: Warning messages only
- **error**: Error messages only

### Viewing Logs

View logs in real-time using the debug view:

1. Press `Ctrl+B` to open debug view
2. Logs update automatically
3. Press `Ctrl+B` again to close

Or view log file directly:

```bash
tail -f ~/.lazytables/logs/lazytables.log
```

## Backups

### Automatic Backups

LazyTables automatically backs up:
- Connection configurations before modifications
- SQL files on save (optional)

Backups are stored in `~/.lazytables/backups/`.

### Manual Backup

To manually backup your data:

```bash
# Backup everything
tar -czf lazytables-backup-$(date +%Y%m%d).tar.gz ~/.lazytables/

# Backup only connections
cp ~/.lazytables/connections.json ~/.lazytables/backups/connections-backup-$(date +%Y%m%d).json
```

### Restoring from Backup

To restore from a backup:

```bash
# Stop LazyTables first, then:
tar -xzf lazytables-backup-20251012.tar.gz -C ~/
```

## Themes

### Available Themes

LazyTables includes built-in themes:
- `default` - Dark theme with blue accents
- `light` - Light theme for bright environments (coming soon)
- `dracula` - Dracula color scheme (coming soon)

### Selecting a Theme

Edit `~/.config/lazytables/config.toml`:

```toml
[general]
theme = "default"
```

Restart LazyTables to apply the theme.

### Custom Themes

Custom themes are not yet supported but are planned for a future release.

## Environment Variables

LazyTables respects the following environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `LAZYTABLES_CONFIG_DIR` | Override config directory | `~/.config/lazytables` |
| `LAZYTABLES_DATA_DIR` | Override data directory | `~/.lazytables` |
| `LAZYTABLES_LOG_LEVEL` | Override log level | `info` |
| `RUST_LOG` | Rust logging filter | Not set |

Example usage:

```bash
# Use custom directories
export LAZYTABLES_CONFIG_DIR="/custom/config"
export LAZYTABLES_DATA_DIR="/custom/data"
lazytables

# Enable debug logging
export LAZYTABLES_LOG_LEVEL="debug"
lazytables
```

## Performance Tuning

### Large Result Sets

For better performance with large result sets, adjust in `config.toml`:

```toml
[query]
max_rows = 10000        # Reduce for faster queries
query_timeout = 30      # Increase for slow queries
```

### Memory Usage

LazyTables uses virtual scrolling for large result sets. Memory usage is typically:
- **Base**: ~20-50MB
- **Per open table**: ~5-10MB (depending on data)

### Network Optimization

For remote databases, consider:
- Using connection pooling (automatic)
- Reducing `max_rows` for faster queries
- Using indexes on frequently queried columns

## Troubleshooting Configuration

### Configuration Not Loading

If your configuration isn't being applied:

1. Verify file location:
   ```bash
   ls -la ~/.config/lazytables/config.toml
   ```

2. Check for syntax errors:
   ```bash
   # Install toml CLI tool
   cargo install --force toml-cli

   # Validate TOML syntax
   toml get ~/.config/lazytables/config.toml .
   ```

3. Reset to defaults:
   ```bash
   rm ~/.config/lazytables/config.toml
   # LazyTables will create new config on next launch
   ```

### Connection Errors

If connections aren't loading:

1. Check file permissions:
   ```bash
   ls -la ~/.lazytables/connections.json
   chmod 600 ~/.lazytables/connections.json
   ```

2. Verify encryption:
   - Connections should load automatically
   - If decryption fails, you may need to re-enter connections

### Log File Issues

If logs aren't being written:

1. Check log directory exists:
   ```bash
   mkdir -p ~/.lazytables/logs
   ```

2. Check permissions:
   ```bash
   chmod 755 ~/.lazytables/logs
   ```

3. Enable file logging in config:
   ```toml
   [logging]
   file_logging = true
   ```

## Migrating Configuration

### From Older Versions

When upgrading from older versions:

1. LazyTables will automatically migrate old configuration
2. Backup files are created before migration
3. Check `~/.lazytables/backups/` for pre-migration backups

### Between Machines

To move LazyTables to a new machine:

```bash
# On old machine
tar -czf lazytables-export.tar.gz ~/.config/lazytables ~/.lazytables/

# Transfer file to new machine, then:
tar -xzf lazytables-export.tar.gz -C ~/
```

---

**Next Steps:**
- Learn [Key Bindings](key-bindings.md) for efficient navigation
- Explore [Guides](guides.md) for productivity tips
- Check [Troubleshooting](troubleshooting.md) for common issues
