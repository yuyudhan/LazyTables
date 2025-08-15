class Lazytables < Formula
  desc "Terminal-based SQL database viewer and editor with vim-style navigation"
  homepage "https://github.com/yuyudhan/LazyTables"
  license "WTFPL"
  version "0.1.3"
  
  # For now, build from source until we have GitHub releases set up
  # Once releases are available, this will download prebuilt binaries
  url "https://github.com/yuyudhan/LazyTables.git",
      branch: "main",
      shallow: false
  
  # Future: Use prebuilt binaries when available
  # if OS.mac? && Hardware::CPU.arm?
  #   url "https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-aarch64-apple-darwin.tar.gz"
  #   sha256 "50ead865a44f5d57fa00e6606cbbd5a67b9a9779513352744d37b7e66b950ddc"
  # elsif OS.mac? && Hardware::CPU.intel?
  #   url "https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-x86_64-apple-darwin.tar.gz"
  #   sha256 "PLACEHOLDER_SHA256_X86_64"
  # elsif OS.linux?
  #   url "https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-x86_64-unknown-linux-gnu.tar.gz"
  #   sha256 "PLACEHOLDER_SHA256_LINUX"
  # end
  
  # Also support HEAD installations for development
  head "https://github.com/yuyudhan/LazyTables.git", branch: "development"
  
  depends_on "rust" => :build

  def install
    # Build from source for now
    system "cargo", "build", "--release", "--locked"
    bin.install "target/release/lazytables"
    
    # Create configuration directories
    (var/"lazytables").mkpath
    (etc/"lazytables").mkpath
  end

  def post_install
    # Create default config directories in user's home
    config_dir = "#{ENV["HOME"]}/.lazytables"
    FileUtils.mkdir_p("#{config_dir}/connections")
    FileUtils.mkdir_p("#{config_dir}/sql_files")
    FileUtils.mkdir_p("#{config_dir}/logs")
    FileUtils.mkdir_p("#{config_dir}/backups")
    
    # Create sample SQL file if it doesn't exist
    sample_sql = "#{config_dir}/sql_files/sample_queries.sql"
    unless File.exist?(sample_sql)
      File.write(sample_sql, <<~SQL)
        -- Sample SQL Queries for LazyTables
        
        -- Show all tables
        SELECT * FROM information_schema.tables 
        WHERE table_schema = 'public';
        
        -- Show table columns
        SELECT * FROM information_schema.columns 
        WHERE table_name = 'your_table_name';
      SQL
    end
  end

  def caveats
    <<~EOS
      LazyTables has been installed!
      
      Configuration will be stored in:
        ~/.lazytables/
      
      To get started:
        lazytables
      
      For help:
        lazytables --help
        
      Note: LazyTables requires a terminal environment and will not work
      when piped or run in non-interactive mode.
    EOS
  end

  test do
    # LazyTables requires a terminal, so we can't run it directly
    # Just check that the binary exists and is executable
    assert_predicate bin/"lazytables", :exist?
    assert_predicate bin/"lazytables", :executable?
  end
end