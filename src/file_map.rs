use std::{collections::HashMap, path::PathBuf};
use serde_json::{Map, Value};



#[derive(Default)]
pub struct GlobalFileMap {
    items: HashMap<String, FileMap>
}

impl GlobalFileMap {
    pub fn new() -> Self {
        GlobalFileMap {
            items: HashMap::new()
        }
    }

    pub fn add(&mut self, component: String, file_map: FileMap) {
        self.items.insert(component, file_map);
    }

    pub fn get(&self, component: &str) -> Option<&FileMap> {
        self.items.get(component)
    }

    pub fn get_by_file_path(&self, file_path: &PathBuf) -> Option<&FileMap> {
        self.items.values().find(|file_map| &file_map.file_path == file_path)
    }

    /// Return all of the contents together
    pub fn get_merged_contents(&self) -> Map<String, Value> {
        let mut merged_contents = Map::new();
        for file_map in self.items.values() {
            for (key, value) in &file_map.contents {
                merged_contents.insert(key.clone(), value.clone());
            }
        }
        merged_contents
    }
}

#[derive(Eq, PartialEq)]
pub struct FileMap {
    file_path: PathBuf,
    contents: Map<String, Value>
}
