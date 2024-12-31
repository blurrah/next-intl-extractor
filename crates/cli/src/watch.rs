use anyhow::{Context, Result};
use glob::Pattern;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};

use crate::messages::MessageHandler;
use next_intl_resolver::extract_translations;

fn process_file_change(
    path: &PathBuf,
    message_handler: &mut MessageHandler,
    output_path: &Path,
) -> Result<()> {
    info!("Processing changed file: {:?}", path);

    let translations = extract_translations(path).context("Failed to extract translations")?;

    message_handler.add_extracted_messages(translations, path.to_string_lossy().to_string());
    message_handler.write_merged_messages(output_path)?;
    info!("Successfully updated translations from {:?}", path);
    Ok(())
}

fn process_file_removal(
    path: &PathBuf,
    message_handler: &mut MessageHandler,
    output_path: &Path,
) -> Result<()> {
    info!("Processing removed file: {:?}", path);

    message_handler.remove_messages_for_file(path.to_string_lossy().as_ref());
    message_handler.write_merged_messages(output_path)?;
    info!("Successfully removed translations from {:?}", path);
    Ok(())
}

/// Watch for file changes and update the message handler with new translations
pub fn watch(
    pattern: &str,
    output_path: &Path,
    message_handler: &mut MessageHandler,
) -> Result<()> {
    let glob_pattern = Pattern::new(pattern).context("Failed to create glob pattern")?;
    debug!("Created glob pattern: {:?}", glob_pattern);

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Err(e) = tx.send(res) {
                error!("Error sending watch event: {}", e);
            }
        },
        Config::default(),
    )
    .context("Failed to create file watcher")?;

    // Watch the current directory recursively
    let current_dir = std::env::current_dir()?;
    watcher
        .watch(&current_dir, RecursiveMode::Recursive)
        .context("Failed to start watching directory")?;

    info!("Started watching for file changes in {:?}...", current_dir);

    // Process initial files that match the pattern
    for entry in glob::glob(pattern)?.flatten() {
        if entry.is_file() {
            debug!("Processing initial file: {:?}", entry);
            process_file_change(&entry, message_handler, output_path)?;
        }
    }

    // Write initial state
    message_handler.write_merged_messages(output_path)?;

    for Event { kind, paths, .. } in rx.into_iter().flatten() {
        for path in paths {
            // Convert absolute path to relative path for glob matching
            let relative_path = path.strip_prefix(&current_dir)?;
            if !glob_pattern.matches_path(relative_path) {
                debug!("Skipping file {:?}", relative_path);
                continue;
            }

            let result = match kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    if path.is_file() {
                        debug!("Processing changed file: {:?}", path);
                        process_file_change(&path, message_handler, output_path)
                    } else {
                        Ok(())
                    }
                }
                EventKind::Remove(_) => {
                    debug!("Processing removed file: {:?}", path);
                    process_file_removal(&path, message_handler, output_path)
                }
                _ => Ok(()),
            };

            if let Err(e) = result {
                error!("Error processing file {:?}: {}", path, e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_env() -> Result<(TempDir, PathBuf, MessageHandler)> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("messages.json");

        // Create initial messages.json
        fs::write(&output_path, "{}")?;

        let message_handler = MessageHandler::new(&output_path)?;

        Ok((temp_dir, output_path, message_handler))
    }

    #[test]
    fn test_watch_file_creation() -> Result<()> {
        let (temp_dir, output_path, mut message_handler) = setup_test_env()?;

        // Create a new file
        let test_file = temp_dir.path().join("test.tsx");
        debug!("Creating test file: {:?}", test_file);
        fs::write(
            &test_file,
            r#"
            import { useTranslations } from 'next-intl';

            export function Test() {
                const t = useTranslations('TestNS');
                return <div>{t('hello')}</div>;
            }
        "#,
        )?;

        // Process the file
        process_file_change(&test_file, &mut message_handler, &output_path)?;

        // Verify the messages were extracted
        let messages = fs::read_to_string(&output_path)?;
        debug!("Output file contents: {}", messages);
        assert!(
            messages.contains("TestNS"),
            "Expected to find TestNS in messages"
        );
        assert!(
            messages.contains("hello"),
            "Expected to find 'hello' in messages"
        );

        Ok(())
    }

    #[test]
    fn test_watch_file_modification() -> Result<()> {
        let (temp_dir, output_path, mut message_handler) = setup_test_env()?;

        // Create initial file
        let test_file = temp_dir.path().join("test.tsx");
        debug!("Creating test file: {:?}", test_file);
        fs::write(
            &test_file,
            r#"
            import { useTranslations } from 'next-intl';

            export function Test() {
                const t = useTranslations('TestNS');
                return <div>{t('hello')}</div>;
            }
        "#,
        )?;

        // Process initial file
        process_file_change(&test_file, &mut message_handler, &output_path)?;

        // Verify initial messages
        let messages = fs::read_to_string(&output_path)?;
        debug!("Initial output file contents: {}", messages);
        assert!(
            messages.contains("TestNS"),
            "Expected to find TestNS in messages"
        );
        assert!(
            messages.contains("hello"),
            "Expected to find 'hello' in messages"
        );

        // Modify the file
        debug!("Modifying test file: {:?}", test_file);
        fs::write(
            &test_file,
            r#"
            import { useTranslations } from 'next-intl';

            export function Test() {
                const t = useTranslations('TestNS');
                return <div>{t('hello')} {t('goodbye')}</div>;
            }
        "#,
        )?;

        // Process modified file
        process_file_change(&test_file, &mut message_handler, &output_path)?;

        // Verify the messages were updated
        let messages = fs::read_to_string(&output_path)?;
        debug!("Output file contents after modification: {}", messages);
        assert!(
            messages.contains("TestNS"),
            "Expected to find TestNS in messages"
        );
        assert!(
            messages.contains("hello"),
            "Expected to find 'hello' in messages"
        );
        assert!(
            messages.contains("goodbye"),
            "Expected to find 'goodbye' in messages"
        );

        Ok(())
    }

    #[test]
    fn test_watch_file_deletion() -> Result<()> {
        let (temp_dir, output_path, mut message_handler) = setup_test_env()?;

        // Create initial file
        let test_file = temp_dir.path().join("test.tsx");
        debug!("Creating test file: {:?}", test_file);
        fs::write(
            &test_file,
            r#"
            import { useTranslations } from 'next-intl';

            export function Test() {
                const t = useTranslations('TestNS');
                return <div>{t('hello')}</div>;
            }
        "#,
        )?;

        // Process initial file
        process_file_change(&test_file, &mut message_handler, &output_path)?;

        // Verify initial messages
        let messages = fs::read_to_string(&output_path)?;
        debug!("Initial output file contents: {}", messages);
        assert!(
            messages.contains("TestNS"),
            "Expected to find TestNS in messages"
        );
        assert!(
            messages.contains("hello"),
            "Expected to find 'hello' in messages"
        );

        // Delete the file
        debug!("Deleting test file: {:?}", test_file);
        fs::remove_file(&test_file)?;

        // Process file removal
        process_file_removal(&test_file, &mut message_handler, &output_path)?;

        // Verify the messages were removed
        let messages = fs::read_to_string(&output_path)?;
        debug!("Final output file contents: {}", messages);
        assert!(
            !messages.contains("TestNS"),
            "TestNS should have been removed"
        );
        assert!(
            !messages.contains("hello"),
            "'hello' should have been removed"
        );

        Ok(())
    }

    #[test]
    fn test_watch_pattern_matching() -> Result<()> {
        let (temp_dir, output_path, mut message_handler) = setup_test_env()?;

        // Create a .tsx file (should be processed)
        let tsx_file = temp_dir.path().join("test.tsx");
        debug!("Creating tsx file: {:?}", tsx_file);
        fs::write(
            &tsx_file,
            r#"
            import { useTranslations } from 'next-intl';
            export function Test() {
                const t = useTranslations('TestNS');
                return <div>{t('hello')}</div>;
            }
        "#,
        )?;

        // Create a .ts file (should be ignored)
        let ts_file = temp_dir.path().join("test.ts");
        debug!("Creating ts file: {:?}", ts_file);
        fs::write(
            &ts_file,
            r#"
            import { useTranslations } from 'next-intl';
            export function test() {
                const t = useTranslations('IgnoreNS');
                return t('ignore');
            }
        "#,
        )?;

        // Process both files
        let pattern = Pattern::new("**/*.tsx")?;
        for file in [&tsx_file, &ts_file] {
            if pattern.matches_path(file) {
                process_file_change(file, &mut message_handler, &output_path)?;
            }
        }

        // Verify only tsx messages were processed
        let messages = fs::read_to_string(&output_path)?;
        debug!("Output file contents: {}", messages);
        assert!(
            messages.contains("TestNS"),
            "Expected to find TestNS in messages"
        );
        assert!(
            messages.contains("hello"),
            "Expected to find 'hello' in messages"
        );
        assert!(
            !messages.contains("IgnoreNS"),
            "IgnoreNS should not be present"
        );
        assert!(
            !messages.contains("ignore"),
            "'ignore' should not be present"
        );

        Ok(())
    }
}
