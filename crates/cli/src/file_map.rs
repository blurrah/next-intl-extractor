use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{from_str, Map, Value};
use std::{collections::HashMap, fs, path::PathBuf, sync::Mutex};

#[derive(Eq, PartialEq)]
pub struct FileMap {
    pub name: String,
    pub file_path: PathBuf,
    pub contents: Value,
}

/// Merge the contents of all the files into a single JSON object
pub fn merge_map_contents_to_json(map: HashMap<String, FileMap>) -> Result<Map<String, Value>> {
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

/// Create initial map that will be used to merge data from files
/// It will also check for duplicate files for the same component and return an error when that happens
pub fn create_initial_map(files: Vec<String>) -> Result<()> {
    let mut map = GLOBAL_FILE_MAP.lock().unwrap();
    for file in files {
        let contents = fs::read_to_string(&file).expect("Unable to read file");
        let data: Value = from_str(&contents).unwrap();
        let file_name = file.split('/').last().unwrap_or("");
        let name = FILENAME_REGEX
            .captures(file_name)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();

        if name.is_empty() {
            log::warn!(
                "File name does not match the expected pattern: {}",
                file_name
            );
            continue;
        }

        // We don't allow multiple files to merge to the same key, show an error when this initially happens
        if map.contains_key(name) {
            let current_file = map
                .get(name)
                .ok_or_else(|| anyhow!("Failed to get file from map"))?
                .file_path
                .to_string_lossy()
                .to_string();

            return Err(anyhow!(
                "Duplicate file found for: {}, [{:?}]",
                name,
                vec![file.clone(), current_file]
            ));
        };

        map.insert(
            name.to_string(),
            FileMap {
                name: name.to_string(),
                file_path: fs::canonicalize(PathBuf::from(file.clone())).unwrap(),
                contents: data.clone(),
            },
        );
    }
    Ok(())
}
