# 001 - Installation

This guide covers installing LazyTables on macOS and Linux systems.

## System Requirements

### Supported Platforms
- **macOS** 10.15 (Catalina) or later
- **Linux** distributions with glibc 2.17+ (Ubuntu 18.04+, CentOS 7+, etc.)
- **Windows** is not currently supported

### Terminal Requirements
- Terminal with 256-color support
- UTF-8 encoding support
- Minimum 80x24 characters (120x40 recommended)

### Prerequisites
- **Rust 1.70+** with cargo (install from https://rustup.rs)

## Installation Methods

### Option 1: Install from crates.io (Recommended)

```bash
cargo install lazytables
```

### Option 2: Install with cargo-binstall (Pre-built binaries)

For faster installation without compilation:

```bash
# Install cargo-binstall if not already installed
cargo install cargo-binstall

# Install lazytables using pre-built binaries
cargo binstall lazytables
```

### Option 3: Install from Git Repository

```bash
cargo install --git https://github.com/yuyudhan/LazyTables.git
```

### Option 4: Build from Source

```bash
# Clone the repository
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables

# Install using cargo
cargo install --path .
```

## Installing Rust

If you don't have Rust installed:

### macOS and Linux

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the on-screen instructions, then reload your shell
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

## Verification

After installation, verify LazyTables is working:

```bash
# Check version
lazytables --version

# Show help
lazytables --help

# Launch the application
lazytables
```

## Configuration

LazyTables stores its configuration in:
- **macOS**: `~/.lazytables/`
- **Linux**: `~/.lazytables/`

On first launch, LazyTables will create:
- Configuration directory
- Default configuration file
- Connections storage file (encrypted)
- SQL files directory

## Troubleshooting

### "Command not found: lazytables"

The cargo bin directory might not be in your PATH:

```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
export PATH="$HOME/.cargo/bin:$PATH"

# Reload your shell
source ~/.bashrc  # or ~/.zshrc
```

### "Error: failed to compile lazytables"

Update Rust to the latest version:

```bash
rustup update
```

### Terminal Compatibility Issues

If you see garbled output or missing characters:

```bash
# Set terminal to support 256 colors
export TERM=xterm-256color

# Verify UTF-8 encoding
echo $LANG  # Should show something like: en_US.UTF-8
```

### macOS Gatekeeper Issues

If macOS blocks the binary:

```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine ~/.cargo/bin/lazytables
```

## Updating LazyTables

To update to the latest version:

```bash
# If installed from crates.io
cargo install lazytables --force

# If installed from git
cargo install --git https://github.com/yuyudhan/LazyTables.git --force

# If using cargo-binstall
cargo binstall lazytables --force
```

## Uninstallation

To remove LazyTables:

```bash
# Uninstall the binary
cargo uninstall lazytables

# Remove configuration (optional)
rm -rf ~/.lazytables
```

## Getting Help

If you encounter issues:

1. **Check requirements** - Ensure Rust 1.70+ is installed
2. **Update Rust** - Run `rustup update`
3. **Search issues** - Check [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
4. **Create an issue** - Include your system details and error messages
5. **Join discussions** - Ask in [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)

## Next Steps

After successful installation:
- Read [002 - First Steps](002-first-steps.md) to get started
- Learn about [003 - Navigation](003-navigation.md) and keyboard shortcuts
- Set up your first database connection in [004 - Managing Connections](004-managing-connections.md)