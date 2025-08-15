# Homebrew Formula for LazyTables
# This formula will be used in the yuyudhan/lazytables tap

class Lazytables < Formula
  desc "Terminal-based SQL database viewer and editor with vim-style navigation"
  homepage "https://github.com/yuyudhan/LazyTables"
  version "0.1.3"
  license "WTFPL"

  # Use the GitHub release tarball
  url "https://github.com/yuyudhan/LazyTables/archive/refs/tags/v0.1.3.tar.gz"
  sha256 "PLACEHOLDER_SHA256"  # This will be updated after creating the release

  # Alternative: Download pre-built binary for faster installation
  if OS.mac? && Hardware::CPU.intel?
    url "https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-x86_64-apple-darwin.tar.gz"
    sha256 "PLACEHOLDER_SHA256_INTEL"
  elsif OS.mac? && Hardware::CPU.arm?
    url "https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-aarch64-apple-darwin.tar.gz"
    sha256 "PLACEHOLDER_SHA256_ARM"
  end

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release", "--locked"
    bin.install "target/release/lazytables"
  end

  test do
    # Test that the binary runs and shows help
    assert_match "LazyTables", shell_output("#{bin}/lazytables --help 2>&1", 0)
  end
end