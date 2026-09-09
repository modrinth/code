use super::super::synced_options::instance_dir;
use super::SERVERS_FILE;
use super::codec::{read_servers, servers_from_bytes};
use super::operations::{compose_instance, effective};
use super::storage::{load_local, write_local_rows};
use super::types::{LocalServer, ServerSource};
use crate::state::{CachedEntry, InstanceLink, InstanceMetadata};
use crate::util::fetch::{DownloadMeta, DownloadReason, fetch_mirrors};
use crate::{ErrorKind, State};
use async_zip::base::read::seek::ZipFileReader;
use quartz_nbt::NbtCompound;
use std::io::Cursor;
use uuid::Uuid;

pub async fn capture_modpack_servers(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    let path = instance_dir(&metadata, &state).join(SERVERS_FILE);
    let servers = read_servers(&path).await?;
    replace_modpack_servers(&metadata, servers, &state).await?;
    if effective(&metadata, &state).await? {
        compose_instance(&metadata, &state).await?;
    }
    Ok(())
}

pub async fn clear_modpack_servers(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    replace_modpack_servers(&metadata, Vec::new(), &state).await?;
    if effective(&metadata, &state).await? {
        compose_instance(&metadata, &state).await?;
    }
    Ok(())
}

async fn replace_modpack_servers(
    metadata: &InstanceMetadata,
    servers: Vec<NbtCompound>,
    state: &State,
) -> crate::Result<()> {
    let mut local = servers
        .into_iter()
        .enumerate()
        .map(|(position, data)| LocalServer {
            id: Uuid::new_v4().to_string(),
            source: ServerSource::Modpack,
            excluded_synced_server_id: None,
            data,
            position: position as i64,
        })
        .collect::<Vec<_>>();
    let mut retained = load_local(&metadata.instance.id, state)
        .await?
        .into_iter()
        .filter(|server| server.source != ServerSource::Modpack)
        .collect::<Vec<_>>();
    for server in &mut retained {
        server.position = local.len() as i64;
        local.push(server.clone());
    }
    let mut tx = state.pool.begin().await?;
    write_local_rows(&mut tx, &metadata.instance.id, &local).await?;
    let version_id = modpack_version_id(&metadata.link);
    sqlx::query!(
        "
		INSERT INTO instance_server_pack_state (instance_id, version_id)
		VALUES (?, ?)
		ON CONFLICT(instance_id) DO UPDATE SET
			version_id = excluded.version_id
		",
        metadata.instance.id,
        version_id,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub(super) async fn pack_state_matches_link(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<bool> {
    let row = sqlx::query!(
        "
		SELECT version_id
		FROM instance_server_pack_state
		WHERE instance_id = ?
		",
        metadata.instance.id,
    )
    .fetch_optional(&state.pool)
    .await?;
    Ok(row.is_some_and(|row| {
        row.version_id.as_deref() == modpack_version_id(&metadata.link)
    }))
}

pub(super) async fn pack_state_exists(
    instance_id: &str,
    state: &State,
) -> crate::Result<bool> {
    Ok(sqlx::query_scalar!(
        r#"
		SELECT EXISTS(
			SELECT 1 FROM instance_server_pack_state WHERE instance_id = ?
		) AS "exists!: bool"
		"#,
        instance_id,
    )
    .fetch_one(&state.pool)
    .await?)
}

pub(super) async fn reconstruct_modpack_servers(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let version_id = modpack_version_id(&metadata.link).ok_or_else(|| {
        ErrorKind::InputError(
            "This modpack does not have a recoverable Modrinth version."
                .to_string(),
        )
    })?;
    let version = CachedEntry::get_version(
        version_id,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| {
        ErrorKind::InputError(format!(
            "Modpack version {version_id} could not be found"
        ))
    })?;
    let primary_file = version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| {
            ErrorKind::InputError(format!(
                "Modpack version {version_id} has no downloadable file"
            ))
        })?;
    let download_meta = DownloadMeta {
        reason: DownloadReason::Modpack,
        game_version: metadata.applied_content_set.game_version.clone(),
        loader: metadata.applied_content_set.loader.as_str().to_string(),
        dependent_on: Some(version_id.to_string()),
    };
    let mrpack = fetch_mirrors(
        &[&primary_file.url],
        primary_file.hashes.get("sha1").map(String::as_str),
        Some(&download_meta),
        None,
        &state.api_semaphore,
        &state.pool,
    )
    .await?;
    let mut archive = ZipFileReader::with_tokio(Cursor::new(&mrpack)).await?;
    let mut selected = None;
    for (index, entry) in archive.file().entries().iter().enumerate() {
        let Ok(filename) = entry.filename().as_str() else {
            continue;
        };
        let priority = if filename == "client-overrides/servers.dat" {
            2
        } else if filename == "overrides/servers.dat" {
            1
        } else {
            continue;
        };
        if selected
            .is_none_or(|(selected_priority, _)| priority >= selected_priority)
        {
            selected = Some((priority, index));
        }
    }

    let servers = if let Some((_, index)) = selected {
        let mut bytes = Vec::new();
        let mut reader = archive.reader_with_entry(index).await?;
        reader.read_to_end_checked(&mut bytes).await?;
        servers_from_bytes(bytes)?
    } else {
        Vec::new()
    };
    replace_modpack_servers(metadata, servers, state).await
}

pub(super) fn is_modpack_link(link: &InstanceLink) -> bool {
    matches!(
        link,
        InstanceLink::ModrinthModpack { .. }
            | InstanceLink::ImportedModpack { .. }
            | InstanceLink::SharedInstance { .. }
    )
}

fn modpack_version_id(link: &InstanceLink) -> Option<&str> {
    match link {
        InstanceLink::ModrinthModpack { version_id, .. } => Some(version_id),
        InstanceLink::ServerProjectModpack {
            content_version_id, ..
        } => Some(content_version_id),
        InstanceLink::ImportedModpack {
            version_id: Some(version_id),
            ..
        } => Some(version_id),
        InstanceLink::SharedInstance {
            modpack_version_id: Some(version_id),
            ..
        } => Some(version_id),
        _ => None,
    }
}
