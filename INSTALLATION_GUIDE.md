# LazyTables Installation Guide

## 🍺 Homebrew Installation (Recommended for macOS)

LazyTables can be installed directly from the main repository using Homebrew.

### Quick Install

```bash
# Option 1: Direct install (simplest)
brew install yuyudhan/lazytables/lazytables

# Option 2: Add tap first, then install
brew tap yuyudhan/lazytables https://github.com/yuyudhan/LazyTables.git
brew install lazytables
```

### Verify Installation

```bash
lazytables --version
# Should output: lazytables 0.1.3
```

## 📦 Build from Source

If you prefer to build from source:

```bash
# Clone the repository
git clone https://github.com/yuyudhan/LazyTables.git
cd LazyTables

# Build release binary
cargo build --release

# Install to system
sudo cp target/release/lazytables /usr/local/bin/
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
- No separate tap repository is needed
- The formula builds from source using Rust, which will be installed automatically if not present
- First installation may take several minutes as it downloads and installs Rust dependencies

## 🔗 Repository Structure

The Homebrew formula is located at:
```
LazyTables/
└── Formula/
    └── lazytables.rb
```

This allows the main repository to serve as both the source code and the Homebrew tap.