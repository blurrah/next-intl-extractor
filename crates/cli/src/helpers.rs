use std::{
    fs,
    path::{Path, PathBuf},
};

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
