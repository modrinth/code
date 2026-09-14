use crate::state::content_store::{FileStorageKind, InstanceFileStorage};
use sqlx::{Sqlite, SqlitePool, Transaction};

pub(crate) async fn instance_storage(
    pool: &SqlitePool,
    instance_id: &str,
) -> crate::Result<Vec<InstanceFileStorage>> {
    Ok(sqlx::query_as!(
		InstanceFileStorage,
		r#"SELECT binding.file_id, binding.blob_sha512, binding.materialization_kind AS "storage_kind: _" FROM store_instance_files binding INNER JOIN instance_files file ON file.id = binding.file_id WHERE file.instance_id = ?"#,
		instance_id
	).fetch_all(pool).await?)
}

pub(crate) async fn file_storage(
    pool: &SqlitePool,
    file_id: &str,
) -> crate::Result<Option<InstanceFileStorage>> {
    Ok(sqlx::query_as!(
		InstanceFileStorage,
		r#"SELECT file_id, blob_sha512, materialization_kind AS "storage_kind: _" FROM store_instance_files WHERE file_id = ?"#,
		file_id
	).fetch_optional(pool).await?)
}

pub(crate) async fn set_file_storage(
    tx: &mut Transaction<'_, Sqlite>,
    file_id: &str,
    sha512: &str,
    storage_kind: FileStorageKind,
) -> crate::Result<()> {
    sqlx::query!("INSERT INTO store_instance_files (file_id, blob_sha512, materialization_kind) VALUES (?, ?, ?) ON CONFLICT(file_id) DO UPDATE SET blob_sha512 = excluded.blob_sha512, materialization_kind = excluded.materialization_kind", file_id, sha512, storage_kind as _)
		.execute(&mut **tx).await?;
    Ok(())
}

pub(in crate::state::content_store) struct InstalledStorage {
    pub(in crate::state::content_store) blob_sha512: String,
    pub(in crate::state::content_store) materialization_kind: FileStorageKind,
    pub(in crate::state::content_store) size: i64,
}

pub(in crate::state::content_store) async fn installed_storage(
    pool: &SqlitePool,
) -> crate::Result<Vec<InstalledStorage>> {
    let rows = sqlx::query!(r#"SELECT binding.blob_sha512, binding.materialization_kind AS "materialization_kind: FileStorageKind", file.size FROM store_instance_files binding INNER JOIN instance_files file ON file.id = binding.file_id WHERE file.missing = 0"#).fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|row| InstalledStorage {
            blob_sha512: row.blob_sha512,
            materialization_kind: row.materialization_kind,
            size: row.size,
        })
        .collect())
}

pub(in crate::state::content_store) async fn instances_using_file(
    pool: &SqlitePool,
    sha512: &str,
) -> crate::Result<Vec<String>> {
    Ok(sqlx::query_scalar!("SELECT DISTINCT file.instance_id FROM instance_files file INNER JOIN store_instance_files binding ON binding.file_id = file.id WHERE binding.blob_sha512 = ?", sha512).fetch_all(pool).await?)
}
