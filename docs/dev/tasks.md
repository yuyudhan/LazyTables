# LazyTables Development Tasks

## 🔄 Recurring Tasks (Ongoing Maintenance)

### ❗ High Priority Recurring
- [ ] **Help Modal Consistency Check** - Ensure help modal (?) aligns with all pane options and shows current available options
- [ ] **Configuration System Implementation** - Support `~/.config/lazytables/conf.yaml` with pane-specific and global configurations, including customizable key bindings with sensible defaults

### 📋 Pane-Specific Key Bindings Documentation
- [ ] **Complete Key Bindings Migration** - Move all keyboard bindings from individual panes to help modal for consistency across:
  - [ ] Connections pane key bindings
  - [ ] Tables pane key bindings
  - [ ] Table details pane key bindings
  - [ ] Table data pane key bindings
  - [ ] Query results pane key bindings
  - [ ] SQL query editor pane key bindings
  - [ ] SQL files pane key bindings

## 🚧 Active Development (In Progress)

### 🔍 Search Functionality
- [ ] **Connection Search Implementation** - Implement search in connections pane triggered by `/` key with arrow key navigation and Enter to select

### 🐛 Bug Fixes
- [ ] **SQL Files Search & Load Issue** - Fix bug where searched files in SQL files pane don't load when Enter is pressed after search

## ✅ Completed Tasks

### 🎯 UI/UX Improvements
- [x] **Help Modal Background** - Fixed help modal to have black background without transparency to prevent background text bleeding through
- [x] **Help System Standardization** - Removed inline help from all panes, moved to centralized help modal triggered by `?` key:
  - [x] SQL files pane help moved to help modal
  - [x] Table details pane help moved to help modal
  - [x] Connections pane help moved to help modal
  - [x] Tables/views pane help moved to help modal
- [x] **Pane Focus Highlighting** - Fixed table details pane to show proper blue highlight only when focused
- [x] **Application Exit Confirmation** - Added confirmation dialog when pressing `Q` to prevent accidental application closure

### 🔧 Navigation & Controls
- [x] **Debug Mode Shortcut Conflict** - Changed debug pane shortcut from `Ctrl+D` to `Ctrl+B` to avoid conflict with table data paging shortcuts
- [x] **Table Navigation Cycling** - Fixed asymmetric cycling behavior in tables/views listing (now cycles both up and down properly)
- [x] **Table Details Scrolling** - Fixed scrolling bug where `j` key would continue incrementing beyond available lines

### 📊 Data Management
- [x] **Table Data Cell Editing** - Implemented in-cell editing functionality with keyboard shortcut, Enter to save, proper toast notifications for updates/errors
- [x] **Table Data Scrolling** - Fixed vertical scrolling issues in table data viewer for rows that don't fit in pane
- [x] **Column Display & Horizontal Scrolling** - Added column count display and implemented horizontal scrolling for tables with many columns
- [x] **Table Data Paging** - Ensured `Ctrl+D` and `Ctrl+U` work properly for next/previous page navigation

### 🔗 Connection Management
- [x] **Connection Edit Modal** - Completed edit functionality for connections pane allowing proper connection modification
- [x] **Connection Search Fix** - Fixed connection search functionality triggered by `/` key to properly display configured connections during search

## 📝 Notes

### Configuration Structure
The configuration system should support:
- Global application settings
- Pane-specific configurations
- Customizable key bindings with fallback to current defaults
- UI look and feel customization

### Search Pattern
Search functionality across panes should follow consistent pattern:
- Triggered by `/` key
- Arrow keys for navigation within results
- Enter key to select/activate
- Escape to exit search mode