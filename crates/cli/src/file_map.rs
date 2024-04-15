use serde_json::{Map, Value};
use std::{collections::HashMap, path::PathBuf};

#[derive(Eq, PartialEq)]
pub struct FileMap {
    // Name of component, already used in HashMap key so not sure if useful..
    pub name: String,
    pub file_path: PathBuf,
    pub contents: Value,
}

pub fn merge_map_contents_to_json(
    map: HashMap<String, FileMap>,
) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
    let mut merged_data: Map<String, Value> = Map::new();

    for (key, value) in map.iter() {
        merged_data.insert(key.clone(), value.contents.clone());
    }

    Ok(merged_data)
}
