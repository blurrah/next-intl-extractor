use crate::file_map::{FileMap, FILENAME_REGEX, GLOBAL_FILE_MAP};
use crate::watch::watch;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use console::{style, Term};
use files::find_files;
use helpers::write_to_output;

use serde_json::{from_str, Map, Value};
use std::{env, fs, path::PathBuf, process::exit, time::Instant};
use thiserror::Error;

pub mod file_map;
pub mod files;
pub mod helpers;
pub mod watch;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Watch for file changes and merge them automatically
    #[arg(short, long, default_value = "false")]
    watch: bool,

    /// Output file
    #[clap(long, short, value_parser = clap::value_parser!(PathBuf), default_value="output.json")]
    output: PathBuf,

    /// Input directory
    #[arg(short, long, value_parser = clap::value_parser!(PathBuf), default_value = ".")]
    input_dir: PathBuf,
}

pub fn main() -> Result<()> {
    let start = Instant::now();
    // Set up logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let term = Term::stdout();
    // Parse arguments
    let args = Args::parse();

    // Check if the input directory exists, if not, use the current directory
    // We might just throw an error as it already defaults back to current directory
    let path = if args.input_dir.exists() && args.input_dir.is_dir() {
        args.input_dir
    } else {
        log::error!(
            "Input directory {} does not exist",
            args.input_dir.display()
        );
        exit(1)
    };

    // If the output file does not have a parent directory, append it to the current working directory
    let output_path = if args.output.parent().is_some() {
        args.output
    } else {
        // Output file does not have a parent directory, append it to the current working directory
        match env::current_dir() {
            Ok(cwd) => {
                let output_file_name = match args.output.file_name() {
                    Some(name) => name,
                    None => {
                        format!(
                            "Failed to get file name from output path: {:?}",
                            args.output
                        );
                        exit(1);
                    }
                };
                let mut output_path = cwd;
                output_path.push(output_file_name);
                output_path
            }
            Err(err) => {
                format!("Failed to get current working directory: {}", err);
                exit(1);
            }
        }
    };

    let files = find_files(&path, &FILENAME_REGEX);

    let mut merged_data: Map<String, Value> = Map::new();

    create_initial_map(files).with_context(|| "An error occured while creating the initial map")?;

    for (key, value) in GLOBAL_FILE_MAP
        .lock()
        .map_err(|e| {
            anyhow!(
                "An error occured while trying to lock GLOBAL_FILE_MAP: {}",
                e
            )
        })?
        .iter()
    {
        merged_data.insert(key.clone(), value.contents.clone());
    }

    // Write the merged data to the output file
    write_to_output(&mut merged_data, &output_path).with_context(|| {
        format!(
            "An error occurred while writing to output file {}",
            &output_path.to_string_lossy()
        )
    })?;

    let duration = start.elapsed();
    log::info!("Time elapsed: {:?}", duration);

    // Initial merge has been done, check if application should keep running in watch mode
    if args.watch {
        term.write_line(&format!("{}", style("Starting in watch mode").yellow()))
            .unwrap_or(());

        // Start watching for file changes, see watch.rs for implementation
        watch(&path, &output_path)
            .with_context(|| "An error occurred while watching for file changes")?;
    }

    Ok(())
}

/// Create initial map that will be used to merge data from files
/// It will also check for duplicate files for the same component and return an error when that happens
fn create_initial_map(files: Vec<String>) -> Result<()> {
    let mut map = GLOBAL_FILE_MAP.lock().unwrap();
    for file in files {
        let contents = fs::read_to_string(&file).expect("Unable to read file");
        let data: Value = from_str(&contents).unwrap();
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

            let current_file = map
                .get(name)
                .ok_or_else(|| anyhow!("Failed to get file from map"))?
                .file_path
                .to_string_lossy()
                .to_string();


            return Err(anyhow!("Duplicate file found for: {}, [{:?}]", name, vec![file.clone(), current_file]));
        };

        map.insert(
            name.to_string(),
            FileMap {
                name: name.to_string(),
                file_path: fs::canonicalize(PathBuf::from(file.clone())).unwrap(),
                contents: data.clone(),
            },
        );
    }
    Ok(())
}
