use std::fmt::Display;
use std::sync::Arc;

use super::RedisTopology;

#[derive(Debug, Clone)]
pub struct KeyBuilder {
    meta_namespace: Arc<str>,
    mode: RedisTopology,
}

impl KeyBuilder {
    pub fn new(
        meta_namespace: impl Into<Arc<str>>,
        mode: RedisTopology,
    ) -> Self {
        Self {
            meta_namespace: meta_namespace.into(),
            mode,
        }
    }

    /// Build a key with the given namespace and logical key. The logical key is used as the key's slot tag.
    pub fn entity(&self, namespace: &str, logical_key: impl Display) -> String {
        let logical_key = logical_key.to_string();
        self.with_slot(namespace, &logical_key, &logical_key)
    }

    /// Build a metadata key with the given namespace and logical key. The slot tag is fixed to `_metadata`.
    pub fn metadata(
        &self,
        namespace: &str,
        logical_key: impl Display,
    ) -> String {
        self.with_slot(namespace, logical_key, "_metadata")
    }

    /// Build a key with the given namespace, logical key, and slot tag.
    pub fn with_slot(
        &self,
        namespace: &str,
        logical_key: impl Display,
        slot_tag: impl Display,
    ) -> String {
        match self.mode {
            RedisTopology::Standalone => {
                format!("{}_{}:{}", self.meta_namespace, namespace, logical_key)
            }
            RedisTopology::Cluster => format!(
                "{}_{}:{{{}}}:{}",
                self.meta_namespace,
                namespace,
                escape_slot_tag(&slot_tag.to_string()),
                logical_key
            ),
        }
    }
}

fn escape_slot_tag(value: &str) -> String {
    if value.is_empty() {
        return "%00".to_string();
    }

    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '%' => escaped.push_str("%25"),
            '{' => escaped.push_str("%7B"),
            '}' => escaped.push_str("%7D"),
            _ => escaped.push(character),
        }
    }
    escaped
}
