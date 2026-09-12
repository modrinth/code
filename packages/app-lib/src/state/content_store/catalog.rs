use super::{
    InstanceFileKind, InstanceFileStorage, StoredFile, StoredFileStatus,
};
use sqlx::{Sqlite, SqlitePool, Transaction};

pub(super) async fn stored_files(
    pool: &SqlitePool,
) -> crate::Result<Vec<StoredFile>> {
    let rows = sqlx::query!("SELECT sha512, sha1, size, relative_path, status, modified_at_ns, last_used_at, sources FROM store_blobs")
		.fetch_all(pool).await?;
    rows.into_iter()
        .map(|row| {
            Ok(StoredFile {
                sha512: row.sha512,
                sha1: row.sha1,
                size: row.size,
                relative_path: row.relative_path,
                status: StoredFileStatus::from_db(&row.status)?,
                modified_at_ns: row.modified_at_ns,
                last_used_at: row.last_used_at,
                sources: row.sources,
            })
        })
        .collect()
}

pub(super) async fn find_files(
    pool: &SqlitePool,
    sha512: Option<&str>,
    sha1: Option<&str>,
) -> crate::Result<Vec<StoredFile>> {
    let rows = sqlx::query!("SELECT sha512, sha1, size, relative_path, status, modified_at_ns, last_used_at, sources FROM store_blobs WHERE (? IS NOT NULL AND sha512 = ?) OR (? IS NULL AND sha1 = ?)", sha512, sha512, sha512, sha1)
		.fetch_all(pool).await?;
    rows.into_iter()
        .map(|row| {
            Ok(StoredFile {
                sha512: row.sha512,
                sha1: row.sha1,
                size: row.size,
                relative_path: row.relative_path,
                status: StoredFileStatus::from_db(&row.status)?,
                modified_at_ns: row.modified_at_ns,
                last_used_at: row.last_used_at,
                sources: row.sources,
            })
        })
        .collect()
}

pub(super) async fn save_file(
    pool: &SqlitePool,
    stored_file: &StoredFile,
) -> crate::Result<()> {
    sqlx::query!("INSERT INTO store_blobs (sha512, sha1, size, relative_path, status, modified_at_ns, sources) VALUES (?, ?, ?, ?, 'ready', ?, ?) ON CONFLICT(sha512) DO UPDATE SET status = 'ready', modified_at_ns = excluded.modified_at_ns, last_used_at = unixepoch(), verified_at = unixepoch(), sources = CASE WHEN excluded.sources = '[]' THEN store_blobs.sources ELSE excluded.sources END",
		stored_file.sha512, stored_file.sha1, stored_file.size, stored_file.relative_path, stored_file.modified_at_ns, stored_file.sources)
		.execute(pool).await?;
    Ok(())
}

pub(super) async fn mark_file_used(
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

pub(super) async fn set_file_status(
    pool: &SqlitePool,
    sha512: &str,
    status: StoredFileStatus,
) -> crate::Result<()> {
    let status = status.as_str();
    sqlx::query!(
        "UPDATE store_blobs SET status = ? WHERE sha512 = ?",
        status,
        sha512
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn instance_storage(
    pool: &SqlitePool,
    instance_id: &str,
) -> crate::Result<Vec<InstanceFileStorage>> {
    let rows = sqlx::query!("SELECT binding.file_id, binding.blob_sha512, binding.materialization_kind FROM store_instance_files binding INNER JOIN instance_files file ON file.id = binding.file_id WHERE file.instance_id = ?", instance_id)
		.fetch_all(pool).await?;
    rows.into_iter()
        .map(|row| {
            Ok(InstanceFileStorage {
                file_id: row.file_id,
                blob_sha512: row.blob_sha512,
                materialization_kind: InstanceFileKind::from_db(
                    &row.materialization_kind,
                )?,
            })
        })
        .collect()
}

pub(crate) async fn file_storage(
    pool: &SqlitePool,
    file_id: &str,
) -> crate::Result<Option<InstanceFileStorage>> {
    let row = sqlx::query!("SELECT file_id, blob_sha512, materialization_kind FROM store_instance_files WHERE file_id = ?", file_id).fetch_optional(pool).await?;
    row.map(|row| {
        Ok(InstanceFileStorage {
            file_id: row.file_id,
            blob_sha512: row.blob_sha512,
            materialization_kind: InstanceFileKind::from_db(
                &row.materialization_kind,
            )?,
        })
    })
    .transpose()
}

pub(super) async fn set_file_storage(
    tx: &mut Transaction<'_, Sqlite>,
    file_id: &str,
    sha512: &str,
    mode: InstanceFileKind,
) -> crate::Result<()> {
    let mode = mode.as_str();
    sqlx::query!("INSERT INTO store_instance_files (file_id, blob_sha512, materialization_kind) VALUES (?, ?, ?) ON CONFLICT(file_id) DO UPDATE SET blob_sha512 = excluded.blob_sha512, materialization_kind = excluded.materialization_kind", file_id, sha512, mode)
		.execute(&mut **tx).await?;
    Ok(())
}

pub(super) async fn retain(
    tx: &mut Transaction<'_, Sqlite>,
    kind: &str,
    owner: &str,
    sha512: &str,
) -> crate::Result<()> {
    sqlx::query!("INSERT OR IGNORE INTO store_retained_refs (owner_kind, owner_id, blob_sha512) VALUES (?, ?, ?)", kind, owner, sha512).execute(&mut **tx).await?;
    Ok(())
}

pub(super) async fn release(
    tx: &mut Transaction<'_, Sqlite>,
    kind: &str,
    owner: &str,
) -> crate::Result<()> {
    sqlx::query!(
        "DELETE FROM store_retained_refs WHERE owner_kind = ? AND owner_id = ?",
        kind,
        owner
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(super) async fn save_file_change(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    instance_id: &str,
    payload: &str,
) -> crate::Result<()> {
    sqlx::query!("INSERT INTO store_operations (id, instance_id, payload) VALUES (?, ?, ?)", id, instance_id, payload).execute(&mut **tx).await?;
    Ok(())
}

pub(super) async fn pending_file_changes(
    pool: &SqlitePool,
    instance_id: Option<&str>,
) -> crate::Result<Vec<String>> {
    Ok(sqlx::query_scalar!("SELECT payload FROM store_operations WHERE ? IS NULL OR instance_id = ? ORDER BY created_at, id", instance_id, instance_id).fetch_all(pool).await?)
}

pub(super) async fn finish_file_change(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
) -> crate::Result<()> {
    sqlx::query!("DELETE FROM store_operations WHERE id = ?", id)
        .execute(&mut **tx)
        .await?;
    release(tx, "operation", id).await
}

pub(super) async fn referenced_files(
    pool: &SqlitePool,
) -> crate::Result<Vec<String>> {
    Ok(sqlx::query_scalar!("SELECT blob_sha512 FROM store_instance_files UNION SELECT blob_sha512 FROM store_retained_refs").fetch_all(pool).await?)
}

pub(super) async fn delete_file(
    pool: &SqlitePool,
    sha512: &str,
) -> crate::Result<()> {
    sqlx::query!("DELETE FROM store_blobs WHERE sha512 = ? AND NOT EXISTS (SELECT 1 FROM store_instance_files WHERE blob_sha512 = ?) AND NOT EXISTS (SELECT 1 FROM store_retained_refs WHERE blob_sha512 = ?)", sha512, sha512, sha512).execute(pool).await?;
    Ok(())
}

pub(crate) async fn setting(
    pool: &SqlitePool,
    key: &str,
) -> crate::Result<Option<String>> {
    Ok(
        sqlx::query_scalar!(
            "SELECT value FROM app_metadata WHERE key = ?",
            key
        )
        .fetch_optional(pool)
        .await?,
    )
}

pub(crate) async fn set_setting(
    pool: &SqlitePool,
    key: &str,
    value: &str,
) -> crate::Result<()> {
    sqlx::query!("INSERT INTO app_metadata (key, value, updated_at) VALUES (?, ?, unixepoch()) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at", key, value).execute(pool).await?;
    Ok(())
}

pub(crate) async fn retained_owners(
    pool: &SqlitePool,
    kind: &str,
) -> crate::Result<Vec<String>> {
    Ok(sqlx::query_scalar!("SELECT DISTINCT owner_id FROM store_retained_refs WHERE owner_kind = ?", kind).fetch_all(pool).await?)
}
