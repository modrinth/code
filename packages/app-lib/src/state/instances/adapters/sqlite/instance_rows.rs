#![allow(dead_code)]

use crate::state::instances::{
    ContentSetStatus, ContentSourceKind, Instance, InstanceLaunchOverrides,
    InstanceLink, InstanceRef, legacy_default_content_set_id,
    legacy_instance_id,
};
use crate::state::{
    Hooks, LauncherFeatureVersion, MemorySettings, Profile,
    ProfileInstallStage, ReleaseChannel, WindowSize,
};
use chrono::{DateTime, TimeZone, Utc};
use serde::de::DeserializeOwned;
use sqlx::{Executor, Sqlite, SqlitePool};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct InstanceRow {
    pub id: String,
    pub path: String,
    pub applied_content_set_id: Option<String>,
    pub install_stage: String,
    pub launcher_feature_version: String,
    pub update_channel: String,
    pub name: String,
    pub icon_path: Option<String>,
    pub created: i64,
    pub modified: i64,
    pub last_played: Option<i64>,
    pub submitted_time_played: i64,
    pub recent_time_played: i64,
}

impl TryFrom<InstanceRow> for Instance {
    type Error = crate::Error;

    fn try_from(row: InstanceRow) -> crate::Result<Self> {
        Ok(Self {
            id: row.id,
            path: row.path,
            applied_content_set_id: row.applied_content_set_id,
            install_stage: ProfileInstallStage::from_str(&row.install_stage),
            launcher_feature_version: LauncherFeatureVersion::from_str(
                &row.launcher_feature_version,
            ),
            update_channel: ReleaseChannel::from_key(&row.update_channel),
            name: row.name,
            icon_path: row.icon_path,
            created: timestamp(row.created),
            modified: timestamp(row.modified),
            last_played: row.last_played.and_then(optional_timestamp),
            submitted_time_played: unsigned(
                row.submitted_time_played,
                "submitted_time_played",
            )?,
            recent_time_played: unsigned(
                row.recent_time_played,
                "recent_time_played",
            )?,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct InstanceLinkRow {
    pub instance_id: String,
    pub link_kind: String,
    pub modrinth_project_id: Option<String>,
    pub modrinth_version_id: Option<String>,
    pub server_project_id: Option<String>,
    pub content_project_id: Option<String>,
    pub content_version_id: Option<String>,
    pub hosting_server_id: Option<String>,
    pub hosting_instance_ids: Option<String>,
    pub hosting_active_instance_id: Option<String>,
    pub shared_instance_id: Option<String>,
}

impl TryFrom<InstanceLinkRow> for InstanceLink {
    type Error = crate::Error;

    fn try_from(row: InstanceLinkRow) -> crate::Result<Self> {
        match row.link_kind.as_str() {
            "unmanaged" => Ok(Self::Unmanaged),
            "modrinth_modpack" => Ok(Self::ModrinthModpack {
                project_id: required(
                    row.modrinth_project_id,
                    "modrinth_project_id",
                )?,
                version_id: required(
                    row.modrinth_version_id,
                    "modrinth_version_id",
                )?,
            }),
            "server_project" => Ok(Self::ServerProject {
                project_id: required(
                    row.server_project_id,
                    "server_project_id",
                )?,
            }),
            "server_project_modpack" => Ok(Self::ServerProjectModpack {
                server_project_id: required(
                    row.server_project_id,
                    "server_project_id",
                )?,
                content_project_id: required(
                    row.content_project_id,
                    "content_project_id",
                )?,
                content_version_id: required(
                    row.content_version_id,
                    "content_version_id",
                )?,
            }),
            "modrinth_hosting" => Ok(Self::ModrinthHosting {
                server_id: parse_uuid(
                    row.hosting_server_id,
                    "hosting_server_id",
                )?,
                instance_ids: parse_optional_json(
                    row.hosting_instance_ids,
                    "hosting_instance_ids",
                )?
                .unwrap_or_default(),
                active_instance_id: parse_optional_uuid(
                    row.hosting_active_instance_id,
                    "hosting_active_instance_id",
                )?,
            }),
            "imported_modpack" => Ok(Self::ImportedModpack {
                project_id: row.modrinth_project_id,
                version_id: row.modrinth_version_id,
            }),
            "shared_instance" => Ok(Self::SharedInstance {
                shared_instance_id: parse_uuid(
                    row.shared_instance_id,
                    "shared_instance_id",
                )?,
            }),
            other => Err(crate::ErrorKind::InputError(format!(
                "Unknown instance link kind {other}"
            ))
            .into()),
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct InstanceLaunchOverridesRow {
    pub instance_id: String,
    pub java_path: Option<String>,
    pub extra_launch_args: Option<String>,
    pub custom_env_vars: Option<String>,
    pub memory: Option<i64>,
    pub force_fullscreen: Option<i64>,
    pub game_resolution_x: Option<i64>,
    pub game_resolution_y: Option<i64>,
    pub hook_pre_launch: Option<String>,
    pub hook_wrapper: Option<String>,
    pub hook_post_exit: Option<String>,
}

impl TryFrom<InstanceLaunchOverridesRow> for InstanceLaunchOverrides {
    type Error = crate::Error;

    fn try_from(row: InstanceLaunchOverridesRow) -> crate::Result<Self> {
        Ok(Self {
            instance_id: row.instance_id,
            java_path: row.java_path,
            extra_launch_args: parse_optional_json(
                row.extra_launch_args,
                "extra_launch_args",
            )?,
            custom_env_vars: parse_optional_json(
                row.custom_env_vars,
                "custom_env_vars",
            )?,
            memory: match row.memory {
                Some(maximum) => Some(MemorySettings {
                    maximum: unsigned(maximum, "memory")? as u32,
                }),
                None => None,
            },
            force_fullscreen: row.force_fullscreen.map(|value| value == 1),
            game_resolution: match (
                row.game_resolution_x,
                row.game_resolution_y,
            ) {
                (Some(x), Some(y)) => Some(WindowSize(
                    unsigned(x, "game_resolution_x")? as u16,
                    unsigned(y, "game_resolution_y")? as u16,
                )),
                _ => None,
            },
            hooks: Hooks {
                pre_launch: row.hook_pre_launch,
                wrapper: row.hook_wrapper,
                post_exit: row.hook_post_exit,
            },
        })
    }
}

pub(crate) async fn get_instance_by_id<'e, E>(
    id: &str,
    exec: E,
) -> crate::Result<Option<Instance>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query_as::<_, InstanceRow>(
        "
		SELECT *
		FROM instances
		WHERE id = ?
		",
    )
    .bind(id)
    .fetch_optional(exec)
    .await?;

    row.map(TryInto::try_into).transpose()
}

pub(crate) async fn get_instance_by_path<'e, E>(
    path: &str,
    exec: E,
) -> crate::Result<Option<Instance>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query_as::<_, InstanceRow>(
        "
		SELECT *
		FROM instances
		WHERE path = ?
		",
    )
    .bind(path)
    .fetch_optional(exec)
    .await?;

    row.map(TryInto::try_into).transpose()
}

pub(crate) async fn list_instances(
    pool: &SqlitePool,
) -> crate::Result<Vec<Instance>> {
    let rows = sqlx::query_as::<_, InstanceRow>(
        "
		SELECT *
		FROM instances
		",
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub(crate) async fn resolve_instance<'e, E>(
    instance: InstanceRef<'_>,
    exec: E,
) -> crate::Result<Option<Instance>>
where
    E: Executor<'e, Database = Sqlite>,
{
    match instance {
        InstanceRef::Id(id) => get_instance_by_id(id, exec).await,
        InstanceRef::Path(path) => get_instance_by_path(path, exec).await,
    }
}

pub(crate) async fn get_instance_link<'e, E>(
    instance_id: &str,
    exec: E,
) -> crate::Result<InstanceLink>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query_as::<_, InstanceLinkRow>(
        "
        SELECT
            instance_id,
            link_kind,
            modrinth_project_id,
            modrinth_version_id,
            server_project_id,
            content_project_id,
            content_version_id,
            hosting_server_id,
            json(hosting_instance_ids) AS hosting_instance_ids,
            hosting_active_instance_id,
            shared_instance_id
        FROM instance_links
        WHERE instance_id = ?
        ",
    )
    .bind(instance_id)
    .fetch_optional(exec)
    .await?;

    match row {
        Some(row) => row.try_into(),
        None => Ok(InstanceLink::Unmanaged),
    }
}

pub(crate) async fn get_instance_groups<'e, E>(
    instance_id: &str,
    exec: E,
) -> crate::Result<Vec<String>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let rows = sqlx::query_scalar::<_, String>(
        "
		SELECT group_name
		FROM instance_groups
		WHERE instance_id = ?
		ORDER BY group_name
		",
    )
    .bind(instance_id)
    .fetch_all(exec)
    .await?;

    Ok(rows)
}

pub(crate) async fn get_instance_launch_overrides<'e, E>(
    instance_id: &str,
    exec: E,
) -> crate::Result<Option<InstanceLaunchOverrides>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query_as::<_, InstanceLaunchOverridesRow>(
        "
		SELECT
			instance_id,
			java_path,
			json(extra_launch_args) AS extra_launch_args,
			json(custom_env_vars) AS custom_env_vars,
			memory,
			force_fullscreen,
			game_resolution_x,
			game_resolution_y,
			hook_pre_launch,
			hook_wrapper,
			hook_post_exit
		FROM instance_launch_overrides
		WHERE instance_id = ?
		",
    )
    .bind(instance_id)
    .fetch_optional(exec)
    .await?;

    row.map(TryInto::try_into).transpose()
}

pub(crate) async fn upsert_instance_from_profile(
    profile: &Profile,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let mut tx = pool.begin().await?;

    let instance_id = legacy_instance_id(&profile.path);
    let content_set_id = legacy_default_content_set_id(&profile.path);
    let icon_path = profile.icon_path.as_deref();
    let loader_version = profile.loader_version.as_deref();

    let install_stage = profile.install_stage.as_str();
    let launcher_feature_version = profile.launcher_feature_version.as_str();
    let update_channel = profile.preferred_update_channel.key();
    let created = profile.created.timestamp();
    let modified = profile.modified.timestamp();
    let last_played = profile.last_played.map(|value| value.timestamp());
    let submitted_time_played = profile.submitted_time_played as i64;
    let recent_time_played = profile.recent_time_played as i64;

    sqlx::query(
        "
		INSERT INTO instances (
			id,
			path,
			applied_content_set_id,
			install_stage,
			launcher_feature_version,
			update_channel,
			name,
			icon_path,
			created,
			modified,
			last_played,
			submitted_time_played,
			recent_time_played
		)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
		ON CONFLICT (id) DO UPDATE SET
			path = excluded.path,
			applied_content_set_id = excluded.applied_content_set_id,
			install_stage = excluded.install_stage,
			launcher_feature_version = excluded.launcher_feature_version,
			update_channel = excluded.update_channel,
			name = excluded.name,
			icon_path = excluded.icon_path,
			created = excluded.created,
			modified = excluded.modified,
			last_played = excluded.last_played,
			submitted_time_played = excluded.submitted_time_played,
			recent_time_played = excluded.recent_time_played
		",
    )
    .bind(&instance_id)
    .bind(profile.path.as_str())
    .bind(&content_set_id)
    .bind(install_stage)
    .bind(launcher_feature_version)
    .bind(update_channel)
    .bind(profile.name.as_str())
    .bind(icon_path)
    .bind(created)
    .bind(modified)
    .bind(last_played)
    .bind(submitted_time_played)
    .bind(recent_time_played)
    .execute(&mut *tx)
    .await?;

    let source_kind = profile_content_source_kind(profile).as_str();
    let content_set_status = ContentSetStatus::Available.as_str();
    let loader = profile.loader.as_str();
    let protocol_version = profile.protocol_version.map(|value| value as i64);

    sqlx::query(
        "
		INSERT INTO instance_content_sets (
			id,
			instance_id,
			name,
			source_kind,
			status,
			game_version,
			protocol_version,
			loader,
			loader_version,
			created,
			modified
		)
		VALUES (?, ?, 'Default', ?, ?, ?, ?, ?, ?, ?, ?)
		ON CONFLICT (id) DO UPDATE SET
			instance_id = excluded.instance_id,
			source_kind = excluded.source_kind,
			status = excluded.status,
			game_version = excluded.game_version,
			protocol_version = excluded.protocol_version,
			loader = excluded.loader,
			loader_version = excluded.loader_version,
			modified = excluded.modified
		",
    )
    .bind(&content_set_id)
    .bind(&instance_id)
    .bind(source_kind)
    .bind(content_set_status)
    .bind(profile.game_version.as_str())
    .bind(protocol_version)
    .bind(loader)
    .bind(loader_version)
    .bind(created)
    .bind(modified)
    .execute(&mut *tx)
    .await?;

    let (
        link_kind,
        modrinth_project_id,
        modrinth_version_id,
        server_project_id,
    ) = profile_link_columns(profile);

    sqlx::query(
        "
		INSERT INTO instance_links (
			instance_id,
			link_kind,
			modrinth_project_id,
			modrinth_version_id,
			server_project_id,
			content_project_id,
			content_version_id,
			hosting_server_id,
			hosting_instance_ids,
			hosting_active_instance_id,
			shared_instance_id
		)
		VALUES (?, ?, ?, ?, ?, NULL, NULL, NULL, NULL, NULL, NULL)
		ON CONFLICT (instance_id) DO UPDATE SET
			link_kind = excluded.link_kind,
			modrinth_project_id = excluded.modrinth_project_id,
			modrinth_version_id = excluded.modrinth_version_id,
			server_project_id = excluded.server_project_id,
			content_project_id = excluded.content_project_id,
			content_version_id = excluded.content_version_id,
			hosting_server_id = excluded.hosting_server_id,
			hosting_instance_ids = excluded.hosting_instance_ids,
			hosting_active_instance_id = excluded.hosting_active_instance_id,
			shared_instance_id = excluded.shared_instance_id
		",
    )
    .bind(&instance_id)
    .bind(link_kind)
    .bind(modrinth_project_id)
    .bind(modrinth_version_id)
    .bind(server_project_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "
		DELETE FROM instance_groups
		WHERE instance_id = ?
		",
    )
    .bind(&instance_id)
    .execute(&mut *tx)
    .await?;

    for group in &profile.groups {
        sqlx::query(
            "
			INSERT OR IGNORE INTO instance_groups (instance_id, group_name)
			VALUES (?, ?)
			",
        )
        .bind(&instance_id)
        .bind(group.as_str())
        .execute(&mut *tx)
        .await?;
    }

    let extra_launch_args = profile
        .extra_launch_args
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    let custom_env_vars = profile
        .custom_env_vars
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    let memory = profile.memory.map(|value| value.maximum as i64);
    let force_fullscreen = profile
        .force_fullscreen
        .map(|value| if value { 1_i64 } else { 0_i64 });
    let game_resolution_x =
        profile.game_resolution.map(|value| value.0 as i64);
    let game_resolution_y =
        profile.game_resolution.map(|value| value.1 as i64);

    sqlx::query(
        "
		INSERT INTO instance_launch_overrides (
			instance_id,
			java_path,
			extra_launch_args,
			custom_env_vars,
			memory,
			force_fullscreen,
			game_resolution_x,
			game_resolution_y,
			hook_pre_launch,
			hook_wrapper,
			hook_post_exit
		)
		VALUES (?, ?, jsonb(?), jsonb(?), ?, ?, ?, ?, ?, ?, ?)
		ON CONFLICT (instance_id) DO UPDATE SET
			java_path = excluded.java_path,
			extra_launch_args = excluded.extra_launch_args,
			custom_env_vars = excluded.custom_env_vars,
			memory = excluded.memory,
			force_fullscreen = excluded.force_fullscreen,
			game_resolution_x = excluded.game_resolution_x,
			game_resolution_y = excluded.game_resolution_y,
			hook_pre_launch = excluded.hook_pre_launch,
			hook_wrapper = excluded.hook_wrapper,
			hook_post_exit = excluded.hook_post_exit
		",
    )
    .bind(&instance_id)
    .bind(profile.java_path.as_deref())
    .bind(extra_launch_args)
    .bind(custom_env_vars)
    .bind(memory)
    .bind(force_fullscreen)
    .bind(game_resolution_x)
    .bind(game_resolution_y)
    .bind(profile.hooks.pre_launch.as_deref())
    .bind(profile.hooks.wrapper.as_deref())
    .bind(profile.hooks.post_exit.as_deref())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

pub(crate) async fn delete_instance_by_path(
    path: &str,
    pool: &SqlitePool,
) -> crate::Result<()> {
    sqlx::query(
        "
		DELETE FROM instances
		WHERE path = ?
		",
    )
    .bind(path)
    .execute(pool)
    .await?;

    Ok(())
}

fn required(value: Option<String>, column: &str) -> crate::Result<String> {
    value.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Missing required instance link column {column}"
        ))
        .into()
    })
}

fn parse_uuid(value: Option<String>, column: &str) -> crate::Result<Uuid> {
    let value = required(value, column)?;

    value.parse().map_err(|err| {
        crate::ErrorKind::InputError(format!("Invalid {column}: {err}")).into()
    })
}

fn parse_optional_uuid(
    value: Option<String>,
    column: &str,
) -> crate::Result<Option<Uuid>> {
    value
        .map(|value| {
            value.parse().map_err(|err| {
                crate::ErrorKind::InputError(format!("Invalid {column}: {err}"))
                    .into()
            })
        })
        .transpose()
}

fn parse_optional_json<T>(
    value: Option<String>,
    column: &str,
) -> crate::Result<Option<T>>
where
    T: DeserializeOwned,
{
    let Some(value) = value else {
        return Ok(None);
    };

    if value == "null" {
        return Ok(None);
    }

    serde_json::from_str(&value).map(Some).map_err(|err| {
        crate::ErrorKind::InputError(format!(
            "Invalid launch override JSON in {column}: {err}"
        ))
        .into()
    })
}

fn timestamp(value: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(value, 0)
        .single()
        .unwrap_or_else(Utc::now)
}

fn optional_timestamp(value: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_opt(value, 0).single()
}

fn unsigned(value: i64, column: &str) -> crate::Result<u64> {
    if value < 0 {
        return Err(crate::ErrorKind::InputError(format!(
            "Expected {column} to be non-negative"
        ))
        .into());
    }

    Ok(value as u64)
}

fn profile_content_source_kind(profile: &Profile) -> ContentSourceKind {
    match &profile.linked_data {
        Some(linked_data) if linked_data.version_id.is_empty() => {
            ContentSourceKind::ServerProject
        }
        Some(_) => ContentSourceKind::ModrinthModpack,
        None => ContentSourceKind::Local,
    }
}

fn profile_link_columns(
    profile: &Profile,
) -> (
    &'static str,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    match &profile.linked_data {
        Some(linked_data) if linked_data.version_id.is_empty() => (
            "server_project",
            None,
            None,
            Some(linked_data.project_id.clone()),
        ),
        Some(linked_data) => (
            "modrinth_modpack",
            Some(linked_data.project_id.clone()),
            Some(linked_data.version_id.clone()),
            None,
        ),
        None => ("unmanaged", None, None, None),
    }
}
