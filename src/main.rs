use std::{env, fs, path::Path};
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

fn main() {
    let args = Args::parse();

    println!("Test: {}", args.watch);
    let files = vec!["test.json"];
    let mut merged_data: Map<String, Value> = Map::new();

    for file in files {
        let contents = fs::read_to_string(file).expect("Unable to read file");
        let data: Value = from_str(&contents).expect("Unable to parse JSON");

        let file_name = file.split('.').next().unwrap();
        merged_data.insert(file_name.to_string(), data);
    }
    let merged_json = json!(merged_data);
    let merged_json_str = to_string_pretty(&merged_json).expect("Unable to serialize JSON");

    find_files();

    fs::write("output.json", merged_json_str).expect("Unable to write file");


}

fn find_files() {
    let current_dir = env::current_dir().unwrap();

    let re = Regex::new(r#"([^\.]+)\.labels\.json$"#).unwrap();

    // Recursively search for files matching the regex
    let mut files: Vec<String> = Vec::new();
    search_files(&current_dir, &re, &mut files);

    // Process the found files
    for file in files {
        // Process each file here
        println!("Found file: {}", file);
    }
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
