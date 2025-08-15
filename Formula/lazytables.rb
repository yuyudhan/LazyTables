class Lazytables < Formula
  desc "Terminal-based SQL database viewer and editor with vim-style navigation"
  homepage "https://github.com/yuyudhan/LazyTables"
  url "https://github.com/yuyudhan/LazyTables/archive/refs/tags/v0.1.3.tar.gz"
  sha256 "PLACEHOLDER_SHA256"  # Will be updated after release
  license "WTFPL"
  version "0.1.3"

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release", "--locked"
    bin.install "target/release/lazytables"
  end

  test do
    # Test that the binary runs and shows version
    assert_match "0.1.3", shell_output("#{bin}/lazytables --version 2>&1", 0)
  end
end