use std::path::{Path, PathBuf};

use notify::{Config, RecommendedWatcher, Watcher};
use serde_json::{Map, Value};

use crate::{helpers::write_to_output, GLOBAL};

pub fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    // multi producer single consumer queue
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(path.as_ref(), notify::RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                // log::info!("Event: {:?}", event);
                check_event(event)
            }
            Err(e) => {
                log::error!("Watch error: {:?}", e);
            }
        }
    }

    Ok(())
}

pub fn check_event(event: notify::Event) {
    match event.kind {
        notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
            log::debug!("File created or modified: {:?}", event.paths);
        }
        notify::EventKind::Remove(_) => {
            log::debug!("File removed: {:?}", event.paths);

            for path in &event.paths {
                let mut map = GLOBAL.lock().unwrap();
                let value = map.iter().find_map(|(key, val)| {
                    if val.file_path == *path {
                        Some(key.clone())
                    } else {
                        None
                    }
                });
                if let Some(value) = value {
                    // Remove the key from the map
                    map.remove(&value);
                }

                // TODO: Move this to separate function once I figure out MutexGuards more properly :)
                let mut merged_data: Map<String, Value> = Map::new();

                for (key, value) in map.iter() {
                    merged_data.insert(key.clone(), value.contents.clone());
                }

                let path = PathBuf::from("output.json");

                write_to_output(&mut merged_data, &path).expect("Test");
            }
        }
        _ => {
            log::debug!("Other event: {:?}", event.paths);
        }
    }
}
