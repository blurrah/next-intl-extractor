use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use serde_json::{Value, Map};

pub struct MessageHandler {
    source_messages: Map<String, Value>,
    extracted_messages: HashMap<String, HashMap<String, String>>,
}

impl MessageHandler {
    pub fn new(source_path: &Path) -> Result<Self> {
        let source_messages = load_source_messages(source_path)?;
        Ok(Self {
            source_messages,
            extracted_messages: HashMap::new(),
        })
    }

    pub fn add_extracted_message(&mut self, namespace: String, key: String) {
        self.extracted_messages
            .entry(namespace)
            .or_default()
            .entry(key)
            .or_default();
    }

    pub fn merge_messages(&self) -> Map<String, Value> {
        // Start with an empty map for the merged messages
        let mut merged = Map::new();

        // Iterate through all namespaces and messages in extracted_messages
        for (namespace, messages) in &self.extracted_messages {
            // Create a new object for this namespace
            let mut namespace_obj = Map::new();

            // Check if this namespace exists in the source messages
            if let Some(Value::Object(source_namespace)) = self.source_messages.get(namespace) {
                // Iterate through all keys in the extracted messages for this namespace
                for key in messages.keys() {
                    if let Some(value) = source_namespace.get(key) {
                        // If the key exists in source, use its value
                        namespace_obj.insert(key.clone(), value.clone());
                    } else {
                        // If the key doesn't exist in source, create a placeholder
                        namespace_obj.insert(key.clone(), Value::String(format!("{}.{}", namespace, key)));
                    }
                }
            } else {
                // If the namespace doesn't exist in source, create placeholders for all keys
                for key in messages.keys() {
                    namespace_obj.insert(key.clone(), Value::String(format!("{}.{}", namespace, key)));
                }
            }

            // Add the namespace object to the merged map
            merged.insert(namespace.clone(), Value::Object(namespace_obj));
        }

        merged
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
            extracted_messages: HashMap::new(),
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
