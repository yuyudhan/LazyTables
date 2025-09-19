# Data Models

Based on the PRD requirements and LazyTables' multi-database architecture, I need to define the core data models that will be shared across the TUI and database adapters. **All configuration and metadata is stored locally in SQLite** for fast access and offline capabilities.

## Connection

**Purpose:** Represents a database connection configuration with database-specific parameters and secure credential storage. Stored locally in SQLite database (~/.lazytables/config.db) for persistence and fast retrieval.

**Key Attributes:**
- id: UUID - Unique identifier for the connection
- name: String - Human-readable connection name
- database_type: DatabaseType - Enum of supported database types
- connection_config: DatabaseConfig - Database-specific connection parameters
- is_active: bool - Current connection status
- created_at: DateTime - Connection creation timestamp
- last_used: DateTime - Last successful connection time

### TypeScript Interface
```typescript
interface Connection {
  id: string;
  name: string;
  database_type: 'PostgreSQL' | 'MySQL' | 'MariaDB' | 'SQLite' | 'Redis';
  connection_config: DatabaseConfig;
  is_active: boolean;
  created_at: string;
  last_used: string;
}

type DatabaseConfig =
  | PostgreSQLConfig
  | MySQLConfig
  | SQLiteConfig
  | RedisConfig;

interface PostgreSQLConfig {
  host: string;
  port: number;
  database: string;
  username: string;
  ssl_mode: 'disable' | 'prefer' | 'require';
  schema?: string;
}
```

### Relationships
- Has many Query objects (query history stored in SQLite)
- Has many TableMetadata objects (cached table information in SQLite)
- Belongs to ConnectionGroup (optional connection organization)

## SQLiteSchema

**Purpose:** Local configuration and metadata storage schema using SQLite as the embedded database for LazyTables configuration.

### SQLite Tables
```sql
-- Connection configurations
CREATE TABLE connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    database_type TEXT NOT NULL,
    connection_config TEXT NOT NULL, -- JSON serialized config
    is_active BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_used DATETIME
);

-- Query history per connection
CREATE TABLE query_history (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    query_text TEXT NOT NULL,
    executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    execution_time_ms INTEGER,
    result_count INTEGER,
    FOREIGN KEY (connection_id) REFERENCES connections(id)
);

-- Cached table metadata for performance
CREATE TABLE table_metadata (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    table_name TEXT NOT NULL,
    schema_name TEXT,
    metadata_json TEXT NOT NULL, -- Serialized table info
    cached_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (connection_id) REFERENCES connections(id)
);

-- Application settings
CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### Relationships
- **Local Storage**: ~/.lazytables/config.db (single SQLite file)
- **Performance**: Fast queries for connection lists, query history, cached metadata
- **Security**: Sensitive credentials still use OS keychain, only references stored in SQLite
