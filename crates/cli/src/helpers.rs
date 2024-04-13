use std::{fs, path::{Path, PathBuf}};

use serde_json::{json, to_string_pretty, Map, Value};

/// Helper function to write a deserialized JSON object to an output file
pub fn write_to_output(
    json_data: &mut Map<String, Value>,
    path: &PathBuf,
) -> Result<(), std::io::Error> {
    let json = json!(json_data);
    let string = to_string_pretty(&json)?;

    fs::write(path, string)?;

    Ok(())
}

/// Helper function to append a given string to a PathBuf
///
/// Example
/// ```rust
/// append_to_path(&PathBuf::from("/path/to"), "file.json"); // "/path/to/file.json"
/// ```
pub fn append_to_path(path: &Path, append: &str) -> PathBuf {
    let mut path = path.to_path_buf().clone().into_os_string();
    path.push(append);
    path.into()
}
