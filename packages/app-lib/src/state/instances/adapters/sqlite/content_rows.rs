#![allow(dead_code)]

use crate::state::instances::{
    ContentEntry, ContentRequirement, ContentSet, ContentSetRemoteRef,
    ContentSetRemoteRefType, ContentSetStatus, ContentSetSyncProvider,
    ContentSetSyncState, ContentSetSyncStatus, ContentSourceKind,
    ContentUpdateCheck, InstanceFile,
};
use crate::state::{ModLoader, ProjectType, ReleaseChannel};
use chrono::{DateTime, TimeZone, Utc};
use sqlx::{Executor, Sqlite};

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct ContentSetRow {
    pub id: String,
    pub instance_id: String,
    pub name: String,
    pub source_kind: String,
    pub status: String,
    pub game_version: String,
    pub protocol_version: Option<i64>,
    pub loader: String,
    pub loader_version: Option<String>,
    pub created: i64,
    pub modified: i64,
}

impl TryFrom<ContentSetRow> for ContentSet {
    type Error = crate::Error;

    fn try_from(row: ContentSetRow) -> crate::Result<Self> {
        Ok(Self {
            id: row.id,
            instance_id: row.instance_id,
            name: row.name,
            source_kind: ContentSourceKind::from_str(&row.source_kind)?,
            status: ContentSetStatus::from_str(&row.status)?,
            game_version: row.game_version,
            protocol_version: row.protocol_version.map(|value| value as u32),
            loader: ModLoader::from_string(&row.loader),
            loader_version: row.loader_version,
            created: timestamp(row.created),
            modified: timestamp(row.modified),
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct ContentSetRemoteRefRow {
    pub content_set_id: String,
    pub ref_type: String,
    pub ref_id: String,
}

impl TryFrom<ContentSetRemoteRefRow> for ContentSetRemoteRef {
    type Error = crate::Error;

    fn try_from(row: ContentSetRemoteRefRow) -> crate::Result<Self> {
        Ok(Self {
            content_set_id: row.content_set_id,
            ref_type: ContentSetRemoteRefType::from_str(&row.ref_type)?,
            ref_id: row.ref_id,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct ContentSetSyncStateRow {
    pub content_set_id: String,
    pub provider: String,
    pub applied_update_id: Option<String>,
    pub latest_available_update_id: Option<String>,
    pub checked_at: Option<i64>,
    pub status: String,
}

impl TryFrom<ContentSetSyncStateRow> for ContentSetSyncState {
    type Error = crate::Error;

    fn try_from(row: ContentSetSyncStateRow) -> crate::Result<Self> {
        Ok(Self {
            content_set_id: row.content_set_id,
            provider: ContentSetSyncProvider::from_str(&row.provider)?,
            applied_update_id: row.applied_update_id,
            latest_available_update_id: row.latest_available_update_id,
            checked_at: row.checked_at.and_then(optional_timestamp),
            status: ContentSetSyncStatus::from_str(&row.status)?,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct InstanceFileRow {
    pub id: String,
    pub instance_id: String,
    pub relative_path: String,
    pub file_name: String,
    pub enabled: i64,
    pub sha1: String,
    pub size: i64,
    pub missing: i64,
    pub added_at: i64,
    pub modified_at: i64,
}

impl TryFrom<InstanceFileRow> for InstanceFile {
    type Error = crate::Error;

    fn try_from(row: InstanceFileRow) -> crate::Result<Self> {
        Ok(Self {
            id: row.id,
            instance_id: row.instance_id,
            relative_path: row.relative_path,
            file_name: row.file_name,
            enabled: row.enabled == 1,
            sha1: row.sha1,
            size: unsigned(row.size, "size")?,
            missing: row.missing == 1,
            added_at: timestamp(row.added_at),
            modified_at: timestamp(row.modified_at),
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct ContentEntryRow {
    pub id: String,
    pub instance_id: String,
    pub content_set_id: String,
    pub file_id: Option<String>,
    pub project_type: String,
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub source_kind: String,
    pub server_requirement: String,
    pub client_requirement: String,
    pub enabled: i64,
    pub added_at: i64,
    pub modified_at: i64,
}

impl TryFrom<ContentEntryRow> for ContentEntry {
    type Error = crate::Error;

    fn try_from(row: ContentEntryRow) -> crate::Result<Self> {
        Ok(Self {
            id: row.id,
            instance_id: row.instance_id,
            content_set_id: row.content_set_id,
            file_id: row.file_id,
            project_type: project_type_from_str(&row.project_type)?,
            project_id: row.project_id,
            version_id: row.version_id,
            source_kind: ContentSourceKind::from_str(&row.source_kind)?,
            server_requirement: ContentRequirement::from_str(
                &row.server_requirement,
            )?,
            client_requirement: ContentRequirement::from_str(
                &row.client_requirement,
            )?,
            enabled: row.enabled == 1,
            added_at: timestamp(row.added_at),
            modified_at: timestamp(row.modified_at),
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct ContentUpdateCheckRow {
    pub content_entry_id: String,
    pub update_channel: String,
    pub update_version_id: Option<String>,
    pub checked_at: i64,
}

impl From<ContentUpdateCheckRow> for ContentUpdateCheck {
    fn from(row: ContentUpdateCheckRow) -> Self {
        Self {
            content_entry_id: row.content_entry_id,
            update_channel: ReleaseChannel::from_key(&row.update_channel),
            update_version_id: row.update_version_id,
            checked_at: timestamp(row.checked_at),
        }
    }
}

pub(crate) async fn get_applied_content_set<'e, E>(
    instance_id: &str,
    exec: E,
) -> crate::Result<Option<ContentSet>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query_as::<_, ContentSetRow>(
        "
		SELECT cs.*
		FROM instances i
		INNER JOIN instance_content_sets cs
			ON cs.id = i.applied_content_set_id
		WHERE i.id = ?
		",
    )
    .bind(instance_id)
    .fetch_optional(exec)
    .await?;

    row.map(TryInto::try_into).transpose()
}

pub(crate) async fn get_content_set<'e, E>(
    content_set_id: &str,
    exec: E,
) -> crate::Result<Option<ContentSet>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query_as::<_, ContentSetRow>(
        "
		SELECT *
		FROM instance_content_sets
		WHERE id = ?
		",
    )
    .bind(content_set_id)
    .fetch_optional(exec)
    .await?;

    row.map(TryInto::try_into).transpose()
}

pub(crate) async fn get_content_sets_for_instance<'e, E>(
    instance_id: &str,
    exec: E,
) -> crate::Result<Vec<ContentSet>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let rows = sqlx::query_as::<_, ContentSetRow>(
        "
		SELECT *
		FROM instance_content_sets
		WHERE instance_id = ?
		ORDER BY created ASC, id ASC
		",
    )
    .bind(instance_id)
    .fetch_all(exec)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub(crate) async fn get_content_set_remote_refs<'e, E>(
    content_set_id: &str,
    exec: E,
) -> crate::Result<Vec<ContentSetRemoteRef>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let rows = sqlx::query_as::<_, ContentSetRemoteRefRow>(
        "
		SELECT *
		FROM instance_content_set_remote_refs
		WHERE content_set_id = ?
		ORDER BY ref_type ASC
		",
    )
    .bind(content_set_id)
    .fetch_all(exec)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub(crate) async fn get_content_set_sync_state<'e, E>(
    content_set_id: &str,
    exec: E,
) -> crate::Result<Option<ContentSetSyncState>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query_as::<_, ContentSetSyncStateRow>(
        "
		SELECT *
		FROM instance_content_set_sync_state
		WHERE content_set_id = ?
		",
    )
    .bind(content_set_id)
    .fetch_optional(exec)
    .await?;

    row.map(TryInto::try_into).transpose()
}

pub(crate) async fn get_instance_files<'e, E>(
    instance_id: &str,
    exec: E,
) -> crate::Result<Vec<InstanceFile>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let rows = sqlx::query_as::<_, InstanceFileRow>(
        "
		SELECT *
		FROM instance_files
		WHERE instance_id = ?
		ORDER BY relative_path ASC
		",
    )
    .bind(instance_id)
    .fetch_all(exec)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub(crate) async fn get_content_entries<'e, E>(
    content_set_id: &str,
    exec: E,
) -> crate::Result<Vec<ContentEntry>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let rows = sqlx::query_as::<_, ContentEntryRow>(
        "
		SELECT *
		FROM instance_content_entries
		WHERE content_set_id = ?
		ORDER BY added_at ASC, id ASC
		",
    )
    .bind(content_set_id)
    .fetch_all(exec)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub(crate) async fn get_content_update_check<'e, E>(
    content_entry_id: &str,
    exec: E,
) -> crate::Result<Option<ContentUpdateCheck>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query_as::<_, ContentUpdateCheckRow>(
        "
		SELECT *
		FROM instance_content_update_checks
		WHERE content_entry_id = ?
		",
    )
    .bind(content_entry_id)
    .fetch_optional(exec)
    .await?;

    Ok(row.map(Into::into))
}

fn project_type_from_str(value: &str) -> crate::Result<ProjectType> {
    match value {
        "mod" => Ok(ProjectType::Mod),
        "datapack" => Ok(ProjectType::DataPack),
        "resourcepack" => Ok(ProjectType::ResourcePack),
        "shader" | "shaderpack" => Ok(ProjectType::ShaderPack),
        other => Err(crate::ErrorKind::InputError(format!(
            "Unknown content project type {other}"
        ))
        .into()),
    }
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
