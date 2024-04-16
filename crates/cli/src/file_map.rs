use serde_json::{Map, Value};
use std::{collections::HashMap, path::PathBuf};

#[derive(Eq, PartialEq)]
pub struct FileMap {
    pub name: String,
    pub file_path: PathBuf,
    pub contents: Value,
}

/// Merge the contents of all the files into a single JSON object
pub fn merge_map_contents_to_json(
    map: HashMap<String, FileMap>,
) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
    let merged_data: Map<String, Value> = map
        .into_iter()
        .map(|(key, value)| (key, value.contents))
        .collect();

    Ok(merged_data)
}
