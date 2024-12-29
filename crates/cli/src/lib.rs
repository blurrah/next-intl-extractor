use std::path::PathBuf;

use wasm_bindgen::prelude::*;

mod files;
mod messages;
mod watch;

use crate::files::find_files;
use crate::messages::MessageHandler;
use anyhow::Error;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "next-intl-resolver")]
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

#[wasm_bindgen]
pub async fn run(args: Vec<String>) -> Result<(), JsValue> {
    let args = CliArguments::parse_from(args);


    // Initialize message handler
    let mut message_handler = MessageHandler::new(&args.output_path)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Find and process files
    let files = find_files(&args.pattern)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    if files.is_empty() {
        return Err(JsValue::from_str(&format!("No files found for pattern: {}", args.pattern)));
    }

    for file in files {
        let translations = next_intl_resolver::extract_translations(&file);

        if let Ok(translations) = translations {
            for (namespace, keys) in translations.iter() {
                for key in keys {
                    message_handler.add_extracted_message(
                        namespace.clone(),
                        key.to_string(),
                        file.to_string_lossy().into_owned(),
                    );
                }
            }
        }
    }

    // Check for conflicts
    let conflicts = message_handler.get_conflicts();
    if !conflicts.is_empty() {
        let mut error_msg = String::from("Found namespace conflicts:\n");
        for conflict in conflicts {
            error_msg.push_str(&format!(
                "Namespace '{}' key '{}' is used in multiple files:\n",
                conflict.namespace, conflict.key
            ));
            for file in &conflict.files {
                error_msg.push_str(&format!("  - {}\n", file));
            }
        }
        return Err(JsValue::from_str(&error_msg));
    }

    // If no conflicts, proceed with merging
    message_handler
        .write_merged_messages(&args.output_path)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(())
}
