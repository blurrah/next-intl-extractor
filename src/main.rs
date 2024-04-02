use std::{env, fs, path::Path};
use console::{style, Term};
use lazy_static::lazy_static;
use serde_json::{from_str, json, to_string_pretty, Map, Value};
use regex::Regex;
use clap::Parser;


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
    let args = Args::parse();
    let term = Term::stdout();

    if args.watch {
        term.write_line(&format!("{}", style("Starting in watch mode").yellow())).unwrap_or(());
    }

    let files = find_files();
    let mut merged_data: Map<String, Value> = Map::new();

    for file in files {
        let contents = fs::read_to_string(&file).expect("Unable to read file");
        let data: Value = from_str(&contents).expect("Unable to parse JSON");
        let file_name = file.split("/").last().unwrap_or("");
        let name = FILENAME_REGEX.captures(&file_name).unwrap().get(1).unwrap().as_str();

        merged_data.insert(name.to_string(), data);
    }
    let merged_json = json!(merged_data);
    let merged_json_str = to_string_pretty(&merged_json).expect("Unable to serialize JSON");


    fs::write("output.json", merged_json_str).expect("Unable to write file");

}

// Find all *.labels.json files in the current directory and its subdirectories
fn find_files() -> Vec<String> {
    let current_dir = env::current_dir().unwrap();

    // Recursively search for files matching the regex
    let mut files: Vec<String> = Vec::new();
    search_files(&current_dir, &FILENAME_REGEX, &mut files);

    // Process the found files
    for file in &files {
        // Process each file here
        println!("Found file: {}", file);
    }

    files
}

/// Recursively search for files in a directory that match a regex
fn search_files(dir: &Path, re: &Regex, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    // Recursively search subdirectories
                    search_files(&path, re, files);
                } else if let Some(file_name) = path.file_name() {
                    if let Some(file_name_str) = file_name.to_str() {
                        if re.is_match(file_name_str) {
                            // Add the file path to the list
                            if let Some(file_path) = path.to_str() {
                                files.push(file_path.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}
