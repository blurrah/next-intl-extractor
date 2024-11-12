use std::path::Path;
use std::{path::PathBuf, process::ExitCode};

use crate::files::find_files;
use crate::messages::MessageHandler;
use anyhow::{anyhow, Error};
use clap::{arg, command, Parser};
use next_intl_extractor::visitor::TranslationFunctionVisitor;
use tracing::{error, info, span, Level};
use tracing_subscriber::FmtSubscriber;

pub mod files;
pub mod messages;

#[derive(Parser, Debug)]
#[command(name = "next-intl-extractor")]
#[command(version = "0.1.0")]
#[command(about = "Extracts next-intl messages")]
#[command(long_about = None)]
struct CliArguments {
    /// Watch for file changes and merge them automatically
    #[arg(short, long, default_value = "false")]
    watch: bool,

    /// Output file
    #[clap(long, short, value_parser = clap::value_parser!(PathBuf))]
    output_path: PathBuf,

    /// Pattern for components to find
    #[arg(short, long, default_value = "**/*.{tsx,ts}")]
    pattern: String,
}

fn run() -> Result<(), Error> {
    let run_span = span!(Level::INFO, "run");
    let _enter = run_span.enter();

    info!("Starting run function");

    // Parse arguments
    let args = CliArguments::parse();
    info!("Arguments parsed: {:?}", args);

    // Check that output file is a .json file
    if args.output_path.extension().unwrap_or_default() != "json" {
        error!("Invalid output file extension");
        return Err(anyhow!("Output file must be a .json file"));
    }

    // Initialize message handler
    let mut message_handler = MessageHandler::new(&args.output_path)?;

    // Find files matching the glob pattern
    let files = find_files(&args.pattern)?;
    info!("Found {} files matching the pattern", files.len());

    // Process each file
    for file in files {
        info!("Processing file: {:?}", file);
        let file_path = Path::new(&file);
        let messages = next_intl_extractor::extract_translations(file_path)?;
        message_handler.add_extracted_messages(messages);
    }

    // After processing all files:
    let merged_messages = message_handler.merge_messages();

    // Write merged_messages to the output file
    message_handler.write_merged_messages(merged_messages, &args.output_path)?;

    if args.watch {
        info!("Watching for file changes");
    }

    info!("Run function completed successfully");
    Ok(())
}

fn main() -> ExitCode {
    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    let root_span = span!(Level::INFO, "cli_execution");
    let _enter = root_span.enter();

    info!("Starting CLI execution");

    // Run the actual application
    match run() {
        Ok(_) => {
            info!("CLI execution completed successfully");
            ExitCode::SUCCESS
        }
        Err(e) => {
            // In release mode, print the error to stderr
            #[cfg(not(debug_assertions))]
            eprintln!("Error: {}", e);

            // In debug mode, use tracing to log the error
            #[cfg(debug_assertions)]
            error!("Error during CLI execution: {}", e);

            ExitCode::FAILURE
        }
    }
}
