#!/usr/bin/env node

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
      "node",
      "cli",
      "--output-path",
      "test-output.json",
      "--pattern",
      "**/*.tsx",
    ];

    console.log("\nTesting CLI with arguments:", testArgs.slice(2));
    try {
      await wasmPkg.run(testArgs);
      console.log("CLI execution completed successfully!");
    } catch (e) {
      console.error("Error running CLI:", e.message);
    }
  } catch (error) {
    console.error("Error:", error);
    process.exit(1);
  }
}

debugWasm().catch(console.error);
