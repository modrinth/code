use chrono::Utc;

pub async fn get_hidden_instance_ids(
    user_id: &str,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<Vec<String>> {
    let rows = sqlx::query!(
        "
		SELECT instance_id
		FROM hidden_instances
		WHERE user_id = ?
		ORDER BY hidden_at DESC
		",
        user_id,
    )
    .fetch_all(exec)
    .await?;

    Ok(rows.into_iter().map(|row| row.instance_id).collect())
}

pub async fn set_instance_hidden(
    user_id: &str,
    instance_id: &str,
    hidden: bool,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<()> {
    if hidden {
        hide_instance(user_id, instance_id, exec).await
    } else {
        unhide_instance(user_id, instance_id, exec).await
    }
}

async fn hide_instance(
    user_id: &str,
    instance_id: &str,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<()> {
    let hidden_at = Utc::now().timestamp();

    sqlx::query!(
        "
		INSERT INTO hidden_instances (user_id, instance_id, hidden_at)
		VALUES (?, ?, ?)
		ON CONFLICT (user_id, instance_id) DO NOTHING
		",
        user_id,
        instance_id,
        hidden_at,
    )
    .execute(exec)
    .await?;

    Ok(())
}

async fn unhide_instance(
    user_id: &str,
    instance_id: &str,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<()> {
    sqlx::query!(
        "
		DELETE FROM hidden_instances
		WHERE user_id = ? AND instance_id = ?
		",
        user_id,
        instance_id,
    )
    .execute(exec)
    .await?;

    Ok(())
}

pub async fn forget_instance(
    instance_id: &str,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<()> {
    sqlx::query!(
        "
		DELETE FROM hidden_instances
		WHERE instance_id = ?
		",
        instance_id,
    )
    .execute(exec)
    .await?;

    Ok(())
}
