# Homebrew Setup for LazyTables

This guide explains how to set up LazyTables for distribution via Homebrew.

## Prerequisites

1. A GitHub account with access to create repositories
2. Homebrew installed on your Mac
3. The LazyTables repository cloned locally

## Setup Steps

### 1. Create the Tap Repository

Create a new **public** repository on GitHub named `homebrew-lazytables` under the `yuyudhan` organization/user.

The repository MUST be public to avoid authentication issues when users run `brew tap`.

### 2. Repository Structure

```
homebrew-lazytables/
├── Formula/
│   └── lazytables.rb
└── README.md
```

### 3. Add the Formula

Copy the formula file from this repository:

```bash
cp Formula/lazytables.rb /path/to/homebrew-lazytables/Formula/
```

### 4. Create a Release

1. Tag the release:
```bash
git tag -a v0.1.3 -m "Release v0.1.3"
git push origin v0.1.3
```

2. Create a GitHub release from the tag
3. The GitHub Actions workflow will automatically build binaries

### 5. Update Formula SHA256

After creating the release:

1. Download the source tarball from GitHub:
```bash
wget https://github.com/yuyudhan/LazyTables/archive/refs/tags/v0.1.3.tar.gz
```

2. Calculate SHA256:
```bash
shasum -a 256 v0.1.3.tar.gz
```

3. Update the formula with the correct SHA256 hash

### 6. Test the Tap

```bash
# Add the tap
brew tap yuyudhan/lazytables

# Install LazyTables
brew install lazytables

# Verify installation
lazytables --version
```

## Alternative: Local Testing

For testing before publishing:

```bash
# Install directly from the formula file
brew install --build-from-source Formula/lazytables.rb

# Or create a local tap
brew tap-new yuyudhan/lazytables
cp Formula/lazytables.rb $(brew --repository)/Library/Taps/yuyudhan/homebrew-lazytables/Formula/
brew install yuyudhan/lazytables/lazytables
```

## Troubleshooting

### Authentication Issues

If users get authentication prompts when running `brew tap`:
- Ensure the tap repository is **public**
- Check that the repository URL uses HTTPS, not SSH
- The repository name must follow the pattern `homebrew-*`

### Build Failures

If the build fails:
- Ensure Rust is installed: `brew install rust`
- Check that all dependencies are available
- Verify the source tarball is accessible

### Formula Issues

Test the formula locally:
```bash
brew audit --strict Formula/lazytables.rb
brew test Formula/lazytables.rb
```

## Publishing Updates

To publish a new version:

1. Update version in `Cargo.toml`
2. Create and push a new tag
3. Update the formula with new URL and SHA256
4. Commit and push to the tap repository
5. Users can update with: `brew upgrade lazytables`

## Notes

- The tap repository must be named `homebrew-lazytables` (not `lazytables`)
- The formula class name should be `Lazytables` (capitalized)
- Always use HTTPS URLs in the formula to avoid authentication
- Keep the tap repository public for easy access