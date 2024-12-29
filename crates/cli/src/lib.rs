use std::path::PathBuf;

use wasm_bindgen::prelude::*;

mod files;
mod messages;
mod watch;

use crate::files::find_files;
use crate::messages::MessageHandler;
use clap::Parser;
use anyhow::anyhow;

// Import console_log and JS functions for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // JS functions for file operations
    #[wasm_bindgen(js_namespace = ["globalThis", "__WASM_HOOKS"])]
    fn write_file(path: &str, content: &str) -> bool;

    #[wasm_bindgen(js_namespace = ["globalThis", "__WASM_HOOKS"])]
    fn read_file(path: &str) -> Option<String>;

    #[wasm_bindgen(js_namespace = ["globalThis", "__WASM_HOOKS"])]
    fn ensure_dir(path: &str) -> bool;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

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
    #[clap(long, short)]
    output_path: String,

    /// Pattern for components to find
    #[arg(short, long, default_value = "**/*.{tsx,ts}")]
    pattern: String,
}

#[wasm_bindgen]
pub async fn run(args: Vec<String>) -> Result<(), JsValue> {
    console_log!("Starting with args: {:?}", args);

    // Convert JS array to Vec<String>
    let args: Vec<String> = args.into_iter().map(|s| s.to_string()).collect();
    console_log!("Converted args: {:?}", args);

    // Parse arguments
    let args = match CliArguments::try_parse_from(&args) {
        Ok(args) => {
            console_log!("Successfully parsed arguments");
            args
        }
        Err(e) => {
            let err_msg = format!("Failed to parse arguments: {}", e);
            console_log!("{}", err_msg);
            return Err(JsValue::from_str(&err_msg));
        }
    };

    console_log!("Parsed arguments: {:?}", args);

    let output_path = PathBuf::from(&args.output_path);
    console_log!("Output path: {:?}", output_path);

    // Create output directory and file using JS functions
    if let Some(parent) = output_path.parent() {
        if !ensure_dir(parent.to_str().unwrap_or("")) {
            return Err(JsValue::from_str("Failed to create output directory"));
        }
    }

    if read_file(&args.output_path).is_none() {
        if !write_file(&args.output_path, "{}") {
            return Err(JsValue::from_str("Failed to create output file"));
        }
        console_log!(
            "Output file does not exist yet. Created: {:?}",
            args.output_path
        );
    }

    // Initialize message handler
    let mut message_handler = match MessageHandler::new(&output_path) {
        Ok(handler) => handler,
        Err(e) => {
            console_log!("Failed to create message handler: {}", e);
            return Err(JsValue::from_str(&format!("Failed to create message handler: {}", e)));
        }
    };

    // Find and process files
    let files = match find_files(&args.pattern) {
        Ok(files) => files,
        Err(e) => {
            console_log!("Failed to find files: {}", e);
            return Err(JsValue::from_str(&format!("Failed to find files: {}", e)));
        }
    };

    if files.is_empty() {
        console_log!("No files found for pattern: {}", args.pattern);
        return Err(JsValue::from_str(&format!("No files found for pattern: {}", args.pattern)));
    }

    console_log!("Found {} files to process", files.len());

    for file in files {
        console_log!("Processing file: {:?}", file);
        let translations = next_intl_resolver::extract_translations(&file);

        match translations {
            Ok(translations) => {
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
            Err(e) => {
                console_log!("Warning: Failed to extract translations from {:?}: {}", file, e);
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
        console_log!("{}", error_msg);
        return Err(JsValue::from_str(&error_msg));
    }

    // If no conflicts, proceed with merging
    match message_handler.write_merged_messages(&output_path) {
        Ok(_) => {
            console_log!("Successfully wrote merged messages to {}", output_path.display());
            Ok(())
        }
        Err(e) => {
            console_log!("Failed to write merged messages: {}", e);
            Err(JsValue::from_str(&format!("Failed to write merged messages: {}", e)))
        }
    }
}
