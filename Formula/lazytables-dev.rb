class LazytablesDev < Formula
  desc "Terminal-based SQL database viewer and editor with vim-style navigation"
  homepage "https://github.com/yuyudhan/LazyTables"
  license "WTFPL"
  
  # For development, build from the current directory
  head do
    url "file://#{File.expand_path("../..", __FILE__)}"
  end

  depends_on "rust" => :build

  def install
    # Build the release binary
    system "cargo", "build", "--release"
    
    # Install the binary
    bin.install "target/release/lazytables"
    
    # Create directories for configuration
    (var/"lazytables").mkpath
    (etc/"lazytables").mkpath
  end

  def post_install
    # Create default config directories
    (var/"lazytables/connections").mkpath
    (var/"lazytables/sql_files").mkpath
    (var/"lazytables/logs").mkpath
    (var/"lazytables/backups").mkpath
  end

  def caveats
    <<~EOS
      LazyTables has been installed!
      
      Configuration will be stored in:
        #{var}/lazytables/
      
      To get started:
        lazytables
      
      For help:
        lazytables --help
    EOS
  end

  test do
    # LazyTables requires a terminal, so we can't run it directly
    # Just check that the binary exists and is executable
    assert_predicate bin/"lazytables", :exist?
    assert_predicate bin/"lazytables", :executable?
  end
end