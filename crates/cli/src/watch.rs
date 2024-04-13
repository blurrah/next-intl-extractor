use std::path::Path;

use notify::{Config, RecommendedWatcher, Watcher};

use crate::GLOBAL_FILE_MAP;

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
                // Remove the item from the GLOBAL_FILE_MAP
                let mut map = GLOBAL_FILE_MAP.lock().unwrap();
                let value = map
                    .iter()
                    .find_map(|(key, val)| if val == path { Some(key.clone()) } else { None });
                if let Some(value) = value {
                    // Remove the key from the map
                    map.remove(&value);
                }

                // TODO: Remove the data from the merged_data and rebuild the output file
            }
        }
        _ => {
            log::debug!("Other event: {:?}", event.paths);
        }
    }
}
