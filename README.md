# LazyTables

**_"Because life's too short for clicking around in database GUIs"_**

**Terminal-based SQL database viewer and editor with vim-style navigation**

⚠️ **This project is in active development and not yet ready for use** - core features are still being implemented.

## Installation

### macOS

```bash
# Using Homebrew (coming soon)
brew install lazytables

# Or build from source
git clone git@github.com:yuyudhan/LazyTables.git
cd LazyTables
cargo build --release
sudo mv target/release/lazytables /usr/local/bin/
```

### Linux

```bash
# Using package manager (coming soon)
# Ubuntu/Debian
sudo apt install lazytables

# Arch Linux
yay -S lazytables

# Or build from source
git clone git@github.com:yuyudhan/LazyTables.git
cd LazyTables
cargo build --release
sudo mv target/release/lazytables /usr/local/bin/
```

## Key Features

- **Four-Pane Layout**: 
  - Connections pane (top-left)
  - Tables/Views pane (middle-left)
  - Table Details pane (bottom-left)
  - Main Content Area (right)

- **Vim-Style Navigation**:
  - `h/j/k/l` - Navigate within panes
  - `Ctrl+h/j/k/l` - Switch between panes
  - `Tab/Shift+Tab` - Cycle through panes
  - `q` - Quit application

- **Modes**:
  - Normal Mode (default) - Navigation and commands
  - Insert Mode (`i`) - Edit data
  - Visual Mode (`v`) - Selection
  - Command Mode (`:`) - Execute commands
  - Query Mode (`Space z q`) - SQL editor

## Development & Contribution

See [docs/dev/README.md](docs/dev/README.md) for development setup and contribution guidelines.

## License

WTFPL - Do What The Fuck You Want To Public License