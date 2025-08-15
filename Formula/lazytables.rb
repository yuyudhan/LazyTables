class Lazytables < Formula
  desc "Terminal-based SQL database viewer and editor with vim-style navigation"
  homepage "https://github.com/yuyudhan/LazyTables"
  license "WTFPL"
  version "0.1.3"
  
  # Binary releases for different architectures
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"  # Will be updated when release is created
    else
      url "https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64"  # Will be updated when release is created
    end
  elsif OS.linux?
    url "https://github.com/yuyudhan/LazyTables/releases/download/v0.1.3/lazytables-v0.1.3-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX"  # Will be updated when release is created
  end
  
  # Fallback to building from source if binary not available
  head do
    url "https://github.com/yuyudhan/LazyTables.git", branch: "main"
    depends_on "rust" => :build
  end
  
  # Option to build from source
  option "with-source", "Build from source instead of using precompiled binary"
  
  depends_on "rust" => :build if build.with?("source") || build.head?

  def install
    if build.with?("source") || build.head?
      # Build from source
      system "cargo", "build", "--release", "--locked"
      bin.install "target/release/lazytables"
    else
      # Install precompiled binary
      bin.install "lazytables"
    end
    
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