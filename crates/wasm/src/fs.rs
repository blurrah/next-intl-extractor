use anyhow::{Error, Result};
use std::path::Path;
use wasm_bindgen::prelude::*;
use next_intl_extractor_cli::fs::FileSystem;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["globalThis", "__WASM_HOOKS"])]
    fn write_file(path: &str, content: &str) -> bool;

    #[wasm_bindgen(js_namespace = ["globalThis", "__WASM_HOOKS"])]
    fn read_file(path: &str) -> Option<String>;

    #[wasm_bindgen(js_namespace = ["globalThis", "__WASM_HOOKS"])]
    fn ensure_dir(path: &str) -> bool;
}

pub struct WasmFileSystem;

impl FileSystem for WasmFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        read_file(path.to_str().unwrap_or(""))
            .ok_or_else(|| Error::msg("Failed to read file"))
    }

    fn write(&self, path: &Path, contents: &str) -> Result<()> {
        if write_file(path.to_str().unwrap_or(""), contents) {
            Ok(())
        } else {
            Err(Error::msg("Failed to write file"))
        }
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        if ensure_dir(path.to_str().unwrap_or("")) {
            Ok(())
        } else {
            Err(Error::msg("Failed to create directory"))
        }
    }

    fn exists(&self, path: &Path) -> bool {
        read_file(path.to_str().unwrap_or("")).is_some()
    }
}
