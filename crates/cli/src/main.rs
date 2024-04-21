use crate::file_map::{create_initial_map, FILENAME_REGEX, GLOBAL_FILE_MAP};
use crate::files::find_files_with_git;
use crate::watch::watch;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use console::{style, Term};
use files::find_files;
use helpers::write_to_output;

use serde_json::{Map, Value};
use std::{env, path::PathBuf, time::Instant};

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

fn main() -> Result<()> {
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
        return Err(anyhow!(
            "Input directory {} does not exist",
            args.input_dir.display()
        ))?;
    };

    // If the output file does not have a parent directory, append it to the current working directory
    let output_path = if args.output.parent().is_some() {
        args.output
    } else {
        // Output file does not have a parent directory, append it to the current working directory
        let cwd = env::current_dir().context("Failed to get current working directory")?;
        let output_file_name = args
            .output
            .file_name()
            .context("Failed to get file name from output path")?;
        let mut output_path = cwd;
        output_path.push(output_file_name);

        output_path
    };

    let files = find_files(&path, &FILENAME_REGEX)?;

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
