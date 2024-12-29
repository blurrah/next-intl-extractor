use std::path::PathBuf;
use wasm_bindgen::prelude::*;

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

// Create our own CLI arguments struct that mirrors the one in the CLI
#[derive(Debug)]
struct WasmCliArgs {
    watch: bool,
    output_path: PathBuf,
    pattern: String,
}

impl WasmCliArgs {
    fn parse_from(args: Vec<String>) -> Result<Self, String> {
        let mut watch = false;
        let mut output_path = None;
        let mut pattern = None;

        let mut args_iter = args.iter().skip(1); // Skip the binary name
        while let Some(arg) = args_iter.next() {
            match arg.as_str() {
                "-w" | "--watch" => watch = true,
                "-o" | "--output-path" => {
                    output_path = args_iter.next().map(|s| PathBuf::from(s));
                }
                "-p" | "--pattern" => {
                    pattern = args_iter.next().map(|s| s.to_string());
                }
                _ => {}
            }
        }

        let output_path = output_path.ok_or_else(|| "Missing required argument: --output-path".to_string())?;
        let pattern = pattern.unwrap_or_else(|| "**/*.{tsx,ts}".to_string());

        Ok(WasmCliArgs {
            watch,
            output_path,
            pattern,
        })
    }
}

struct WasmFileSystem;

impl WasmFileSystem {
    fn read_file(path: &std::path::Path) -> std::io::Result<String> {
        read_file(path.to_str().unwrap_or(""))
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))
    }

    fn write_file(path: &std::path::Path, contents: &str) -> std::io::Result<()> {
        if write_file(path.to_str().unwrap_or(""), contents) {
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to write file"))
        }
    }

    fn create_dir_all(path: &std::path::Path) -> std::io::Result<()> {
        if ensure_dir(path.to_str().unwrap_or("")) {
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to create directory"))
        }
    }
}

#[wasm_bindgen]
pub async fn run(args: Vec<String>) -> Result<(), JsValue> {
    console_log!("Starting with args: {:?}", args);

    // Parse arguments using our wasm-specific argument parser
    let args = WasmCliArgs::parse_from(args)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse arguments: {}", e)))?;

    console_log!("Parsed arguments: {:?}", args);

    // Create output directory if it doesn't exist
    if let Some(parent) = args.output_path.parent() {
        WasmFileSystem::create_dir_all(parent)
            .map_err(|e| JsValue::from_str(&format!("Failed to create directory: {}", e)))?;
    }

    // Initialize empty output file if it doesn't exist
    if WasmFileSystem::read_file(&args.output_path).is_err() {
        WasmFileSystem::write_file(&args.output_path, "{}")
            .map_err(|e| JsValue::from_str(&format!("Failed to create output file: {}", e)))?;
    }

    // TODO: Implement the rest of the CLI functionality using WasmFileSystem
    // This will require some refactoring of the CLI code to accept custom file system operations,
    // but we'll do that in a separate PR to keep the changes minimal and focused.

    Ok(())
}
