# Database Schema

LazyTables uses **local SQLite** for configuration storage and metadata caching. The schema is optimized for fast TUI operations and minimal overhead.

## Local SQLite Configuration Schema

**Location:** `~/.lazytables/config.db`

```sql
-- Database connection configurations
CREATE TABLE connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    database_type TEXT NOT NULL CHECK (database_type IN ('PostgreSQL', 'MySQL', 'MariaDB', 'SQLite', 'Redis')),
    host TEXT,
    port INTEGER,
    database_name TEXT,
    username TEXT,
    connection_params TEXT, -- JSON for database-specific parameters
    is_active BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_used DATETIME,
    last_error TEXT
);

-- Query execution history for each connection
CREATE TABLE query_history (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    query_text TEXT NOT NULL,
    query_hash TEXT NOT NULL, -- SHA256 of query for deduplication
    executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    execution_time_ms INTEGER,
    rows_affected INTEGER,
    success BOOLEAN DEFAULT 1,
    error_message TEXT,
    FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE CASCADE
);

-- Table metadata cache for performance
CREATE TABLE table_metadata_cache (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    schema_name TEXT,
    table_name TEXT NOT NULL,
    table_type TEXT, -- 'table', 'view', 'materialized_view'
    column_count INTEGER,
    row_estimate INTEGER,
    size_bytes INTEGER,
    metadata_json TEXT NOT NULL, -- Full table schema
    cached_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME, -- TTL for cache invalidation
    FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE CASCADE,
    UNIQUE(connection_id, schema_name, table_name)
);

-- SQL files and snippets
CREATE TABLE sql_files (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    connection_id TEXT, -- Optional: associate with specific connection
    content_hash TEXT, -- For change detection
    last_modified DATETIME,
    is_favorite BOOLEAN DEFAULT 0,
    tags TEXT, -- JSON array of tags
    FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE SET NULL
);

-- Application settings and preferences
CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    setting_type TEXT DEFAULT 'string', -- 'string', 'number', 'boolean', 'json'
    description TEXT,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_query_history_connection ON query_history(connection_id, executed_at DESC);
CREATE INDEX idx_query_history_hash ON query_history(query_hash);
CREATE INDEX idx_table_cache_connection ON table_metadata_cache(connection_id);
CREATE INDEX idx_table_cache_expires ON table_metadata_cache(expires_at);
CREATE INDEX idx_connections_type ON connections(database_type);
CREATE INDEX idx_connections_active ON connections(is_active, last_used DESC);
```

## Schema Design Principles

**Performance Optimized:**
- Indexes on frequently queried columns (connection lookups, query history)
- Query hash for deduplication of repeated queries
- TTL-based cache expiration for table metadata

**TUI-Focused:**
- Fast connection list retrieval for ConnectionsPane
- Efficient query history for autocomplete and recent queries
- Cached table metadata reduces database round-trips

**Data Integrity:**
- Foreign key constraints with CASCADE for cleanup
- CHECK constraints for valid database types
- Unique constraints to prevent duplicates
