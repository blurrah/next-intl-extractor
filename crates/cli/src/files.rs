use anyhow::Result;
use glob::glob;
use std::path::PathBuf;

/// Find all files that match a glob pattern
pub fn find_files(glob_pattern: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in glob(glob_pattern)?.flatten() {
        if entry.is_file() {
            files.push(entry);
        }
    }

    Ok(files)
}
