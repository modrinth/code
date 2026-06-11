#![allow(dead_code)]

use crate::state::instances::{
    Instance, InstanceLaunchOverrides, InstanceLink, InstanceRef,
};
use crate::state::{
    Hooks, LauncherFeatureVersion, MemorySettings, ProfileInstallStage,
    ReleaseChannel, WindowSize,
};
use chrono::{DateTime, TimeZone, Utc};
use serde::de::DeserializeOwned;
use sqlx::{Executor, Sqlite};
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
