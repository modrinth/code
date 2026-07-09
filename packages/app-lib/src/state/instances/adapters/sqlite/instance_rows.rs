#![allow(dead_code)]

use crate::state::instances::{
    ContentSet, ContentSetStatus, ContentSetSyncStatus, ContentSourceKind,
    Instance, InstanceLaunchContext, InstanceLaunchOverrides,
    InstanceLaunchOverridesData, InstanceLink, SharedInstanceAttachment,
    SharedInstanceRole, playtime_to_storage,
};
use crate::state::{
    InstanceInstallStage, LauncherFeatureVersion, ModLoader, ReleaseChannel,
};
use chrono::{DateTime, TimeZone, Utc};
use serde::de::DeserializeOwned;
use sqlx::{Executor, Sqlite, SqlitePool, Transaction};
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
            install_stage: InstanceInstallStage::from_str(&row.install_stage),
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
    pub shared_instance_role: Option<String>,
    pub shared_instance_manager_id: Option<String>,
    pub shared_instance_linked_user_id: Option<String>,
    pub imported_name: Option<String>,
    pub imported_version_number: Option<String>,
    pub imported_filename: Option<String>,
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
                name: row.imported_name,
                version_number: row.imported_version_number,
                filename: row.imported_filename,
            }),
            "shared_instance" => Ok(Self::SharedInstance {
                modpack_project_id: row.modrinth_project_id,
                modpack_version_id: row.modrinth_version_id,
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
    pub overrides: String,
}

#[derive(Debug)]
pub(crate) struct InstanceMetadataRecord {
    pub instance: Instance,
    pub applied_content_set: ContentSet,
    pub link: InstanceLink,
    pub shared_instance: Option<SharedInstanceAttachment>,
    pub groups: Vec<String>,
    pub launch_overrides: InstanceLaunchOverrides,
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct InstanceDisplayInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, sqlx::FromRow)]
struct InstanceMetadataRow {
    id: String,
    path: String,
    applied_content_set_id: Option<String>,
    install_stage: String,
    launcher_feature_version: String,
    update_channel: String,
    name: String,
    icon_path: Option<String>,
    created: i64,
    modified: i64,
    last_played: Option<i64>,
    submitted_time_played: i64,
    recent_time_played: i64,
    content_set_id: Option<String>,
    content_set_instance_id: Option<String>,
    content_set_name: Option<String>,
    content_set_source_kind: Option<String>,
    content_set_status: Option<String>,
    content_set_game_version: Option<String>,
    content_set_protocol_version: Option<i64>,
    content_set_loader: Option<String>,
    content_set_loader_version: Option<String>,
    content_set_created: Option<i64>,
    content_set_modified: Option<i64>,
    link_kind: String,
    modrinth_project_id: Option<String>,
    modrinth_version_id: Option<String>,
    server_project_id: Option<String>,
    content_project_id: Option<String>,
    content_version_id: Option<String>,
    hosting_server_id: Option<String>,
    hosting_instance_ids: Option<String>,
    hosting_active_instance_id: Option<String>,
    shared_instance_id: Option<String>,
    shared_instance_role: Option<String>,
    shared_instance_manager_id: Option<String>,
    shared_instance_linked_user_id: Option<String>,
    shared_sync_applied_update_id: Option<String>,
    shared_sync_latest_available_update_id: Option<String>,
    shared_sync_status: Option<String>,
    imported_name: Option<String>,
    imported_version_number: Option<String>,
    imported_filename: Option<String>,
    groups: String,
    launch_overrides: Option<String>,
}

impl TryFrom<InstanceLaunchOverridesRow> for InstanceLaunchOverrides {
    type Error = crate::Error;

    fn try_from(row: InstanceLaunchOverridesRow) -> crate::Result<Self> {
        let data =
            serde_json::from_str::<InstanceLaunchOverridesData>(&row.overrides)
                .map_err(|err| {
                    crate::ErrorKind::InputError(format!(
                        "Invalid launch overrides JSON: {err}"
                    ))
                    .as_error()
                })?;

        Ok(data.into_launch_overrides(row.instance_id))
    }
}

impl InstanceMetadataRow {
    fn into_record(self) -> crate::Result<InstanceMetadataRecord> {
        let instance_id = self.id.clone();
        let instance = InstanceRow {
            id: self.id,
            path: self.path,
            applied_content_set_id: self.applied_content_set_id,
            install_stage: self.install_stage,
            launcher_feature_version: self.launcher_feature_version,
            update_channel: self.update_channel,
            name: self.name,
            icon_path: self.icon_path,
            created: self.created,
            modified: self.modified,
            last_played: self.last_played,
            submitted_time_played: self.submitted_time_played,
            recent_time_played: self.recent_time_played,
        }
        .try_into()?;
        let applied_content_set = ContentSet {
            id: required(self.content_set_id, "instance_content_sets.id")?,
            instance_id: required(
                self.content_set_instance_id,
                "instance_content_sets.instance_id",
            )?,
            name: required(
                self.content_set_name,
                "instance_content_sets.name",
            )?,
            source_kind: ContentSourceKind::from_str(&required(
                self.content_set_source_kind,
                "instance_content_sets.source_kind",
            )?)?,
            status: ContentSetStatus::from_str(&required(
                self.content_set_status,
                "instance_content_sets.status",
            )?)?,
            game_version: required(
                self.content_set_game_version,
                "instance_content_sets.game_version",
            )?,
            protocol_version: self
                .content_set_protocol_version
                .map(|value| value as u32),
            loader: ModLoader::from_string(&required(
                self.content_set_loader,
                "instance_content_sets.loader",
            )?),
            loader_version: self.content_set_loader_version,
            created: timestamp(required_i64(
                self.content_set_created,
                "instance_content_sets.created",
            )?),
            modified: timestamp(required_i64(
                self.content_set_modified,
                "instance_content_sets.modified",
            )?),
        };
        let link = InstanceLinkRow {
            instance_id: instance_id.clone(),
            link_kind: self.link_kind,
            modrinth_project_id: self.modrinth_project_id,
            modrinth_version_id: self.modrinth_version_id,
            server_project_id: self.server_project_id,
            content_project_id: self.content_project_id,
            content_version_id: self.content_version_id,
            hosting_server_id: self.hosting_server_id,
            hosting_instance_ids: self.hosting_instance_ids,
            hosting_active_instance_id: self.hosting_active_instance_id,
            shared_instance_id: self.shared_instance_id.clone(),
            shared_instance_role: self.shared_instance_role.clone(),
            shared_instance_manager_id: self.shared_instance_manager_id.clone(),
            shared_instance_linked_user_id: self
                .shared_instance_linked_user_id
                .clone(),
            imported_name: self.imported_name,
            imported_version_number: self.imported_version_number,
            imported_filename: self.imported_filename,
        }
        .try_into()?;
        let shared_instance = shared_instance_attachment(
            self.shared_instance_id,
            self.shared_instance_role,
            self.shared_instance_manager_id,
            self.shared_instance_linked_user_id,
            self.shared_sync_status,
            self.shared_sync_applied_update_id,
            self.shared_sync_latest_available_update_id,
        )?;
        let groups = parse_groups(self.groups)?;
        let launch_overrides =
            launch_overrides_from_json(instance_id, self.launch_overrides)?;

        Ok(InstanceMetadataRecord {
            instance,
            applied_content_set,
            link,
            shared_instance,
            groups,
            launch_overrides,
        })
    }

    fn into_launch_context(self) -> crate::Result<InstanceLaunchContext> {
        let record = self.into_record()?;

        Ok(InstanceLaunchContext {
            instance: record.instance,
            applied_content_set: record.applied_content_set,
            link: record.link,
            launch_overrides: record.launch_overrides,
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
    let row = sqlx::query_as!(
        InstanceRow,
        "
		SELECT *
		FROM instances
		WHERE id = ?
		",
        id,
    )
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
    let row = sqlx::query_as!(
        InstanceRow,
        "
		SELECT *
		FROM instances
		WHERE path = ?
		",
        path,
    )
    .fetch_optional(exec)
    .await?;

    row.map(TryInto::try_into).transpose()
}

pub(crate) async fn get_instance_path_by_id<'e, E>(
    id: &str,
    exec: E,
) -> crate::Result<Option<String>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let path = sqlx::query_scalar!(
        "
        SELECT path
        FROM instances
        WHERE id = ?
        ",
        id,
    )
    .fetch_optional(exec)
    .await?;

    Ok(path)
}

pub(crate) async fn get_instance_display_info<'e, E>(
    id: &str,
    exec: E,
) -> crate::Result<Option<InstanceDisplayInfo>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query_as!(
        InstanceDisplayInfo,
        "
        SELECT id, name
        FROM instances
        WHERE id = ?
        ",
        id,
    )
    .fetch_optional(exec)
    .await?;

    Ok(row)
}

pub(crate) async fn get_instance_metadata_by_id(
    id: &str,
    pool: &SqlitePool,
) -> crate::Result<Option<InstanceMetadataRecord>> {
    let row = sqlx::query_as!(
        InstanceMetadataRow,
        r#"
        SELECT
            i.id AS "id!: String",
            i.path AS "path!: String",
            i.applied_content_set_id AS "applied_content_set_id?: String",
            i.install_stage AS "install_stage!: String",
            i.launcher_feature_version AS "launcher_feature_version!: String",
            i.update_channel AS "update_channel!: String",
            i.name AS "name!: String",
            i.icon_path AS "icon_path?: String",
            i.created AS "created!: i64",
            i.modified AS "modified!: i64",
            i.last_played AS "last_played?: i64",
            i.submitted_time_played AS "submitted_time_played!: i64",
            i.recent_time_played AS "recent_time_played!: i64",
            cs.id AS "content_set_id?: String",
            cs.instance_id AS "content_set_instance_id?: String",
            cs.name AS "content_set_name?: String",
            cs.source_kind AS "content_set_source_kind?: String",
            cs.status AS "content_set_status?: String",
            cs.game_version AS "content_set_game_version?: String",
            cs.protocol_version AS "content_set_protocol_version?: i64",
            cs.loader AS "content_set_loader?: String",
            cs.loader_version AS "content_set_loader_version?: String",
            cs.created AS "content_set_created?: i64",
            cs.modified AS "content_set_modified?: i64",
            COALESCE(link.link_kind, 'unmanaged') AS "link_kind!: String",
            link.modrinth_project_id AS "modrinth_project_id?: String",
            link.modrinth_version_id AS "modrinth_version_id?: String",
            link.server_project_id AS "server_project_id?: String",
            link.content_project_id AS "content_project_id?: String",
            link.content_version_id AS "content_version_id?: String",
            link.hosting_server_id AS "hosting_server_id?: String",
            json(link.hosting_instance_ids) AS "hosting_instance_ids?: String",
            link.hosting_active_instance_id AS "hosting_active_instance_id?: String",
            link.shared_instance_id AS "shared_instance_id?: String",
            link.shared_instance_role AS "shared_instance_role?: String",
            link.shared_instance_manager_id AS "shared_instance_manager_id?: String",
            link.shared_instance_linked_user_id AS "shared_instance_linked_user_id?: String",
            sync.applied_update_id AS "shared_sync_applied_update_id?: String",
            sync.latest_available_update_id AS "shared_sync_latest_available_update_id?: String",
            sync.status AS "shared_sync_status?: String",
            link.imported_name AS "imported_name?: String",
            link.imported_version_number AS "imported_version_number?: String",
            link.imported_filename AS "imported_filename?: String",
            COALESCE((
                SELECT json_group_array(group_name)
                FROM (
                    SELECT group_name
                    FROM instance_groups
                    WHERE instance_id = i.id
                    ORDER BY group_name
                )
            ), '[]') AS "groups!: String",
            json(overrides.overrides) AS "launch_overrides?: String"
        FROM instances i
        LEFT JOIN instance_content_sets cs
            ON cs.id = i.applied_content_set_id
            AND cs.instance_id = i.id
        LEFT JOIN instance_links link
            ON link.instance_id = i.id
        LEFT JOIN instance_content_set_sync_state sync
            ON sync.content_set_id = cs.id
            AND sync.provider = 'shared_instance'
        LEFT JOIN instance_launch_overrides overrides
            ON overrides.instance_id = i.id
        WHERE i.id = ?
        "#,
        id,
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };
    let record =
        hydrate_shared_instance_fields(row.into_record()?, pool).await?;

    Ok(Some(record))
}

pub(crate) async fn get_instance_metadata_many(
    ids: &[&str],
    pool: &SqlitePool,
) -> crate::Result<Vec<InstanceMetadataRecord>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let ids_json = serde_json::to_string(ids)?;
    let rows = sqlx::query_as!(
        InstanceMetadataRow,
        r#"
        WITH requested AS (
            SELECT value AS id, key AS ord
            FROM json_each(?)
        )
        SELECT
            i.id AS "id!: String",
            i.path AS "path!: String",
            i.applied_content_set_id AS "applied_content_set_id?: String",
            i.install_stage AS "install_stage!: String",
            i.launcher_feature_version AS "launcher_feature_version!: String",
            i.update_channel AS "update_channel!: String",
            i.name AS "name!: String",
            i.icon_path AS "icon_path?: String",
            i.created AS "created!: i64",
            i.modified AS "modified!: i64",
            i.last_played AS "last_played?: i64",
            i.submitted_time_played AS "submitted_time_played!: i64",
            i.recent_time_played AS "recent_time_played!: i64",
            cs.id AS "content_set_id?: String",
            cs.instance_id AS "content_set_instance_id?: String",
            cs.name AS "content_set_name?: String",
            cs.source_kind AS "content_set_source_kind?: String",
            cs.status AS "content_set_status?: String",
            cs.game_version AS "content_set_game_version?: String",
            cs.protocol_version AS "content_set_protocol_version?: i64",
            cs.loader AS "content_set_loader?: String",
            cs.loader_version AS "content_set_loader_version?: String",
            cs.created AS "content_set_created?: i64",
            cs.modified AS "content_set_modified?: i64",
            COALESCE(link.link_kind, 'unmanaged') AS "link_kind!: String",
            link.modrinth_project_id AS "modrinth_project_id?: String",
            link.modrinth_version_id AS "modrinth_version_id?: String",
            link.server_project_id AS "server_project_id?: String",
            link.content_project_id AS "content_project_id?: String",
            link.content_version_id AS "content_version_id?: String",
            link.hosting_server_id AS "hosting_server_id?: String",
            json(link.hosting_instance_ids) AS "hosting_instance_ids?: String",
            link.hosting_active_instance_id AS "hosting_active_instance_id?: String",
            link.shared_instance_id AS "shared_instance_id?: String",
            link.shared_instance_role AS "shared_instance_role?: String",
            link.shared_instance_manager_id AS "shared_instance_manager_id?: String",
            link.shared_instance_linked_user_id AS "shared_instance_linked_user_id?: String",
            sync.applied_update_id AS "shared_sync_applied_update_id?: String",
            sync.latest_available_update_id AS "shared_sync_latest_available_update_id?: String",
            sync.status AS "shared_sync_status?: String",
            link.imported_name AS "imported_name?: String",
            link.imported_version_number AS "imported_version_number?: String",
            link.imported_filename AS "imported_filename?: String",
            COALESCE((
                SELECT json_group_array(group_name)
                FROM (
                    SELECT group_name
                    FROM instance_groups
                    WHERE instance_id = i.id
                    ORDER BY group_name
                )
            ), '[]') AS "groups!: String",
            json(overrides.overrides) AS "launch_overrides?: String"
        FROM requested
        INNER JOIN instances i
            ON i.id = requested.id
        LEFT JOIN instance_content_sets cs
            ON cs.id = i.applied_content_set_id
            AND cs.instance_id = i.id
        LEFT JOIN instance_links link
            ON link.instance_id = i.id
        LEFT JOIN instance_content_set_sync_state sync
            ON sync.content_set_id = cs.id
            AND sync.provider = 'shared_instance'
        LEFT JOIN instance_launch_overrides overrides
            ON overrides.instance_id = i.id
        ORDER BY requested.ord
        "#,
        ids_json,
    )
    .fetch_all(pool)
    .await?;

    let mut records = Vec::with_capacity(rows.len());
    for row in rows {
        records.push(
            hydrate_shared_instance_fields(row.into_record()?, pool).await?,
        );
    }

    Ok(records)
}

pub(crate) async fn list_instance_metadata(
    pool: &SqlitePool,
) -> crate::Result<Vec<InstanceMetadataRecord>> {
    let rows = sqlx::query_as!(
        InstanceMetadataRow,
        r#"
        SELECT
            i.id AS "id!: String",
            i.path AS "path!: String",
            i.applied_content_set_id AS "applied_content_set_id?: String",
            i.install_stage AS "install_stage!: String",
            i.launcher_feature_version AS "launcher_feature_version!: String",
            i.update_channel AS "update_channel!: String",
            i.name AS "name!: String",
            i.icon_path AS "icon_path?: String",
            i.created AS "created!: i64",
            i.modified AS "modified!: i64",
            i.last_played AS "last_played?: i64",
            i.submitted_time_played AS "submitted_time_played!: i64",
            i.recent_time_played AS "recent_time_played!: i64",
            cs.id AS "content_set_id?: String",
            cs.instance_id AS "content_set_instance_id?: String",
            cs.name AS "content_set_name?: String",
            cs.source_kind AS "content_set_source_kind?: String",
            cs.status AS "content_set_status?: String",
            cs.game_version AS "content_set_game_version?: String",
            cs.protocol_version AS "content_set_protocol_version?: i64",
            cs.loader AS "content_set_loader?: String",
            cs.loader_version AS "content_set_loader_version?: String",
            cs.created AS "content_set_created?: i64",
            cs.modified AS "content_set_modified?: i64",
            COALESCE(link.link_kind, 'unmanaged') AS "link_kind!: String",
            link.modrinth_project_id AS "modrinth_project_id?: String",
            link.modrinth_version_id AS "modrinth_version_id?: String",
            link.server_project_id AS "server_project_id?: String",
            link.content_project_id AS "content_project_id?: String",
            link.content_version_id AS "content_version_id?: String",
            link.hosting_server_id AS "hosting_server_id?: String",
            json(link.hosting_instance_ids) AS "hosting_instance_ids?: String",
            link.hosting_active_instance_id AS "hosting_active_instance_id?: String",
            link.shared_instance_id AS "shared_instance_id?: String",
            link.shared_instance_role AS "shared_instance_role?: String",
            link.shared_instance_manager_id AS "shared_instance_manager_id?: String",
            link.shared_instance_linked_user_id AS "shared_instance_linked_user_id?: String",
            sync.applied_update_id AS "shared_sync_applied_update_id?: String",
            sync.latest_available_update_id AS "shared_sync_latest_available_update_id?: String",
            sync.status AS "shared_sync_status?: String",
            link.imported_name AS "imported_name?: String",
            link.imported_version_number AS "imported_version_number?: String",
            link.imported_filename AS "imported_filename?: String",
            COALESCE((
                SELECT json_group_array(group_name)
                FROM (
                    SELECT group_name
                    FROM instance_groups
                    WHERE instance_id = i.id
                    ORDER BY group_name
                )
            ), '[]') AS "groups!: String",
            json(overrides.overrides) AS "launch_overrides?: String"
        FROM instances i
        LEFT JOIN instance_content_sets cs
            ON cs.id = i.applied_content_set_id
            AND cs.instance_id = i.id
        LEFT JOIN instance_links link
            ON link.instance_id = i.id
        LEFT JOIN instance_content_set_sync_state sync
            ON sync.content_set_id = cs.id
            AND sync.provider = 'shared_instance'
        LEFT JOIN instance_launch_overrides overrides
            ON overrides.instance_id = i.id
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut records = Vec::with_capacity(rows.len());
    for row in rows {
        records.push(
            hydrate_shared_instance_fields(row.into_record()?, pool).await?,
        );
    }

    Ok(records)
}

pub(crate) async fn get_instance_launch_context(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<Option<InstanceLaunchContext>> {
    let row = sqlx::query_as!(
        InstanceMetadataRow,
        r#"
        SELECT
            i.id AS "id!: String",
            i.path AS "path!: String",
            i.applied_content_set_id AS "applied_content_set_id?: String",
            i.install_stage AS "install_stage!: String",
            i.launcher_feature_version AS "launcher_feature_version!: String",
            i.update_channel AS "update_channel!: String",
            i.name AS "name!: String",
            i.icon_path AS "icon_path?: String",
            i.created AS "created!: i64",
            i.modified AS "modified!: i64",
            i.last_played AS "last_played?: i64",
            i.submitted_time_played AS "submitted_time_played!: i64",
            i.recent_time_played AS "recent_time_played!: i64",
            cs.id AS "content_set_id?: String",
            cs.instance_id AS "content_set_instance_id?: String",
            cs.name AS "content_set_name?: String",
            cs.source_kind AS "content_set_source_kind?: String",
            cs.status AS "content_set_status?: String",
            cs.game_version AS "content_set_game_version?: String",
            cs.protocol_version AS "content_set_protocol_version?: i64",
            cs.loader AS "content_set_loader?: String",
            cs.loader_version AS "content_set_loader_version?: String",
            cs.created AS "content_set_created?: i64",
            cs.modified AS "content_set_modified?: i64",
            COALESCE(link.link_kind, 'unmanaged') AS "link_kind!: String",
            link.modrinth_project_id AS "modrinth_project_id?: String",
            link.modrinth_version_id AS "modrinth_version_id?: String",
            link.server_project_id AS "server_project_id?: String",
            link.content_project_id AS "content_project_id?: String",
            link.content_version_id AS "content_version_id?: String",
            link.hosting_server_id AS "hosting_server_id?: String",
            json(link.hosting_instance_ids) AS "hosting_instance_ids?: String",
            link.hosting_active_instance_id AS "hosting_active_instance_id?: String",
            link.shared_instance_id AS "shared_instance_id?: String",
            link.shared_instance_role AS "shared_instance_role?: String",
            link.shared_instance_manager_id AS "shared_instance_manager_id?: String",
            link.shared_instance_linked_user_id AS "shared_instance_linked_user_id?: String",
            sync.applied_update_id AS "shared_sync_applied_update_id?: String",
            sync.latest_available_update_id AS "shared_sync_latest_available_update_id?: String",
            sync.status AS "shared_sync_status?: String",
            link.imported_name AS "imported_name?: String",
            link.imported_version_number AS "imported_version_number?: String",
            link.imported_filename AS "imported_filename?: String",
            '[]' AS "groups!: String",
            json(overrides.overrides) AS "launch_overrides?: String"
        FROM instances i
        LEFT JOIN instance_content_sets cs
            ON cs.id = i.applied_content_set_id
            AND cs.instance_id = i.id
        LEFT JOIN instance_links link
            ON link.instance_id = i.id
        LEFT JOIN instance_content_set_sync_state sync
            ON sync.content_set_id = cs.id
            AND sync.provider = 'shared_instance'
        LEFT JOIN instance_launch_overrides overrides
            ON overrides.instance_id = i.id
        WHERE i.id = ?
        "#,
        instance_id,
    )
    .fetch_optional(pool)
    .await?;

    row.map(InstanceMetadataRow::into_launch_context)
        .transpose()
}

pub(crate) async fn list_instances(
    pool: &SqlitePool,
) -> crate::Result<Vec<Instance>> {
    let rows = sqlx::query_as!(
        InstanceRow,
        "
		SELECT *
		FROM instances
		",
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub(crate) async fn get_instance_link<'e, E>(
    instance_id: &str,
    exec: E,
) -> crate::Result<InstanceLink>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query_as!(
        InstanceLinkRow,
        r#"
        SELECT
            instance_id,
            link_kind,
            modrinth_project_id,
            modrinth_version_id,
            server_project_id,
            content_project_id,
            content_version_id,
            hosting_server_id,
            json(hosting_instance_ids) AS "hosting_instance_ids?: String",
            hosting_active_instance_id,
            shared_instance_id,
            shared_instance_role,
            shared_instance_manager_id,
            shared_instance_linked_user_id,
            imported_name,
            imported_version_number,
            imported_filename
        FROM instance_links
        WHERE instance_id = ?
        "#,
        instance_id,
    )
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
    let rows = sqlx::query_scalar!(
        "
		SELECT group_name
		FROM instance_groups
		WHERE instance_id = ?
		ORDER BY group_name
		",
        instance_id,
    )
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
    let row = sqlx::query_as!(
        InstanceLaunchOverridesRow,
        r#"
		SELECT
			instance_id,
			json(overrides) AS "overrides!: String"
		FROM instance_launch_overrides
		WHERE instance_id = ?
		"#,
        instance_id,
    )
    .fetch_optional(exec)
    .await?;

    row.map(TryInto::try_into).transpose()
}

pub(crate) async fn insert_instance(
    instance: &Instance,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let id = instance.id.as_str();
    let path = instance.path.as_str();
    let applied_content_set_id = instance.applied_content_set_id.as_deref();
    let install_stage = instance.install_stage.as_str();
    let launcher_feature_version = instance.launcher_feature_version.as_str();
    let update_channel = instance.update_channel.key();
    let name = instance.name.as_str();
    let icon_path = instance.icon_path.as_deref();
    let created = instance.created.timestamp();
    let modified = instance.modified.timestamp();
    let last_played = instance.last_played.map(|value| value.timestamp());
    let submitted_time_played = playtime_to_storage(
        instance.submitted_time_played,
        "submitted_time_played",
    )?;
    let recent_time_played =
        playtime_to_storage(instance.recent_time_played, "recent_time_played")?;

    sqlx::query!(
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
		",
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
        recent_time_played,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn update_instance(
    instance: &Instance,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let id = instance.id.as_str();
    let path = instance.path.as_str();
    let applied_content_set_id = instance.applied_content_set_id.as_deref();
    let install_stage = instance.install_stage.as_str();
    let launcher_feature_version = instance.launcher_feature_version.as_str();
    let update_channel = instance.update_channel.key();
    let name = instance.name.as_str();
    let icon_path = instance.icon_path.as_deref();
    let modified = instance.modified.timestamp();
    let last_played = instance.last_played.map(|value| value.timestamp());
    let submitted_time_played = playtime_to_storage(
        instance.submitted_time_played,
        "submitted_time_played",
    )?;
    let recent_time_played =
        playtime_to_storage(instance.recent_time_played, "recent_time_played")?;

    sqlx::query!(
        "
		UPDATE instances
		SET
			path = ?,
			applied_content_set_id = ?,
			install_stage = ?,
			launcher_feature_version = ?,
			update_channel = ?,
			name = ?,
			icon_path = ?,
			modified = ?,
			last_played = ?,
			submitted_time_played = ?,
			recent_time_played = ?
		WHERE id = ?
		",
        path,
        applied_content_set_id,
        install_stage,
        launcher_feature_version,
        update_channel,
        name,
        icon_path,
        modified,
        last_played,
        submitted_time_played,
        recent_time_played,
        id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn upsert_instance_link(
    instance_id: &str,
    link: &InstanceLink,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let columns = instance_link_columns(link)?;
    let modrinth_project_id = columns.modrinth_project_id.as_deref();
    let modrinth_version_id = columns.modrinth_version_id.as_deref();
    let server_project_id = columns.server_project_id.as_deref();
    let content_project_id = columns.content_project_id.as_deref();
    let content_version_id = columns.content_version_id.as_deref();
    let hosting_server_id = columns.hosting_server_id.as_deref();
    let hosting_instance_ids = columns.hosting_instance_ids.as_deref();
    let hosting_active_instance_id =
        columns.hosting_active_instance_id.as_deref();
    let imported_name = columns.imported_name.as_deref();
    let imported_version_number = columns.imported_version_number.as_deref();
    let imported_filename = columns.imported_filename.as_deref();

    sqlx::query!(
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
			imported_name,
			imported_version_number,
			imported_filename
		)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?, jsonb(?), ?, ?, ?, ?)
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
			imported_name = excluded.imported_name,
			imported_version_number = excluded.imported_version_number,
			imported_filename = excluded.imported_filename
		",
        instance_id,
        columns.link_kind,
        modrinth_project_id,
        modrinth_version_id,
        server_project_id,
        content_project_id,
        content_version_id,
        hosting_server_id,
        hosting_instance_ids,
        hosting_active_instance_id,
        imported_name,
        imported_version_number,
        imported_filename,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn set_shared_instance_attachment(
    instance_id: &str,
    attachment: Option<&SharedInstanceAttachment>,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let shared_instance_id = attachment.map(|value| value.id.to_string());
    let shared_instance_role =
        attachment.map(|value| value.role.as_str().to_string());
    let shared_instance_manager_id =
        attachment.and_then(|value| value.manager_id.as_deref());
    let shared_instance_linked_user_id =
        attachment.and_then(|value| value.linked_user_id.as_deref());
    let shared_instance_access_token =
        attachment.and_then(|value| value.access_token.as_deref());
    let shared_instance_server_manager_name =
        attachment.and_then(|value| value.server_manager_name.as_deref());
    let shared_instance_server_manager_icon_url =
        attachment.and_then(|value| value.server_manager_icon_url.as_deref());

    sqlx::query!(
        "
		INSERT INTO instance_links (
			instance_id,
			link_kind,
			shared_instance_id,
			shared_instance_role,
			shared_instance_manager_id,
			shared_instance_linked_user_id
		)
		VALUES (?, 'unmanaged', ?, ?, ?, ?)
		ON CONFLICT (instance_id) DO UPDATE SET
			link_kind = CASE
				WHEN excluded.shared_instance_id IS NULL
					AND instance_links.link_kind = 'shared_instance'
					THEN 'unmanaged'
				ELSE instance_links.link_kind
			END,
			shared_instance_id = excluded.shared_instance_id,
			shared_instance_role = excluded.shared_instance_role,
			shared_instance_manager_id = excluded.shared_instance_manager_id,
			shared_instance_linked_user_id = excluded.shared_instance_linked_user_id
		",
        instance_id,
        shared_instance_id,
        shared_instance_role,
        shared_instance_manager_id,
        shared_instance_linked_user_id,
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "
		UPDATE instance_links
		SET shared_instance_access_token = ?,
			shared_instance_server_manager_name = ?,
			shared_instance_server_manager_icon_url = ?
		WHERE instance_id = ?
		",
    )
    .bind(shared_instance_access_token)
    .bind(shared_instance_server_manager_name)
    .bind(shared_instance_server_manager_icon_url)
    .bind(instance_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn replace_instance_groups(
    instance_id: &str,
    groups: &[String],
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    sqlx::query!(
        "
		DELETE FROM instance_groups
		WHERE instance_id = ?
		",
        instance_id,
    )
    .execute(&mut **tx)
    .await?;

    for group in groups {
        sqlx::query!(
            "
			INSERT OR IGNORE INTO instance_groups (instance_id, group_name)
			VALUES (?, ?)
			",
            instance_id,
            group,
        )
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

pub(crate) async fn upsert_instance_launch_overrides(
    overrides: &InstanceLaunchOverrides,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let overrides_data =
        serde_json::to_string(&InstanceLaunchOverridesData::from(overrides))?;
    let instance_id = overrides.instance_id.as_str();

    sqlx::query!(
        "
		INSERT INTO instance_launch_overrides (
			instance_id,
			overrides
		)
		VALUES (?, jsonb(?))
		ON CONFLICT (instance_id) DO UPDATE SET
			overrides = excluded.overrides
		",
        instance_id,
        overrides_data,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn delete_instance_by_id(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<()> {
    sqlx::query!(
        "
		DELETE FROM instances
		WHERE id = ?
		",
        instance_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

struct InstanceLinkColumns {
    link_kind: &'static str,
    modrinth_project_id: Option<String>,
    modrinth_version_id: Option<String>,
    server_project_id: Option<String>,
    content_project_id: Option<String>,
    content_version_id: Option<String>,
    hosting_server_id: Option<String>,
    hosting_instance_ids: Option<String>,
    hosting_active_instance_id: Option<String>,
    imported_name: Option<String>,
    imported_version_number: Option<String>,
    imported_filename: Option<String>,
}

fn instance_link_columns(
    link: &InstanceLink,
) -> crate::Result<InstanceLinkColumns> {
    match link {
        InstanceLink::Unmanaged => Ok(InstanceLinkColumns {
            link_kind: "unmanaged",
            modrinth_project_id: None,
            modrinth_version_id: None,
            server_project_id: None,
            content_project_id: None,
            content_version_id: None,
            hosting_server_id: None,
            hosting_instance_ids: None,
            hosting_active_instance_id: None,
            imported_name: None,
            imported_version_number: None,
            imported_filename: None,
        }),
        InstanceLink::ModrinthModpack {
            project_id,
            version_id,
        } => Ok(InstanceLinkColumns {
            link_kind: "modrinth_modpack",
            modrinth_project_id: Some(project_id.clone()),
            modrinth_version_id: Some(version_id.clone()),
            server_project_id: None,
            content_project_id: None,
            content_version_id: None,
            hosting_server_id: None,
            hosting_instance_ids: None,
            hosting_active_instance_id: None,
            imported_name: None,
            imported_version_number: None,
            imported_filename: None,
        }),
        InstanceLink::ServerProject { project_id } => Ok(InstanceLinkColumns {
            link_kind: "server_project",
            modrinth_project_id: None,
            modrinth_version_id: None,
            server_project_id: Some(project_id.clone()),
            content_project_id: None,
            content_version_id: None,
            hosting_server_id: None,
            hosting_instance_ids: None,
            hosting_active_instance_id: None,
            imported_name: None,
            imported_version_number: None,
            imported_filename: None,
        }),
        InstanceLink::ServerProjectModpack {
            server_project_id,
            content_project_id,
            content_version_id,
        } => Ok(InstanceLinkColumns {
            link_kind: "server_project_modpack",
            modrinth_project_id: None,
            modrinth_version_id: None,
            server_project_id: Some(server_project_id.clone()),
            content_project_id: Some(content_project_id.clone()),
            content_version_id: Some(content_version_id.clone()),
            hosting_server_id: None,
            hosting_instance_ids: None,
            hosting_active_instance_id: None,
            imported_name: None,
            imported_version_number: None,
            imported_filename: None,
        }),
        InstanceLink::ModrinthHosting {
            server_id,
            instance_ids,
            active_instance_id,
        } => Ok(InstanceLinkColumns {
            link_kind: "modrinth_hosting",
            modrinth_project_id: None,
            modrinth_version_id: None,
            server_project_id: None,
            content_project_id: None,
            content_version_id: None,
            hosting_server_id: Some(server_id.to_string()),
            hosting_instance_ids: Some(serde_json::to_string(instance_ids)?),
            hosting_active_instance_id: active_instance_id
                .map(|value| value.to_string()),
            imported_name: None,
            imported_version_number: None,
            imported_filename: None,
        }),
        InstanceLink::ImportedModpack {
            project_id,
            version_id,
            name,
            version_number,
            filename,
        } => Ok(InstanceLinkColumns {
            link_kind: "imported_modpack",
            modrinth_project_id: project_id.clone(),
            modrinth_version_id: version_id.clone(),
            server_project_id: None,
            content_project_id: None,
            content_version_id: None,
            hosting_server_id: None,
            hosting_instance_ids: None,
            hosting_active_instance_id: None,
            imported_name: name.clone(),
            imported_version_number: version_number.clone(),
            imported_filename: filename.clone(),
        }),
        InstanceLink::SharedInstance {
            modpack_project_id,
            modpack_version_id,
        } => Ok(InstanceLinkColumns {
            link_kind: "shared_instance",
            modrinth_project_id: modpack_project_id.clone(),
            modrinth_version_id: modpack_version_id.clone(),
            server_project_id: None,
            content_project_id: None,
            content_version_id: None,
            hosting_server_id: None,
            hosting_instance_ids: None,
            hosting_active_instance_id: None,
            imported_name: None,
            imported_version_number: None,
            imported_filename: None,
        }),
    }
}

fn required(value: Option<String>, column: &str) -> crate::Result<String> {
    value.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Missing required instance link column {column}"
        ))
        .into()
    })
}

fn required_i64(value: Option<i64>, column: &str) -> crate::Result<i64> {
    value.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Missing required instance metadata column {column}"
        ))
        .into()
    })
}

fn parse_groups(value: String) -> crate::Result<Vec<String>> {
    serde_json::from_str(&value).map_err(|err| {
        crate::ErrorKind::InputError(format!(
            "Invalid instance groups JSON: {err}"
        ))
        .into()
    })
}

fn launch_overrides_from_json(
    instance_id: String,
    value: Option<String>,
) -> crate::Result<InstanceLaunchOverrides> {
    match value {
        Some(overrides) if overrides != "null" => InstanceLaunchOverridesRow {
            instance_id,
            overrides,
        }
        .try_into(),
        _ => Ok(InstanceLaunchOverrides::empty(instance_id)),
    }
}

async fn hydrate_shared_instance_fields(
    mut record: InstanceMetadataRecord,
    pool: &SqlitePool,
) -> crate::Result<InstanceMetadataRecord> {
    if let Some(attachment) = record.shared_instance.as_mut() {
        let (access_token, server_manager_name, server_manager_icon_url) =
            shared_instance_fields(&record.instance.id, pool).await?;
        attachment.access_token = access_token;
        attachment.server_manager_name = server_manager_name;
        attachment.server_manager_icon_url = server_manager_icon_url;
    }

    Ok(record)
}

async fn shared_instance_fields(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<(Option<String>, Option<String>, Option<String>)> {
    Ok(
        sqlx::query_as::<_, (Option<String>, Option<String>, Option<String>)>(
            "
		SELECT
			shared_instance_access_token,
			shared_instance_server_manager_name,
			shared_instance_server_manager_icon_url
		FROM instance_links
		WHERE instance_id = ?
		",
        )
        .bind(instance_id)
        .fetch_optional(pool)
        .await?
        .unwrap_or((None, None, None)),
    )
}

fn shared_instance_attachment(
    shared_instance_id: Option<String>,
    shared_instance_role: Option<String>,
    shared_instance_manager_id: Option<String>,
    shared_instance_linked_user_id: Option<String>,
    shared_sync_status: Option<String>,
    applied_update_id: Option<String>,
    latest_available_update_id: Option<String>,
) -> crate::Result<Option<SharedInstanceAttachment>> {
    let Some(id) = shared_instance_id else {
        return Ok(None);
    };

    let role = match shared_instance_role {
        Some(role) => SharedInstanceRole::from_stored_str(&role)?,
        None => SharedInstanceRole::Member,
    };
    let status = match shared_sync_status {
        Some(status) => ContentSetSyncStatus::from_str(&status)?,
        None => ContentSetSyncStatus::Unknown,
    };

    Ok(Some(SharedInstanceAttachment {
        id,
        role,
        manager_id: shared_instance_manager_id,
        server_manager_name: None,
        server_manager_icon_url: None,
        linked_user_id: shared_instance_linked_user_id,
        access_token: None,
        status,
        applied_version: optional_i32(applied_update_id, "applied_update_id")?,
        latest_version: optional_i32(
            latest_available_update_id,
            "latest_available_update_id",
        )?,
    }))
}

fn optional_i32(
    value: Option<String>,
    column: &str,
) -> crate::Result<Option<i32>> {
    value
        .map(|value| {
            value.parse().map_err(|err| {
                crate::ErrorKind::InputError(format!("Invalid {column}: {err}"))
                    .into()
            })
        })
        .transpose()
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
