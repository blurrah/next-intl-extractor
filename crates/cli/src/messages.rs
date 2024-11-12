use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use serde_json::{Value, Map};

#[derive(Default)]
pub struct MessageMap {
    messages: HashMap<String, Either<String, Box<MessageMap>>>,
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}


pub struct MessageHandler {
    source_messages: Map<String, Value>,
    extracted_messages: MessageMap,
}

impl MessageHandler {
    pub fn new(source_path: &Path) -> Result<Self> {
        let source_messages = load_source_messages(source_path)?;
        Ok(Self {
            source_messages,
            extracted_messages: MessageMap::default(),
        })
    }

    /// Add a new message to the extracted messages
    pub fn add_extracted_message(&mut self, namespace: String, key: String) {
        let parts: Vec<&str> = namespace.split('.').collect();
        let mut current = &mut self.extracted_messages.messages;

        // Navigate through all but the last part
        for &part in parts.iter() {
            current = match current.entry(part.to_string()).or_insert_with(|| {
                Either::Right(Box::new(MessageMap::default()))
            }) {
                Either::Right(map) => &mut map.messages,
                Either::Left(_) => unreachable!("Should never have a string value while traversing"),
            };
        }

        // Insert the final key as a Left value
        current.insert(key, Either::Left(String::new()));
    }

    /// Add a set of extracted messages to the extracted messages
    pub fn add_extracted_messages(&mut self, messages: HashMap<String, HashSet<String>>) {
        for (namespace, keys) in messages {
            for key in keys {
                self.add_extracted_message(namespace.clone(), key);
            }
        }
    }

    pub fn merge_messages(&self) -> Map<String, Value> {
        let mut merged = Map::new();
        self.merge_recursive(&self.extracted_messages, &mut merged, None);
        merged
    }

    fn merge_recursive(&self, message_map: &MessageMap, output: &mut Map<String, Value>, prefix: Option<&str>) {
        for (key, value) in &message_map.messages {
            match value {
                Either::Left(_) => {
                    let full_key = if let Some(p) = prefix {
                        format!("{}.{}", p, key)
                    } else {
                        key.clone()
                    };

                    // Look up in source messages
                    if let Some(source_value) = self.lookup_in_source(&full_key, key) {
                        output.insert(key.clone(), source_value);
                    } else {
                        output.insert(key.clone(), Value::String(full_key));
                    }
                }
                Either::Right(nested) => {
                    let mut nested_map = Map::new();
                    self.merge_recursive(nested, &mut nested_map, Some(key));
                    output.insert(key.clone(), Value::Object(nested_map));
                }
            }
        }
    }

    fn lookup_in_source(&self, full_key: &str, key: &str) -> Option<Value> {
        let parts: Vec<&str> = full_key.split('.').collect();
        let mut current = &self.source_messages;

        for (i, &part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                return current.get(key).cloned();
            }

            current = match current.get(part)?.as_object() {
                Some(obj) => obj,
                None => return None,
            };
        }
        None
    }

    pub fn write_merged_messages(&self, messages: Map<String, Value>, output_path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(&messages)?;
        fs::write(output_path, json)?;
        Ok(())
    }
}

fn load_source_messages(path: &Path) -> Result<Map<String, Value>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read source file: {}", path.display()))?;
    let json: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from: {}", path.display()))?;

    match json {
        Value::Object(map) => Ok(map),
        _ => anyhow::bail!("Source file does not contain a JSON object"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_message_handler() -> MessageHandler {
        let source_messages = json!({
            "namespace1": {
                "key1": "value1",
                "key2": "value2",
                "key3": "value3"
            },
            "namespace2": {
                "key4": "value4",
                "key5": "value5"
            }
        });

        MessageHandler {
            source_messages: source_messages.as_object().unwrap().clone(),
            extracted_messages: MessageMap::default(),
        }
    }

    #[test]
    fn test_merge_messages_with_existing_keys() {
        let mut handler = create_test_message_handler();
        handler.add_extracted_message("namespace1".to_string(), "key1".to_string());
        handler.add_extracted_message("namespace1".to_string(), "key2".to_string());

        let merged = handler.merge_messages();

        assert_eq!(merged.len(), 1);
        let namespace1 = merged.get("namespace1").unwrap().as_object().unwrap();
        assert_eq!(namespace1.len(), 2);
        assert_eq!(namespace1.get("key1").unwrap(), "value1");
        assert_eq!(namespace1.get("key2").unwrap(), "value2");
        assert!(namespace1.get("key3").is_none());
    }

    #[test]
    fn test_merge_messages_with_new_keys() {
        let mut handler = create_test_message_handler();
        handler.add_extracted_message("namespace1".to_string(), "key1".to_string());
        handler.add_extracted_message("namespace1".to_string(), "new_key".to_string());

        let merged = handler.merge_messages();

        assert_eq!(merged.len(), 1);
        let namespace1 = merged.get("namespace1").unwrap().as_object().unwrap();
        assert_eq!(namespace1.len(), 2);
        assert_eq!(namespace1.get("key1").unwrap(), "value1");
        assert_eq!(namespace1.get("new_key").unwrap(), "namespace1.new_key");
    }

    #[test]
    fn test_merge_messages_with_new_namespace() {
        let mut handler = create_test_message_handler();
        handler.add_extracted_message("new_namespace".to_string(), "new_key".to_string());

        let merged = handler.merge_messages();

        assert_eq!(merged.len(), 1);
        let new_namespace = merged.get("new_namespace").unwrap().as_object().unwrap();
        assert_eq!(new_namespace.len(), 1);
        assert_eq!(new_namespace.get("new_key").unwrap(), "new_namespace.new_key");
    }

    #[test]
    fn test_merge_messages_with_multiple_namespaces() {
        let mut handler = create_test_message_handler();
        handler.add_extracted_message("namespace1".to_string(), "key1".to_string());
        handler.add_extracted_message("namespace2".to_string(), "key4".to_string());
        handler.add_extracted_message("namespace2".to_string(), "new_key".to_string());

        let merged = handler.merge_messages();

        assert_eq!(merged.len(), 2);
        let namespace1 = merged.get("namespace1").unwrap().as_object().unwrap();
        assert_eq!(namespace1.len(), 1);
        assert_eq!(namespace1.get("key1").unwrap(), "value1");

        let namespace2 = merged.get("namespace2").unwrap().as_object().unwrap();
        assert_eq!(namespace2.len(), 2);
        assert_eq!(namespace2.get("key4").unwrap(), "value4");
        assert_eq!(namespace2.get("new_key").unwrap(), "namespace2.new_key");
    }
}
