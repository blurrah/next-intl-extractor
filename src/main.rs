use std::{collections::HashMap, env, fs, hash::Hash, path::{Path, PathBuf}, process::{exit, ExitCode}};
use console::{style, Term};
use files::search_files;
use lazy_static::lazy_static;
use notify::{Config, RecommendedWatcher, Watcher};
use serde_json::{from_str, json, to_string_pretty, Map, Value};
use regex::Regex;
use clap::Parser;

pub mod files;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Watch for file changes and merge them automatically
    #[arg(short, long, default_value = "false")]
    watch: bool
}

lazy_static! {
    static ref FILENAME_REGEX: Regex = Regex::new(r#"([^\.]+)\.labels\.json$"#).unwrap();
}


fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
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

fn check_event(event: notify::Event) {
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

fn main() {
    // Set up logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let term = Term::stdout();
    let path = env::current_dir().unwrap();
    let mut file_map: HashMap<String, PathBuf> = HashMap::new();

    let files = find_files();
    let mut merged_data: Map<String, Value> = Map::new();

    for file in files {
        let contents = fs::read_to_string(&file).expect("Unable to read file");
        let data: Value = from_str(&contents).expect("Unable to parse JSON");
        let file_name = file.split("/").last().unwrap_or("");
        let name = FILENAME_REGEX.captures(&file_name).unwrap().get(1).unwrap().as_str();

        if file_map.contains_key(name) {
            let current_file = file_map.get(name).unwrap().to_str().unwrap();
            term.write_line(&format!("{}", style(format!("❌ Duplicate file found for: {}, [\"{}\", \"{}\"]", name, file, current_file)).red())).unwrap_or(());
            exit(1)
        }

        // Save to hashmap for later use
        file_map.insert(
            name.to_string(),
            PathBuf::from(file.clone())
        );

        merged_data.insert(name.to_string(), data);
    }
    let merged_json = json!(merged_data);
    let merged_json_str = to_string_pretty(&merged_json).expect("Unable to serialize JSON");


    fs::write("output.json", merged_json_str).expect("Unable to write file");

    // Initial merge has been done, check if application should keep running in watch mode
    if args.watch {
        term.write_line(&format!("{}", style("Starting in watch mode").yellow())).unwrap_or(());

        if let Err(error) = watch(path) {
            log::error!("An error occurred while watching for file changes: {}", error);
        }
    }

}

// Find all *.labels.json files in the current directory and its subdirectories
fn find_files() -> Vec<String> {
    let current_dir = env::current_dir().unwrap();

    // Recursively search for files matching the regex
    // let mut files: Vec<String> = Vec::new();
    let files = search_files(&current_dir, &FILENAME_REGEX);

    // Process the found files
    for file in &files {
        // Process each file here
        println!("Found file: {}", file);
    }

    files
}


