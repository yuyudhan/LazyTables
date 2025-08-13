# 004 - Managing Connections

Learn how to add, configure, and manage database connections in LazyTables.

## Connection Management Overview

LazyTables stores all your database connections securely and provides easy management through the Connections pane. Connections are encrypted at rest and support various authentication methods.

## Adding Connections

### Basic Connection Setup

1. **Open Connections pane**: Press `c` or navigate with `Ctrl+h`
2. **Add new connection**: Press `a`
3. **Fill connection form**: Use the connection dialog
4. **Test connection**: LazyTables automatically tests connectivity
5. **Save**: Press `Enter` to save, `Esc` to cancel

### Connection Form Fields

**Required Fields**:
- **Name**: Display name for the connection
- **Type**: Database type (PostgreSQL, MySQL, SQLite, etc.)
- **Host**: Database server hostname or IP
- **Port**: Database server port
- **Username**: Database username
- **Password**: Database password

**Optional Fields**:
- **Database**: Specific database name (can connect to server without specifying)
- **Schema**: Default schema (for databases that support it)
- **SSL Mode**: SSL connection requirements
- **Connection Timeout**: Timeout in seconds
- **Tags**: Organize connections with tags

## Database-Specific Configuration

### PostgreSQL Connections

```
Name: Production PostgreSQL
Type: PostgreSQL
Host: postgres.company.com
Port: 5432
Username: app_user
Password: ••••••••••
Database: production_db
Schema: public
SSL Mode: require
```

**Advanced PostgreSQL Options**:
- **SSL Mode**: `disable`, `allow`, `prefer`, `require`, `verify-ca`, `verify-full`
- **SSL Certificate**: Path to client certificate
- **SSL Key**: Path to client private key
- **SSL Root Certificate**: Path to CA certificate
- **Connection Pool Size**: Maximum concurrent connections
- **Statement Timeout**: Query timeout in milliseconds

### MySQL/MariaDB Connections

```
Name: Dev MySQL
Type: MySQL
Host: localhost
Port: 3306
Username: developer
Password: ••••••••••
Database: development
SSL Mode: preferred
Character Set: utf8mb4
```

**Advanced MySQL Options**:
- **Character Set**: `utf8`, `utf8mb4`, `latin1`
- **Collation**: Database collation
- **SQL Mode**: MySQL SQL mode settings
- **Local Infile**: Enable/disable LOCAL INFILE
- **Multi Statements**: Allow multiple statements

### SQLite Connections

```
Name: Local SQLite
Type: SQLite
File Path: /path/to/database.db
Read Only: false
Journal Mode: WAL
```

**SQLite-Specific Options**:
- **File Path**: Path to SQLite database file
- **Read Only**: Open in read-only mode
- **Journal Mode**: `DELETE`, `TRUNCATE`, `PERSIST`, `MEMORY`, `WAL`
- **Synchronous**: `OFF`, `NORMAL`, `FULL`, `EXTRA`
- **Cache Size**: Page cache size

### Connection Examples

**Local Development**:
```
Name: Local Dev
Type: PostgreSQL
Host: localhost
Port: 5432
Username: postgres
Password: postgres
Database: myapp_development
SSL Mode: disable
Tags: local, development
```

**Docker Container**:
```
Name: Docker PostgreSQL
Type: PostgreSQL
Host: localhost
Port: 15432
Username: dbuser
Password: dbpass
Database: app_db
SSL Mode: disable
Tags: docker, testing
```

**Remote Production**:
```
Name: Production DB
Type: MySQL
Host: db.production.com
Port: 3306
Username: readonly_user
Password: secure_password
Database: production
SSL Mode: required
Connection Timeout: 30
Tags: production, readonly
```

**Cloud Database (AWS RDS)**:
```
Name: AWS RDS
Type: PostgreSQL
Host: mydb.cluster-abc123.us-west-2.rds.amazonaws.com
Port: 5432
Username: app_user
Password: ••••••••••
Database: production
SSL Mode: require
Tags: aws, cloud, production
```

## Connection Security

### Password Storage

LazyTables uses industry-standard encryption for storing sensitive data:
- **Encryption**: AES-256 encryption for passwords
- **Key Derivation**: PBKDF2 with system-specific salt
- **Storage**: Encrypted connection files in user config directory
- **Memory**: Passwords cleared from memory when not in use

### SSL/TLS Configuration

**SSL Modes**:
- **disable**: No SSL encryption (not recommended for production)
- **allow**: SSL if available, plain text otherwise
- **prefer**: SSL preferred, fallback to plain text
- **require**: SSL required, fail if not available
- **verify-ca**: SSL with CA certificate verification
- **verify-full**: SSL with full certificate verification

**Certificate Setup**:
```toml
# In connection advanced settings
ssl_cert = "/path/to/client.crt"
ssl_key = "/path/to/client.key"
ssl_ca = "/path/to/ca.crt"
```

### Connection String Import

Import connections from existing tools:

```bash
# PostgreSQL connection string
postgresql://username:password@host:port/database

# MySQL connection string  
mysql://username:password@host:port/database

# From environment variable
export DATABASE_URL="postgresql://user:pass@localhost/db"
lazytables --import-env
```

## Managing Existing Connections

### Connection Actions

In the Connections pane:

```bash
# Selection and basic actions
j, k                # Navigate connection list
Enter               # Connect/disconnect
Space               # Show connection details

# Connection management
a                   # Add new connection
e                   # Edit selected connection
d                   # Delete connection (with confirmation)
c                   # Clone connection
r                   # Refresh connection status
t                   # Test connection
```

### Connection States

Visual indicators show connection status:
- **🟢 Connected**: Active connection with green indicator
- **⚪ Disconnected**: Available but not connected
- **🔴 Error**: Connection failed with error details
- **🟡 Connecting**: Connection in progress
- **⏸️ Paused**: Connection temporarily disabled

### Editing Connections

1. **Select connection**: Navigate to connection in list
2. **Edit**: Press `e`
3. **Modify fields**: Update any connection parameters
4. **Test changes**: Connection is automatically tested
5. **Save**: Press `Enter` to save, `Esc` to discard changes

### Organizing Connections

**Tags**: Group connections logically
```
Tags: production, aws, readonly
Tags: local, development, testing
Tags: mysql, legacy, migration
```

**Folders**: Organize by environment or project
```
📁 Production
  └── 🗄️ Main Database
  └── 🗄️ Analytics DB
📁 Development  
  └── 🗄️ Local PostgreSQL
  └── 🗄️ Test MySQL
```

## Connection Troubleshooting

### Common Connection Issues

**"Connection refused"**:
- Verify database server is running
- Check host and port are correct
- Confirm firewall allows connections
- Test with `telnet host port`

**"Authentication failed"**:
- Verify username and password
- Check user has database access permissions
- Confirm database exists
- Try connecting with database client

**"SSL connection failed"**:
- Check SSL mode requirements
- Verify SSL certificates are valid
- Try lower SSL security mode for testing
- Check server SSL configuration

**"Timeout connecting"**:
- Increase connection timeout value
- Check network connectivity
- Verify DNS resolution
- Test with ping/traceroute

### Connection Diagnostics

**Test Connection**:
```bash
# In connections pane
t                   # Test selected connection
```

**Connection Details**:
```bash
# View connection info
Space               # Show connection details overlay
# Shows: server version, connection status, active queries
```

**Error Logs**:
```bash
# View detailed error messages
:logs               # Show application logs
:connection-log     # Show connection-specific logs
```

## Advanced Connection Features

### Connection Pooling

Optimize performance with connection pools:

```toml
[connections.pooling]
min_connections = 2         # Minimum pool size
max_connections = 10        # Maximum pool size
idle_timeout = 300          # Idle connection timeout (seconds)
acquire_timeout = 30        # Pool acquisition timeout (seconds)
```

### Auto-Connect

Automatically connect to frequently used databases:

```toml
[connections.auto_connect]
enabled = true
connections = ["Local Dev", "Production Read-Only"]
retry_attempts = 3
retry_delay = 5
```

### Connection Profiles

Save different configurations for the same database:

```
Base Connection: Production DB
├── Profile: Admin (full access)
├── Profile: ReadOnly (read-only user)  
└── Profile: Analytics (analytics schema)
```

### Backup and Export

**Export Connections**:
```bash
:export-connections /path/to/backup.json
```

**Import Connections**:
```bash
:import-connections /path/to/backup.json
```

**Sync Across Devices**:
- Export connections to cloud storage
- Use git to version control connection configs
- Sync encrypted connection files

## Best Practices

### Security Best Practices

1. **Use read-only accounts** for production browsing
2. **Enable SSL/TLS** for remote connections
3. **Rotate passwords** regularly
4. **Use connection-specific users** rather than shared accounts
5. **Limit connection timeouts** to prevent hanging connections

### Organization Best Practices

1. **Use descriptive names**: "Production MySQL (Read-Only)" vs "MySQL1"
2. **Tag connections** by environment, purpose, and access level
3. **Group related connections** in folders
4. **Document special configurations** in connection names or tags
5. **Test connections regularly** to catch issues early

### Performance Best Practices

1. **Configure appropriate timeouts** for your network
2. **Use connection pooling** for frequently accessed databases
3. **Limit concurrent connections** to avoid overwhelming servers
4. **Close unused connections** to free resources
5. **Monitor connection health** with regular tests

## Connection Configuration File

LazyTables stores connections in:
- **macOS**: `~/Library/Application Support/LazyTables/connections.toml`
- **Linux**: `~/.config/lazytables/connections.toml`

**Example configuration**:
```toml
[connections.local_dev]
name = "Local Development"
type = "PostgreSQL"
host = "localhost"
port = 5432
username = "developer"
password = "encrypted:base64encodedpassword"
database = "myapp_dev"
ssl_mode = "disable"
tags = ["local", "development"]

[connections.production]
name = "Production (Read-Only)"
type = "MySQL"  
host = "prod-db.company.com"
port = 3306
username = "readonly"
password = "encrypted:base64encodedpassword"
database = "production"
ssl_mode = "require"
tags = ["production", "readonly"]
```

## Next Steps

Now that you can manage connections effectively:
- Learn advanced [005 - Querying Data](005-querying-data.md) techniques
- Explore [006 - Configuration](006-configuration.md) options
- Master [007 - Keyboard Shortcuts](007-keyboard-shortcuts.md) for efficiency

Your database connections are now organized and secure! 🔐