use std::path::Path;

use notify::{Config, RecommendedWatcher, Watcher};

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
            log::info!("File created or modified: {:?}", event.paths);
        }
        notify::EventKind::Remove(_) => {
            log::info!("File removed: {:?}", event.paths);
        }
        _ => {
            log::info!("Other event: {:?}", event.paths);
        }
    }
}
