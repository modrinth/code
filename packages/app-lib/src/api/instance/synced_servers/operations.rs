use super::super::synced_options::{
    CheckpointStatus, checkpoint, detach_link, ensure_link, finish_checkpoint,
    instance_dir, instance_is_running, instance_option_enabled,
    instance_option_supported, sha1_bytes, sha1_file, sync_files_are_protected,
};
use crate::state::{InstanceMetadata, SyncedOption};
use crate::{ErrorKind, State};
use quartz_nbt::NbtCompound;
use uuid::Uuid;

use super::SERVERS_FILE;
use super::codec::{
    read_servers, server_address, server_hidden, server_identity_address,
    server_identity_name, servers_to_bytes, update_server_data, write_servers,
};
use super::modpack::{
    is_modpack_link, pack_state_exists, pack_state_matches_link,
    reconstruct_modpack_servers,
};
use super::storage::{
    begin_server_checkpoint, canonical_exists, commit_server_state,
    generated_path, load_local, load_projection_entries, read_canonical,
    server_revision, write_local,
};
use super::types::{
    CanonicalServer, DesyncServerMode, LocalServer, ProjectionEntry,
    ProjectionOwner, ServerRecord, ServerSource, SyncedServer,
};
use std::collections::HashSet;

pub(in crate::api::instance) async fn seed_servers(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    if is_modpack_link(&metadata.link)
        && (!pack_state_matches_link(metadata, state).await?
            || !pack_state_exists(&metadata.instance.id, state).await?)
        && let Err(error) = reconstruct_modpack_servers(metadata, state).await
    {
        tracing::warn!(
            "Failed to reconstruct the server pack state for {}: {error}",
            metadata.instance.id
        );
    }
    let local_entries = load_local(&metadata.instance.id, state).await?;
    if is_modpack_link(&metadata.link)
        && !pack_state_exists(&metadata.instance.id, state).await?
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

pub(in crate::api::instance) async fn merge_servers_from_instance(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    if is_modpack_link(&metadata.link)
        && (!pack_state_matches_link(metadata, state).await?
            || !pack_state_exists(&metadata.instance.id, state).await?)
        && let Err(error) = reconstruct_modpack_servers(metadata, state).await
    {
        tracing::warn!(
            "Failed to reconstruct the server pack state for {}: {error}",
            metadata.instance.id
        );
    }
    let local_entries = load_local(&metadata.instance.id, state).await?;
    if is_modpack_link(&metadata.link)
        && !pack_state_exists(&metadata.instance.id, state).await?
    {
        return Err(ErrorKind::InputError(
            "This linked modpack cannot join multiplayer syncing until its supplied server list has been reconstructed."
                .to_string(),
        )
        .into());
    }
    let path = instance_dir(metadata, state).join(SERVERS_FILE);
    let mut candidates = read_servers(&path).await?;
    for local in &local_entries {
        let local_address = server_identity_address(&local.data);
        if let Some(index) = candidates.iter().position(|data| {
            data == &local.data
                || (!local_address.is_empty()
                    && server_identity_address(data) == local_address)
        }) {
            candidates.remove(index);
        }
    }

    let mut canonical = read_canonical(state).await?;
    for data in candidates {
        let address = server_identity_address(&data);
        let exists = canonical.iter().any(|server| {
            server.data == data
                || (!address.is_empty()
                    && server_identity_address(&server.data) == address)
        });
        if !exists {
            canonical.push(CanonicalServer {
                id: Uuid::new_v4().to_string(),
                data,
            });
        }
    }
    if commit_server_state(Some(&canonical), None, state).await? {
        regenerate_servers(state).await?;
    }
    Ok(())
}

pub(in crate::api::instance) async fn ensure_servers(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    if is_modpack_link(&metadata.link)
        && !pack_state_matches_link(metadata, state).await?
        && let Err(error) = reconstruct_modpack_servers(metadata, state).await
    {
        tracing::warn!(
            "Failed to reconstruct the server pack state for {}; retaining its last known per-instance server metadata: {error}",
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

pub(in crate::api::instance) async fn detach_servers(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let generated = generated_path(state, &metadata.instance.id);
    let local = instance_dir(metadata, state).join(SERVERS_FILE);
    detach_link(&generated, &local).await
}

pub(in crate::api::instance) async fn reconcile_servers(
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
    let checkpoint = checkpoint(
        &metadata.instance.id,
        SyncedOption::MultiplayerServers,
        "default",
        state,
    )
    .await?;
    if checkpoint
        .as_ref()
        .is_some_and(|value| value.status == CheckpointStatus::Pending)
    {
        return compose_instance(metadata, state).await;
    }
    let actual = sha1_file(&local_path).await?;
    let revision = server_revision(state).await?;
    if checkpoint
        .as_ref()
        .map(|value| value.expected_sha1.as_str())
        == Some(actual.as_str())
    {
        if checkpoint
            .as_ref()
            .is_some_and(|value| value.source_revision == revision)
        {
            return Ok(());
        }
        return compose_instance(metadata, state).await;
    }

    let current = read_servers(&local_path).await?;
    let projections =
        load_projection_entries(&metadata.instance.id, state).await?;
    let projection_matches = match_projection_entries(&current, &projections);
    let mut matched = HashSet::new();
    let mut canonical = read_canonical(state).await?;
    let mut locals = load_local(&metadata.instance.id, state).await?;

    let mut canonical_order = Vec::new();

    for ((position, data), projection) in
        current.into_iter().enumerate().zip(projection_matches)
    {
        if let Some(projection) = projection {
            matched.insert(projection.id.clone());
            match projection.owner {
                ProjectionOwner::Synced => {
                    canonical_order.push(projection.id.clone());
                    if let Some(server) = canonical
                        .iter_mut()
                        .find(|server| server.id == projection.id)
                        && data != projection.data
                    {
                        server.data = data;
                    }
                }
                ProjectionOwner::Instance => {
                    if let Some(server) = locals
                        .iter_mut()
                        .find(|server| server.id == projection.id)
                    {
                        if data != projection.data {
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
        !projections.iter().any(|projection| {
            projection.owner == ProjectionOwner::Synced
                && projection.id == server.id
                && !matched.contains(&projection.id)
        })
    });
    locals.retain(|server| {
        !projections.iter().any(|projection| {
            projection.owner == ProjectionOwner::Instance
                && projection.id == server.id
                && !matched.contains(&projection.id)
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
    let canonical_changed = commit_server_state(
        Some(&canonical),
        Some((&metadata.instance.id, &locals)),
        state,
    )
    .await?;
    if canonical_changed {
        regenerate_servers(state).await
    } else {
        compose_instance(metadata, state).await
    }
}

fn match_projection_entries<'a>(
    current: &[NbtCompound],
    projections: &'a [ProjectionEntry],
) -> Vec<Option<&'a ProjectionEntry>> {
    let mut matches = vec![None; current.len()];
    let mut matched_projections = HashSet::new();

    for (position, data) in current.iter().enumerate() {
        let candidate = projections
            .iter()
            .enumerate()
            .filter(|(index, projection)| {
                !matched_projections.contains(index) && projection.data == *data
            })
            .min_by_key(|(_, projection)| {
                projection.position.abs_diff(position as i64)
            })
            .map(|(index, _)| index);
        if let Some(index) = candidate {
            matches[position] = Some(index);
            matched_projections.insert(index);
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
        let candidate = projections
            .iter()
            .enumerate()
            .filter(|(index, projection)| {
                !matched_projections.contains(index)
                    && server_identity_address(&projection.data) == address
            })
            .min_by_key(|(_, projection)| {
                projection.position.abs_diff(position as i64)
            })
            .map(|(index, _)| index);
        if let Some(index) = candidate {
            matches[position] = Some(index);
            matched_projections.insert(index);
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
        let candidate = projections
            .iter()
            .enumerate()
            .filter(|(index, projection)| {
                !matched_projections.contains(index)
                    && server_identity_name(&projection.data) == name
            })
            .min_by_key(|(_, projection)| {
                projection.position.abs_diff(position as i64)
            })
            .map(|(index, _)| index);
        if let Some(index) = candidate {
            matches[position] = Some(index);
            matched_projections.insert(index);
        }
    }

    matches
        .into_iter()
        .map(|projection| projection.map(|index| &projections[index]))
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
            regenerate_servers(state).await?;
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
            compose_instance(metadata, state).await?;
        }
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
        regenerate_servers(state).await
    } else {
        let mut locals = load_local(&metadata.instance.id, state).await?;
        locals.retain(|server| server.id != server_id);
        write_local(&metadata.instance.id, &locals, state).await?;
        compose_instance(metadata, state).await
    }
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
        excluded_synced_server_id: (mode
            == DesyncServerMode::KeepInOtherInstances)
            .then(|| server.id.clone()),
        data: server.data,
        position: local_position,
    });
    if mode == DesyncServerMode::RemoveFromOtherInstances {
        canonical.retain(|candidate| candidate.id != server_id);
    }
    let canonical_changed = commit_server_state(
        Some(&canonical),
        Some((instance_id, &locals)),
        &state,
    )
    .await?;
    if canonical_changed {
        regenerate_servers(&state).await?;
    } else {
        compose_instance(&metadata, &state).await?;
    }
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

pub(super) async fn compose_instance(
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
    begin_server_checkpoint(
        &metadata.instance.id,
        &records,
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
    finish_checkpoint(
        &metadata.instance.id,
        SyncedOption::MultiplayerServers,
        "default",
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
        .filter_map(|server| server.excluded_synced_server_id.as_deref())
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

pub(super) async fn effective(
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
    if !instance_option_enabled(metadata, SyncedOption::MultiplayerServers) {
        return Ok(false);
    }
    let global_enabled = sqlx::query_scalar!(
        r#"
		SELECT EXISTS(
			SELECT 1 FROM sync_feature_settings
			WHERE feature = 'multiplayer_servers' AND globally_enabled = 1
		) AS "enabled!: bool"
		"#,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(instance_option_supported(
        metadata,
        SyncedOption::MultiplayerServers,
        global_enabled,
        state,
    )
    .await)
}
