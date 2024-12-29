use std::path::Path;
use anyhow::Result;

/// Trait for abstracting file system operations
/// This is due to the wasm approach where we need to use JS hooks to access the filesystem
pub trait FileSystem {
    /// Read a file's contents as a string
    fn read_to_string(&self, path: &Path) -> Result<String>;

    /// Write a string to a file, creating the file if it doesn't exist
    fn write(&self, path: &Path, contents: &str) -> Result<()>;

    /// Create a directory and all its parent components if they are missing
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Check if a path exists
    fn exists(&self, path: &Path) -> bool;
}

/// Default implementation using std::fs
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    fn write(&self, path: &Path, contents: &str) -> Result<()> {
        Ok(std::fs::write(path, contents)?)
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        Ok(std::fs::create_dir_all(path)?)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

/// Get the default file system implementation
pub fn default_fs() -> StdFileSystem {
    StdFileSystem
}
