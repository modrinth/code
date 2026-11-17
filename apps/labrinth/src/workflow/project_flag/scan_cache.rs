use anyhow::{Context, Result};
use serde_json::Value;

use crate::database::PgPool;

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

pub(super) async fn load_scan(
    pool: &PgPool,
    scan_kind: &str,
    input_hash: &[u8],
    model_version: i64,
) -> Result<Option<Value>> {
    sqlx::query_scalar!(
        r#"
        SELECT result
        FROM project_flag_scans
        WHERE scan_kind = $1 AND input_hash = $2 AND model_version = $3
        "#,
        scan_kind,
        input_hash,
        model_version,
    )
    .fetch_optional(pool)
    .await
    .context("loading project flag scan")
}

pub(super) async fn persist_scan(
    pool: &PgPool,
    scan_kind: &str,
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
        scan_kind,
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
