# LazyTables Installation Guide

## 🍺 Homebrew Installation (Recommended for macOS)

LazyTables can be installed via Homebrew with precompiled binaries for fast installation.

### Quick Install

```bash
# Option 1: Direct install with binary (fastest - no compilation needed)
brew install yuyudhan/lazytables/lazytables

# Option 2: Build from source
brew install yuyudhan/lazytables/lazytables --build-from-source

# Option 3: Install latest development version
brew install yuyudhan/lazytables/lazytables --HEAD
```

### Verify Installation

```bash
lazytables --version
# Should output: lazytables 0.1.3
```

## 📦 Direct Installation

### Using Make (Easiest)

```bash
# Clone and install
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables
make install
```

### Manual Build from Source

```bash
# Clone the repository
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables

# Build release binary
cargo build --release

# Install to system
sudo cp target/release/lazytables /usr/local/bin/
```

### Download Prebuilt Binary

Download the latest release from [GitHub Releases](https://github.com/yuyudhan/LazyTables/releases):

```bash
# For macOS ARM64 (M1/M2)
curl -L https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-aarch64-apple-darwin.tar.gz | tar xz
sudo mv lazytables /usr/local/bin/

# For macOS Intel
curl -L https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-x86_64-apple-darwin.tar.gz | tar xz
sudo mv lazytables /usr/local/bin/
```

## 🔧 Troubleshooting

### If `brew install` doesn't find the formula:

1. Make sure you're using the full path:
   ```bash
   brew install yuyudhan/lazytables/lazytables
   ```

2. Or explicitly tap the repository:
   ```bash
   brew tap yuyudhan/lazytables https://github.com/yuyudhan/LazyTables.git
   brew install lazytables
   ```

### If you get "formula not found" after tapping:

Update your tap:
```bash
brew update
brew search lazytables
```

### To upgrade to a new version:

```bash
brew update
brew upgrade lazytables
```

### To uninstall:

```bash
brew uninstall lazytables
brew untap yuyudhan/lazytables
```

## 🚀 Running LazyTables

Once installed, simply run:

```bash
lazytables
```

This will launch the TUI application. Press `?` for help on keyboard shortcuts.

## 📝 Notes

- The formula is maintained in the main LazyTables repository under `Formula/lazytables.rb`
- Binary releases are available for immediate installation (no compilation needed)
- When binaries are available, Homebrew installs them directly (fast installation)
- Building from source requires Rust, which will be installed automatically if not present
- First source build may take several minutes to download and compile dependencies

## 🔗 Repository Structure

The Homebrew formula is located at:
```
LazyTables/
└── Formula/
    └── lazytables.rb
```

This allows the main repository to serve as both the source code and the Homebrew tap.