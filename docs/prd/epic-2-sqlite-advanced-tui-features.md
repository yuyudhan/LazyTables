# Epic 2: SQLite Integration + Advanced TUI Features

**Epic Type**: Brownfield Enhancement
**Epic Status**: Planned
**Dependencies**: Epic 1 (Multi-Database Foundation + Enhanced UI)
**Target Completion**: Q1 2025

## Epic Goal

Expand LazyTables' database support to include SQLite file-based databases while delivering advanced TUI features that enhance the user experience across all database types, building on the multi-database foundation established in Epic 1.

## Epic Description

### Existing System Context

- **Current relevant functionality**: Multi-database support for PostgreSQL, MySQL, MariaDB with unified adapter interface
- **Technology stack**: Rust + Ratatui + SQLx adapters + async connection management
- **Integration points**: Database adapter trait, connection manager, six-pane TUI layout

### Enhancement Details

- **What's being added/changed**: SQLite file-based database support + advanced TUI features (syntax highlighting, enhanced export, improved navigation)
- **How it integrates**: New SQLite adapter implementing existing DatabaseAdapter trait + enhanced TUI widgets for all database types
- **Success criteria**: File-based database browsing works seamlessly, syntax highlighting improves query experience, enhanced features work across all supported databases

## Stories

### Story 2.1: SQLite File-Based Database Support
Add SQLite adapter with file browser integration, enabling users to open .sqlite/.db files directly and browse them using the existing six-pane interface.

**Key Features**:
- File picker integration for SQLite database files
- SQLite-specific metadata display (file size, tables, indexes)
- Support for in-memory databases (:memory:)
- SQLite PRAGMA command support
- File path display in connection status

### Story 2.2: Advanced SQL Query Editor with Syntax Highlighting
Enhance the query editor with database-specific syntax highlighting, bracket matching, and query formatting that works across PostgreSQL, MySQL, and SQLite.

**Key Features**:
- Database-specific SQL syntax highlighting
- Bracket/parentheses matching
- Query auto-formatting capabilities
- Error highlighting for syntax errors
- Keyword completion based on database type

### Story 2.3: Enhanced Data Export and Visualization
Implement advanced data export capabilities (CSV, JSON, SQL) and improved data type rendering for large datasets across all database types.

**Key Features**:
- Multiple export formats (CSV, JSON, SQL INSERT statements)
- Streaming export for large datasets (>10MB)
- Progress indicators for long-running exports
- Data type-specific rendering (JSON pretty-print, binary data display)
- Export configuration options (headers, delimiters, formatting)

## Compatibility Requirements

- ✅ Existing PostgreSQL, MySQL, MariaDB adapters remain unchanged
- ✅ Six-pane layout and vim navigation patterns preserved
- ✅ Connection management interface accommodates file-based connections
- ✅ Performance targets maintained (sub-100ms startup, 60fps scrolling)

## Risk Mitigation

- **Primary Risk**: SQLite file operations could block TUI responsiveness during large file scanning
- **Mitigation**: Implement async file scanning with progress indicators and streaming metadata loading
- **Rollback Plan**: Feature flags allow disabling SQLite support, reverting to Epic 1 functionality

## Definition of Done

- ✅ All stories completed with acceptance criteria met
- ✅ Existing PostgreSQL/MySQL functionality verified through regression testing
- ✅ SQLite files open and browse correctly with file picker integration
- ✅ Syntax highlighting enhances query editing across all database types
- ✅ Export functionality works reliably for datasets up to 10MB
- ✅ Performance benchmarks met (startup time, memory usage, scrolling performance)

## Strategic Impact Analysis

### Value Delivery
- **User Segment Expansion**: SQLite support opens LazyTables to data analysts, app developers, and researchers using local databases
- **Feature Parity**: Advanced TUI features bring LazyTables closer to GUI tool capabilities while maintaining terminal efficiency
- **Foundation Building**: Establishes patterns for complex UI features that will benefit Epic 3 (Redis integration)

### Risk Assessment
- **Low Risk**: SQLite adapter follows established patterns from Epic 1
- **Medium Risk**: Syntax highlighting complexity could impact performance
- **Mitigation**: Feature flags and performance monitoring ensure safe rollout

### Dependencies
- Epic 1 completion (multi-database adapter foundation)
- Architecture validation passed (91% readiness confirmed)
- TUI performance baseline established

## Integration Requirements

- Preserve all existing PostgreSQL, MySQL, MariaDB functionality and user workflows
- Establish patterns for file-based database connections
- Create foundation for advanced syntax highlighting across all database types
- Implement streaming data export capabilities for large datasets