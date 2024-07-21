use std::{
    fs,
    path::{Path, PathBuf},
};

use notify::{Config, RecommendedWatcher, Watcher};
use serde_json::{Map, Value};

use crate::{
    file_map::{FileMap, GLOBAL_FILE_MAP},
    helpers::write_to_output,
    FILENAME_REGEX,
};

pub fn watch<P: AsRef<Path>>(path: P, output_path: &PathBuf) -> notify::Result<()> {
    // multi producer single consumer queue
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(path.as_ref(), notify::RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                // log::info!("Event: {:?}", event);
                check_event(event, output_path)
            }
            Err(e) => {
                log::error!("Watch error: {:?}", e);
            }
        }
    }

    Ok(())
}

/// Check file update events and update the output accordingly
pub fn check_event(event: notify::Event, output_path: &PathBuf) {
    match event.kind {
        notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
            let mut merged_data: Map<String, Value> = Map::new();
            let mut map = GLOBAL_FILE_MAP.lock().unwrap();

            log::debug!("File created or modified: {:?}", event.paths);
            for path in &event.paths {
                // Get the file name
                let file_name = if let Some(file_name) = path.file_name() {
                    log::debug!("File name found: {:?}", file_name);
                    file_name
                } else {
                    log::debug!("File name not found: {:?}", path);
                    continue;
                };

                if !file_name.to_string_lossy().ends_with("labels.json") {
                    // Do not run if the file is not a JSON file
                    log::debug!("File is not a JSON file: {}", path.to_str().unwrap_or(""));
                    continue;
                }

                let absolute_path = fs::canonicalize(path).expect("Unable to get absolute path");

                // Check if the file is already in the map
                if let Some(value) = map
                    .iter_mut()
                    .find(|(_, val)| val.file_path == absolute_path)
                {
                    // Update the contents
                    log::debug!("Found existing file {}, updating contents", value.1.name);
                    let contents = std::fs::read_to_string(path).expect("Unable to read file");
                    value.1.contents =
                        serde_json::from_str(&contents).expect("Unable to parse JSON");
                } else {
                    // Check if the component name already exists in the map
                    let file_name = path.to_str().unwrap_or("").split('/').last().unwrap_or("");

                    let name = match FILENAME_REGEX.captures(file_name) {
                        Some(captures) => captures.get(1).map(|m| m.as_str()).unwrap_or(""),
                        None => "",
                    };

                    if name.is_empty() {
                        log::warn!("Component name not found in file name: {}", file_name);
                        continue;
                    }

                    if map.contains_key(name) {
                        // This component already exists in the map with a different file name, show a warning.
                        // We don't want to merge multiple files for one component
                        log::warn!(
                            "Component {} already exists in the map with a different name, original file is {} and new file is {}",
                            name, map.get(name).unwrap().file_path.to_string_lossy(), absolute_path.to_string_lossy()
                        );
                    } else {
                        let contents = std::fs::read_to_string(path).expect("Unable to read file");
                        let value: Value =
                            serde_json::from_str(&contents).expect("Unable to parse JSON");
                        // Insert the file into the map
                        map.insert(
                            name.to_string(),
                            FileMap {
                                name: name.to_string(),
                                file_path: path.to_path_buf(),
                                contents: value,
                            },
                        );
                    }
                }
            }
            // TODO: Move this to separate function once I figure out MutexGuards more properly :)
            for (key, value) in map.iter() {
                merged_data.insert(key.clone(), value.contents.clone());
            }

            write_to_output(&mut merged_data, output_path).expect("Test");
        }
        notify::EventKind::Remove(_) => {
            log::debug!("File removed: {:?}", event.paths);
            let mut merged_data: Map<String, Value> = Map::new();
            let mut map = GLOBAL_FILE_MAP.lock().unwrap();

            for path in &event.paths {
                // Get the file name
                let file_name = if let Some(file_name) = path.file_name() {
                    log::debug!("File name found: {:?}", file_name);
                    file_name
                } else {
                    log::debug!("File name not found: {:?}", path);
                    continue;
                };

                if !file_name.to_string_lossy().ends_with("labels.json") {
                    // Do not run if the file is not a JSON file
                    log::debug!("File is not a JSON file: {}", path.to_str().unwrap_or(""));
                    continue;
                }

                let absolute_path = match fs::canonicalize(path) {
                    Ok(path) => path,
                    Err(e) => {
                        log::debug!(
                            "Unable to get absolute path for {}: {}",
                            path.to_string_lossy(),
                            e
                        );
                        continue;
                    }
                };

                let value = map.iter().find_map(|(key, val)| {
                    if val.file_path == absolute_path {
                        Some(key.clone())
                    } else {
                        None
                    }
                });
                if let Some(value) = value {
                    // Remove the key from the map
                    map.remove(&value);
                }
            }
            // TODO: Move this to separate function once I figure out MutexGuards more properly :)
            for (key, value) in map.iter() {
                merged_data.insert(key.clone(), value.contents.clone());
            }

            write_to_output(&mut merged_data, output_path).expect("Test");
        }
        // We don't need to handle other events
        _ => (),
    }
}
