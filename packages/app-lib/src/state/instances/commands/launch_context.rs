use crate::state::InstanceInstallStage;
use crate::state::instances::{
    InstanceLaunchContext, adapters::sqlite::instance_rows, playtime_to_storage,
};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

pub(crate) async fn get_instance_launch_context(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<Option<InstanceLaunchContext>> {
    instance_rows::get_instance_launch_context(instance_id, pool).await
}

pub(crate) async fn set_instance_install_stage(
    instance_id: &str,
    install_stage: InstanceInstallStage,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let install_stage = install_stage.as_str();
    let modified = Utc::now().timestamp();

    sqlx::query!(
        "
		UPDATE instances
		SET install_stage = ?, modified = ?
		WHERE id = ?
		",
        install_stage,
        modified,
        instance_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn set_applied_content_set_loader_version(
    instance_id: &str,
    loader_version: Option<&str>,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let modified = Utc::now().timestamp();

    sqlx::query!(
        "
		UPDATE instance_content_sets
		SET loader_version = ?, modified = ?
		WHERE id = (
			SELECT applied_content_set_id
			FROM instances
			WHERE id = ?
		)
		",
        loader_version,
        modified,
        instance_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn set_applied_content_set_protocol_version(
    instance_id: &str,
    protocol_version: Option<u32>,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let protocol_version = protocol_version.map(i64::from);
    let modified = Utc::now().timestamp();

    sqlx::query!(
        "
		UPDATE instance_content_sets
		SET protocol_version = ?, modified = ?
		WHERE id = (
			SELECT applied_content_set_id
			FROM instances
			WHERE id = ?
		)
		",
        protocol_version,
        modified,
        instance_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn set_instance_last_played(
    instance_id: &str,
    last_played: DateTime<Utc>,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let last_played = last_played.timestamp();
    let modified = Utc::now().timestamp();

    sqlx::query!(
        "
		UPDATE instances
		SET last_played = ?, modified = ?
		WHERE id = ?
		",
        last_played,
        modified,
        instance_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn add_instance_recent_playtime(
    instance_id: &str,
    seconds: u64,
    pool: &SqlitePool,
) -> crate::Result<()> {
    if seconds == 0 {
        return Ok(());
    }

    let seconds = playtime_to_storage(seconds, "recent_time_played")?;
    let max_playtime = i64::MAX;
    let max_playtime_before_increment = max_playtime - seconds;
    let modified = Utc::now().timestamp();

    sqlx::query!(
        "
		UPDATE instances
		SET
			recent_time_played = CASE
				WHEN recent_time_played < 0 THEN ?
				WHEN recent_time_played > ? THEN ?
				ELSE recent_time_played + ?
			END,
			modified = ?
		WHERE id = ?
		",
        seconds,
        max_playtime_before_increment,
        max_playtime,
        seconds,
        modified,
        instance_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn mark_instance_playtime_submitted(
    instance_id: &str,
    recent_time_played: u64,
    pool: &SqlitePool,
) -> crate::Result<()> {
    if recent_time_played == 0 {
        return Ok(());
    }

    let recent_time_played =
        playtime_to_storage(recent_time_played, "recent_time_played")?;
    let max_playtime = i64::MAX;
    let max_playtime_before_increment = max_playtime - recent_time_played;
    let modified = Utc::now().timestamp();

    sqlx::query!(
        "
		UPDATE instances
		SET
			submitted_time_played = CASE
				WHEN submitted_time_played < 0 THEN ?
				WHEN submitted_time_played > ? THEN ?
				ELSE submitted_time_played + ?
			END,
			recent_time_played = 0,
			modified = ?
		WHERE id = ?
		",
        recent_time_played,
        max_playtime_before_increment,
        max_playtime,
        recent_time_played,
        modified,
        instance_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}
