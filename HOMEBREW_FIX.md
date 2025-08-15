# Fix Homebrew Tap Authentication Issue

## The Problem
When you run `brew tap yuyudhan/lazytables`, it's asking for authentication because the repository `homebrew-lazytables` doesn't exist yet.

## Solution - Step by Step

### 1. Create the Tap Repository on GitHub

Go to GitHub and create a new repository:
- Repository name: `homebrew-lazytables` (MUST be this exact name)
- Make it **PUBLIC** (This is critical - private repos require authentication)
- Initialize with README

### 2. Clone the New Repository

```bash
git clone https://github.com/yuyudhan/homebrew-lazytables.git
cd homebrew-lazytables
```

### 3. Create Formula Directory Structure

```bash
mkdir -p Formula
```

### 4. Add the LazyTables Formula

Copy the formula from this repository:
```bash
cp /path/to/LazyTables/Formula/lazytables.rb Formula/
```

### 5. Update Formula with Correct SHA256

First, create the release tag in LazyTables repo:
```bash
cd /path/to/LazyTables
git tag -a v0.1.3 -m "Release v0.1.3"
git push origin v0.1.3
```

Then get the SHA256 of the source tarball:
```bash
wget https://github.com/yuyudhan/LazyTables/archive/refs/tags/v0.1.3.tar.gz
shasum -a 256 v0.1.3.tar.gz
```

Update the SHA256 in `Formula/lazytables.rb`

### 6. Commit and Push the Formula

```bash
git add Formula/lazytables.rb
git commit -m "Add LazyTables v0.1.3 formula"
git push origin main
```

### 7. Test the Tap

Now users (including you) can install without authentication:
```bash
brew tap yuyudhan/lazytables
brew install lazytables
```

## Alternative: Quick Fix Using GitHub UI

1. Go to https://github.com/new
2. Create repository named `homebrew-lazytables`
3. Make it PUBLIC
4. Use GitHub's web UI to create `Formula/lazytables.rb`
5. Paste the formula content with correct SHA256

## Important Notes

- The repository MUST be named `homebrew-lazytables` (not just `lazytables`)
- The repository MUST be PUBLIC
- The formula file MUST be in a `Formula/` directory
- The formula class name should be `Lazytables` (capitalized)

## Creating the GitHub Release

Run the provided script:
```bash
./scripts/create-github-release.sh
```

This will:
1. Build the release binary
2. Create GitHub release with the tag
3. Upload the binary
4. Calculate SHA256 for the formula