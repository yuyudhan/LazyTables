# Installation Guide

## Prerequisites

Before installing LazyTables, ensure you have:

- **Rust 1.70+** - LazyTables is written in Rust and requires the Rust toolchain
- **Terminal with 256 color support** - For proper rendering of the UI
- **macOS or Linux** - Windows is not currently supported

## Installing Rust

If you don't have Rust installed, install it using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, restart your terminal and verify:

```bash
rustc --version
cargo --version
```

## Installation Methods

### Method 1: Install from crates.io (Recommended)

The easiest way to install LazyTables is from crates.io:

```bash
cargo install lazytables
```

This will download, compile, and install the latest stable version.

**Installation time:** 3-5 minutes (compiles from source)

### Method 2: Install with cargo-binstall (Faster)

For faster installation using pre-built binaries:

```bash
# Install cargo-binstall if you haven't already
cargo install cargo-binstall

# Install lazytables using pre-built binaries
cargo binstall lazytables
```

**Installation time:** 30-60 seconds (downloads binary)

### Method 3: Install from Source

For the latest development version or to contribute:

```bash
# Clone the repository
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables

# Install from source
cargo install --path .
```

### Method 4: Build for Development

If you're developing LazyTables:

```bash
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables

# Run in development mode with auto-reload (requires cargo-watch)
make install-deps  # Install cargo-watch
make dev           # Run with auto-reload

# Or run directly
cargo run
```

## Verifying Installation

After installation, verify LazyTables is installed correctly:

```bash
# Check version
lazytables --version

# Should output: lazytables 0.2.1
```

## First Run

Launch LazyTables for the first time:

```bash
lazytables
```

On first launch, LazyTables will:
1. Create configuration directory at `~/.config/lazytables/`
2. Create data directory at `~/.lazytables/`
3. Initialize default configuration files
4. Display the welcome screen

## Updating

To update to the latest version:

```bash
# If installed via cargo install
cargo install lazytables --force

# If installed via cargo-binstall
cargo binstall lazytables --force
```

## Uninstalling

To remove LazyTables from your system:

```bash
cargo uninstall lazytables
```

To also remove configuration and data files:

```bash
cargo uninstall lazytables
rm -rf ~/.config/lazytables
rm -rf ~/.lazytables
```

## Troubleshooting Installation

### "cargo: command not found"

Rust is not installed or not in your PATH. Install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then restart your terminal.

### Compilation Errors

Ensure you have the latest Rust version:

```bash
rustup update
```

### Database Driver Issues

If you encounter database connection issues, ensure you have the necessary system libraries:

**PostgreSQL (macOS):**
```bash
brew install postgresql
```

**MySQL (macOS):**
```bash
brew install mysql-client
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install libpq-dev libmysqlclient-dev libsqlite3-dev
```

### Permission Issues

If you get permission errors during installation:

```bash
# On macOS/Linux, ensure cargo bin directory is in PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## Next Steps

After installation:

1. Read the [Quick Start Guide](../README.md#quick-start) to get started
2. Review [Key Bindings](key-bindings.md) to learn navigation
3. Check out [Configuration](configuration.md) to customize LazyTables
4. Explore [Guides](guides.md) for productivity tips

---

**Need help?** [Open an issue](https://github.com/yuyudhan/LazyTables/issues) or check [Troubleshooting](troubleshooting.md).
