use anyhow::{Context, Result};
use glob::Pattern;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::messages::MessageHandler;
use next_intl_resolver::extract_translations;

async fn process_file_change(
    // Need to borrow the path as it can be used for error handling outside of this function
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

async fn process_file_removal(
    // Need to borrow the path as it can be used for error handling outside of this function
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
pub async fn watch(
    pattern: &str,
    output_path: &Path,
    message_handler: &mut MessageHandler,
) -> Result<()> {
    let glob_pattern = Pattern::new(pattern).context("Failed to create glob pattern")?;

    let (tx, mut rx) = mpsc::channel(32);

    // Ensure we have a runtime handle for the watcher
    let handle = Handle::current();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let tx = tx.clone();
            handle.spawn(async move {
                if let Err(e) = tx.send(res).await {
                    error!("Error sending watch event: {}", e);
                }
            });
        },
        Config::default(),
    )
    .context("Failed to create file watcher")?;

    watcher
        .watch(Path::new("."), RecursiveMode::Recursive)
        .context("Failed to start watching directory")?;

    info!("Started watching for file changes...");

    while let Some(Ok(Event { kind, paths, .. })) = rx.recv().await {
        for path in paths {
            if !glob_pattern.matches(path.to_string_lossy().as_ref()) {
                info!("Skipping file {:?}", path);
                continue;
            }

            let result = match kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    process_file_change(&path, message_handler, output_path).await
                }
                EventKind::Remove(_) => {
                    process_file_removal(&path, message_handler, output_path).await
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
    use tokio::time::{sleep, Duration};

    async fn setup_test_env() -> Result<(TempDir, PathBuf, MessageHandler)> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("messages.json");

        // Create initial messages.json
        fs::write(&output_path, "{}")?;

        let message_handler = MessageHandler::new(&output_path)?;

        Ok((temp_dir, output_path, message_handler))
    }

    #[tokio::test]
    async fn test_watch_file_creation() -> Result<()> {
        let (temp_dir, output_path, mut message_handler) = setup_test_env().await?;

        // Start watching in the same task since watch is now properly async
        let _watch_path = temp_dir.path().to_path_buf();
        let pattern = "**/*.{ts,tsx}";
        let async_output_path = output_path.clone();

        // Just call watch directly since it's already async
        watch(pattern, &async_output_path, &mut message_handler).await?;

        // Create a new file
        let test_file = temp_dir.path().join("test.tsx");
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

        // Give time for the watcher to process
        sleep(Duration::from_millis(100)).await;

        // Verify the messages were extracted
        let messages = fs::read_to_string(&output_path)?;
        assert!(messages.contains("TestNS"));
        assert!(messages.contains("hello"));

        Ok(())
    }

    #[tokio::test]
    async fn test_watch_file_modification() -> Result<()> {
        let (temp_dir, output_path, mut message_handler) = setup_test_env().await?;

        // Create initial file
        let test_file = temp_dir.path().join("test.tsx");
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

        // Start watching
        let _watch_path = temp_dir.path().to_path_buf();
        let pattern = "**/*.{ts,tsx}";
        let async_output_path = output_path.clone();
        let _watch_handle =
            tokio::spawn(
                async move { watch(pattern, &async_output_path, &mut message_handler).await },
            );

        // Give watcher time to start
        sleep(Duration::from_millis(100)).await;

        // Modify the file
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

        // Give time for the watcher to process
        sleep(Duration::from_millis(100)).await;

        // Verify the messages were updated
        let messages = fs::read_to_string(&output_path)?;
        assert!(messages.contains("TestNS"));
        assert!(messages.contains("hello"));
        assert!(messages.contains("goodbye"));

        Ok(())
    }

    #[tokio::test]
    async fn test_watch_file_deletion() -> Result<()> {
        let (temp_dir, output_path, mut message_handler) = setup_test_env().await?;

        // Create initial file
        let test_file = temp_dir.path().join("test.tsx");
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

        // Start watching
        let _watch_path = temp_dir.path().to_path_buf();
        let pattern = "**/*.{ts,tsx}";
        let async_output_path = output_path.clone();
        let _watch_handle =
            tokio::spawn(
                async move { watch(pattern, &async_output_path, &mut message_handler).await },
            );

        // Give watcher time to start and process initial file
        sleep(Duration::from_millis(100)).await;

        // Verify initial messages
        let messages = fs::read_to_string(&output_path)?;
        assert!(messages.contains("TestNS"));
        assert!(messages.contains("hello"));

        // Delete the file
        fs::remove_file(&test_file)?;

        // Give time for the watcher to process
        sleep(Duration::from_millis(100)).await;

        // Verify the messages were removed
        let messages = fs::read_to_string(&output_path)?;
        assert!(!messages.contains("TestNS"));
        assert!(!messages.contains("hello"));

        Ok(())
    }

    #[tokio::test]
    async fn test_watch_pattern_matching() -> Result<()> {
        let (temp_dir, output_path, mut message_handler) = setup_test_env().await?;

        // Start watching with specific pattern
        let pattern = "**/*.tsx"; // Only watch tsx files
        let async_output_path = output_path.clone();
        let _watch_handle =
            tokio::spawn(
                async move { watch(pattern, &async_output_path, &mut message_handler).await },
            );

        // Give watcher time to start
        sleep(Duration::from_millis(100)).await;

        // Create a .tsx file (should be watched)
        let tsx_file = temp_dir.path().join("test.tsx");
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

        // Give time for the watcher to process
        sleep(Duration::from_millis(100)).await;

        // Verify only tsx messages were processed
        let messages = fs::read_to_string(&output_path)?;
        assert!(messages.contains("TestNS"));
        assert!(messages.contains("hello"));
        assert!(!messages.contains("IgnoreNS"));
        assert!(!messages.contains("ignore"));

        Ok(())
    }
}
