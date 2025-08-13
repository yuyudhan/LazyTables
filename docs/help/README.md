# LazyTables User Documentation

Welcome to the LazyTables help documentation! This directory contains comprehensive user guides to help you master LazyTables and work efficiently with your databases.

## Documentation Index

### 🚀 Getting Started

1. **[001 - Installation](001-installation.md)**
   - System requirements and platform support
   - Installation methods for macOS and Linux
   - Verification and troubleshooting
   - Configuration directory setup

2. **[002 - First Steps](002-first-steps.md)**
   - Launching LazyTables for the first time
   - Understanding the four-pane interface
   - Basic navigation and essential commands
   - Adding your first database connection

### 🧭 Core Features

3. **[003 - Navigation](003-navigation.md)**
   - Master vim-style navigation throughout LazyTables
   - Movement within and between panes
   - Modal interface (Normal, Insert, Visual, Command modes)
   - Search and filtering techniques

4. **[004 - Managing Connections](004-managing-connections.md)**
   - Adding and configuring database connections
   - Connection security and SSL setup
   - Organizing connections with tags and folders
   - Troubleshooting connection issues

5. **[005 - Querying Data](005-querying-data.md)**
   - Using the integrated query editor
   - Writing and executing SQL queries
   - Navigating large result sets
   - Query management and templates

### ⚙️ Customization & Advanced Usage

6. **[006 - Configuration](006-configuration.md)**
   - Comprehensive configuration options
   - Themes and visual customization
   - Key binding customization
   - Performance tuning

## Quick Start Guide

New to LazyTables? Follow this path:

1. **Install LazyTables**: Start with [001 - Installation](001-installation.md)
2. **Learn the basics**: Read [002 - First Steps](002-first-steps.md)
3. **Master navigation**: Study [003 - Navigation](003-navigation.md)
4. **Set up databases**: Follow [004 - Managing Connections](004-managing-connections.md)
5. **Start querying**: Explore [005 - Querying Data](005-querying-data.md)

## Essential Concepts

### The Four-Pane Interface

LazyTables uses a consistent four-pane layout:

```
┌─────────────┬─────────────────────────────┐
│ Connections │                             │
├─────────────┤                             │
│ Tables/     │        Main Content         │
│ Views       │          Area               │
├─────────────┤                             │
│ Table       │                             │
│ Details     │                             │
└─────────────┴─────────────────────────────┘
```

### Vim-Style Navigation

LazyTables follows vim navigation principles:
- **h/j/k/l** for movement within panes
- **Ctrl+h/j/k/l** for switching between panes
- **Modal interface** with Normal, Insert, Visual, and Command modes
- **Keyboard-first** approach for maximum efficiency

### Key Shortcuts Summary

```bash
# Essential shortcuts
:q                  # Quit LazyTables (command mode)
?                   # Show help
c, t, d, m         # Jump to Connections, Tables, Details, Main panes
Enter              # Connect/select/activate
a                  # Add (connection, query, etc.)
e                  # Edit selected item
/                  # Search

# Query execution
Ctrl+Enter         # Execute query
:                  # Command mode
```

## Database Support

### Currently Supported
- **PostgreSQL** - Full support with all features
- **MySQL/MariaDB** - Core functionality
- **SQLite** - Local database files

### Planned Support
- Oracle Database
- Microsoft SQL Server
- ClickHouse
- Redis (key-value browsing)
- MongoDB (document browsing)

## Getting Help

### Built-in Help
- Press **?** anytime for keyboard shortcut reference
- Use **:help** in command mode for detailed help
- Check the status bar for current mode and active pane

### External Resources
- **[GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)**: Report bugs and request features
- **[GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)**: Community support and ideas
- **[Developer Documentation](../dev/)**: Contributing and development information

### Troubleshooting

**Common issues and solutions**:

1. **LazyTables won't start**: Check [001 - Installation](001-installation.md#troubleshooting-installation)
2. **Can't connect to database**: See [004 - Managing Connections](004-managing-connections.md#connection-troubleshooting)
3. **Navigation not working**: Review [003 - Navigation](003-navigation.md#troubleshooting-navigation)
4. **Performance issues**: Check [006 - Configuration](006-configuration.md#performance-configuration)

## Feature Status

### ✅ Available Features
- Four-pane terminal interface
- Database connection management
- Basic query execution
- Result navigation and display
- Configuration system
- Secure password storage

### 🚧 In Development
- Advanced query editor with syntax highlighting
- Data export functionality
- Query history and favorites
- Advanced table browsing
- Performance monitoring

### 📋 Planned Features
- Data editing capabilities
- Plugin system
- Custom themes
- Query templates
- Advanced search and filtering

## Tips for New Users

1. **Start with a local database** for practice
2. **Use ?** frequently to reference keyboard shortcuts
3. **Practice vim navigation** if you're unfamiliar with it
4. **Configure your theme** for comfortable viewing
5. **Set up read-only connections** for production databases

## Tips for Power Users

1. **Customize key bindings** to match your workflow
2. **Use leader key sequences** for complex operations
3. **Set up connection profiles** for different environments
4. **Configure auto-connect** for frequently used databases
5. **Use query templates** for common operations

## Community and Contributing

LazyTables is an open-source project that welcomes contributions:

- **Code Contributions**: See [Developer Documentation](../dev/)
- **Bug Reports**: Use [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
- **Feature Requests**: Share ideas in [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)
- **Documentation**: Help improve these guides
- **Testing**: Try new features and report feedback

## Feedback and Updates

We're continuously improving LazyTables based on user feedback:

- **Star the project** on GitHub if you find it useful
- **Share your experience** in discussions
- **Report issues** to help us fix problems
- **Request features** that would improve your workflow

---

**Ready to become a LazyTables expert?** Start with [001 - Installation](001-installation.md) and work through the documentation at your own pace.

**"Because life's too short for clicking around in database GUIs"** 🚀