use log::{debug, info};
use regex::Regex;
/// This file contains all the filesystem operations that are needed for the application
/// to function. This includes finding files, reading files, and writing files.
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Find all *.labels.json files in the current directory and its subdirectories
pub fn find_files(dir: &PathBuf, re: &Regex) -> Vec<String> {
    let files = search_files(dir, re);

    // Only run this code path if the debug log level is actually enabled
    if log::log_enabled!(log::Level::Debug) {
        for file in &files {
            let relative_path = Path::new(file).strip_prefix(dir).unwrap().to_string_lossy();
            debug!("Found file: {}", relative_path);
        }
    }

    files
}

/// Recursively search for files in a directory that match a regex
/// and collect them into a vector.
pub fn search_files(dir: &Path, re: &Regex) -> Vec<String> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok) // Convert Results to Options, filtering out errors
        .map(|entry| entry.path().to_owned())
        .filter(|path| path.is_file()) // Ensure we're dealing with files
        .filter_map(|path| {
            // Extract the file name as a &str
            path.clone()
                .file_name()
                .and_then(|name| name.to_str())
                // Match it against the regex, returning the path if it matches
                .and_then(|name| if re.is_match(name) { Some(path) } else { None })
        })
        .filter_map(|path| path.to_str().map(String::from)) // Convert PathBuf to String
        .collect()
}
