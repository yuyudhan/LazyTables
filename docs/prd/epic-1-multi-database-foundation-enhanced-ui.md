# Epic 1: Multi-Database Foundation + Enhanced UI

**Epic Goal**: Establish LazyTables as a multi-database terminal tool by adding MySQL/MariaDB support and delivering foundational UI/UX improvements that create the framework for comprehensive database management capabilities.

**Integration Requirements**:
- Preserve all existing PostgreSQL functionality and user workflows
- Establish patterns for database-specific UI adaptations
- Create foundation for syntax highlighting and enhanced query editing
- Implement unified connection management for multiple database types

## Story 1.1: Database Adapter Architecture Enhancement

As a developer working with multiple database types,
I want LazyTables to support a unified adapter interface for different databases,
so that I can seamlessly switch between PostgreSQL and MySQL connections using familiar LazyTables workflows.

### Acceptance Criteria

1. Enhanced adapter trait supports database-specific capabilities while maintaining unified interface
2. MySQL and MariaDB adapters implement full connection, query, and metadata operations
3. Database type detection and appropriate adapter selection works automatically
4. Connection pooling supports multiple database types simultaneously
5. Error handling provides database-specific error messages and recovery suggestions

### Integration Verification

- **IV1**: All existing PostgreSQL connections continue to work exactly as before with no configuration changes required
- **IV2**: Existing six-pane navigation and vim keybindings remain unchanged for PostgreSQL workflows
- **IV3**: PostgreSQL query execution performance maintains current benchmark levels (sub-50ms for simple queries)

## Story 1.2: Enhanced Connection Management Interface

As a database administrator managing multiple database types,
I want an improved connection creation interface that handles database-specific configuration,
so that I can efficiently set up MySQL and PostgreSQL connections with appropriate defaults and validation.

### Acceptance Criteria

1. Connection creation modal displays database-specific configuration fields
2. Database type selection provides appropriate default ports and SSL settings
3. Connection string parsing supports MySQL and PostgreSQL formats automatically
4. Connection testing validates database-specific requirements before saving
5. Connection list displays database type indicators and connection health status

### Integration Verification

- **IV1**: Existing PostgreSQL connection configurations load and function without modification
- **IV2**: Current connection navigation shortcuts (c, a, Enter, d) work identically for all connection types
- **IV3**: Connection creation workflow preserves existing two-step process (type selection → configuration)

## Story 1.3: Database-Adaptive Table Browser

As a developer exploring different database schemas,
I want the tables pane to display database-appropriate metadata and objects,
so that I can understand MySQL table engines, PostgreSQL schemas, and database-specific features using consistent navigation.

### Acceptance Criteria

1. Tables pane displays database-specific object types (PostgreSQL schemas, MySQL engines)
2. Table metadata shows relevant information per database type (indexes, constraints, triggers)
3. Database-specific icons and indicators provide visual database type context
4. Object browsing supports database-specific hierarchies (schema.table vs database.table)
5. Navigation shortcuts work consistently across all database types

### Integration Verification

- **IV1**: PostgreSQL table browsing maintains existing behavior with enhanced metadata display
- **IV2**: Table selection and data viewing shortcuts (t, j/k, Enter) work identically across databases
- **IV3**: Existing table detail display in details pane shows additional metadata without breaking layout

## Story 1.4: Multi-Database Query Editor Foundation

As a developer writing SQL for different databases,
I want enhanced query editor capabilities with database-aware features,
so that I can write MySQL and PostgreSQL queries efficiently with appropriate syntax guidance and execution.

### Acceptance Criteria

1. Query editor detects active database connection and adjusts syntax validation accordingly
2. Basic keyword highlighting works for MySQL and PostgreSQL syntax differences
3. Query execution (Ctrl+Enter) handles database-specific SQL dialects correctly
4. Error messages provide database-specific SQL syntax guidance
5. Query history maintains database context for executed queries

### Integration Verification

- **IV1**: Existing PostgreSQL query execution workflows continue unchanged
- **IV2**: Current query editor shortcuts (q, i, Ctrl+Enter, Ctrl+S) function identically
- **IV3**: SQL file management preserves existing behavior while adding database context awareness

## Story 1.5: Enhanced Results Display and Export

As a developer analyzing query results from different databases,
I want improved results display with export capabilities,
so that I can efficiently view, navigate, and export data regardless of database type with performance suitable for large datasets.

### Acceptance Criteria

1. Results pane displays database-specific data types appropriately (MySQL JSON, PostgreSQL arrays)
2. Pagination controls handle large result sets efficiently across database types
3. CSV and JSON export functionality works with streaming for datasets over 1MB
4. Export progress indication appears for operations exceeding 2 seconds
5. Results navigation maintains vim-style shortcuts with enhanced data type rendering

### Integration Verification

- **IV1**: PostgreSQL result display maintains current performance and navigation behavior
- **IV2**: Results pane navigation shortcuts (r, j/k/h/l, gg/G) work identically
- **IV3**: Large dataset handling preserves existing memory usage patterns and scrolling performance