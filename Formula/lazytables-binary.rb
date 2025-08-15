class LazytablesBinary < Formula
  desc "Terminal-based SQL database viewer and editor with vim-style navigation (pre-built binary)"
  homepage "https://github.com/yuyudhan/LazyTables"
  version "0.1.3"
  license "WTFPL"

  if OS.mac? && Hardware::CPU.intel?
    url "https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-x86_64-apple-darwin.tar.gz"
    sha256 "PLACEHOLDER_INTEL_SHA256"
  elsif OS.mac? && Hardware::CPU.arm?
    url "https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-aarch64-apple-darwin.tar.gz"
    sha256 "PLACEHOLDER_ARM_SHA256"
  else
    url "https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-x86_64-linux.tar.gz"
    sha256 "PLACEHOLDER_LINUX_SHA256"
  end

  def install
    bin.install "lazytables"
  end

  test do
    assert_match "0.1.3", shell_output("#{bin}/lazytables --version 2>&1", 0)
  end
end