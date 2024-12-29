#!/usr/bin/env node
import { mkdir, readFile, writeFile } from "node:fs/promises";

// Workaround for accessing the filesystem from Rust functions
// by setting up JS hooks it can call directly that use `fs`
// To be honest: not a big fan of this so might stop the wasm approach for CLI's
globalThis.__WASM_HOOKS = {
  async write_file(path, content) {
    try {
      await writeFile(path, content);
      return true;
    } catch (error) {
      console.error("Error writing file:", error);
      return false;
    }
  },

  async read_file(path) {
    try {
      const content = await readFile(path, "utf-8");
      return content;
    } catch (error) {
      if (error.code !== "ENOENT") {
        console.error("Error reading file:", error);
      }
      return null;
    }
  },

  async ensure_dir(path) {
    try {
      await mkdir(path, { recursive: true });
      return true;
    } catch (error) {
      console.error("Error creating directory:", error);
      return false;
    }
  },
};

async function debugWasm() {
  try {
    console.log("Loading Wasm module...");
    const wasmPkg = await import(
      "../crates/cli/pkg/next_intl_extractor_cli.js"
    );

    console.log("Wasm module loaded successfully!");
    console.log("Available exports:", Object.keys(wasmPkg));

    // Set up mock process.argv
    const testArgs = [
      "next-intl-extractor",
      "--output-path",
      "test-output.json",
      "--pattern",
      "**/*.tsx",
    ];

    console.log("\nTesting CLI with arguments:", testArgs);
    try {
      await wasmPkg.run(testArgs);
      console.log("CLI execution completed successfully!");
    } catch (e) {
      console.error("Error running CLI:", e);
      console.error("Error details:", e.stack);
    }
  } catch (error) {
    console.error("Error:", error);
    process.exit(1);
  }
}

debugWasm().catch(console.error);
