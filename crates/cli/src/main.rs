use std::{path::PathBuf, process::ExitCode};

use anyhow::{Error, anyhow};
use clap::{arg, command, Parser};
use next_intl_extractor::visitor::TranslationFunctionVisitor;
use tracing::{info, error, span, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> ExitCode {
    // Initialize tracing
    tracing_subscriber::fmt().init();

    let root_span = span!(Level::INFO, "cli_execution");
    let _enter = root_span.enter();

    info!("Starting CLI execution");

    match run() {
        Ok(_) => {
            info!("CLI execution completed successfully");
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("Error during CLI execution: {}", e);
            ExitCode::FAILURE
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Watch for file changes and merge them automatically
    #[arg(short, long, default_value = "false")]
    watch: bool,

    /// Output file
    #[clap(long, short, value_parser = clap::value_parser!(PathBuf))]
    output: PathBuf,

    /// Pattern for components to find
    #[arg(short, long, value_parser = clap::value_parser!(PathBuf), default_value = "**/*.{tsx,ts}")]
    pattern: PathBuf
}


fn run() -> Result<(), Error> {
    let run_span = span!(Level::INFO, "run");
    let _enter = run_span.enter();

    info!("Starting run function");

    // Parse arguments
    let args = Args::parse();
    info!("Arguments parsed: {:?}", args);

    // Check that output file is a .json file
    if args.output.extension().unwrap_or_default() != "json" {
        error!("Invalid output file extension");
        return Err(anyhow!("Output file must be a .json file"));
    }

    // TODO: Add your main logic here
    info!("Main logic execution would go here");

    info!("Run function completed successfully");
    Ok(())
}
