# Introduction

This document outlines the complete TUI (Terminal User Interface) architecture for **LazyTables**, a terminal-based SQL database viewer and editor designed for developers who value keyboard-driven workflows. Built with Rust and featuring vim motions throughout, it provides a fast, intuitive interface for database management without leaving the terminal.

This architecture serves as the single source of truth for AI-driven development, ensuring consistency across all system components including the TUI framework, database adapters, state management, and cross-platform terminal compatibility.

## Starter Template or Existing Project

**Status:** Brownfield Enhancement Project - Building on existing LazyTables foundation

**Current Foundation:**
- Rust-based TUI application using Ratatui framework
- Six-pane fixed layout with vim-style navigation
- PostgreSQL support via async adapters
- Basic SQL query editor and file management
- Connection management with secure credential storage

**Enhancement Goals:**
- Multi-database support (MySQL, MariaDB, SQLite, Redis)
- Enhanced query editor with syntax highlighting
- Advanced data export capabilities
- Database-specific UI adaptations

**Constraints Imposed:**
- Must preserve existing six-pane layout and vim navigation
- Backward compatibility with current PostgreSQL connections
- Terminal-only interface (no GUI components)
- Single binary deployment model

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-01-19 | v1.0 | Initial TUI architecture for multi-database support | Architect |
