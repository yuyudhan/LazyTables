# LazyTables Themes

This directory contains theme files for LazyTables. Themes are defined in TOML format and control the visual appearance of the application.

## Theme Locations

LazyTables searches for themes in the following locations (in order of priority):

1. `~/.config/lazytables/themes/` - User-installed themes
2. `~/.local/share/lazytables/themes/` - User data directory
3. `/usr/share/lazytables/themes/` - System-wide themes (Unix only)
4. `./themes/` - Development/bundled themes

## Installing Themes

### Method 1: Using the CLI

```bash
# Install a theme file
lazytables theme install path/to/theme.toml

# List available themes
lazytables theme list

# Show theme directories
lazytables theme dirs

# Export built-in themes
lazytables theme export ./my-themes
```

### Method 2: Manual Installation

Copy your theme file to `~/.config/lazytables/themes/` directory:

```bash
mkdir -p ~/.config/lazytables/themes
cp mytheme.toml ~/.config/lazytables/themes/
```

## Using a Theme

To use a theme, update your LazyTables configuration file (`~/.config/lazytables/config.toml`):

```toml
[ui]
theme = "LazyDark"  # Use the theme name from the TOML file
```

## Creating Custom Themes

Themes are defined in TOML format. Here's the structure:

```toml
name = "MyTheme"
author = "Your Name"

[colors]
# Core UI colors
background = "#0d0d0d"
foreground = "#cdd6f4"
text = "#ffffff"
selection_bg = "#45475a"
cursor = "#f5e0dc"

# Pane colors
pane_background = "#181825"
border = "#313244"
active_border = "#74c7ec"
inactive_pane = "#45475a"

# Component colors
header_fg = "#cba6f7"
status_bg = "#313244"
status_fg = "#cdd6f4"
primary_highlight = "#74c7ec"

# Table colors
table_header_bg = "#313244"
table_header_fg = "#cba6f7"
table_row_bg = "#181825"
table_row_alt_bg = "#1e1e2e"
selected_cell_bg = "#45475a"

# Modal colors
modal_bg = "#0d0d0d"
modal_border = "#74c7ec"
modal_title = "#74c7ec"

# Input field colors
input_bg = "#1e1e2e"
input_fg = "#ffffff"
input_border = "#45475a"
input_active_border = "#74c7ec"
input_placeholder = "#6c7086"

# Button colors
button_bg = "#0000ff"
button_fg = "#000000"
button_active_bg = "#74c7ec"
button_active_fg = "#000000"

# Status colors
success = "#00ff00"
error = "#ff0000"
warning = "#ffff00"
info = "#00ffff"

# SQL editor colors
editor_bg = "#1e1e2e"
editor_fg = "#cdd6f4"
editor_line_number = "#6c7086"
editor_cursor_line = "#313244"
editor_selection = "#45475a"

# Syntax highlighting
syntax_keyword = "#cba6f7"
syntax_string = "#a6e3a1"
syntax_number = "#fab387"
syntax_comment = "#6c7086"
syntax_function = "#89b4fa"
syntax_operator = "#f5c2e7"

# Toast colors
toast_success_bg = "#285028"
toast_error_bg = "#502828"
toast_warning_bg = "#505028"
toast_info_bg = "#283c50"

# Help colors
help_bg = "#1e1e2e"
help_fg = "#cdd6f4"
help_header = "#cba6f7"
help_key = "#74c7ec"
help_description = "#bac2de"
```

## Color Format

Colors must be specified in hexadecimal format with a `#` prefix:
- `#RGB` - 3-digit hex (e.g., `#f00` for red)
- `#RRGGBB` - 6-digit hex (e.g., `#ff0000` for red)

## Built-in Themes

LazyTables comes with two built-in themes:

- **LazyDark** - A dark theme based on Catppuccin Mocha
- **LazyLight** - A light theme based on Catppuccin Latte

## Theme Development Tips

1. **Start with an existing theme**: Export the built-in themes and modify them
2. **Test incrementally**: Change a few colors at a time and test
3. **Consider accessibility**: Ensure sufficient contrast between text and backgrounds
4. **Be consistent**: Use a cohesive color palette throughout

## Sharing Themes

If you create a theme you'd like to share with the community:

1. Fork the LazyTables repository
2. Add your theme file to the `themes/` directory
3. Submit a pull request

## Troubleshooting

### Theme not loading?

1. Check that the theme file is valid TOML:
   ```bash
   cat ~/.config/lazytables/themes/mytheme.toml | toml-test
   ```

2. Verify all required color fields are present

3. Check the theme name in your config matches the `name` field in the theme file

4. Run `lazytables theme list` to see if your theme is detected

### Colors look wrong?

- Ensure your terminal supports true color (24-bit color)
- Check that colors are in valid hex format (#RRGGBB)
- Some terminals may render colors differently - test in multiple terminals