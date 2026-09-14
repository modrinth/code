use sqlx::{Sqlite, SqlitePool, Transaction};

pub(in crate::state::content_store) async fn retain(
    tx: &mut Transaction<'_, Sqlite>,
    kind: &str,
    owner: &str,
    sha512: &str,
) -> crate::Result<()> {
    sqlx::query!("INSERT OR IGNORE INTO store_retained_refs (owner_kind, owner_id, blob_sha512) VALUES (?, ?, ?)", kind, owner, sha512).execute(&mut **tx).await?;
    Ok(())
}

pub(in crate::state::content_store) async fn release(
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

pub(in crate::state::content_store) async fn referenced_files(
    pool: &SqlitePool,
) -> crate::Result<Vec<String>> {
    Ok(sqlx::query_scalar!("SELECT blob_sha512 FROM store_instance_files UNION SELECT blob_sha512 FROM store_retained_refs").fetch_all(pool).await?)
}

pub(crate) async fn retained_owners(
    pool: &SqlitePool,
    kind: &str,
) -> crate::Result<Vec<String>> {
    Ok(sqlx::query_scalar!("SELECT DISTINCT owner_id FROM store_retained_refs WHERE owner_kind = ?", kind).fetch_all(pool).await?)
}
