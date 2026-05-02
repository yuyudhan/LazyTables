#!/bin/sh
# shellcheck disable=SC3043  # 'local' is widely supported in dash/bash/ash/zsh
# LazyTables installer — downloads a pre-built binary from GitHub Releases.
# Usage: curl -fsSL https://raw.githubusercontent.com/yuyudhan/LazyTables/main/scripts/install.sh | sh
set -eu

REPO="yuyudhan/LazyTables"
BINARY="lazytables"
INSTALL_DIR="${LAZYTABLES_INSTALL_DIR:-$HOME/.local/bin}"

# --- helpers ----------------------------------------------------------------

say() { printf '%s\n' "$@"; }
err() { say "Error: $*" >&2; exit 1; }

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        err "need '$1' (command not found)"
    fi
}

# --- detect platform --------------------------------------------------------

detect_platform() {
    local os arch target

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin) os="apple-darwin" ;;
        Linux)  os="unknown-linux-gnu" ;;
        *)      err "Unsupported OS: $os (only macOS and Linux are supported)" ;;
    esac

    case "$arch" in
        x86_64|amd64)   arch="x86_64" ;;
        aarch64|arm64)   arch="aarch64" ;;
        *)               err "Unsupported architecture: $arch" ;;
    esac

    target="${arch}-${os}"
    say "$target"
}

# --- fetch latest version ---------------------------------------------------

fetch_latest_tag() {
    local url="https://api.github.com/repos/${REPO}/releases/latest"
    local tag

    if command -v curl > /dev/null 2>&1; then
        tag="$(curl -fsSL "$url" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"
    elif command -v wget > /dev/null 2>&1; then
        tag="$(wget -qO- "$url" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"
    else
        err "need 'curl' or 'wget' to download files"
    fi

    if [ -z "$tag" ]; then
        err "could not determine latest release tag"
    fi

    say "$tag"
}

# --- download & extract -----------------------------------------------------

download() {
    local url="$1" dest="$2"
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL "$url" -o "$dest"
    elif command -v wget > /dev/null 2>&1; then
        wget -qO "$dest" "$url"
    fi
}

# --- main -------------------------------------------------------------------

main() {
    say "LazyTables installer"
    say ""

    local target tag version archive_name download_url tmp_dir

    target="$(detect_platform)"
    say "Detected platform: $target"

    say "Fetching latest release..."
    tag="$(fetch_latest_tag)"
    # tag may be "v0.2.3" or "0.2.3"; normalise for archive name
    version="$tag"
    case "$version" in
        v*) ;; # already has v prefix
        *)  version="v${version}" ;;
    esac

    archive_name="${BINARY}-${version}-${target}.tar.gz"
    download_url="https://github.com/${REPO}/releases/download/${tag}/${archive_name}"

    say "Downloading ${archive_name}..."

    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT

    download "$download_url" "${tmp_dir}/${archive_name}" ||
        err "download failed — check that a release exists for ${target} at\n  ${download_url}"

    say "Extracting..."
    tar xzf "${tmp_dir}/${archive_name}" -C "$tmp_dir"

    if [ ! -f "${tmp_dir}/${BINARY}" ]; then
        err "archive did not contain '${BINARY}' binary"
    fi

    # Install binary
    mkdir -p "$INSTALL_DIR"

    if [ -w "$INSTALL_DIR" ]; then
        mv "${tmp_dir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
        chmod +x "${INSTALL_DIR}/${BINARY}"
    else
        say "Installing to ${INSTALL_DIR} requires elevated permissions."
        sudo mv "${tmp_dir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
        sudo chmod +x "${INSTALL_DIR}/${BINARY}"
    fi

    say ""
    say "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"

    # PATH hint
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            say ""
            say "WARNING: ${INSTALL_DIR} is not in your PATH."
            say "Add it by running:"
            say ""
            say "  export PATH=\"${INSTALL_DIR}:\$PATH\""
            say ""
            say "To make it permanent, add that line to your ~/.bashrc, ~/.zshrc, or equivalent."
            ;;
    esac

    # Verify
    if command -v "$BINARY" > /dev/null 2>&1; then
        say ""
        "$BINARY" --version 2>/dev/null || true
        say ""
        say "Run 'lazytables' to get started!"
    fi
}

main
