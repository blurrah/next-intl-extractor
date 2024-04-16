use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{Map, Value};
use std::{collections::HashMap, path::PathBuf, sync::Mutex};

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

lazy_static! {
    // Regex to match the file name
    pub static ref FILENAME_REGEX: Regex = Regex::new(r#"([^\.]+)\.labels\.json$"#).unwrap();

    // Global file map to store all the file contents
    pub static ref GLOBAL_FILE_MAP: Mutex<HashMap<String,FileMap>> = Mutex::new(HashMap::new());
}
