use clap::Parser;
use console::{style, Term};
use files::find_files;
use lazy_static::lazy_static;
use helpers::write_to_output;
use regex::Regex;
use serde_json::{from_str, Map, Value};
use std::{
    collections::HashMap,
    env, fs,
    path::PathBuf,
    process::exit,
    sync::Mutex,
};
use watch::watch;

pub mod files;
pub mod helpers;
pub mod watch;

#[derive(Debug, Clone)]
struct DuplicateFileError {
    component: String,
    file_paths: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Watch for file changes and merge them automatically
    #[arg(short, long, default_value = "false")]
    watch: bool,
}

lazy_static! {
    static ref FILENAME_REGEX: Regex = Regex::new(r#"([^\.]+)\.labels\.json$"#).unwrap();

    // There can only be one label file per component, holding these references to make sure there aren't duplicates
    static ref GLOBAL_FILE_MAP: Mutex<HashMap<String, PathBuf>> = Mutex::new(HashMap::new());
}

fn main() {
    // Set up logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let term = Term::stdout();
    // Parse arguments
    let args = Args::parse();
    // Default path is the current working directory (could extend this with arguments)
    let path = env::current_dir().unwrap();

    let files = find_files(&FILENAME_REGEX);

    let mut merged_data: Map<String, Value> = Map::new();

    if let Err(e) = merge_data(files, &mut merged_data) {
        let error_line = format!(
            "❌ Duplicate file found for: {}, [{}]",
            e.component,
            e.file_paths.join(", ")
        );
        term.write_line(&format!("{}", style(error_line).red())).unwrap_or(());
        exit(1);
    }

    // Write the merged data to the output file
    if let Err(er) = write_to_output(&mut merged_data, &path) {
        log::error!("An error occurred while writing to output file: {}", er);
        exit(1);
    }

    // Initial merge has been done, check if application should keep running in watch mode
    if args.watch {
        term.write_line(&format!("{}", style("Starting in watch mode").yellow()))
            .unwrap_or(());

        if let Err(error) = watch(path) {
            log::error!(
                "An error occurred while watching for file changes: {}",
                error
            );
        }
    }
}

/// Merge data from given files into a single deserialized JSON object
/// It will also check for duplicate files for the same component and return an error when that happens
fn merge_data(
    files: Vec<String>,
    merged_data: &mut Map<String, Value>,
) -> Result<(), DuplicateFileError> {
    let mut map = GLOBAL_FILE_MAP.lock().unwrap();
    for file in files {
        let contents = fs::read_to_string(&file).expect("Unable to read file");
        let data: Value = from_str(&contents).expect("Unable to parse JSON");
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
            let current_file = map.get(name).unwrap().to_str().unwrap();

            return Err(DuplicateFileError {
                component: String::from(name),
                file_paths: vec![file.clone(), String::from(current_file)],
            });
        };

        // Save unique component and file combination
        map.insert(name.to_string(), PathBuf::from(file.clone()));

        merged_data.insert(name.to_string(), data);
    }
    Ok(())
}
