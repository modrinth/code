#![allow(dead_code)]

use crate::state::instances::{
    ContentEntry, ContentRequirement, ContentSet, ContentSetRemoteRef,
    ContentSetRemoteRefType, ContentSetStatus, ContentSetSyncProvider,
    ContentSetSyncState, ContentSetSyncStatus, ContentSourceKind,
    ContentUpdateCheck, InstanceFile,
};
use crate::state::{ModLoader, ProjectType, ReleaseChannel};
use chrono::{DateTime, TimeZone, Utc};
use sqlx::{Executor, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

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

pub(crate) async fn insert_content_set(
	content_set: &ContentSet,
	tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
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
		VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
		",
	)
	.bind(content_set.id.as_str())
	.bind(content_set.instance_id.as_str())
	.bind(content_set.name.as_str())
	.bind(content_set.source_kind.as_str())
	.bind(content_set.status.as_str())
	.bind(content_set.game_version.as_str())
	.bind(content_set.protocol_version.map(|value| value as i64))
	.bind(content_set.loader.as_str())
	.bind(content_set.loader_version.as_deref())
	.bind(content_set.created.timestamp())
	.bind(content_set.modified.timestamp())
	.execute(&mut **tx)
	.await?;

	Ok(())
}

pub(crate) async fn update_content_set(
	content_set: &ContentSet,
	tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
	sqlx::query(
		"
		UPDATE instance_content_sets
		SET
			name = ?,
			source_kind = ?,
			status = ?,
			game_version = ?,
			protocol_version = ?,
			loader = ?,
			loader_version = ?,
			modified = ?
		WHERE id = ?
		",
	)
	.bind(content_set.name.as_str())
	.bind(content_set.source_kind.as_str())
	.bind(content_set.status.as_str())
	.bind(content_set.game_version.as_str())
	.bind(content_set.protocol_version.map(|value| value as i64))
	.bind(content_set.loader.as_str())
	.bind(content_set.loader_version.as_deref())
	.bind(content_set.modified.timestamp())
	.bind(content_set.id.as_str())
	.execute(&mut **tx)
	.await?;

	Ok(())
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

pub(crate) async fn mark_instance_files_missing(
	instance_id: &str,
	tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
	sqlx::query(
		"
		UPDATE instance_files
		SET
			missing = 1,
			modified_at = ?
		WHERE instance_id = ?
		",
	)
	.bind(Utc::now().timestamp())
	.bind(instance_id)
	.execute(&mut **tx)
	.await?;

	Ok(())
}

pub(crate) async fn upsert_instance_file(
	file: &InstanceFile,
	tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
	sqlx::query(
		"
		INSERT INTO instance_files (
			id,
			instance_id,
			relative_path,
			file_name,
			enabled,
			sha1,
			size,
			missing,
			added_at,
			modified_at
		)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
		ON CONFLICT (instance_id, relative_path) DO UPDATE SET
			file_name = excluded.file_name,
			enabled = excluded.enabled,
			sha1 = excluded.sha1,
			size = excluded.size,
			missing = excluded.missing,
			modified_at = excluded.modified_at
		",
	)
	.bind(file.id.as_str())
	.bind(file.instance_id.as_str())
	.bind(file.relative_path.as_str())
	.bind(file.file_name.as_str())
	.bind(i64::from(file.enabled))
	.bind(file.sha1.as_str())
	.bind(file.size as i64)
	.bind(i64::from(file.missing))
	.bind(file.added_at.timestamp())
	.bind(file.modified_at.timestamp())
	.execute(&mut **tx)
	.await?;

	Ok(())
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

pub(crate) struct UpsertInstanceFile<'a> {
	pub instance_id: &'a str,
	pub relative_path: &'a str,
	pub file_name: &'a str,
	pub enabled: bool,
	pub sha1: &'a str,
	pub size: u64,
	pub missing: bool,
}

pub(crate) async fn get_instance_file_by_relative_path(
	instance_id: &str,
	relative_path: &str,
	pool: &SqlitePool,
) -> crate::Result<Option<InstanceFile>> {
	let row = sqlx::query_as::<_, InstanceFileRow>(
		"
		SELECT *
		FROM instance_files
		WHERE instance_id = ? AND relative_path = ?
		",
	)
	.bind(instance_id)
	.bind(relative_path)
	.fetch_optional(pool)
	.await?;

	row.map(TryInto::try_into).transpose()
}

pub(crate) async fn upsert_instance_file_from_parts(
	input: UpsertInstanceFile<'_>,
	pool: &SqlitePool,
) -> crate::Result<InstanceFile> {
	let existing =
		get_instance_file_by_relative_path(input.instance_id, input.relative_path, pool)
			.await?;
	let file = InstanceFile {
		id: existing
			.as_ref()
			.map(|file| file.id.clone())
			.unwrap_or_else(|| format!("instance-file:{}", Uuid::new_v4())),
		instance_id: input.instance_id.to_string(),
		relative_path: input.relative_path.to_string(),
		file_name: input.file_name.to_string(),
		enabled: input.enabled,
		sha1: input.sha1.to_string(),
		size: input.size,
		missing: input.missing,
		added_at: existing
			.as_ref()
			.map(|file| file.added_at)
			.unwrap_or_else(Utc::now),
		modified_at: Utc::now(),
	};

	let mut tx = pool.begin().await?;
	upsert_instance_file(&file, &mut tx).await?;
	tx.commit().await?;

	Ok(file)
}

pub(crate) async fn rename_instance_file(
	instance_id: &str,
	old_relative_path: &str,
	new_relative_path: &str,
	new_file_name: &str,
	enabled: bool,
	pool: &SqlitePool,
) -> crate::Result<Option<InstanceFile>> {
	sqlx::query(
		"
		UPDATE instance_files
		SET
			relative_path = ?,
			file_name = ?,
			enabled = ?,
			modified_at = ?
		WHERE instance_id = ? AND relative_path = ?
		",
	)
	.bind(new_relative_path)
	.bind(new_file_name)
	.bind(i64::from(enabled))
	.bind(Utc::now().timestamp())
	.bind(instance_id)
	.bind(old_relative_path)
	.execute(pool)
	.await?;

	get_instance_file_by_relative_path(instance_id, new_relative_path, pool).await
}

pub(crate) async fn remove_instance_file_by_relative_path(
	instance_id: &str,
	relative_path: &str,
	pool: &SqlitePool,
) -> crate::Result<()> {
	sqlx::query(
		"
		DELETE FROM instance_files
		WHERE instance_id = ? AND relative_path = ?
		",
	)
	.bind(instance_id)
	.bind(relative_path)
	.execute(pool)
	.await?;

	Ok(())
}

pub(crate) struct UpsertContentEntry<'a> {
	pub instance_id: &'a str,
	pub content_set_id: &'a str,
	pub file_id: Option<&'a str>,
	pub project_type: ProjectType,
	pub project_id: Option<&'a str>,
	pub version_id: Option<&'a str>,
	pub source_kind: ContentSourceKind,
	pub server_requirement: ContentRequirement,
	pub client_requirement: ContentRequirement,
	pub enabled: bool,
}

pub(crate) async fn get_content_entry_by_id(
	id: &str,
	pool: &SqlitePool,
) -> crate::Result<Option<ContentEntry>> {
	let row = sqlx::query_as::<_, ContentEntryRow>(
		"
		SELECT *
		FROM instance_content_entries
		WHERE id = ?
		",
	)
	.bind(id)
	.fetch_optional(pool)
	.await?;

	row.map(TryInto::try_into).transpose()
}

pub(crate) async fn get_content_entry_by_file(
	content_set_id: &str,
	file_id: &str,
	pool: &SqlitePool,
) -> crate::Result<Option<ContentEntry>> {
	let row = sqlx::query_as::<_, ContentEntryRow>(
		"
		SELECT *
		FROM instance_content_entries
		WHERE content_set_id = ? AND file_id = ?
		ORDER BY modified_at DESC
		LIMIT 1
		",
	)
	.bind(content_set_id)
	.bind(file_id)
	.fetch_optional(pool)
	.await?;

	row.map(TryInto::try_into).transpose()
}

pub(crate) async fn upsert_content_entry_from_parts(
	input: UpsertContentEntry<'_>,
	pool: &SqlitePool,
) -> crate::Result<ContentEntry> {
	let existing_id = if let Some(file_id) = input.file_id {
		sqlx::query_scalar::<_, String>(
			"
			SELECT id
			FROM instance_content_entries
			WHERE content_set_id = ? AND file_id = ?
			ORDER BY modified_at DESC
			LIMIT 1
			",
		)
		.bind(input.content_set_id)
		.bind(file_id)
		.fetch_optional(pool)
		.await?
	} else if let (Some(project_id), Some(version_id)) =
		(input.project_id, input.version_id)
	{
		sqlx::query_scalar::<_, String>(
			"
			SELECT id
			FROM instance_content_entries
			WHERE content_set_id = ?
				AND project_id = ?
				AND version_id = ?
			ORDER BY modified_at DESC
			LIMIT 1
			",
		)
		.bind(input.content_set_id)
		.bind(project_id)
		.bind(version_id)
		.fetch_optional(pool)
		.await?
	} else {
		None
	};
	let now = Utc::now();
	let entry = ContentEntry {
		id: existing_id
			.unwrap_or_else(|| format!("content-entry:{}", Uuid::new_v4())),
		instance_id: input.instance_id.to_string(),
		content_set_id: input.content_set_id.to_string(),
		file_id: input.file_id.map(ToString::to_string),
		project_type: input.project_type,
		project_id: input.project_id.map(ToString::to_string),
		version_id: input.version_id.map(ToString::to_string),
		source_kind: input.source_kind,
		server_requirement: input.server_requirement,
		client_requirement: input.client_requirement,
		enabled: input.enabled,
		added_at: now,
		modified_at: now,
	};

	let added_at = get_content_entry_by_id(&entry.id, pool)
		.await?
		.map(|entry| entry.added_at)
		.unwrap_or(entry.added_at);

	sqlx::query(
		"
		INSERT INTO instance_content_entries (
			id,
			instance_id,
			content_set_id,
			file_id,
			project_type,
			project_id,
			version_id,
			source_kind,
			server_requirement,
			client_requirement,
			enabled,
			added_at,
			modified_at
		)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
		ON CONFLICT(id) DO UPDATE SET
			file_id = excluded.file_id,
			project_type = excluded.project_type,
			project_id = excluded.project_id,
			version_id = excluded.version_id,
			source_kind = excluded.source_kind,
			server_requirement = excluded.server_requirement,
			client_requirement = excluded.client_requirement,
			enabled = excluded.enabled,
			modified_at = excluded.modified_at
		",
	)
	.bind(entry.id.as_str())
	.bind(entry.instance_id.as_str())
	.bind(entry.content_set_id.as_str())
	.bind(entry.file_id.as_deref())
	.bind(entry.project_type.get_name())
	.bind(entry.project_id.as_deref())
	.bind(entry.version_id.as_deref())
	.bind(entry.source_kind.as_str())
	.bind(entry.server_requirement.as_str())
	.bind(entry.client_requirement.as_str())
	.bind(i64::from(entry.enabled))
	.bind(added_at.timestamp())
	.bind(entry.modified_at.timestamp())
	.execute(pool)
	.await?;

	get_content_entry_by_id(&entry.id, pool).await?.ok_or_else(|| {
		crate::ErrorKind::OtherError(format!(
			"Failed to read content entry {} after upsert",
			entry.id
		))
		.into()
	})
}

pub(crate) async fn set_content_entry_enabled_for_file(
	content_set_id: &str,
	file_id: &str,
	enabled: bool,
	pool: &SqlitePool,
) -> crate::Result<()> {
	sqlx::query(
		"
		UPDATE instance_content_entries
		SET enabled = ?, modified_at = ?
		WHERE content_set_id = ? AND file_id = ?
		",
	)
	.bind(i64::from(enabled))
	.bind(Utc::now().timestamp())
	.bind(content_set_id)
	.bind(file_id)
	.execute(pool)
	.await?;

	Ok(())
}

pub(crate) async fn remove_content_entries_for_file(
	content_set_id: &str,
	file_id: &str,
	pool: &SqlitePool,
) -> crate::Result<()> {
	sqlx::query(
		"
		DELETE FROM instance_content_entries
		WHERE content_set_id = ? AND file_id = ?
		",
	)
	.bind(content_set_id)
	.bind(file_id)
	.execute(pool)
	.await?;

	Ok(())
}

pub(crate) async fn upsert_content_update_check(
	content_entry_id: &str,
	update_channel: ReleaseChannel,
	update_version_id: Option<&str>,
	pool: &SqlitePool,
) -> crate::Result<()> {
	sqlx::query(
		"
		INSERT INTO instance_content_update_checks (
			content_entry_id,
			update_channel,
			update_version_id,
			checked_at
		)
		VALUES (?, ?, ?, ?)
		ON CONFLICT(content_entry_id) DO UPDATE SET
			update_channel = excluded.update_channel,
			update_version_id = excluded.update_version_id,
			checked_at = excluded.checked_at
		",
	)
	.bind(content_entry_id)
	.bind(update_channel.key())
	.bind(update_version_id)
	.bind(Utc::now().timestamp())
	.execute(pool)
	.await?;

	Ok(())
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
