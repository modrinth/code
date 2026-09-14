use super::retention_rows::release;
use crate::state::content_store::model::FileChangeJournal;
use sqlx::{Sqlite, SqlitePool, Transaction};

pub(in crate::state::content_store) async fn save_file_change(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
    instance_id: &str,
    journal: &FileChangeJournal,
) -> crate::Result<()> {
    let payload = serde_json::to_string(journal)?;
    sqlx::query!("INSERT INTO store_operations (id, instance_id, payload) VALUES (?, ?, ?)", id, instance_id, payload).execute(&mut **tx).await?;
    Ok(())
}

pub(in crate::state::content_store) async fn pending_file_changes(
    pool: &SqlitePool,
    instance_id: Option<&str>,
) -> crate::Result<Vec<String>> {
    Ok(sqlx::query_scalar!("SELECT payload FROM store_operations WHERE ? IS NULL OR instance_id = ? ORDER BY created_at, id", instance_id, instance_id).fetch_all(pool).await?)
}

pub(in crate::state::content_store) async fn finish_file_change(
    tx: &mut Transaction<'_, Sqlite>,
    id: &str,
) -> crate::Result<()> {
    sqlx::query!("DELETE FROM store_operations WHERE id = ?", id)
        .execute(&mut **tx)
        .await?;
    release(tx, "operation", id).await
}

pub(in crate::state::content_store) fn decode_journal(
    payload: &str,
) -> serde_json::Result<FileChangeJournal> {
    serde_json::from_str(payload)
}
