# Progress

Just keeping track of progress and my ideas here since I only touch this once every few months

## Ideas

Crates workspace that contains the following:
- **cli** - For setting up the CLI logic such as running and watching files and showing logging/error messages
- **resolver** - For resolving the source files and extracting the messages from them

## CLI
- Resolves the existing source.json file with it's messages
- Run the resolver on the glob pattern given as input by the user
- Merge the messages from the resolver with the existing source.json file
    - If the source.json already has a message for a key, use the existing message
    - If the source.json doesn't have a message for a key, use the key as a message
- Write the merged messages to the output file

If watch mode is enabled:
- Run a watcher on the glob pattern given as input by the user
- When a file is changed, run the resolver on it
- Merge the messages from the resolver with the existing source.json file
    - If the source.json already has a message for a key, use the existing message
    - If the source.json doesn't have a message for a key, use the key as a message
- Write the merged messages to the output file

## Resolver
Uses Oxc to parse the source files and extract messages from the translation functions

TODO:
- Show warning if different files use the same namespace/key
    - Potentially error if this happens behind a flag
