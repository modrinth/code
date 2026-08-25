use super::synced_options::{
    detach_link, ensure_link, instance_dir, instance_is_running,
    instance_option_enabled, nbt_from_bytes, nbt_to_bytes, read_nbt_file,
    record_materialization, safe_instance_id, sha1_file,
    sync_files_are_protected, synced_options_path, write_nbt_file,
};
use crate::state::{CachedEntry, InstanceLink, InstanceMetadata, SyncedOption};
use crate::util::fetch::{DownloadMeta, DownloadReason, fetch_mirrors};
use crate::{ErrorKind, State};
use async_zip::base::read::seek::ZipFileReader;
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const SERVERS_FILE: &str = "servers.dat";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerSource {
    UserSynced,
    Modpack,
    LinkedServerProject,
    LocalDesynced,
}

impl ServerSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::UserSynced => "user_synced",
            Self::Modpack => "modpack",
            Self::LinkedServerProject => "linked_server_project",
            Self::LocalDesynced => "local_desynced",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "user_synced" => Some(Self::UserSynced),
            "modpack" => Some(Self::Modpack),
            "linked_server_project" => Some(Self::LinkedServerProject),
            "local_desynced" => Some(Self::LocalDesynced),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesyncServerMode {
    KeepInOtherInstances,
    RemoveFromOtherInstances,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncedServer {
    pub id: String,
    pub name: String,
    pub address: String,
    pub accept_textures: Option<bool>,
}

#[derive(Clone, Debug)]
pub(crate) struct ServerRecord {
    pub id: String,
    pub source: ServerSource,
    pub data: NbtCompound,
}

impl ServerRecord {
    pub fn name(&self) -> String {
        self.data
            .get::<_, &str>("name")
            .unwrap_or_default()
            .to_string()
    }

    pub fn address(&self) -> String {
        self.data
            .get::<_, &str>("ip")
            .unwrap_or_default()
            .to_string()
    }

    pub fn icon(&self) -> Option<String> {
        self.data.get::<_, &str>("icon").ok().map(ToOwned::to_owned)
    }

    pub fn hidden(&self) -> bool {
        self.data.get::<_, i8>("hidden").unwrap_or(0) != 0
    }

    pub fn accept_textures(&self) -> Option<bool> {
        self.data
            .get::<_, i8>("acceptTextures")
            .ok()
            .map(|value| value != 0)
    }
}

#[derive(Clone)]
struct CanonicalServer {
    id: String,
    data: NbtCompound,
}

#[derive(Clone)]
struct LocalServer {
    id: String,
    source: ServerSource,
    canonical_id: Option<String>,
    data: NbtCompound,
    position: i64,
}

#[derive(Clone)]
struct SnapshotServer {
    id: String,
    source: ServerSource,
    data: NbtCompound,
}

pub(super) async fn seed_servers(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    if is_modpack_link(&metadata.link)
        && (!baseline_matches_link(metadata, state).await?
            || !baseline_was_reconstructed(&metadata.instance.id, state)
                .await?)
        && let Err(error) = reconstruct_modpack_servers(metadata, state).await
    {
        tracing::warn!(
            "Failed to reconstruct the server baseline for {}: {error}",
            metadata.instance.id
        );
    }
    let local_entries = load_local(&metadata.instance.id, state).await?;
    if is_modpack_link(&metadata.link)
        && !baseline_was_reconstructed(&metadata.instance.id, state).await?
    {
        return Err(ErrorKind::InputError(
			"This linked modpack cannot be used as the multiplayer base until its supplied server list has been reconstructed. Choose an unmanaged instance instead."
				.to_string(),
		)
		.into());
    }
    let path = instance_dir(metadata, state).join(SERVERS_FILE);
    let mut servers = read_servers(&path).await?;
    if is_modpack_link(&metadata.link) {
        for local in &local_entries {
            if let Some(index) = servers.iter().position(|data| {
                data == &local.data
                    || server_address(data) == server_address(&local.data)
            }) {
                servers.remove(index);
            }
        }
    } else {
        sqlx::query!(
            "DELETE FROM instance_server_entries WHERE instance_id = ?",
            metadata.instance.id,
        )
        .execute(&state.pool)
        .await?;
    }
    sqlx::query!(
        "DELETE FROM instance_server_snapshots WHERE instance_id = ?",
        metadata.instance.id,
    )
    .execute(&state.pool)
    .await?;
    let canonical = servers
        .into_iter()
        .map(|data| CanonicalServer {
            id: Uuid::new_v4().to_string(),
            data,
        })
        .collect::<Vec<_>>();
    write_canonical(state, &canonical).await?;
    regenerate_servers(state).await
}

pub(super) async fn ensure_servers(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    if is_modpack_link(&metadata.link)
        && !baseline_matches_link(metadata, state).await?
        && let Err(error) = reconstruct_modpack_servers(metadata, state).await
    {
        tracing::warn!(
            "Failed to reconstruct the server baseline for {}; preserving its current server list locally: {error}",
            metadata.instance.id
        );
        let path = instance_dir(metadata, state).join(SERVERS_FILE);
        let servers = read_servers(&path).await?;
        replace_modpack_servers(metadata, servers, false, state).await?;
    }
    let canonical_path = canonical_path(state);
    if !canonical_path.exists() {
        if !is_modpack_link(&metadata.link) {
            let servers =
                read_servers(&instance_dir(metadata, state).join(SERVERS_FILE))
                    .await?;
            let canonical = servers
                .into_iter()
                .map(|data| CanonicalServer {
                    id: Uuid::new_v4().to_string(),
                    data,
                })
                .collect::<Vec<_>>();
            write_canonical(state, &canonical).await?;
        } else {
            write_canonical(state, &[]).await?;
        }
    }
    compose_instance(metadata, state).await
}

pub(super) async fn detach_servers(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let generated = generated_path(state, &metadata.instance.id);
    let local = instance_dir(metadata, state).join(SERVERS_FILE);
    detach_link(&generated, &local).await
}

pub async fn capture_modpack_servers(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    let path = instance_dir(&metadata, &state).join(SERVERS_FILE);
    let servers = read_servers(&path).await?;
    replace_modpack_servers(&metadata, servers, true, &state).await?;
    if effective(&metadata, &state).await? {
        compose_instance(&metadata, &state).await?;
    }
    Ok(())
}

pub async fn clear_modpack_servers(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    replace_modpack_servers(&metadata, Vec::new(), true, &state).await?;
    if effective(&metadata, &state).await? {
        compose_instance(&metadata, &state).await?;
    }
    Ok(())
}

async fn replace_modpack_servers(
    metadata: &InstanceMetadata,
    servers: Vec<NbtCompound>,
    reconstructed: bool,
    state: &State,
) -> crate::Result<()> {
    let mut tx = state.pool.begin().await?;
    sqlx::query!(
		"DELETE FROM instance_server_entries WHERE instance_id = ? AND source = 'modpack'",
		metadata.instance.id,
	)
	.execute(&mut *tx)
	.await?;
    for (position, server) in servers.into_iter().enumerate() {
        let id = Uuid::new_v4().to_string();
        let source = ServerSource::Modpack.as_str();
        let nbt = nbt_to_bytes(&server)?;
        let position = position as i64;
        sqlx::query!(
            "
			INSERT INTO instance_server_entries
				(instance_id, id, source, canonical_id, nbt, position)
			VALUES (?, ?, ?, NULL, ?, ?)
			",
            metadata.instance.id,
            id,
            source,
            nbt,
            position,
        )
        .execute(&mut *tx)
        .await?;
    }
    let version_id = modpack_version_id(&metadata.link);
    sqlx::query!(
        "
		INSERT INTO instance_server_baselines
			(instance_id, version_id, reconstructed)
		VALUES (?, ?, ?)
		ON CONFLICT(instance_id) DO UPDATE SET
			version_id = excluded.version_id,
			reconstructed = excluded.reconstructed
		",
        metadata.instance.id,
        version_id,
        reconstructed,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn baseline_matches_link(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<bool> {
    let row = sqlx::query!(
        "
		SELECT version_id
		FROM instance_server_baselines
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

async fn baseline_was_reconstructed(
    instance_id: &str,
    state: &State,
) -> crate::Result<bool> {
    Ok(sqlx::query_scalar!(
        r#"
		SELECT reconstructed AS "reconstructed!: bool"
		FROM instance_server_baselines
		WHERE instance_id = ?
		"#,
        instance_id,
    )
    .fetch_optional(&state.pool)
    .await?
    .unwrap_or(false))
}

async fn reconstruct_modpack_servers(
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
    replace_modpack_servers(metadata, servers, true, state).await
}

pub(super) async fn reconcile_servers(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    if !effective(metadata, state).await? {
        return Ok(());
    }
    let local_path = instance_dir(metadata, state).join(SERVERS_FILE);
    if !local_path.exists() {
        return compose_instance(metadata, state).await;
    }
    let expected = sqlx::query_scalar!(
        "
		SELECT expected_sha1 FROM synced_option_materializations
		WHERE instance_id = ? AND option = 'multiplayer_servers' AND family = ''
		",
        metadata.instance.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .flatten();
    let actual = sha1_file(&local_path).await?;
    if expected.as_deref() == Some(actual.as_str()) {
        return Ok(());
    }

    let current = read_servers(&local_path).await?;
    let snapshots = load_snapshots(&metadata.instance.id, state).await?;
    let mut matched = HashSet::new();
    let mut canonical = read_canonical(state).await?;
    let mut locals = load_local(&metadata.instance.id, state).await?;

    for (position, data) in current.into_iter().enumerate() {
        let snapshot = snapshots
            .iter()
            .filter(|snapshot| !matched.contains(&snapshot.id))
            .find(|snapshot| snapshot.data == data)
            .or_else(|| {
                snapshots
                    .iter()
                    .filter(|snapshot| !matched.contains(&snapshot.id))
                    .find(|snapshot| {
                        server_address(&snapshot.data) == server_address(&data)
                    })
            })
            .or_else(|| snapshots.get(position));

        if let Some(snapshot) = snapshot {
            matched.insert(snapshot.id.clone());
            match snapshot.source {
                ServerSource::UserSynced => {
                    if let Some(server) = canonical
                        .iter_mut()
                        .find(|server| server.id == snapshot.id)
                    {
                        server.data = data;
                    }
                }
                _ => {
                    if let Some(server) = locals
                        .iter_mut()
                        .find(|server| server.id == snapshot.id)
                    {
                        server.data = data;
                        server.position = position as i64;
                    }
                }
            }
        } else {
            canonical.push(CanonicalServer {
                id: Uuid::new_v4().to_string(),
                data,
            });
        }
    }

    canonical.retain(|server| {
        !snapshots.iter().any(|snapshot| {
            snapshot.source == ServerSource::UserSynced
                && snapshot.id == server.id
                && !matched.contains(&snapshot.id)
        })
    });
    write_canonical(state, &canonical).await?;
    write_local(&metadata.instance.id, &locals, state).await?;
    regenerate_servers(state).await
}

pub(crate) async fn list_server_records(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<Vec<ServerRecord>> {
    if effective(metadata, state).await? {
        let snapshots = load_snapshots(&metadata.instance.id, state).await?;
        if !snapshots.is_empty() {
            return Ok(snapshots
                .into_iter()
                .map(|snapshot| ServerRecord {
                    id: snapshot.id,
                    source: snapshot.source,
                    data: snapshot.data,
                })
                .collect());
        }
    }
    Ok(
        read_servers(&instance_dir(metadata, state).join(SERVERS_FILE))
            .await?
            .into_iter()
            .map(|data| ServerRecord {
                id: Uuid::new_v4().to_string(),
                source: ServerSource::LocalDesynced,
                data,
            })
            .collect(),
    )
}

pub(crate) async fn add_user_server(
    metadata: &InstanceMetadata,
    mut data: NbtCompound,
    state: &State,
) -> crate::Result<String> {
    let id = Uuid::new_v4().to_string();
    if effective(metadata, state).await? {
        let mut canonical = read_canonical(state).await?;
        canonical.push(CanonicalServer {
            id: id.clone(),
            data,
        });
        write_canonical(state, &canonical).await?;
        regenerate_servers(state).await?;
    } else {
        let path = instance_dir(metadata, state).join(SERVERS_FILE);
        let mut servers = read_servers(&path).await?;
        let insert_index = servers
            .iter()
            .position(server_hidden)
            .unwrap_or(servers.len());
        servers.insert(insert_index, std::mem::take(&mut data));
        write_servers(&path, &servers).await?;
    }
    Ok(id)
}

pub(crate) async fn update_server(
    metadata: &InstanceMetadata,
    server_id: &str,
    data: NbtCompound,
    state: &State,
) -> crate::Result<()> {
    if effective(metadata, state).await? {
        let snapshots = load_snapshots(&metadata.instance.id, state).await?;
        let snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.id == server_id)
            .ok_or_else(|| {
                ErrorKind::InputError("Unknown server".to_string())
            })?;
        if snapshot.source == ServerSource::UserSynced {
            let mut canonical = read_canonical(state).await?;
            let server = canonical
                .iter_mut()
                .find(|server| server.id == server_id)
                .ok_or_else(|| {
                    ErrorKind::InputError("Unknown server".to_string())
                })?;
            server.data = data;
            write_canonical(state, &canonical).await?;
        } else {
            let mut locals = load_local(&metadata.instance.id, state).await?;
            let server = locals
                .iter_mut()
                .find(|server| server.id == server_id)
                .ok_or_else(|| {
                    ErrorKind::InputError("Unknown server".to_string())
                })?;
            server.data = data;
            write_local(&metadata.instance.id, &locals, state).await?;
        }
        regenerate_servers(state).await?;
        return Ok(());
    }
    Err(ErrorKind::InputError(
        "Stable server editing requires multiplayer syncing for this instance."
            .to_string(),
    )
    .into())
}

pub(crate) async fn update_server_by_index(
    metadata: &InstanceMetadata,
    index: usize,
    name: String,
    address: String,
    accept_textures: Option<bool>,
    state: &State,
) -> crate::Result<()> {
    if effective(metadata, state).await? {
        let records = list_server_records(metadata, state).await?;
        let record = records
            .get(index)
            .filter(|record| !record.hidden())
            .ok_or_else(|| {
                ErrorKind::InputError(format!(
                    "No editable server at index {index}"
                ))
            })?;
        let mut data = record.data.clone();
        update_server_data(&mut data, name, address, accept_textures);
        return update_server(metadata, &record.id, data, state).await;
    }
    let path = instance_dir(metadata, state).join(SERVERS_FILE);
    let mut servers = read_servers(&path).await?;
    let server = servers
        .get_mut(index)
        .filter(|server| !server_hidden(server))
        .ok_or_else(|| {
            ErrorKind::InputError(format!(
                "No editable server at index {index}"
            ))
        })?;
    update_server_data(server, name, address, accept_textures);
    write_servers(&path, &servers).await
}

pub(crate) async fn remove_server(
    metadata: &InstanceMetadata,
    server_id: &str,
    state: &State,
) -> crate::Result<()> {
    if !effective(metadata, state).await? {
        return Err(ErrorKind::InputError(
			"Stable server removal requires multiplayer syncing for this instance."
				.to_string(),
		)
		.into());
    }
    let snapshots = load_snapshots(&metadata.instance.id, state).await?;
    let snapshot = snapshots
        .iter()
        .find(|snapshot| snapshot.id == server_id)
        .ok_or_else(|| {
        ErrorKind::InputError("Unknown server".to_string())
    })?;
    if snapshot.source == ServerSource::UserSynced {
        let mut canonical = read_canonical(state).await?;
        canonical.retain(|server| server.id != server_id);
        write_canonical(state, &canonical).await?;
    } else {
        let mut locals = load_local(&metadata.instance.id, state).await?;
        locals.retain(|server| server.id != server_id);
        write_local(&metadata.instance.id, &locals, state).await?;
    }
    regenerate_servers(state).await
}

pub(crate) async fn remove_server_by_index(
    metadata: &InstanceMetadata,
    index: usize,
    state: &State,
) -> crate::Result<()> {
    if effective(metadata, state).await? {
        let records = list_server_records(metadata, state).await?;
        let record = records
            .get(index)
            .filter(|record| !record.hidden())
            .ok_or_else(|| {
                ErrorKind::InputError(format!(
                    "No removable server at index {index}"
                ))
            })?;
        return remove_server(metadata, &record.id, state).await;
    }
    let path = instance_dir(metadata, state).join(SERVERS_FILE);
    let mut servers = read_servers(&path).await?;
    if servers.get(index).is_none_or(server_hidden) {
        return Err(ErrorKind::InputError(format!(
            "No removable server at index {index}"
        ))
        .into());
    }
    servers.remove(index);
    write_servers(&path, &servers).await
}

pub async fn desync_server(
    instance_id: &str,
    server_id: &str,
    mode: DesyncServerMode,
) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    let mut canonical = read_canonical(&state).await?;
    let server = canonical
        .iter()
        .find(|server| server.id == server_id)
        .cloned()
        .ok_or_else(|| {
            ErrorKind::InputError(
                "Only a synced user server can be desynced.".to_string(),
            )
        })?;
    let mut locals = load_local(instance_id, &state).await?;
    locals.push(LocalServer {
        id: Uuid::new_v4().to_string(),
        source: ServerSource::LocalDesynced,
        canonical_id: (mode == DesyncServerMode::KeepInOtherInstances)
            .then(|| server.id.clone()),
        data: server.data,
        position: locals.len() as i64,
    });
    if mode == DesyncServerMode::RemoveFromOtherInstances {
        canonical.retain(|candidate| candidate.id != server_id);
        write_canonical(&state, &canonical).await?;
    }
    write_local(instance_id, &locals, &state).await?;
    regenerate_servers(&state).await?;
    let _ = metadata;
    Ok(())
}

pub async fn list_synced_servers() -> crate::Result<Vec<SyncedServer>> {
    let state = State::get().await?;
    Ok(read_canonical(&state)
        .await?
        .into_iter()
        .map(|server| {
            let CanonicalServer { id, data } = server;
            SyncedServer {
                id,
                name: data
                    .get::<_, &str>("name")
                    .unwrap_or_default()
                    .to_string(),
                address: server_address(&data),
                accept_textures: data
                    .get::<_, i8>("acceptTextures")
                    .ok()
                    .map(|value| value != 0),
            }
        })
        .collect())
}

pub async fn update_synced_server(server: SyncedServer) -> crate::Result<()> {
    let state = State::get().await?;
    let mut canonical = read_canonical(&state).await?;
    let target = canonical
        .iter_mut()
        .find(|candidate| candidate.id == server.id)
        .ok_or_else(|| {
            ErrorKind::InputError("Unknown synced server".to_string())
        })?;
    update_server_data(
        &mut target.data,
        server.name,
        server.address,
        server.accept_textures,
    );
    write_canonical(&state, &canonical).await?;
    regenerate_servers(&state).await
}

pub async fn remove_synced_server(server_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let mut canonical = read_canonical(&state).await?;
    let previous_len = canonical.len();
    canonical.retain(|server| server.id != server_id);
    if canonical.len() == previous_len {
        return Err(
            ErrorKind::InputError("Unknown synced server".to_string()).into()
        );
    }
    write_canonical(&state, &canonical).await?;
    regenerate_servers(&state).await
}

async fn compose_instance(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let canonical = read_canonical(state).await?;
    let locals = load_local(&metadata.instance.id, state).await?;
    let exclusions = locals
        .iter()
        .filter_map(|server| server.canonical_id.as_deref())
        .collect::<HashSet<_>>();
    let mut records = canonical
        .into_iter()
        .filter(|server| !exclusions.contains(server.id.as_str()))
        .map(|server| ServerRecord {
            id: server.id,
            source: ServerSource::UserSynced,
            data: server.data,
        })
        .collect::<Vec<_>>();
    let mut locals = locals;
    locals.sort_by_key(|server| server.position);
    records.extend(locals.into_iter().map(|server| ServerRecord {
        id: server.id,
        source: server.source,
        data: server.data,
    }));

    let generated = generated_path(state, &metadata.instance.id);
    write_servers(
        &generated,
        &records
            .iter()
            .map(|record| record.data.clone())
            .collect::<Vec<_>>(),
    )
    .await?;
    write_snapshots(&metadata.instance.id, &records, state).await?;
    let target = instance_dir(metadata, state).join(SERVERS_FILE);
    let mode = ensure_link(&generated, &target).await?;
    record_materialization(
        &metadata.instance.id,
        SyncedOption::MultiplayerServers,
        "",
        Some(&sha1_file(&generated).await?),
        mode,
        state,
    )
    .await
}

async fn regenerate_servers(state: &State) -> crate::Result<()> {
    for metadata in crate::state::list_instances(&state.pool).await? {
        if effective(&metadata, state).await? {
            compose_instance(&metadata, state).await?;
        }
    }
    Ok(())
}

async fn effective(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<bool> {
    if sync_files_are_protected(metadata)
        || instance_is_running(metadata, state)
        || !instance_option_enabled(metadata, SyncedOption::MultiplayerServers)
        || is_linked_server_project(&metadata.link)
    {
        return Ok(false);
    }
    Ok(sqlx::query_scalar!(
        r#"
		SELECT EXISTS(
			SELECT 1 FROM global_synced_options_overrides
			WHERE option = 'multiplayer_servers' AND enabled = 1
		) AS "enabled!: bool"
		"#,
    )
    .fetch_one(&state.pool)
    .await?)
}

async fn read_canonical(state: &State) -> crate::Result<Vec<CanonicalServer>> {
    let path = canonical_path(state);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let root = read_nbt_file(&path).await?;
    let Some(list) = root.get::<_, &NbtList>("Servers").ok() else {
        return Ok(Vec::new());
    };
    Ok(list
        .iter()
        .filter_map(|tag| {
            let NbtTag::Compound(record) = tag else {
                return None;
            };
            Some(CanonicalServer {
                id: record.get::<_, &str>("Id").ok()?.to_string(),
                data: record.get::<_, &NbtCompound>("Data").ok()?.clone(),
            })
        })
        .collect())
}

async fn write_canonical(
    state: &State,
    servers: &[CanonicalServer],
) -> crate::Result<()> {
    let mut list = NbtList::new();
    for server in servers {
        let mut record = NbtCompound::new();
        record.insert("Id", server.id.clone());
        record.insert("Data", server.data.clone());
        list.push(record);
    }
    let mut root = NbtCompound::new();
    root.insert("Version", 1_i32);
    root.insert("Servers", list);
    write_nbt_file(&canonical_path(state), &root).await
}

pub(crate) async fn read_servers(
    path: &Path,
) -> crate::Result<Vec<NbtCompound>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    servers_from_root(read_nbt_file(path).await?)
}

fn servers_from_bytes(bytes: Vec<u8>) -> crate::Result<Vec<NbtCompound>> {
    servers_from_root(nbt_from_bytes(bytes)?)
}

fn servers_from_root(root: NbtCompound) -> crate::Result<Vec<NbtCompound>> {
    let Some(list) = root.get::<_, &NbtList>("servers").ok() else {
        return Ok(Vec::new());
    };
    Ok(list
        .iter()
        .filter_map(|tag| match tag {
            NbtTag::Compound(compound) => Some(compound.clone()),
            _ => None,
        })
        .collect())
}

pub(crate) async fn write_servers(
    path: &Path,
    servers: &[NbtCompound],
) -> crate::Result<()> {
    let mut list = NbtList::new();
    for server in servers {
        list.push(server.clone());
    }
    let mut root = NbtCompound::new();
    root.insert("servers", list);
    write_nbt_file(path, &root).await
}

pub(crate) fn server_data(
    name: String,
    address: String,
    accept_textures: Option<bool>,
) -> NbtCompound {
    let mut server = NbtCompound::new();
    server.insert("name", name);
    server.insert("ip", address);
    if let Some(accept_textures) = accept_textures {
        server.insert("acceptTextures", i8::from(accept_textures));
    }
    server.insert("hidden", 0_i8);
    server
}

pub(crate) fn update_server_data(
    server: &mut NbtCompound,
    name: String,
    address: String,
    accept_textures: Option<bool>,
) {
    server.insert("name", name);
    server.insert("ip", address);
    match accept_textures {
        Some(value) => server.insert("acceptTextures", i8::from(value)),
        None => {
            server.inner_mut().remove("acceptTextures");
        }
    }
}

fn server_hidden(server: &NbtCompound) -> bool {
    server.get::<_, i8>("hidden").unwrap_or(0) != 0
}

fn server_address(server: &NbtCompound) -> String {
    server.get::<_, &str>("ip").unwrap_or_default().to_string()
}

async fn load_local(
    instance_id: &str,
    state: &State,
) -> crate::Result<Vec<LocalServer>> {
    let rows = sqlx::query!(
        "
		SELECT id, source, canonical_id, nbt, position
		FROM instance_server_entries
		WHERE instance_id = ?
		ORDER BY position
		",
        instance_id,
    )
    .fetch_all(&state.pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(LocalServer {
                id: row.id,
                source: ServerSource::from_str(&row.source).ok_or_else(
                    || {
                        ErrorKind::InputError(format!(
                            "Unknown server source {}",
                            row.source
                        ))
                    },
                )?,
                canonical_id: row.canonical_id,
                data: nbt_from_bytes(row.nbt)?,
                position: row.position,
            })
        })
        .collect()
}

async fn write_local(
    instance_id: &str,
    servers: &[LocalServer],
    state: &State,
) -> crate::Result<()> {
    let mut tx = state.pool.begin().await?;
    sqlx::query!(
        "DELETE FROM instance_server_entries WHERE instance_id = ?",
        instance_id,
    )
    .execute(&mut *tx)
    .await?;
    for server in servers {
        let source = server.source.as_str();
        let nbt = nbt_to_bytes(&server.data)?;
        sqlx::query!(
            "
			INSERT INTO instance_server_entries
				(instance_id, id, source, canonical_id, nbt, position)
			VALUES (?, ?, ?, ?, ?, ?)
			",
            instance_id,
            server.id,
            source,
            server.canonical_id,
            nbt,
            server.position,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn load_snapshots(
    instance_id: &str,
    state: &State,
) -> crate::Result<Vec<SnapshotServer>> {
    let rows = sqlx::query!(
        "
		SELECT server_id, source, nbt
		FROM instance_server_snapshots
		WHERE instance_id = ?
		ORDER BY position
		",
        instance_id,
    )
    .fetch_all(&state.pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(SnapshotServer {
                id: row.server_id,
                source: ServerSource::from_str(&row.source).ok_or_else(
                    || {
                        ErrorKind::InputError(format!(
                            "Unknown server source {}",
                            row.source
                        ))
                    },
                )?,
                data: nbt_from_bytes(row.nbt)?,
            })
        })
        .collect()
}

async fn write_snapshots(
    instance_id: &str,
    servers: &[ServerRecord],
    state: &State,
) -> crate::Result<()> {
    let mut tx = state.pool.begin().await?;
    sqlx::query!(
        "DELETE FROM instance_server_snapshots WHERE instance_id = ?",
        instance_id,
    )
    .execute(&mut *tx)
    .await?;
    for (position, server) in servers.iter().enumerate() {
        let source = server.source.as_str();
        let nbt = nbt_to_bytes(&server.data)?;
        let position = position as i64;
        sqlx::query!(
            "
			INSERT INTO instance_server_snapshots
				(instance_id, server_id, source, nbt, position)
			VALUES (?, ?, ?, ?, ?)
			",
            instance_id,
            server.id,
            source,
            nbt,
            position,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

fn canonical_path(state: &State) -> PathBuf {
    synced_options_path(state).join("servers/canonical.nbt")
}

fn generated_path(state: &State, instance_id: &str) -> PathBuf {
    synced_options_path(state)
        .join("servers/generated")
        .join(safe_instance_id(instance_id))
        .join(SERVERS_FILE)
}

fn is_linked_server_project(link: &InstanceLink) -> bool {
    matches!(
        link,
        InstanceLink::ServerProject { .. }
            | InstanceLink::ServerProjectModpack { .. }
            | InstanceLink::ModrinthHosting { .. }
    )
}

fn is_modpack_link(link: &InstanceLink) -> bool {
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
