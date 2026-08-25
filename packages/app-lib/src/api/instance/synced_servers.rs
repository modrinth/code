use super::synced_options::{
    detach_link, ensure_link, finish_materialization, instance_dir,
    instance_is_running, instance_option_enabled, materialization,
    nbt_from_bytes, nbt_to_bytes, read_nbt_file, safe_instance_id, sha1_bytes,
    sha1_file, sync_files_are_protected, synced_options_path,
};
use crate::state::{CachedEntry, InstanceLink, InstanceMetadata, SyncedOption};
use crate::util::fetch::{DownloadMeta, DownloadReason, fetch_mirrors};
use crate::{ErrorKind, State};
use async_zip::base::read::seek::ZipFileReader;
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};
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
    position: i64,
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
    for local in &local_entries {
        let local_address = server_identity_address(&local.data);
        if let Some(index) = servers.iter().position(|data| {
            data == &local.data
                || (!local_address.is_empty()
                    && server_identity_address(data) == local_address)
        }) {
            servers.remove(index);
        }
    }
    let canonical = servers
        .into_iter()
        .map(|data| CanonicalServer {
            id: Uuid::new_v4().to_string(),
            data,
        })
        .collect::<Vec<_>>();
    commit_server_state(Some(&canonical), None, state).await?;
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
            "Failed to reconstruct the server baseline for {}; retaining its last known per-instance server metadata: {error}",
            metadata.instance.id
        );
    }
    if !canonical_exists(state).await? {
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
            commit_server_state(Some(&canonical), None, state).await?;
        } else {
            commit_server_state(Some(&[]), None, state).await?;
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
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    let path = instance_dir(&metadata, &state).join(SERVERS_FILE);
    let servers = read_servers(&path).await?;
    replace_modpack_servers(&metadata, servers, true, &state).await?;
    regenerate_servers(&state).await?;
    Ok(())
}

pub async fn clear_modpack_servers(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    replace_modpack_servers(&metadata, Vec::new(), true, &state).await?;
    regenerate_servers(&state).await?;
    Ok(())
}

async fn replace_modpack_servers(
    metadata: &InstanceMetadata,
    servers: Vec<NbtCompound>,
    reconstructed: bool,
    state: &State,
) -> crate::Result<()> {
    let mut local = servers
        .into_iter()
        .enumerate()
        .map(|(position, data)| LocalServer {
            id: Uuid::new_v4().to_string(),
            source: ServerSource::Modpack,
            canonical_id: None,
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
    bump_server_revision(&mut tx, false).await?;
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
    canonical_initialized(state).await?;
    let materialized = materialization(
        &metadata.instance.id,
        SyncedOption::MultiplayerServers,
        "",
        state,
    )
    .await?;
    if materialized.as_ref().is_some_and(|value| value.pending) {
        return compose_instance(metadata, state).await;
    }
    let actual = sha1_file(&local_path).await?;
    let revision = server_revision(state).await?;
    if materialized
        .as_ref()
        .and_then(|value| value.expected_sha1.as_deref())
        == Some(actual.as_str())
    {
        if materialized
            .as_ref()
            .is_some_and(|value| value.canonical_revision == revision)
        {
            return Ok(());
        }
        return compose_instance(metadata, state).await;
    }

    let current = read_servers(&local_path).await?;
    let snapshots = load_snapshots(&metadata.instance.id, state).await?;
    let snapshot_matches = match_snapshots(&current, &snapshots);
    let mut matched = HashSet::new();
    let mut canonical = read_canonical(state).await?;
    let mut locals = load_local(&metadata.instance.id, state).await?;

    let mut canonical_order = Vec::new();

    for ((position, data), snapshot) in
        current.into_iter().enumerate().zip(snapshot_matches)
    {
        if let Some(snapshot) = snapshot {
            matched.insert(snapshot.id.clone());
            match snapshot.source {
                ServerSource::UserSynced => {
                    canonical_order.push(snapshot.id.clone());
                    if let Some(server) = canonical
                        .iter_mut()
                        .find(|server| server.id == snapshot.id)
                        && data != snapshot.data
                    {
                        server.data = data;
                    }
                }
                _ => {
                    if let Some(server) = locals
                        .iter_mut()
                        .find(|server| server.id == snapshot.id)
                    {
                        if data != snapshot.data {
                            server.data = data;
                        }
                        server.position = position as i64;
                    }
                }
            }
        } else {
            let id = Uuid::new_v4().to_string();
            canonical_order.push(id.clone());
            canonical.push(CanonicalServer { id, data });
        }
    }

    canonical.retain(|server| {
        !snapshots.iter().any(|snapshot| {
            snapshot.source == ServerSource::UserSynced
                && snapshot.id == server.id
                && !matched.contains(&snapshot.id)
        })
    });
    locals.retain(|server| {
        !snapshots.iter().any(|snapshot| {
            snapshot.source != ServerSource::UserSynced
                && snapshot.id == server.id
                && !matched.contains(&snapshot.id)
        })
    });
    let by_id = canonical
        .iter()
        .cloned()
        .map(|server| (server.id.clone(), server))
        .collect::<std::collections::HashMap<_, _>>();
    let visible = canonical_order.iter().cloned().collect::<HashSet<_>>();
    let mut ordered = canonical_order
        .into_iter()
        .filter_map(|id| by_id.get(&id).cloned());
    for server in &mut canonical {
        if visible.contains(&server.id)
            && let Some(next) = ordered.next()
        {
            *server = next;
        }
    }
    commit_server_state(
        Some(&canonical),
        Some((&metadata.instance.id, &locals)),
        state,
    )
    .await?;
    regenerate_servers(state).await
}

fn match_snapshots<'a>(
    current: &[NbtCompound],
    snapshots: &'a [SnapshotServer],
) -> Vec<Option<&'a SnapshotServer>> {
    let mut matches = vec![None; current.len()];
    let mut matched_snapshots = HashSet::new();

    for (position, data) in current.iter().enumerate() {
        let candidate = snapshots
            .iter()
            .enumerate()
            .filter(|(index, snapshot)| {
                !matched_snapshots.contains(index) && snapshot.data == *data
            })
            .min_by_key(|(_, snapshot)| {
                snapshot.position.abs_diff(position as i64)
            })
            .map(|(index, _)| index);
        if let Some(index) = candidate {
            matches[position] = Some(index);
            matched_snapshots.insert(index);
        }
    }

    for (position, data) in current.iter().enumerate() {
        if matches[position].is_some() {
            continue;
        }
        let address = server_identity_address(data);
        if address.is_empty() {
            continue;
        }
        let candidate = snapshots
            .iter()
            .enumerate()
            .filter(|(index, snapshot)| {
                !matched_snapshots.contains(index)
                    && server_identity_address(&snapshot.data) == address
            })
            .min_by_key(|(_, snapshot)| {
                snapshot.position.abs_diff(position as i64)
            })
            .map(|(index, _)| index);
        if let Some(index) = candidate {
            matches[position] = Some(index);
            matched_snapshots.insert(index);
        }
    }

    for (position, data) in current.iter().enumerate() {
        if matches[position].is_some() {
            continue;
        }
        let name = server_identity_name(data);
        if name.is_empty() {
            continue;
        }
        let candidate = snapshots
            .iter()
            .enumerate()
            .filter(|(index, snapshot)| {
                !matched_snapshots.contains(index)
                    && server_identity_name(&snapshot.data) == name
            })
            .min_by_key(|(_, snapshot)| {
                snapshot.position.abs_diff(position as i64)
            })
            .map(|(index, _)| index);
        if let Some(index) = candidate {
            matches[position] = Some(index);
            matched_snapshots.insert(index);
        }
    }

    matches
        .into_iter()
        .map(|snapshot| snapshot.map(|index| &snapshots[index]))
        .collect()
}

pub(crate) async fn list_server_records(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<Vec<ServerRecord>> {
    let _guard = state.lock_synced_options().await;
    list_server_records_locked(metadata, state).await
}

async fn list_server_records_locked(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<Vec<ServerRecord>> {
    if participating(metadata, state).await? {
        return compose_records(metadata, state).await;
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
    let _guard = state.lock_synced_options().await;
    let id = Uuid::new_v4().to_string();
    if participating(metadata, state).await? {
        let mut canonical = read_canonical(state).await?;
        canonical.push(CanonicalServer {
            id: id.clone(),
            data,
        });
        commit_server_state(Some(&canonical), None, state).await?;
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
    let _guard = state.lock_synced_options().await;
    update_server_locked(metadata, server_id, data, state).await
}

async fn update_server_locked(
    metadata: &InstanceMetadata,
    server_id: &str,
    data: NbtCompound,
    state: &State,
) -> crate::Result<()> {
    if participating(metadata, state).await? {
        let records = compose_records(metadata, state).await?;
        let record = records
            .iter()
            .find(|record| record.id == server_id)
            .ok_or_else(|| {
                ErrorKind::InputError("Unknown server".to_string())
            })?;
        if record.source == ServerSource::UserSynced {
            let mut canonical = read_canonical(state).await?;
            let server = canonical
                .iter_mut()
                .find(|server| server.id == server_id)
                .ok_or_else(|| {
                    ErrorKind::InputError("Unknown server".to_string())
                })?;
            server.data = data;
            commit_server_state(Some(&canonical), None, state).await?;
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
    let _guard = state.lock_synced_options().await;
    if participating(metadata, state).await? {
        let records = list_server_records_locked(metadata, state).await?;
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
        return update_server_locked(metadata, &record.id, data, state).await;
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
    let _guard = state.lock_synced_options().await;
    remove_server_locked(metadata, server_id, state).await
}

async fn remove_server_locked(
    metadata: &InstanceMetadata,
    server_id: &str,
    state: &State,
) -> crate::Result<()> {
    if !participating(metadata, state).await? {
        return Err(ErrorKind::InputError(
			"Stable server removal requires multiplayer syncing for this instance."
				.to_string(),
		)
		.into());
    }
    let records = compose_records(metadata, state).await?;
    let record = records
        .iter()
        .find(|record| record.id == server_id)
        .ok_or_else(|| ErrorKind::InputError("Unknown server".to_string()))?;
    if record.source == ServerSource::UserSynced {
        let mut canonical = read_canonical(state).await?;
        canonical.retain(|server| server.id != server_id);
        commit_server_state(Some(&canonical), None, state).await?;
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
    let _guard = state.lock_synced_options().await;
    if participating(metadata, state).await? {
        let records = list_server_records_locked(metadata, state).await?;
        let record = records
            .get(index)
            .filter(|record| !record.hidden())
            .ok_or_else(|| {
                ErrorKind::InputError(format!(
                    "No removable server at index {index}"
                ))
            })?;
        return remove_server_locked(metadata, &record.id, state).await;
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
    let _guard = state.lock_synced_options().await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    if !participating(&metadata, &state).await? {
        return Err(ErrorKind::InputError(
            "This instance is not participating in multiplayer server syncing."
                .to_string(),
        )
        .into());
    }
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
    let local_position = compose_records(&metadata, &state)
        .await?
        .iter()
        .position(|record| record.id == server_id)
        .unwrap_or_default() as i64;
    let mut locals = load_local(instance_id, &state).await?;
    locals.push(LocalServer {
        id: Uuid::new_v4().to_string(),
        source: ServerSource::LocalDesynced,
        canonical_id: (mode == DesyncServerMode::KeepInOtherInstances)
            .then(|| server.id.clone()),
        data: server.data,
        position: local_position,
    });
    if mode == DesyncServerMode::RemoveFromOtherInstances {
        canonical.retain(|candidate| candidate.id != server_id);
    }
    commit_server_state(Some(&canonical), Some((instance_id, &locals)), &state)
        .await?;
    regenerate_servers(&state).await?;
    Ok(())
}

pub async fn list_synced_servers() -> crate::Result<Vec<SyncedServer>> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
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
    let _guard = state.lock_synced_options().await;
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
    commit_server_state(Some(&canonical), None, &state).await?;
    regenerate_servers(&state).await
}

pub async fn remove_synced_server(server_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let _guard = state.lock_synced_options().await;
    let mut canonical = read_canonical(&state).await?;
    let previous_len = canonical.len();
    canonical.retain(|server| server.id != server_id);
    if canonical.len() == previous_len {
        return Err(
            ErrorKind::InputError("Unknown synced server".to_string()).into()
        );
    }
    commit_server_state(Some(&canonical), None, &state).await?;
    regenerate_servers(&state).await
}

async fn compose_instance(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let records = compose_records(metadata, state).await?;
    let bytes = servers_to_bytes(
        &records
            .iter()
            .map(|record| record.data.clone())
            .collect::<Vec<_>>(),
    )?;
    let expected = sha1_bytes(&bytes);
    let revision = server_revision(state).await?;
    begin_server_materialization(
        &metadata.instance.id,
        &records,
        &bytes,
        &expected,
        revision,
        state,
    )
    .await?;
    let generated = generated_path(state, &metadata.instance.id);
    if let Some(parent) = generated.parent() {
        crate::util::io::create_dir_all(parent).await?;
    }
    crate::util::io::write(&generated, &bytes).await?;
    let target = instance_dir(metadata, state).join(SERVERS_FILE);
    let mode = ensure_link(&generated, &target).await?;
    finish_materialization(
        &metadata.instance.id,
        SyncedOption::MultiplayerServers,
        "",
        mode,
        state,
    )
    .await
}

async fn compose_records(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<Vec<ServerRecord>> {
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
    for server in locals {
        let position = usize::try_from(server.position)
            .unwrap_or(records.len())
            .min(records.len());
        records.insert(
            position,
            ServerRecord {
                id: server.id,
                source: server.source,
                data: server.data,
            },
        );
    }
    Ok(records)
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
        || instance_is_running(metadata, state).await?
    {
        return Ok(false);
    }
    participating(metadata, state).await
}

async fn participating(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<bool> {
    if !instance_option_enabled(metadata, SyncedOption::MultiplayerServers)
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

pub(super) async fn canonical_exists(state: &State) -> crate::Result<bool> {
    canonical_initialized(state).await
}

async fn canonical_initialized(state: &State) -> crate::Result<bool> {
    Ok(sqlx::query_scalar!(
        r#"
		SELECT initialized AS "initialized!: bool"
		FROM synced_option_revisions
		WHERE option = 'multiplayer_servers'
		"#,
    )
    .fetch_one(&state.pool)
    .await?)
}

async fn read_canonical(state: &State) -> crate::Result<Vec<CanonicalServer>> {
    canonical_initialized(state).await?;
    let rows = sqlx::query!(
        "
		SELECT id, nbt
		FROM synced_server_entries
		ORDER BY position
		",
    )
    .fetch_all(&state.pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(CanonicalServer {
                id: row.id,
                data: nbt_from_bytes(row.nbt)?,
            })
        })
        .collect()
}

async fn commit_server_state(
    canonical: Option<&[CanonicalServer]>,
    local: Option<(&str, &[LocalServer])>,
    state: &State,
) -> crate::Result<i64> {
    let canonical_changed = canonical.is_some();
    let mut tx = state.pool.begin().await?;
    if let Some(canonical) = canonical {
        write_canonical_rows(&mut tx, canonical).await?;
    }
    if let Some((instance_id, local)) = local {
        write_local_rows(&mut tx, instance_id, local).await?;
    }
    let revision = bump_server_revision(&mut tx, canonical_changed).await?;
    tx.commit().await?;
    Ok(revision)
}

async fn write_canonical_rows(
    tx: &mut Transaction<'_, Sqlite>,
    servers: &[CanonicalServer],
) -> crate::Result<()> {
    sqlx::query!("DELETE FROM synced_server_entries")
        .execute(&mut **tx)
        .await?;
    for (position, server) in servers.iter().enumerate() {
        let nbt = nbt_to_bytes(&server.data)?;
        let position = position as i64;
        sqlx::query!(
            "
			INSERT INTO synced_server_entries (id, nbt, position)
			VALUES (?, ?, ?)
			",
            server.id,
            nbt,
            position,
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn bump_server_revision(
    tx: &mut Transaction<'_, Sqlite>,
    initialize: bool,
) -> crate::Result<i64> {
    Ok(sqlx::query_scalar!(
        r#"
		UPDATE synced_option_revisions
		SET revision = revision + 1,
			initialized = CASE WHEN ? THEN 1 ELSE initialized END
		WHERE option = 'multiplayer_servers'
		RETURNING revision AS "revision!: i64"
		"#,
        initialize,
    )
    .fetch_one(&mut **tx)
    .await?)
}

async fn server_revision(state: &State) -> crate::Result<i64> {
    Ok(sqlx::query_scalar!(
        r#"
		SELECT revision AS "revision!: i64"
		FROM synced_option_revisions
		WHERE option = 'multiplayer_servers'
		"#,
    )
    .fetch_one(&state.pool)
    .await?)
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
    let list = root.get::<_, &NbtList>("servers").map_err(|_| {
        ErrorKind::InputError(
            "servers.dat does not contain a valid servers list".to_string(),
        )
    })?;
    list.iter()
        .map(|tag| match tag {
            NbtTag::Compound(compound) => Ok(compound.clone()),
            _ => Err(ErrorKind::InputError(
                "servers.dat contains an invalid server entry".to_string(),
            )
            .into()),
        })
        .collect()
}

pub(crate) async fn write_servers(
    path: &Path,
    servers: &[NbtCompound],
) -> crate::Result<()> {
    if let Some(parent) = path.parent() {
        crate::util::io::create_dir_all(parent).await?;
    }
    crate::util::io::write(path, servers_to_bytes(servers)?).await?;
    Ok(())
}

fn servers_to_bytes(servers: &[NbtCompound]) -> crate::Result<Vec<u8>> {
    let mut list = NbtList::new();
    for server in servers {
        list.push(server.clone());
    }
    let mut root = NbtCompound::new();
    root.insert("servers", list);
    nbt_to_bytes(&root)
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

fn server_identity_address(server: &NbtCompound) -> String {
    server_address(server).trim().to_ascii_lowercase()
}

fn server_identity_name(server: &NbtCompound) -> String {
    server
        .get::<_, &str>("name")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
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
    commit_server_state(None, Some((instance_id, servers)), state).await?;
    Ok(())
}

async fn write_local_rows(
    tx: &mut Transaction<'_, Sqlite>,
    instance_id: &str,
    servers: &[LocalServer],
) -> crate::Result<()> {
    sqlx::query!(
        "DELETE FROM instance_server_entries WHERE instance_id = ?",
        instance_id,
    )
    .execute(&mut **tx)
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
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn load_snapshots(
    instance_id: &str,
    state: &State,
) -> crate::Result<Vec<SnapshotServer>> {
    let rows = sqlx::query!(
        "
		SELECT server_id, source, nbt, position
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
                position: row.position,
            })
        })
        .collect()
}

async fn begin_server_materialization(
    instance_id: &str,
    servers: &[ServerRecord],
    baseline: &[u8],
    expected_sha1: &str,
    canonical_revision: i64,
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
    sqlx::query!(
        "
		INSERT INTO synced_option_materializations
			(instance_id, option, family, expected_sha1, baseline,
			 canonical_revision, pending, link_mode)
		VALUES (?, 'multiplayer_servers', '', ?, ?, ?, 1, 'copy')
		ON CONFLICT(instance_id, option, family) DO UPDATE SET
			expected_sha1 = excluded.expected_sha1,
			baseline = excluded.baseline,
			canonical_revision = excluded.canonical_revision,
			pending = 1
		",
        instance_id,
        expected_sha1,
        baseline,
        canonical_revision,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
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
