# 006 - Configuration

Customize LazyTables to match your workflow with comprehensive configuration options for appearance, behavior, and functionality.

## Configuration Overview

LazyTables uses TOML configuration files for easy customization. Configuration is loaded from multiple sources with the following precedence:

1. Command line arguments (highest priority)
2. Environment variables
3. User configuration file
4. System configuration file
5. Built-in defaults (lowest priority)

## Configuration File Locations

### User Configuration

**macOS**: `~/Library/Application Support/LazyTables/config.toml`
**Linux**: `~/.config/lazytables/config.toml`

### System Configuration

**macOS**: `/etc/lazytables/config.toml`
**Linux**: `/etc/lazytables/config.toml`

### Environment Override

```bash
export LAZYTABLES_CONFIG="/custom/path/to/config.toml"
lazytables
```

## Basic Configuration

### Creating Your First Configuration

LazyTables creates a default configuration on first run:

```toml
# ~/.config/lazytables/config.toml

[display]
theme = "default"
show_line_numbers = true
page_size = 100
wrap_text = false
column_separator = " │ "

[behavior]
auto_connect = true
confirm_exit = false
save_query_history = true
auto_save_interval = 300  # seconds

[editor]
tab_width = 4
insert_final_newline = true
trim_trailing_whitespace = true
syntax_highlighting = true

[keys]
command_quit = ":q"
help = "?"
search = "/"
```

### Basic Customization

**Change theme**:
```toml
[display]
theme = "dark"  # Options: default, dark, light, solarized
```

**Adjust page size**:
```toml
[display]
page_size = 50  # Rows per page in results
```

**Disable confirmations**:
```toml
[behavior]
confirm_exit = false
confirm_delete = false
```

## Display Configuration

### Theme Settings

**Available Themes**:
```toml
[display]
theme = "default"    # Default blue theme
# theme = "dark"     # Dark theme with high contrast
# theme = "light"    # Light theme for bright terminals
# theme = "solarized"# Solarized color scheme
# theme = "gruvbox"  # Gruvbox color scheme
# theme = "nord"     # Nord color scheme
```

**Custom Colors**:
```toml
[display.colors]
background = "#282c34"
foreground = "#abb2bf"
cursor = "#61afef"
selection = "#3e4451"

# Pane colors
active_border = "#61afef"
inactive_border = "#5c6370"
title = "#e06c75"

# Text colors
keyword = "#c678dd"
string = "#98c379"
comment = "#5c6370"
number = "#d19a66"
```

### Layout Options

**Pane Configuration**:
```toml
[display.layout]
sidebar_width = 25      # Percentage of screen width
show_status_bar = true
show_line_numbers = true
show_column_types = true
highlight_current_row = true

# Pane proportions (percentages of sidebar height)
connections_height = 30
tables_height = 40
details_height = 30
```

**Result Display**:
```toml
[display.results]
page_size = 100
max_column_width = 50
show_row_numbers = true
null_display = "∅"
truncate_long_text = true
date_format = "%Y-%m-%d %H:%M:%S"
```

## Behavior Configuration

### Auto-Connect Settings

```toml
[behavior.auto_connect]
enabled = true
last_connection = true
favorite_connections = ["Local Dev", "Staging DB"]
retry_attempts = 3
retry_delay = 2  # seconds
```

### Query Behavior

```toml
[behavior.queries]
auto_limit = 1000           # Automatic LIMIT for SELECT queries
warn_large_results = 5000   # Warn when results exceed this count
save_history = true
history_size = 100
auto_format = true          # Auto-format SQL queries
explain_slow_queries = true # Auto-explain queries > 5s
```

### Auto-Save Settings

```toml
[behavior.auto_save]
enabled = true
interval = 300              # Auto-save every 5 minutes
save_query_on_exit = true
backup_connections = true
```

## Editor Configuration

### Vim-Style Settings

```toml
[editor]
mode = "vim"                # Options: vim, emacs, default
tab_width = 4
expand_tabs = true          # Convert tabs to spaces
show_matching_brackets = true
auto_indent = true
word_wrap = false

# Vim-specific settings
relative_line_numbers = false
show_mode = true
leader_key = " "            # Space as leader key
```

### Syntax Highlighting

```toml
[editor.syntax]
enabled = true
highlight_keywords = true
highlight_strings = true
highlight_comments = true
highlight_numbers = true

# Custom keyword highlighting
custom_keywords = ["UPSERT", "MERGE", "LATERAL"]

# SQL dialect
dialect = "auto"            # Options: auto, postgres, mysql, sqlite
```

### Auto-Completion (Future Feature)

```toml
[editor.completion]
enabled = true
table_names = true
column_names = true
sql_keywords = true
functions = true
trigger_length = 2          # Characters before showing completions
```

## Key Bindings

### Default Key Mapping

```toml
[keys]
# Application
command_quit = ":q"
help = "?"
command_mode = ":"

# Navigation
pane_left = ["h", "Ctrl+h"]
pane_right = ["l", "Ctrl+l"]
pane_up = ["k", "Ctrl+k"]
pane_down = ["j", "Ctrl+j"]
next_pane = "Tab"
prev_pane = "Shift+Tab"

# Pane focus
focus_connections = "c"
focus_tables = "t"
focus_details = "d"
focus_main = "m"

# Actions
search = "/"
add_connection = "a"
edit_connection = "e"
delete_connection = "d"
refresh = "r"
execute_query = "Ctrl+Enter"
```

### Custom Key Bindings

```toml
[keys.custom]
# Custom shortcuts
quick_select = "Space s"    # Quick SELECT * FROM table
show_create = "Space c"     # SHOW CREATE TABLE
explain_query = "Space e"   # EXPLAIN query
format_query = "Space f"    # Format SQL query

# Leader key sequences (Space as leader)
[keys.leader]
query_history = "h"         # Space+h for query history
export_csv = "x c"          # Space+x+c for CSV export
export_json = "x j"         # Space+x+j for JSON export
```

## Database Configuration

### Connection Defaults

```toml
[database.defaults]
connection_timeout = 30     # Connection timeout in seconds
query_timeout = 120        # Query timeout in seconds
pool_size = 5              # Connection pool size
ssl_mode = "prefer"        # Default SSL mode
```

### Database-Specific Settings

**PostgreSQL**:
```toml
[database.postgresql]
search_path = "public"
statement_timeout = "30s"
lock_timeout = "10s"
application_name = "LazyTables"
```

**MySQL**:
```toml
[database.mysql]
charset = "utf8mb4"
sql_mode = "STRICT_TRANS_TABLES,ERROR_FOR_DIVISION_BY_ZERO"
time_zone = "+00:00"
```

**SQLite**:
```toml
[database.sqlite]
journal_mode = "WAL"
synchronous = "NORMAL"
cache_size = 10000
foreign_keys = true
```

## Performance Configuration

### Memory Settings

```toml
[performance.memory]
result_cache_size = "100MB"     # Cache size for query results
connection_pool_size = 10       # Max connections per database
lazy_load_threshold = 1000      # Lazy load for results > threshold
```

### Rendering Performance

```toml
[performance.rendering]
fps_limit = 60                  # Maximum FPS for UI updates
virtual_scrolling = true        # Enable virtual scrolling
debounce_search = 200           # Search debounce in milliseconds
batch_updates = true            # Batch UI updates for performance
```

## Logging Configuration

### Log Levels

```toml
[logging]
level = "info"                  # Options: debug, info, warn, error
file = "~/.config/lazytables/lazytables.log"
max_size = "10MB"
max_files = 5
console_output = false          # Also log to console

[logging.modules]
database = "debug"              # Detailed database logging
ui = "info"                    # UI event logging
queries = "info"               # Query execution logging
```

## Export Configuration

### Default Export Settings

```toml
[export]
default_format = "csv"
include_headers = true
null_representation = ""
date_format = "iso8601"
quote_strings = true
delimiter = ","

# Export directory
output_directory = "~/Documents/LazyTables-Exports"
auto_timestamp = true           # Add timestamp to filenames
```

### Format-Specific Settings

```toml
[export.csv]
delimiter = ","
quote_char = '"'
escape_char = "\\"
line_terminator = "\n"

[export.json]
pretty_print = true
array_format = true             # true = array, false = json-lines

[export.sql]
include_create_table = false
include_drop_table = false
batch_size = 1000
```

## Security Configuration

### Connection Security

```toml
[security.connections]
encrypt_passwords = true
store_passwords = true          # false = prompt every time
ssl_verify_certificates = true
connection_timeout = 30

# Password encryption
encryption_algorithm = "AES-256-GCM"
key_derivation = "PBKDF2"
iterations = 100000
```

### Query Security

```toml
[security.queries]
prevent_destructive_queries = false    # Block DROP, DELETE, TRUNCATE
require_where_clause = false          # Require WHERE in UPDATE/DELETE
log_all_queries = false               # Security audit logging
max_query_length = 100000             # Prevent extremely long queries
```

## Environment Variables

### Configuration via Environment

```bash
# Override config file location
export LAZYTABLES_CONFIG="/path/to/config.toml"

# Theme override
export LAZYTABLES_THEME="dark"

# Behavior overrides
export LAZYTABLES_AUTO_CONNECT="false"
export LAZYTABLES_PAGE_SIZE="50"

# Database defaults
export LAZYTABLES_CONNECTION_TIMEOUT="45"
export LAZYTABLES_QUERY_TIMEOUT="180"

# Logging
export LAZYTABLES_LOG_LEVEL="debug"
export LAZYTABLES_LOG_FILE="/tmp/lazytables.log"
```

### Connection String Environment

```bash
# Auto-import from DATABASE_URL
export DATABASE_URL="postgresql://user:pass@localhost/db"
lazytables --import-env

# Multiple database URLs
export DEV_DATABASE_URL="postgresql://dev:dev@localhost/dev_db"
export STAGING_DATABASE_URL="postgresql://staging:pass@staging.com/db"
```

## Configuration Management

### Validation

```bash
# Validate configuration
lazytables --validate-config

# Show current configuration
lazytables --show-config

# Show configuration sources
lazytables --config-sources
```

### Backup and Sync

```bash
# Backup configuration
cp ~/.config/lazytables/config.toml ~/backups/

# Version control configuration
cd ~/.config/lazytables
git init
git add config.toml
git commit -m "Initial LazyTables configuration"

# Sync across machines
# Use git, Dropbox, or other sync solutions
```

### Configuration Profiles

**Multiple profiles for different environments**:
```bash
# Work profile
lazytables --config ~/.config/lazytables/work.toml

# Personal profile
lazytables --config ~/.config/lazytables/personal.toml

# Demo profile (read-only, safe settings)
lazytables --config ~/.config/lazytables/demo.toml
```

## Advanced Configuration

### Plugin Configuration (Future)

```toml
[plugins]
enabled = true
auto_update = false
directory = "~/.config/lazytables/plugins"

[plugins.installed]
csv_export = { version = "1.0.0", enabled = true }
json_formatter = { version = "0.5.0", enabled = true }
```

### Custom Commands (Future)

```toml
[commands]
# Custom command definitions
show_users = "SELECT id, username, email FROM users ORDER BY created_at DESC"
table_sizes = """
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC
"""
```

## Troubleshooting Configuration

### Common Issues

**Configuration not loading**:
- Check file permissions: `chmod 644 config.toml`
- Validate TOML syntax with online validator
- Check for typos in configuration keys
- Use `--show-config` to see active configuration

**Theme not applying**:
- Verify theme name is correct
- Check terminal color support: `tput colors`
- Try different theme to isolate issue
- Reset to default: `theme = "default"`

**Key bindings not working**:
- Check for conflicting terminal shortcuts
- Verify key syntax in configuration
- Test with default key bindings
- Check terminal capabilities

## Configuration Examples

### Minimal Configuration

```toml
# Bare minimum configuration
[display]
theme = "dark"
page_size = 50

[keys]
command_quit = ":q"
```

### Power User Configuration

```toml
# Advanced configuration for experienced users
[display]
theme = "gruvbox"
show_line_numbers = true
page_size = 200
column_separator = " ┃ "

[behavior]
auto_connect = true
save_query_history = true
auto_save_interval = 120
warn_large_results = 1000

[editor]
mode = "vim"
tab_width = 2
expand_tabs = true
relative_line_numbers = true
leader_key = " "

[keys.leader]
format_query = "f"
export_csv = "x"
show_history = "h"
explain_query = "e"

[performance]
virtual_scrolling = true
fps_limit = 120
result_cache_size = "500MB"

[logging]
level = "debug"
file = "~/.local/share/lazytables/debug.log"
```

### Read-Only/Demo Configuration

```toml
# Safe configuration for demonstrations
[display]
theme = "light"
show_line_numbers = false

[behavior]
confirm_exit = true
save_query_history = false
auto_connect = false

[security]
prevent_destructive_queries = true
require_where_clause = true
log_all_queries = true
```

## Next Steps

With your configuration customized:
- Master [007 - Keyboard Shortcuts](007-keyboard-shortcuts.md) for your key bindings
- Learn [008 - Tips and Tricks](008-tips-and-tricks.md) for advanced usage
- Explore [009 - Troubleshooting](009-troubleshooting.md) if you encounter issues

Your LazyTables setup is now tailored to your workflow! ⚙️