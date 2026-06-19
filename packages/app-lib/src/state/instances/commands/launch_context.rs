use crate::state::InstanceInstallStage;
use crate::state::instances::{
    InstanceLaunchContext, InstanceLaunchOverrides,
    adapters::sqlite::{content_rows, instance_rows},
};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

pub(crate) async fn get_instance_launch_context(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<Option<InstanceLaunchContext>> {
    let Some(instance) =
        instance_rows::get_instance_by_id(instance_id, pool).await?
    else {
        return Ok(None);
    };

    let applied_content_set =
        content_rows::get_applied_content_set(&instance.id, pool)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Instance {} has no applied content set",
                    instance.id
                ))
            })?;
    let link = instance_rows::get_instance_link(&instance.id, pool).await?;
    let launch_overrides =
        instance_rows::get_instance_launch_overrides(&instance.id, pool)
            .await?
            .unwrap_or_else(|| {
                InstanceLaunchOverrides::empty(instance.id.clone())
            });

    Ok(Some(InstanceLaunchContext {
        instance,
        applied_content_set,
        link,
        launch_overrides,
    }))
}

pub(crate) async fn set_instance_install_stage(
    instance_id: &str,
    install_stage: InstanceInstallStage,
    pool: &SqlitePool,
) -> crate::Result<()> {
    sqlx::query(
        "
		UPDATE instances
		SET install_stage = ?, modified = ?
		WHERE id = ?
		",
    )
    .bind(install_stage.as_str())
    .bind(Utc::now().timestamp())
    .bind(instance_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn set_applied_content_set_loader_version(
    instance_id: &str,
    loader_version: Option<&str>,
    pool: &SqlitePool,
) -> crate::Result<()> {
    sqlx::query(
        "
		UPDATE instance_content_sets
		SET loader_version = ?, modified = ?
		WHERE id = (
			SELECT applied_content_set_id
			FROM instances
			WHERE id = ?
		)
		",
    )
    .bind(loader_version)
    .bind(Utc::now().timestamp())
    .bind(instance_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn set_applied_content_set_protocol_version(
    instance_id: &str,
    protocol_version: Option<u32>,
    pool: &SqlitePool,
) -> crate::Result<()> {
    sqlx::query(
        "
		UPDATE instance_content_sets
		SET protocol_version = ?, modified = ?
		WHERE id = (
			SELECT applied_content_set_id
			FROM instances
			WHERE id = ?
		)
		",
    )
    .bind(protocol_version.map(i64::from))
    .bind(Utc::now().timestamp())
    .bind(instance_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn set_instance_last_played(
    instance_id: &str,
    last_played: DateTime<Utc>,
    pool: &SqlitePool,
) -> crate::Result<()> {
    sqlx::query(
        "
		UPDATE instances
		SET last_played = ?, modified = ?
		WHERE id = ?
		",
    )
    .bind(last_played.timestamp())
    .bind(Utc::now().timestamp())
    .bind(instance_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn add_instance_recent_playtime(
    instance_id: &str,
    seconds: u64,
    pool: &SqlitePool,
) -> crate::Result<()> {
    sqlx::query(
        "
		UPDATE instances
		SET recent_time_played = recent_time_played + ?, modified = ?
		WHERE id = ?
		",
    )
    .bind(seconds as i64)
    .bind(Utc::now().timestamp())
    .bind(instance_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn mark_instance_playtime_submitted(
    instance_id: &str,
    recent_time_played: u64,
    pool: &SqlitePool,
) -> crate::Result<()> {
    sqlx::query(
        "
		UPDATE instances
		SET
			submitted_time_played = submitted_time_played + ?,
			recent_time_played = 0,
			modified = ?
		WHERE id = ?
		",
    )
    .bind(recent_time_played as i64)
    .bind(Utc::now().timestamp())
    .bind(instance_id)
    .execute(pool)
    .await?;

    Ok(())
}
