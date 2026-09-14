use sqlx::SqlitePool;

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
