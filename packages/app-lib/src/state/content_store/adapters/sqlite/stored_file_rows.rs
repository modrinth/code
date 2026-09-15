use crate::state::content_store::StoredFileStatus;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StoredFileMetadata {
    pub sha512: String,
    pub size: i64,
    pub status: StoredFileStatus,
    pub modified_as: i64,
    pub last_used_at: i64,
    pub sources: String,
}

pub(in crate::state::content_store) async fn stored_files(
    pool: &SqlitePool,
) -> crate::Result<Vec<StoredFileMetadata>> {
    Ok(sqlx::query_as!(
		StoredFileMetadata,
		r#"SELECT sha512, size, status AS "status: _", modified_as, last_used_at, sources FROM store_blobs"#
	).fetch_all(pool).await?)
}

pub(crate) async fn find_file(
    pool: &SqlitePool,
    sha512: &str,
) -> crate::Result<Option<StoredFileMetadata>> {
    Ok(sqlx::query_as!(
		StoredFileMetadata,
		r#"SELECT sha512, size, status AS "status: _", modified_as, last_used_at, sources FROM store_blobs WHERE sha512 = ?"#,
		sha512
	).fetch_optional(pool).await?)
}

pub(in crate::state::content_store) async fn known_hashes(
    pool: &SqlitePool,
) -> crate::Result<HashSet<String>> {
    Ok(sqlx::query_scalar!("SELECT sha512 FROM store_blobs")
        .fetch_all(pool)
        .await?
        .into_iter()
        .collect())
}

pub(in crate::state::content_store) async fn cleanup_candidates(
    pool: &SqlitePool,
) -> crate::Result<Vec<StoredFileMetadata>> {
    Ok(sqlx::query_as!(
		StoredFileMetadata,
		r#"SELECT sha512, size, status AS "status: _", modified_as, last_used_at, sources FROM store_blobs WHERE NOT EXISTS (SELECT 1 FROM store_instance_files WHERE blob_sha512 = store_blobs.sha512) AND NOT EXISTS (SELECT 1 FROM store_retained_refs WHERE blob_sha512 = store_blobs.sha512) ORDER BY last_used_at"#
	).fetch_all(pool).await?)
}

pub(in crate::state::content_store) async fn save_file(
    pool: &SqlitePool,
    stored_file: &StoredFileMetadata,
) -> crate::Result<()> {
    sqlx::query!("INSERT INTO store_blobs (sha512, size, status, modified_as, sources) VALUES (?, ?, 'ready', ?, ?) ON CONFLICT(sha512) DO UPDATE SET status = 'ready', modified_as = excluded.modified_as, last_used_at = unixepoch(), verified_at = unixepoch(), sources = CASE WHEN excluded.sources = '[]' THEN store_blobs.sources ELSE excluded.sources END",
		stored_file.sha512, stored_file.size, stored_file.modified_as, stored_file.sources)
		.execute(pool).await?;
    Ok(())
}

pub(in crate::state::content_store) async fn mark_file_used(
    pool: &SqlitePool,
    sha512: &str,
) -> crate::Result<()> {
    sqlx::query!(
        "UPDATE store_blobs SET last_used_at = unixepoch() WHERE sha512 = ?",
        sha512
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(in crate::state::content_store) async fn set_file_status(
    pool: &SqlitePool,
    sha512: &str,
    status: StoredFileStatus,
) -> crate::Result<()> {
    sqlx::query!(
        "UPDATE store_blobs SET status = ? WHERE sha512 = ?",
        status as _,
        sha512
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(in crate::state::content_store) async fn delete_file(
    pool: &SqlitePool,
    sha512: &str,
) -> crate::Result<()> {
    sqlx::query!("DELETE FROM store_blobs WHERE sha512 = ? AND NOT EXISTS (SELECT 1 FROM store_instance_files WHERE blob_sha512 = ?) AND NOT EXISTS (SELECT 1 FROM store_retained_refs WHERE blob_sha512 = ?)", sha512, sha512, sha512).execute(pool).await?;
    Ok(())
}

pub(in crate::state::content_store) fn file_sources(
    stored_file: &StoredFileMetadata,
) -> crate::Result<Vec<String>> {
    Ok(serde_json::from_str(&stored_file.sources)?)
}

pub(in crate::state::content_store) fn encode_sources(
    sources: &[String],
) -> crate::Result<String> {
    Ok(serde_json::to_string(sources)?)
}
