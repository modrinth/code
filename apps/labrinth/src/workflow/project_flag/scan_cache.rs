use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::database::PgPool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProjectFlagScanKind {
    Explicit,
    AiGenerated,
    Text,
    Rules,
}

impl ProjectFlagScanKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::AiGenerated => "ai_generated",
            Self::Text => "text",
            Self::Rules => "rules",
        }
    }
}

impl std::fmt::Display for ProjectFlagScanKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Bump when evaluator semantics change without a context/rule/module change.
pub(super) const RULES_CACHE_VERSION: i64 = 5;

/// Canonicalizes every JSON object, including nested maps in rule parameters
/// and context, while preserving array order visible to scripts.
pub(crate) fn rules_input_hash(
    value: &impl serde::Serialize,
) -> Result<blake3::Hash> {
    let mut value =
        serde_json::to_value(value).context("serializing rule cache input")?;
    value.sort_all_objects();
    let bytes =
        serde_json::to_vec(&value).context("encoding rule cache input")?;
    Ok(blake3::hash(&bytes))
}

/// Hashes the content and question identities with an unambiguous boundary.
pub(super) fn input_hash(
    content_key: &str,
    context_hash: &[u8],
) -> blake3::Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(content_key.len() as u64).to_be_bytes());
    hasher.update(content_key.as_bytes());
    hasher.update(context_hash);
    hasher.finalize()
}

pub(crate) async fn load_scan(
    pool: &PgPool,
    scan_kind: ProjectFlagScanKind,
    input_hash: &[u8],
    model_version: i64,
) -> Result<Option<Value>> {
    sqlx::query_scalar!(
        r#"
        SELECT result
        FROM project_flag_scans
        WHERE scan_kind = $1 AND input_hash = $2 AND model_version = $3
        "#,
        scan_kind.as_str(),
        input_hash,
        model_version,
    )
    .fetch_optional(pool)
    .await
    .context("loading project flag scan")
}

pub(super) async fn persist_scan(
    pool: &PgPool,
    scan_kind: ProjectFlagScanKind,
    input_hash: &[u8],
    model_version: i64,
    result: &Value,
) -> Result<Value> {
    sqlx::query!(
        r#"
        INSERT INTO project_flag_scans
            (scan_kind, input_hash, model_version, result)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (scan_kind, input_hash, model_version) DO NOTHING
        "#,
        scan_kind.as_str(),
        input_hash,
        model_version,
        result,
    )
    .execute(pool)
    .await
    .context("persisting project flag scan")?;

    load_scan(pool, scan_kind, input_hash, model_version)
        .await?
        .context("persisted project flag scan is missing")
}

#[cfg(test)]
mod tests {
    use super::{ProjectFlagScanKind, rules_input_hash};
    use serde_json::json;

    #[test]
    fn scan_kind_preserves_stored_names() {
        for (kind, name) in [
            (ProjectFlagScanKind::Explicit, "explicit"),
            (ProjectFlagScanKind::AiGenerated, "ai_generated"),
            (ProjectFlagScanKind::Text, "text"),
            (ProjectFlagScanKind::Rules, "rules"),
        ] {
            assert_eq!(kind.as_str(), name);
            assert_eq!(kind.to_string(), name);
            let value = json!(name);
            assert_eq!(serde_json::to_value(kind).unwrap(), value);
            assert_eq!(
                serde_json::from_value::<ProjectFlagScanKind>(value).unwrap(),
                kind
            );
        }
    }

    #[test]
    fn canonicalizes_nested_object_order() {
        let first: serde_json::Value = serde_json::from_str(
            r#"{"modules":{"b":"two","a":"one"},"params":{"z":1,"a":2}}"#,
        )
        .unwrap();
        let second: serde_json::Value = serde_json::from_str(
            r#"{"params":{"a":2,"z":1},"modules":{"a":"one","b":"two"}}"#,
        )
        .unwrap();
        assert_eq!(
            rules_input_hash(&first).unwrap(),
            rules_input_hash(&second).unwrap()
        );
    }

    #[test]
    fn invalidates_on_each_input_change() {
        let input = json!({
            "context": {"project": {"title": "original"}, "scanned_files": [], "ai": {"decisions_by_rule": {"1": {"question": null}}, "images": []}},
            "rules": {"expression": "[]", "decision_questions": {"q": "original"}},
            "modules": {"helper": "original"},
            "scan_error": null,
        });
        let original = rules_input_hash(&input).unwrap();
        for path in [
            "/context/project/title",
            "/context/scanned_files",
            "/context/ai/decisions_by_rule/1/question",
            "/context/ai/images",
            "/rules/expression",
            "/rules/decision_questions/q",
            "/modules/helper",
            "/scan_error",
        ] {
            let mut changed = input.clone();
            *changed.pointer_mut(path).unwrap() = json!("changed");
            assert_ne!(original, rules_input_hash(&changed).unwrap(), "{path}");
        }
        assert_ne!(
            rules_input_hash(&json!([1, 2])).unwrap(),
            rules_input_hash(&json!([2, 1])).unwrap(),
        );
    }
}
