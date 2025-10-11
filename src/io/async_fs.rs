// FilePath: src/io/async_fs.rs

//! Async file system operations
//!
//! This module provides async wrappers around file system operations using tokio::fs
//! to prevent blocking the UI thread. All operations include timeout handling to
//! prevent indefinite hangs.

use crate::core::error::{LazyTablesError, Result};
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::time::timeout;

/// Default timeout for file operations (5 seconds)
const FILE_OP_TIMEOUT: Duration = Duration::from_secs(5);

/// Read file contents to a string asynchronously with timeout
///
/// # Arguments
/// * `path` - Path to the file to read
///
/// # Returns
/// * `Ok(String)` - File contents
/// * `Err` - If file doesn't exist, permission denied, timeout, or I/O error
///
/// # Example
/// ```no_run
/// use lazytables::io::async_fs::read_to_string;
///
/// #[tokio::main]
/// async fn main() {
///     let content = read_to_string("/path/to/file.txt").await.unwrap();
///     println!("File content: {}", content);
/// }
/// ```
pub async fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref().to_path_buf();
    let path_display = path.display().to_string();

    crate::log_debug!("Reading file asynchronously: {}", path_display);

    let result = timeout(FILE_OP_TIMEOUT, fs::read_to_string(&path)).await;

    match result {
        Ok(Ok(contents)) => {
            crate::log_debug!(
                "Successfully read {} bytes from {}",
                contents.len(),
                path_display
            );
            Ok(contents)
        }
        Ok(Err(e)) => {
            let error_msg = format!("Failed to read file {}: {}", path_display, e);
            crate::log_error!("{}", error_msg);
            Err(LazyTablesError::Other(error_msg))
        }
        Err(_) => {
            let error_msg = format!(
                "Timeout reading file {} (exceeded {} seconds)",
                path_display,
                FILE_OP_TIMEOUT.as_secs()
            );
            crate::log_error!("{}", error_msg);
            Err(LazyTablesError::Other(error_msg))
        }
    }
}

/// Write contents to a file asynchronously with timeout
///
/// # Arguments
/// * `path` - Path to the file to write
/// * `contents` - Contents to write to the file
///
/// # Returns
/// * `Ok(())` - File written successfully
/// * `Err` - If permission denied, timeout, or I/O error
///
/// # Example
/// ```no_run
/// use lazytables::io::async_fs::write;
///
/// #[tokio::main]
/// async fn main() {
///     write("/path/to/file.txt", "Hello, world!").await.unwrap();
/// }
/// ```
pub async fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    let path = path.as_ref().to_path_buf();
    let path_display = path.display().to_string();
    let contents_ref = contents.as_ref();
    let byte_count = contents_ref.len();

    crate::log_debug!(
        "Writing {} bytes to file asynchronously: {}",
        byte_count,
        path_display
    );

    let result = timeout(FILE_OP_TIMEOUT, fs::write(&path, contents_ref)).await;

    match result {
        Ok(Ok(())) => {
            crate::log_debug!(
                "Successfully wrote {} bytes to {}",
                byte_count,
                path_display
            );
            Ok(())
        }
        Ok(Err(e)) => {
            let error_msg = format!("Failed to write file {}: {}", path_display, e);
            crate::log_error!("{}", error_msg);
            Err(LazyTablesError::Other(error_msg))
        }
        Err(_) => {
            let error_msg = format!(
                "Timeout writing file {} (exceeded {} seconds)",
                path_display,
                FILE_OP_TIMEOUT.as_secs()
            );
            crate::log_error!("{}", error_msg);
            Err(LazyTablesError::Other(error_msg))
        }
    }
}

/// Create a directory and all of its parent components asynchronously with timeout
///
/// # Arguments
/// * `path` - Path to the directory to create
///
/// # Returns
/// * `Ok(())` - Directory created successfully or already exists
/// * `Err` - If permission denied, timeout, or I/O error
///
/// # Example
/// ```no_run
/// use lazytables::io::async_fs::create_dir_all;
///
/// #[tokio::main]
/// async fn main() {
///     create_dir_all("/path/to/directory").await.unwrap();
/// }
/// ```
pub async fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref().to_path_buf();
    let path_display = path.display().to_string();

    crate::log_debug!("Creating directory asynchronously: {}", path_display);

    let result = timeout(FILE_OP_TIMEOUT, fs::create_dir_all(&path)).await;

    match result {
        Ok(Ok(())) => {
            crate::log_debug!("Successfully created directory: {}", path_display);
            Ok(())
        }
        Ok(Err(e)) => {
            let error_msg = format!("Failed to create directory {}: {}", path_display, e);
            crate::log_error!("{}", error_msg);
            Err(LazyTablesError::Other(error_msg))
        }
        Err(_) => {
            let error_msg = format!(
                "Timeout creating directory {} (exceeded {} seconds)",
                path_display,
                FILE_OP_TIMEOUT.as_secs()
            );
            crate::log_error!("{}", error_msg);
            Err(LazyTablesError::Other(error_msg))
        }
    }
}

/// Read directory entries asynchronously with timeout
///
/// # Arguments
/// * `path` - Path to the directory to read
///
/// # Returns
/// * `Ok(Vec<DirEntry>)` - List of directory entries
/// * `Err` - If directory doesn't exist, permission denied, timeout, or I/O error
///
/// # Example
/// ```no_run
/// use lazytables::io::async_fs::read_dir;
///
/// #[tokio::main]
/// async fn main() {
///     let entries = read_dir("/path/to/directory").await.unwrap();
///     for entry in entries {
///         println!("Found: {:?}", entry.file_name());
///     }
/// }
/// ```
pub async fn read_dir<P: AsRef<Path>>(path: P) -> Result<Vec<fs::DirEntry>> {
    let path = path.as_ref().to_path_buf();
    let path_display = path.display().to_string();

    crate::log_debug!("Reading directory asynchronously: {}", path_display);

    // Read directory entries
    let result = timeout(FILE_OP_TIMEOUT, async {
        let mut dir = fs::read_dir(&path).await?;
        let mut entries = Vec::new();

        while let Some(entry) = dir.next_entry().await? {
            entries.push(entry);
        }

        Ok::<Vec<fs::DirEntry>, std::io::Error>(entries)
    })
    .await;

    match result {
        Ok(Ok(entries)) => {
            crate::log_debug!(
                "Successfully read {} entries from {}",
                entries.len(),
                path_display
            );
            Ok(entries)
        }
        Ok(Err(e)) => {
            let error_msg = format!("Failed to read directory {}: {}", path_display, e);
            crate::log_error!("{}", error_msg);
            Err(LazyTablesError::Other(error_msg))
        }
        Err(_) => {
            let error_msg = format!(
                "Timeout reading directory {} (exceeded {} seconds)",
                path_display,
                FILE_OP_TIMEOUT.as_secs()
            );
            crate::log_error!("{}", error_msg);
            Err(LazyTablesError::Other(error_msg))
        }
    }
}

/// Check if a path exists asynchronously with timeout
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// * `Ok(true)` - Path exists
/// * `Ok(false)` - Path does not exist
/// * `Err` - If timeout or I/O error
pub async fn exists<P: AsRef<Path>>(path: P) -> Result<bool> {
    let path = path.as_ref().to_path_buf();

    let result = timeout(FILE_OP_TIMEOUT, async {
        tokio::fs::metadata(&path).await.is_ok()
    })
    .await;

    match result {
        Ok(exists) => Ok(exists),
        Err(_) => {
            let error_msg = format!(
                "Timeout checking if path exists (exceeded {} seconds)",
                FILE_OP_TIMEOUT.as_secs()
            );
            Err(LazyTablesError::Other(error_msg))
        }
    }
}

/// Remove/delete a file asynchronously with timeout
///
/// # Arguments
/// * `path` - Path to the file to remove
///
/// # Returns
/// * `Ok(())` - File removed successfully
/// * `Err` - If file doesn't exist, permission denied, timeout, or I/O error
pub async fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref().to_path_buf();
    let path_display = path.display().to_string();

    crate::log_debug!("Removing file asynchronously: {}", path_display);

    let result = timeout(FILE_OP_TIMEOUT, fs::remove_file(&path)).await;

    match result {
        Ok(Ok(())) => {
            crate::log_debug!("Successfully removed file: {}", path_display);
            Ok(())
        }
        Ok(Err(e)) => {
            let error_msg = format!("Failed to remove file {}: {}", path_display, e);
            crate::log_error!("{}", error_msg);
            Err(LazyTablesError::Io(e))
        }
        Err(_) => {
            let error_msg = format!(
                "Timeout removing file {} (exceeded {} seconds)",
                path_display,
                FILE_OP_TIMEOUT.as_secs()
            );
            crate::log_error!("{}", error_msg);
            Err(LazyTablesError::Other(error_msg))
        }
    }
}

/// Rename a file or directory asynchronously with timeout
///
/// # Arguments
/// * `from` - Path to the file/directory to rename
/// * `to` - New path for the file/directory
///
/// # Returns
/// * `Ok(())` - File/directory renamed successfully
/// * `Err` - If source doesn't exist, permission denied, timeout, or I/O error
pub async fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    let from = from.as_ref().to_path_buf();
    let to = to.as_ref().to_path_buf();
    let from_display = from.display().to_string();
    let to_display = to.display().to_string();

    crate::log_debug!(
        "Renaming asynchronously: {} to {}",
        from_display,
        to_display
    );

    let result = timeout(FILE_OP_TIMEOUT, fs::rename(&from, &to)).await;

    match result {
        Ok(Ok(())) => {
            crate::log_debug!("Successfully renamed {} to {}", from_display, to_display);
            Ok(())
        }
        Ok(Err(e)) => {
            let error_msg = format!("Failed to rename {} to {}: {}", from_display, to_display, e);
            crate::log_error!("{}", error_msg);
            Err(LazyTablesError::Io(e))
        }
        Err(_) => {
            let error_msg = format!(
                "Timeout renaming {} to {} (exceeded {} seconds)",
                from_display,
                to_display,
                FILE_OP_TIMEOUT.as_secs()
            );
            crate::log_error!("{}", error_msg);
            Err(LazyTablesError::Other(error_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_read_to_string_success() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_content = "Hello, LazyTables!";
        temp_file.write_all(test_content.as_bytes()).unwrap();

        let result = read_to_string(temp_file.path()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content);
    }

    #[tokio::test]
    async fn test_read_to_string_nonexistent() {
        let result = read_to_string("/nonexistent/file.txt").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let test_content = "Test content";

        let result = write(&file_path, test_content).await;
        assert!(result.is_ok());

        // Verify file was written
        let read_result = read_to_string(&file_path).await;
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_content);
    }

    #[tokio::test]
    async fn test_create_dir_all_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nested_path = temp_dir.path().join("a").join("b").join("c");

        let result = create_dir_all(&nested_path).await;
        assert!(result.is_ok());

        // Verify directory was created
        assert!(nested_path.exists());
        assert!(nested_path.is_dir());
    }

    #[tokio::test]
    async fn test_read_dir_success() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create some test files
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        write(&file1, "content1").await.unwrap();
        write(&file2, "content2").await.unwrap();

        let result = read_dir(temp_dir.path()).await;
        assert!(result.is_ok());

        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn test_exists() {
        let temp_file = NamedTempFile::new().unwrap();

        // Existing file
        let result = exists(temp_file.path()).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Non-existing file
        let result = exists("/nonexistent/file.txt").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
