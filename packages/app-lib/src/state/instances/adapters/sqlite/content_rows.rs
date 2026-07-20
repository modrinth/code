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
    let row = sqlx::query_as!(
        ContentSetRow,
        "
		SELECT cs.*
		FROM instances i
		INNER JOIN instance_content_sets cs
			ON cs.id = i.applied_content_set_id
		WHERE i.id = ?
		",
        instance_id,
    )
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
    let row = sqlx::query_as!(
        ContentSetRow,
        "
		SELECT *
		FROM instance_content_sets
		WHERE id = ?
		",
        content_set_id,
    )
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
    let rows = sqlx::query_as!(
        ContentSetRow,
        "
		SELECT *
		FROM instance_content_sets
		WHERE instance_id = ?
		ORDER BY created ASC, id ASC
		",
        instance_id,
    )
    .fetch_all(exec)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub(crate) async fn insert_content_set(
    content_set: &ContentSet,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let id = content_set.id.as_str();
    let instance_id = content_set.instance_id.as_str();
    let name = content_set.name.as_str();
    let source_kind = content_set.source_kind.as_str();
    let status = content_set.status.as_str();
    let game_version = content_set.game_version.as_str();
    let protocol_version =
        content_set.protocol_version.map(|value| value as i64);
    let loader = content_set.loader.as_str();
    let loader_version = content_set.loader_version.as_deref();
    let created = content_set.created.timestamp();
    let modified = content_set.modified.timestamp();

    sqlx::query!(
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
        modified,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn update_content_set(
    content_set: &ContentSet,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let id = content_set.id.as_str();
    let name = content_set.name.as_str();
    let source_kind = content_set.source_kind.as_str();
    let status = content_set.status.as_str();
    let game_version = content_set.game_version.as_str();
    let protocol_version =
        content_set.protocol_version.map(|value| value as i64);
    let loader = content_set.loader.as_str();
    let loader_version = content_set.loader_version.as_deref();
    let modified = content_set.modified.timestamp();

    sqlx::query!(
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
        name,
        source_kind,
        status,
        game_version,
        protocol_version,
        loader,
        loader_version,
        modified,
        id,
    )
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
    let rows = sqlx::query_as!(
        ContentSetRemoteRefRow,
        "
		SELECT *
		FROM instance_content_set_remote_refs
		WHERE content_set_id = ?
		ORDER BY ref_type ASC
		",
        content_set_id,
    )
    .fetch_all(exec)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub(crate) async fn upsert_content_set_remote_ref(
    remote_ref: &ContentSetRemoteRef,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let content_set_id = remote_ref.content_set_id.as_str();
    let ref_type = remote_ref.ref_type.as_str();
    let ref_id = remote_ref.ref_id.as_str();

    sqlx::query!(
        "
		INSERT INTO instance_content_set_remote_refs (
			content_set_id,
			ref_type,
			ref_id
		)
		VALUES (?, ?, ?)
		ON CONFLICT (content_set_id, ref_type) DO UPDATE SET
			ref_id = excluded.ref_id
		",
        content_set_id,
        ref_type,
        ref_id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn delete_content_set_remote_ref(
    content_set_id: &str,
    ref_type: ContentSetRemoteRefType,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let ref_type = ref_type.as_str();

    sqlx::query!(
        "
		DELETE FROM instance_content_set_remote_refs
		WHERE content_set_id = ? AND ref_type = ?
		",
        content_set_id,
        ref_type,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn get_content_set_sync_state<'e, E>(
    content_set_id: &str,
    exec: E,
) -> crate::Result<Option<ContentSetSyncState>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let row = sqlx::query_as!(
        ContentSetSyncStateRow,
        "
		SELECT *
		FROM instance_content_set_sync_state
		WHERE content_set_id = ?
		",
        content_set_id,
    )
    .fetch_optional(exec)
    .await?;

    row.map(TryInto::try_into).transpose()
}

pub(crate) async fn upsert_content_set_sync_state(
    sync_state: &ContentSetSyncState,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let content_set_id = sync_state.content_set_id.as_str();
    let provider = sync_state.provider.as_str();
    let applied_update_id = sync_state.applied_update_id.as_deref();
    let latest_available_update_id =
        sync_state.latest_available_update_id.as_deref();
    let checked_at = sync_state.checked_at.map(|value| value.timestamp());
    let status = sync_state.status.as_str();

    sqlx::query!(
        "
		INSERT INTO instance_content_set_sync_state (
			content_set_id,
			provider,
			applied_update_id,
			latest_available_update_id,
			checked_at,
			status
		)
		VALUES (?, ?, ?, ?, ?, ?)
		ON CONFLICT (content_set_id) DO UPDATE SET
			provider = excluded.provider,
			applied_update_id = excluded.applied_update_id,
			latest_available_update_id = excluded.latest_available_update_id,
			checked_at = excluded.checked_at,
			status = excluded.status
		",
        content_set_id,
        provider,
        applied_update_id,
        latest_available_update_id,
        checked_at,
        status,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn delete_content_set_sync_state(
    content_set_id: &str,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    sqlx::query!(
        "
		DELETE FROM instance_content_set_sync_state
		WHERE content_set_id = ?
		",
        content_set_id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn get_instance_files<'e, E>(
    instance_id: &str,
    exec: E,
) -> crate::Result<Vec<InstanceFile>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let rows = sqlx::query_as!(
        InstanceFileRow,
        "
		SELECT *
		FROM instance_files
		WHERE instance_id = ?
		ORDER BY relative_path ASC
		",
        instance_id,
    )
    .fetch_all(exec)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub(crate) async fn mark_instance_files_missing(
    instance_id: &str,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let modified_at = Utc::now().timestamp();

    sqlx::query!(
        "
		UPDATE instance_files
		SET
			missing = 1,
			modified_at = ?
		WHERE instance_id = ?
		",
        modified_at,
        instance_id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn upsert_instance_file(
    file: &InstanceFile,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let id = file.id.as_str();
    let instance_id = file.instance_id.as_str();
    let relative_path = file.relative_path.as_str();
    let file_name = file.file_name.as_str();
    let enabled = i64::from(file.enabled);
    let sha1 = file.sha1.as_str();
    let size = file.size as i64;
    let missing = i64::from(file.missing);
    let added_at = file.added_at.timestamp();
    let modified_at = file.modified_at.timestamp();

    sqlx::query!(
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
        id,
        instance_id,
        relative_path,
        file_name,
        enabled,
        sha1,
        size,
        missing,
        added_at,
        modified_at,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn insert_content_entry(
    entry: &ContentEntry,
    tx: &mut Transaction<'_, Sqlite>,
) -> crate::Result<()> {
    let id = entry.id.as_str();
    let instance_id = entry.instance_id.as_str();
    let content_set_id = entry.content_set_id.as_str();
    let file_id = entry.file_id.as_deref();
    let project_type = entry.project_type.get_name();
    let project_id = entry.project_id.as_deref();
    let version_id = entry.version_id.as_deref();
    let source_kind = entry.source_kind.as_str();
    let server_requirement = entry.server_requirement.as_str();
    let client_requirement = entry.client_requirement.as_str();
    let enabled = i64::from(entry.enabled);
    let added_at = entry.added_at.timestamp();
    let modified_at = entry.modified_at.timestamp();

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
		",
    )
    .bind(id)
    .bind(instance_id)
    .bind(content_set_id)
    .bind(file_id)
    .bind(project_type)
    .bind(project_id)
    .bind(version_id)
    .bind(source_kind)
    .bind(server_requirement)
    .bind(client_requirement)
    .bind(enabled)
    .bind(added_at)
    .bind(modified_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn restore_instance_content_snapshot(
    instance_id: &str,
    files: &[InstanceFile],
    entries: &[ContentEntry],
    pool: &SqlitePool,
) -> crate::Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "
		DELETE FROM instance_content_entries
		WHERE instance_id = ?
		",
    )
    .bind(instance_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "
		DELETE FROM instance_files
		WHERE instance_id = ?
		",
    )
    .bind(instance_id)
    .execute(&mut *tx)
    .await?;

    for file in files {
        upsert_instance_file(file, &mut tx).await?;
    }
    for entry in entries {
        insert_content_entry(entry, &mut tx).await?;
    }

    tx.commit().await?;
    Ok(())
}

pub(crate) async fn get_content_entries<'e, E>(
    content_set_id: &str,
    exec: E,
) -> crate::Result<Vec<ContentEntry>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let rows = sqlx::query_as!(
        ContentEntryRow,
        "
		SELECT *
		FROM instance_content_entries
		WHERE content_set_id = ?
		ORDER BY added_at ASC, id ASC
		",
        content_set_id,
    )
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
    let row = sqlx::query_as!(
        ContentUpdateCheckRow,
        "
		SELECT *
		FROM instance_content_update_checks
		WHERE content_entry_id = ?
		",
        content_entry_id,
    )
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
    let row = sqlx::query_as!(
        InstanceFileRow,
        "
		SELECT *
		FROM instance_files
		WHERE instance_id = ? AND relative_path = ?
		",
        instance_id,
        relative_path,
    )
    .fetch_optional(pool)
    .await?;

    row.map(TryInto::try_into).transpose()
}

pub(crate) async fn upsert_instance_file_from_parts(
    input: UpsertInstanceFile<'_>,
    pool: &SqlitePool,
) -> crate::Result<InstanceFile> {
    let existing = get_instance_file_by_relative_path(
        input.instance_id,
        input.relative_path,
        pool,
    )
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
    let enabled = i64::from(enabled);
    let modified_at = Utc::now().timestamp();
    let mut tx = pool.begin().await?;

    let source_id = sqlx::query_scalar!(
        "
		SELECT id
		FROM instance_files
		WHERE instance_id = ? AND relative_path = ?
		",
        instance_id,
        old_relative_path,
    )
    .fetch_optional(&mut *tx)
    .await?;
    let target_id = sqlx::query_scalar!(
        "
		SELECT id
		FROM instance_files
		WHERE instance_id = ? AND relative_path = ?
		",
        instance_id,
        new_relative_path,
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let (Some(source_id), Some(target_id)) =
        (source_id.as_deref(), target_id.as_deref())
        && source_id != target_id
    {
        sqlx::query!(
            "
				DELETE FROM instance_content_entries
				WHERE id IN (
					SELECT target_entry.id
					FROM instance_content_entries target_entry
					WHERE target_entry.instance_id = ?
						AND target_entry.file_id = ?
						AND EXISTS (
							SELECT 1
							FROM instance_content_entries source_entry
							WHERE source_entry.instance_id = target_entry.instance_id
								AND source_entry.content_set_id = target_entry.content_set_id
								AND source_entry.file_id = ?
						)
				)
				",
            instance_id,
            target_id,
            source_id,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "
				UPDATE instance_content_entries
				SET file_id = ?, modified_at = ?
				WHERE instance_id = ? AND file_id = ?
				",
            source_id,
            modified_at,
            instance_id,
            target_id,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "
				DELETE FROM instance_files
				WHERE id = ?
				",
            target_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query!(
        "
		UPDATE instance_files
		SET
			relative_path = ?,
			file_name = ?,
			enabled = ?,
			missing = 0,
			modified_at = ?
		WHERE instance_id = ? AND relative_path = ?
		",
        new_relative_path,
        new_file_name,
        enabled,
        modified_at,
        instance_id,
        old_relative_path,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    get_instance_file_by_relative_path(instance_id, new_relative_path, pool)
        .await
}

pub(crate) async fn remove_instance_file_by_relative_path(
    instance_id: &str,
    relative_path: &str,
    pool: &SqlitePool,
) -> crate::Result<()> {
    sqlx::query!(
        "
		DELETE FROM instance_files
		WHERE instance_id = ? AND relative_path = ?
		",
        instance_id,
        relative_path,
    )
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
    let row = sqlx::query_as!(
        ContentEntryRow,
        "
		SELECT *
		FROM instance_content_entries
		WHERE id = ?
		",
        id,
    )
    .fetch_optional(pool)
    .await?;

    row.map(TryInto::try_into).transpose()
}

pub(crate) async fn get_content_entry_by_file(
    content_set_id: &str,
    file_id: &str,
    pool: &SqlitePool,
) -> crate::Result<Option<ContentEntry>> {
    let row = sqlx::query_as!(
        ContentEntryRow,
        "
		SELECT *
		FROM instance_content_entries
		WHERE content_set_id = ? AND file_id = ?
		ORDER BY modified_at DESC
		LIMIT 1
		",
        content_set_id,
        file_id,
    )
    .fetch_optional(pool)
    .await?;

    row.map(TryInto::try_into).transpose()
}

pub(crate) async fn upsert_content_entry_from_parts(
    input: UpsertContentEntry<'_>,
    pool: &SqlitePool,
) -> crate::Result<ContentEntry> {
    let existing_id = if let Some(file_id) = input.file_id {
        sqlx::query_scalar!(
            "
			SELECT id
			FROM instance_content_entries
			WHERE content_set_id = ? AND file_id = ?
			ORDER BY modified_at DESC
			LIMIT 1
			",
            input.content_set_id,
            file_id,
        )
        .fetch_optional(pool)
        .await?
    } else if let (Some(project_id), Some(version_id)) =
        (input.project_id, input.version_id)
    {
        sqlx::query_scalar!(
            "
			SELECT id
			FROM instance_content_entries
			WHERE content_set_id = ?
				AND project_id = ?
				AND version_id = ?
			ORDER BY modified_at DESC
			LIMIT 1
			",
            input.content_set_id,
            project_id,
            version_id,
        )
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
    let id = entry.id.as_str();
    let entry_instance_id = entry.instance_id.as_str();
    let content_set_id = entry.content_set_id.as_str();
    let file_id = entry.file_id.as_deref();
    let project_type = entry.project_type.get_name();
    let project_id = entry.project_id.as_deref();
    let version_id = entry.version_id.as_deref();
    let source_kind = entry.source_kind.as_str();
    let server_requirement = entry.server_requirement.as_str();
    let client_requirement = entry.client_requirement.as_str();
    let enabled = i64::from(entry.enabled);
    let added_at = added_at.timestamp();
    let modified_at = entry.modified_at.timestamp();

    sqlx::query!(
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
        id,
        entry_instance_id,
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
        modified_at,
    )
    .execute(pool)
    .await?;

    get_content_entry_by_id(&entry.id, pool)
        .await?
        .ok_or_else(|| {
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
) -> crate::Result<bool> {
    let enabled = i64::from(enabled);
    let modified_at = Utc::now().timestamp();

    let result = sqlx::query!(
        "
		UPDATE instance_content_entries
		SET enabled = ?, modified_at = ?
		WHERE content_set_id = ? AND file_id = ?
		",
        enabled,
        modified_at,
        content_set_id,
        file_id,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub(crate) async fn remove_content_entries_for_file(
    content_set_id: &str,
    file_id: &str,
    pool: &SqlitePool,
) -> crate::Result<()> {
    sqlx::query!(
        "
		DELETE FROM instance_content_entries
		WHERE content_set_id = ? AND file_id = ?
		",
        content_set_id,
        file_id,
    )
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
    let update_channel = update_channel.key();
    let checked_at = Utc::now().timestamp();

    sqlx::query!(
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
        content_entry_id,
        update_channel,
        update_version_id,
        checked_at,
    )
    .execute(pool)
    .await?;

    Ok(())
}

fn project_type_from_str(value: &str) -> crate::Result<ProjectType> {
    ProjectType::from_name(value).ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Unknown content project type {value}"
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
