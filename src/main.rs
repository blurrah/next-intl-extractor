use std::{collections::HashMap, env, fs, path::PathBuf, process::exit};
use console::{style, Term};
use files::find_files;
use watch::watch;
use lazy_static::lazy_static;
use serde_json::{from_str, json, to_string_pretty, Map, Value};
use regex::Regex;
use clap::Parser;

pub mod files;
pub mod watch;

#[derive(Debug, Clone)]
struct DuplicateFileError {
    component: String,
    file_paths: Vec<String>
}


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

fn main() {
    // Set up logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let term = Term::stdout();
    // Parse arguments
    let args = Args::parse();
    // Default path is the current working directory (could extend this with arguments)
    let path = env::current_dir().unwrap();

    // There can only be one file per component, holding these references to make sure there aren't duplicates
    let file_map: HashMap<String, PathBuf> = HashMap::new();

    let files = find_files(&FILENAME_REGEX);

    let mut merged_data: Map<String, Value> = Map::new();

    if let Err(e) = merge_data(files, file_map, &mut merged_data) {
        term.write_line(&format!("{}", style(format!("❌ Duplicate file found for: {}, [{}]", e.component, e.file_paths.join(", "))).red())).unwrap_or(());
            exit(1);
    }

    let merged_json = json!(merged_data);
    let merged_string = match to_string_pretty(&merged_json) {
        Ok(str) => str,
        Err(e) => {
            log::error!("An error occurred while serializing JSON: {}", e);
            exit(1)
        }
    };

    fs::write("output.json", merged_string).expect("Unable to write file");

    // Initial merge has been done, check if application should keep running in watch mode
    if args.watch {
        term.write_line(&format!("{}", style("Starting in watch mode").yellow())).unwrap_or(());

        if let Err(error) = watch(path) {
            log::error!("An error occurred while watching for file changes: {}", error);
        }
    }

}

fn merge_data(files: Vec<String>, mut file_map: HashMap<String, PathBuf>, merged_data: &mut Map<String, Value>) -> Result<(), DuplicateFileError> {
    for file in files {
        let contents = fs::read_to_string(&file).expect("Unable to read file");
        let data: Value = from_str(&contents).expect("Unable to parse JSON");
        let file_name = file.split('/').last().unwrap_or("");
        let name = FILENAME_REGEX.captures(file_name).unwrap().get(1).unwrap().as_str();

        // We don't allow multiple files to merge to the same key, show an error when this initially happens
        if file_map.contains_key(name) {
            let current_file = file_map.get(name).unwrap().to_str().unwrap();

            return Err(DuplicateFileError {
                component: String::from(name),
                file_paths: vec![file.clone(), String::from(current_file)],
            });
        };

        // Save unique component and file combination
        file_map.insert(
            name.to_string(),
            PathBuf::from(file.clone())
        );

        merged_data.insert(name.to_string(), data);
    }
    Ok(())
}
