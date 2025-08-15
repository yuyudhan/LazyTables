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

### Supported Terminals
- **macOS**: iTerm2, Terminal.app, Alacritty, Wezterm
- **Linux**: GNOME Terminal, Konsole, xfce4-terminal, Alacritty, Wezterm

## macOS Installation

### Option 1: Homebrew (Recommended)

```bash
# Add the LazyTables tap (coming soon)
brew tap yuyudhan/lazytables
brew install lazytables
```

### Option 2: Direct Download

```bash
# Download latest release
curl -L https://github.com/yuyudhan/LazyTables/releases/latest/download/lazytables-macos.tar.gz -o lazytables.tar.gz

# Extract and install
tar -xzf lazytables.tar.gz
sudo mv lazytables /usr/local/bin/
chmod +x /usr/local/bin/lazytables
```

### Option 3: Build from Source

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone and build
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables
cargo build --release

# Install binary
sudo cp target/release/lazytables /usr/local/bin/
```

## Linux Installation

### Option 1: Package Repository (Ubuntu/Debian)

```bash
# Add LazyTables repository (coming soon)
curl -fsSL https://repo.lazytables.dev/gpg.key | sudo gpg --dearmor -o /usr/share/keyrings/lazytables.gpg
echo "deb [signed-by=/usr/share/keyrings/lazytables.gpg] https://repo.lazytables.dev/deb stable main" | sudo tee /etc/apt/sources.list.d/lazytables.list

# Update and install
sudo apt update
sudo apt install lazytables
```

### Option 2: Arch Linux (AUR)

```bash
# Using yay
yay -S lazytables-git

# Using paru
paru -S lazytables-git

# Manual AUR installation
git clone https://aur.archlinux.org/lazytables-git.git
cd lazytables-git
makepkg -si
```

### Option 3: RPM-based Distributions (CentOS, RHEL, Fedora)

```bash
# Add LazyTables repository (coming soon)
sudo dnf config-manager --add-repo https://repo.lazytables.dev/rpm/lazytables.repo

# Install
sudo dnf install lazytables
```

### Option 4: Direct Download

```bash
# Download latest release for your architecture
curl -L https://github.com/yuyudhan/LazyTables/releases/latest/download/lazytables-linux-x86_64.tar.gz -o lazytables.tar.gz

# Extract and install
tar -xzf lazytables.tar.gz
sudo mv lazytables /usr/local/bin/
chmod +x /usr/local/bin/lazytables
```

### Option 5: Build from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install system dependencies (Ubuntu/Debian)
sudo apt install build-essential pkg-config libssl-dev

# Install system dependencies (CentOS/RHEL/Fedora)
sudo dnf install gcc pkg-config openssl-devel

# Clone and build
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables
cargo build --release

# Install binary
sudo cp target/release/lazytables /usr/local/bin/
```

## Verification

After installation, verify LazyTables is working:

```bash
# Check version
lazytables --version

# Show help
lazytables --help

# Test launch (will show connection setup)
lazytables
```

## Configuration

### Default Configuration Location

LazyTables stores its configuration in:
- **macOS**: `~/Library/Application Support/LazyTables/`
- **Linux**: `~/.config/lazytables/`

### Initial Setup

On first launch, LazyTables will create:
- Configuration directory
- Default configuration file
- Connections storage file (encrypted)

## Troubleshooting Installation

### Common Issues

#### "Command not found: lazytables"

**Cause**: Binary not in PATH or not executable

**Solutions**:
```bash
# Check if binary exists
which lazytables

# Check PATH includes /usr/local/bin
echo $PATH

# Make binary executable
chmod +x /usr/local/bin/lazytables

# Add to PATH if needed (add to ~/.bashrc or ~/.zshrc)
export PATH="/usr/local/bin:$PATH"
```

#### "Permission denied"

**Cause**: Binary doesn't have execute permissions

**Solution**:
```bash
chmod +x /usr/local/bin/lazytables
```

#### "Library not found" (Linux)

**Cause**: Missing system libraries

**Solutions**:
```bash
# Ubuntu/Debian
sudo apt install libc6 libssl3

# CentOS/RHEL/Fedora
sudo dnf install glibc openssl-libs

# Check required libraries
ldd /usr/local/bin/lazytables
```

#### Terminal Compatibility Issues

**Symptoms**: Garbled output, missing characters, incorrect colors

**Solutions**:
1. **Check terminal capabilities**:
   ```bash
   echo $TERM
   tput colors
   ```

2. **Set terminal to support 256 colors**:
   ```bash
   export TERM=xterm-256color
   ```

3. **Verify UTF-8 encoding**:
   ```bash
   echo $LANG
   # Should include UTF-8, like: en_US.UTF-8
   ```

#### macOS Gatekeeper Issues

**Symptoms**: "cannot be opened because the developer cannot be verified"

**Solution**:
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/lazytables

# Or allow in System Preferences > Security & Privacy
```

### Build from Source Issues

#### Rust Not Installed

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustc --version
```

#### Compilation Errors

```bash
# Update Rust toolchain
rustup update

# Clean build artifacts
cargo clean

# Build with verbose output
cargo build --release --verbose
```

#### Missing System Dependencies

**Linux (Ubuntu/Debian)**:
```bash
sudo apt install build-essential pkg-config libssl-dev
```

**Linux (CentOS/RHEL/Fedora)**:
```bash
sudo dnf install gcc pkg-config openssl-devel
```

**macOS**:
```bash
xcode-select --install
```

## Updating LazyTables

### Package Manager Updates

```bash
# Homebrew (macOS)
brew update && brew upgrade lazytables

# APT (Ubuntu/Debian)
sudo apt update && sudo apt upgrade lazytables

# DNF (Fedora/CentOS)
sudo dnf update lazytables

# AUR (Arch Linux)
yay -Syu lazytables-git
```

### Manual Updates

1. Download the latest release
2. Replace the existing binary
3. Restart any running instances

## Uninstallation

### Package Manager Removal

```bash
# Homebrew (macOS)
brew uninstall lazytables

# APT (Ubuntu/Debian)
sudo apt remove lazytables

# DNF (Fedora/CentOS)
sudo dnf remove lazytables

# AUR (Arch Linux)
yay -R lazytables-git
```

### Manual Removal

```bash
# Remove binary
sudo rm /usr/local/bin/lazytables

# Remove configuration (optional)
# macOS
rm -rf ~/Library/Application\ Support/LazyTables

# Linux
rm -rf ~/.config/lazytables
```

## Getting Help

If you encounter installation issues:

1. **Check system requirements** - Ensure your system is supported
2. **Review error messages** - Look for specific error details
3. **Search existing issues** - Check [GitHub Issues](https://github.com/yuyudhan/LazyTables/issues)
4. **Create a new issue** - Include system details and error messages
5. **Join discussions** - Ask for help in [GitHub Discussions](https://github.com/yuyudhan/LazyTables/discussions)

## Next Steps

After successful installation:
- Read [002 - First Steps](002-first-steps.md) to get started
- Learn about [003 - Navigation](003-navigation.md) and keyboard shortcuts
- Set up your first database connection in [004 - Managing Connections](004-managing-connections.md)