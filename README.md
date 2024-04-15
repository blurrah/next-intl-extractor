# i18n-label-merger

Simple CLI utility that collects i18n label files (denoted as `<Identifier>.labels.json`) and merges them to a single JSON object for translation services like Transifex.

## Why?

Could have done this just as easily in a TypeScript project, as it's a simple and non-heavy task. But doing it in Rust to get more acquainted with it and the ecosystem around CLI applications, file system operations and (de)serialization of JSON.

So naturally the codebase is quite a mess with mixed concepts, will be better as I keep improving it.

## What should it do once ready?

- Take all labels.json files and create one single output json
    - Allow passing working directory and output filename
- Watch mode with incremental JSON updates
    - Debounce updates when for example git operations are being done
- Warn when invalid values are used (Only `String(String)` and `Object(Map<String, RecursiveReference>)` should be allowed)
    - Show where the invalid value is being used in the source file
