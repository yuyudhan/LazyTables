# LazyTables Configurable Hotkeys

LazyTables now supports fully configurable hotkeys for pane switching and navigation! You can customize all hotkeys to match your workflow.

## Quick Start

1. **LazyTables automatically creates a configuration file on first run.** To customize:
   ```bash
   # Copy the template to customize your settings
   cp config/config.template.toml ~/.lazytables/config.toml
   ```

2. **Edit the hotkeys section** in `~/.lazytables/config.toml`:
   ```toml
   [keybindings.pane_hotkeys]
   connections = "F1"        # Jump to Connections pane
   tables = "F2"            # Jump to Tables pane  
   details = "F3"           # Jump to Table Details pane
   tabular_output = "F4"    # Jump to Tabular Output pane
   sql_files = "F5"         # Jump to SQL Files pane
   query_window = "F6"      # Jump to Query Window pane

   [keybindings.navigation]
   focus_left = "Ctrl+h"     # Move focus left
   focus_down = "Ctrl+j"     # Move focus down
   focus_up = "Ctrl+k"       # Move focus up
   focus_right = "Ctrl+l"    # Move focus right
   cycle_forward = "Tab"     # Cycle focus forward
   cycle_backward = "Shift+Tab"  # Cycle focus backward
   ```

3. **Restart LazyTables** to apply your new hotkeys!

## Default Hotkeys

| Action | Default Hotkey | Description |
|--------|----------------|-------------|
| **Pane Switching** | | |
| Connections | `F1` | Jump directly to Connections pane |
| Tables | `F2` | Jump directly to Tables pane |
| Table Details | `F3` | Jump directly to Table Details pane |
| Tabular Output | `F4` | Jump directly to Tabular Output pane |
| SQL Files | `F5` | Jump directly to SQL Files pane |
| Query Window | `F6` | Jump directly to Query Window pane |
| **Navigation** | | |
| Focus Left | `Ctrl+h` | Move focus to the left pane |
| Focus Down | `Ctrl+j` | Move focus to the pane below |
| Focus Up | `Ctrl+k` | Move focus to the pane above |
| Focus Right | `Ctrl+l` | Move focus to the right pane |
| Cycle Forward | `Tab` | Cycle through panes in order |
| Cycle Backward | `Shift+Tab` | Cycle through panes in reverse |

## Hotkey Format

You can use any of these key formats in your configuration:

### Single Keys
- **Letters**: `"a"`, `"b"`, `"z"`
- **Numbers**: `"1"`, `"2"`, `"0"`
- **Symbols**: `"!"`, `"@"`, `"#"`
- **Function Keys**: `"F1"`, `"F2"`, ... `"F12"`
- **Special Keys**: `"Enter"`, `"Esc"`, `"Space"`, `"Tab"`, `"Backspace"`
- **Arrow Keys**: `"Up"`, `"Down"`, `"Left"`, `"Right"`

### With Modifiers
- **Ctrl**: `"Ctrl+a"`, `"Ctrl+F1"`
- **Alt**: `"Alt+a"`, `"Alt+F1"`
- **Shift**: `"Shift+a"`, `"Shift+F1"`
- **Multiple**: `"Ctrl+Shift+a"`, `"Ctrl+Alt+F5"`

## Example Configurations

### Number Keys for Quick Access
```toml
[keybindings.pane_hotkeys]
connections = "1"
tables = "2"
details = "3"
tabular_output = "4"
sql_files = "5"
query_window = "6"
```

### Alt+Arrow Keys for Navigation
```toml
[keybindings.navigation]
focus_left = "Alt+Left"
focus_down = "Alt+Down"
focus_up = "Alt+Up"
focus_right = "Alt+Right"
```

### Custom Layout
```toml
[keybindings.pane_hotkeys]
connections = "Ctrl+1"       # Ctrl+1 for connections
tables = "Ctrl+2"           # Ctrl+2 for tables
details = "Ctrl+3"          # Ctrl+3 for details
tabular_output = "Ctrl+4"   # Ctrl+4 for results
sql_files = "Ctrl+5"        # Ctrl+5 for files
query_window = "Ctrl+6"     # Ctrl+6 for query editor
```

## Within-Pane Navigation

Note that these hotkeys control **pane-to-pane** movement. Within each pane, you still use vim-style navigation:

- `j` / `k` - Move up/down within lists and tables
- `h` / `l` - Move left/right within tables
- `gg` / `G` - Jump to first/last item
- `0` / `$` - Jump to first/last column (in tables)
- `Enter` / `Space` - Select item or open table

## Troubleshooting

**Hotkeys not working?**
1. Check your configuration file syntax with a TOML validator
2. Ensure the file is saved to `~/.lazytables/config.toml`
3. Restart LazyTables after making changes
4. Check for typos in key names (case sensitive)

**Key conflicts?**
- Some terminals may capture certain key combinations
- Try alternative keys if your preferred combination doesn't work
- Function keys (F1-F12) usually work reliably across terminals

**Need help?**
- Check the full documentation in `docs/user/configuration.md`
- Look at `config/config.template.toml` for more examples
- Report issues on GitHub if you find bugs

## Advanced Usage

For more advanced configuration options, see the full configuration documentation. You can also customize:
- Editor settings
- Theme preferences  
- Connection defaults
- Performance settings

Enjoy your customized LazyTables experience! 🚀