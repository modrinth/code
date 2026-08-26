use sqlx::{Sqlite, SqlitePool, Transaction};

#[derive(Clone, Debug, sqlx::FromRow)]
pub(crate) struct ScreenshotRow {
    pub id: String,
    pub instance_id: String,
    pub file_name: String,
    pub content_hash: String,
    pub file_size: i64,
    pub modified_at: i64,
    pub created_at: i64,
    pub group_id: Option<String>,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub(crate) struct ScreenshotGroupRow {
    pub id: String,
    pub name: String,
}

pub(crate) async fn list_screenshots(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<Vec<ScreenshotRow>> {
    Ok(sqlx::query_as!(
        ScreenshotRow,
        r#"
		SELECT
			screenshots.id,
			screenshots.instance_id,
			screenshots.file_name,
			screenshots.content_hash,
			screenshots.file_size,
			screenshots.modified_at,
			screenshots.created_at,
			memberships.group_id AS "group_id?"
		FROM screenshots
		LEFT JOIN screenshot_group_memberships memberships
			ON memberships.screenshot_id = screenshots.id
		WHERE screenshots.instance_id = ?
		ORDER BY screenshots.file_name, screenshots.id
		"#,
        instance_id,
    )
    .fetch_all(pool)
    .await?)
}

pub(crate) async fn get_screenshot_by_key(
    instance_id: &str,
    file_name: &str,
    pool: &SqlitePool,
) -> crate::Result<Option<ScreenshotRow>> {
    Ok(sqlx::query_as!(
        ScreenshotRow,
        r#"
		SELECT
			screenshots.id,
			screenshots.instance_id,
			screenshots.file_name,
			screenshots.content_hash,
			screenshots.file_size,
			screenshots.modified_at,
			screenshots.created_at,
			memberships.group_id AS "group_id?"
		FROM screenshots
		LEFT JOIN screenshot_group_memberships memberships
			ON memberships.screenshot_id = screenshots.id
		WHERE screenshots.instance_id = ? AND screenshots.file_name = ?
		"#,
        instance_id,
        file_name,
    )
    .fetch_optional(pool)
    .await?)
}

pub(crate) async fn copy_group_membership(
    source_id: &str,
    edited_id: &str,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    sqlx::query!(
        r#"
		INSERT INTO screenshot_group_memberships (screenshot_id, group_id)
		SELECT ?, group_id
		FROM screenshot_group_memberships
		WHERE screenshot_id = ?
		ON CONFLICT (screenshot_id) DO UPDATE SET group_id = excluded.group_id
		"#,
        edited_id,
        source_id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn insert_screenshot(
    row: &ScreenshotRow,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    sqlx::query!(
        "
		INSERT INTO screenshots (
			id,
			instance_id,
			file_name,
			content_hash,
			file_size,
			modified_at,
			created_at
		)
		VALUES (?, ?, ?, ?, ?, ?, ?)
		",
        row.id,
        row.instance_id,
        row.file_name,
        row.content_hash,
        row.file_size,
        row.modified_at,
        row.created_at,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn update_screenshot(
    row: &ScreenshotRow,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    sqlx::query!(
        "
		UPDATE screenshots
		SET
			instance_id = ?,
			file_name = ?,
			content_hash = ?,
			file_size = ?,
			modified_at = ?,
			created_at = ?
		WHERE id = ?
		",
        row.instance_id,
        row.file_name,
        row.content_hash,
        row.file_size,
        row.modified_at,
        row.created_at,
        row.id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn delete_screenshot(
    id: &str,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    sqlx::query!("DELETE FROM screenshots WHERE id = ?", id)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

pub(crate) async fn move_screenshot(
    source_instance_id: &str,
    source_file_name: &str,
    target_instance_id: &str,
    target_file_name: &str,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "DELETE FROM screenshots WHERE instance_id = ? AND file_name = ?",
        target_instance_id,
        target_file_name,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "
		UPDATE screenshots
		SET instance_id = ?, file_name = ?
		WHERE instance_id = ? AND file_name = ?
		",
        target_instance_id,
        target_file_name,
        source_instance_id,
        source_file_name,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

pub(crate) async fn delete_screenshot_by_key(
    instance_id: &str,
    file_name: &str,
    pool: &SqlitePool,
) -> crate::Result<()> {
    sqlx::query!(
        "DELETE FROM screenshots WHERE instance_id = ? AND file_name = ?",
        instance_id,
        file_name,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn list_groups(
    pool: &SqlitePool,
) -> crate::Result<Vec<ScreenshotGroupRow>> {
    Ok(sqlx::query_as!(
        ScreenshotGroupRow,
        "
        SELECT id, name
		FROM screenshot_groups
		ORDER BY display_order, name, id
		",
    )
    .fetch_all(pool)
    .await?)
}
