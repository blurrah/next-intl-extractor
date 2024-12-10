# next-intl-extractor

> [!CAUTION]
> This is work in progress and does not properly work yet
> This is also a learning project until I get more acquainted with the Rust ecosystem

Few simple CLI tools to help with `next-intl` project.

## Why?

Partly because I could really use these tools but mostly to get acquainted with the Rust ecosystem for creating development tooling within the JS ecosystem.

So naturally the codebase is quite a mess with mixed concepts, will become better as I keep improving on it.

## What should it do once ready?

- Read your component files for `useTranslation` and `getTranslation` usage and extract those labels into a source.json automatically.
- Take the existing source.json labels and use them when available
- Optional: Automatically translate into configured destination languages
- Watch mode with incremental JSON updates
    - Debounce updates when for example git operations are being done
- Warn when invalid values are used (Only `String(String)` and `Object(Map<String, RecursiveReference>)` should be allowed)
    - Show where the invalid value is being used in the source file
- Available on NPM as bin
    - With binary-install for now, possibly napi-rs or wasm-pack for js api
