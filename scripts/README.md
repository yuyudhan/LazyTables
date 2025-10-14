# LazyTables Scripts

Collection of development and release automation scripts.

## create-release-files.sh

Creates binstall-compatible release archives for multiple platforms.

### Requirements

```bash
# Install cross for cross-compilation (Note: v0.2.5 has issues on macOS)
cargo install cross --version 0.2.4

# Or use latest (but Linux builds may fail on macOS)
cargo install cross

# Ensure Docker Desktop is running (for Linux builds on macOS)
```

**Known Issue:** `cross` v0.2.5 has issues building Linux targets on macOS. If Linux builds fail, either:
1. Downgrade: `cargo install cross --version 0.2.4 --force`
2. Build macOS targets only: `./scripts/create-release-files.sh -t x86_64-apple-darwin -t aarch64-apple-darwin`
3. Use GitHub Actions once billing is resolved

### Usage Examples

**Build all targets (auto-detect version from Cargo.toml):**
```bash
./scripts/create-release-files.sh
```

**Build all targets with specific version:**
```bash
./scripts/create-release-files.sh -v 0.2.4
```

**Build only macOS targets:**
```bash
./scripts/create-release-files.sh -t x86_64-apple-darwin -t aarch64-apple-darwin
```

**Build only Linux targets:**
```bash
./scripts/create-release-files.sh -t x86_64-unknown-linux-gnu -t aarch64-unknown-linux-gnu
```

**Build single target with version:**
```bash
./scripts/create-release-files.sh -v 0.2.4 -t aarch64-apple-darwin
```

**Show help:**
```bash
./scripts/create-release-files.sh --help
```

### Available Targets

- `x86_64-apple-darwin` - macOS Intel
- `aarch64-apple-darwin` - macOS Apple Silicon
- `x86_64-unknown-linux-gnu` - Linux x86_64
- `aarch64-unknown-linux-gnu` - Linux ARM64

### Output Location

Archives are created in `target/release/binstall-builds/`:
```
lazytables-v0.2.4-x86_64-apple-darwin.tar.gz
lazytables-v0.2.4-x86_64-apple-darwin.tar.gz.sha256
lazytables-v0.2.4-aarch64-apple-darwin.tar.gz
lazytables-v0.2.4-aarch64-apple-darwin.tar.gz.sha256
...
```

### Publishing to GitHub

```bash
# Create release
gh release create v0.2.4

# Upload all files
gh release upload v0.2.4 target/release/binstall-builds/*

# Or upload specific files
gh release upload v0.2.4 target/release/binstall-builds/lazytables-v0.2.4-*.tar.gz target/release/binstall-builds/lazytables-v0.2.4-*.sha256
```

### Testing Locally

```bash
# Extract and test binary
cd target/release/binstall-builds
tar xzf lazytables-v0.2.4-aarch64-apple-darwin.tar.gz
./lazytables --version
```

### Users Installing

Once released, users can install with:
```bash
cargo binstall lazytables
```
