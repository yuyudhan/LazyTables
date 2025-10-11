# User Guides & Workflows

## Getting Started

### First Steps

1. Launch: `lazytables`
2. Press `?` for help, `1-6` to jump between panes
3. Press `1` → `a` to add connection
4. Press `Enter` to connect
5. Press `2` to browse tables

### Vim Navigation

- **Movement**: `h/j/k/l` (left/down/up/right)
- **Jump**: `gg` (top), `G` (bottom), `0` (line start), `$` (line end)
- **Search**: `/` to filter, `n`/`N` for next/previous match

---

## Common Workflows

### Database Exploration

```
1 → Select connection → Enter
2 → j/k to browse → Enter to open table
3 → View schema details
t → Toggle Data/Schema view
4 → Navigate cells with h/j/k/l
```

### Query Development

```
5 → i → Write query
Ctrl+Enter → Execute
4 → View results
Ctrl+S → Save query
```

### Data Editing

```
2 → Select table → Enter
4 → Navigate with h/j/k/l
i or Enter → Edit cell → Enter to save
r → Refresh data
```

### File Management

```
6 → Browse saved queries
Enter → Load file
r → Rename
d → Delete
Ctrl+N → New timestamped query
```

### Multi-Table Analysis

```
2 → Open first table
Repeat for more tables (each opens new tab)
S/D → Switch between tabs
x → Close tab
```

---

## Productivity Tips

### Navigation

- Use `1-6` for direct pane access (faster than Tab)
- `/` to filter long lists
- `gg`/`G` for instant top/bottom jump
- `Ctrl+h/j/k/l` for directional navigation

### Query Editor

- **Execute at cursor**: Place cursor on any SQL statement, press `Ctrl+Enter`
- **Save snippets**: Save common queries as files
- **Multi-statement**: Execute each with cursor + `Ctrl+Enter`
- **Auto-complete**: Tab to accept suggestions

### Table Viewer

- Press `t` for schema view: columns, indexes, foreign keys, constraints, statistics
- `yy` to copy row as CSV
- `/` to search, `n`/`N` to jump between matches
- `r` to refresh after external changes

### Connections

- Use connection strings for quick setup: `postgresql://user:pass@host:port/db`
- Include environment in names: "Production DB", "Dev DB"
- Test connections on save

### Files

- `Ctrl+N` for timestamped queries (good for exploration)
- Rename important queries with descriptive names
- Delete old files to keep pane clean

---

## Advanced Features

### Large Result Sets

- Virtual scrolling handles thousands of rows smoothly
- Auto-pagination for >10K rows (configurable)
- Use LIMIT in queries for better performance

### Query History

- Automatically tracked per connection
- View in debug mode (`Ctrl+B`)

### Debug Mode

Press `Ctrl+B` for:
- Real-time logs
- Connection status
- Query execution details

---

## Best Practices

### Security

- Credentials automatically encrypted with AES-GCM
- Set permissions: `chmod 600 ~/.lazytables/connections.json`
- Use read-only accounts for production when possible

### Query Optimization

```sql
-- Always use LIMIT for exploration
SELECT * FROM large_table LIMIT 100;

-- Check performance
EXPLAIN ANALYZE SELECT * FROM users WHERE email = 'test@example.com';
```

### Organization

- Use descriptive connection names with environment
- Name query files clearly: `daily_report.sql`, `user_analysis.sql`
- Delete unused queries regularly

### Performance

- Use indexes for filtered columns
- Close unused tabs
- Set appropriate `max_rows` in config
- Refresh only when needed

---

## Database-Specific Tips

**PostgreSQL**: Use `EXPLAIN ANALYZE`, check `pg_stat_user_tables` for statistics

**MySQL/MariaDB**: Schema view shows engine type, use `EXPLAIN` for optimization

**SQLite**: Use `EXPLAIN QUERY PLAN`, check `PRAGMA` commands

---

## Keyboard Patterns

Create muscle memory:

```
1 → a → [fill] → Enter          # Add connection
2 → / → [search] → Enter         # Find table
5 → i → [query] → Ctrl+Enter → 4 # Write and execute
6 → Enter → 5 → Ctrl+Enter       # Load and run saved query
```

---

**More help**: [Key Bindings](key-bindings.md) | [Configuration](configuration.md) | [Troubleshooting](troubleshooting.md)
