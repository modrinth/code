use super::{PackLibrary, SyncedPack};
use crate::state::content_store::{
    BlobLease, content_file_path, hash_file, input, validate_digest,
    validate_relative,
};
use crate::state::instances::adapters::sqlite::{content_rows, instance_rows};
use crate::state::{CachedEntry, State};
use crate::util::io;
use bytes::Bytes;
use std::path::PathBuf;

fn directory(state: &State) -> PathBuf {
    state.directories.synced_options_dir().join("packs")
}

pub(super) async fn read_library(state: &State) -> crate::Result<PackLibrary> {
    match io::read(directory(state).join("packs.json")).await {
        Ok(bytes) => {
            let mut library: PackLibrary = serde_json::from_slice(&bytes)?;
            for pack in library.packs.values_mut() {
                pack.item.file_name = pack
                    .item
                    .file_name
                    .trim_end_matches(".disabled")
                    .to_string();
                pack.item.file_path = pack
                    .item
                    .file_path
                    .trim_end_matches(".disabled")
                    .to_string();
            }
            for placements in library.instances.values_mut() {
                for placement in placements.values_mut() {
                    placement.path = placement
                        .path
                        .trim_end_matches(".disabled")
                        .to_string();
                    if let Some(path) =
                        &mut placement.resource_pack_selection_path
                    {
                        *path = path.trim_end_matches(".disabled").to_string();
                    }
                }
            }
            Ok(library)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(PackLibrary::default())
        }
        Err(error) => Err(error.into()),
    }
}

pub(super) async fn write_library(
    library: &PackLibrary,
    state: &State,
) -> crate::Result<()> {
    let _lease = state.content_store.lease().await;
    let previous = read_library(state).await?;
    let mut library = library.clone();
    for (id, pack) in &mut library.packs {
        let deferred = previous.packs.get(id).is_some_and(|previous| {
            previous.sha1 == pack.sha1
                && previous.blob_sha512 == pack.blob_sha512
                && (pack.migration_error.is_some()
                    || previous.blob_sha512.is_none())
        });
        let blob = match state
            .content_store
            .lookup(
                pack.blob_sha512.as_deref(),
                Some(&pack.sha1),
                Some(pack.item.size),
            )
            .await
        {
            Ok(Some(blob)) => blob,
            Ok(None) | Err(_) if deferred => continue,
            Ok(None) => {
                return Err(input(
                    "A synced pack needs repair or re-import before its library can be saved",
                ));
            }
            Err(error) => return Err(error),
        };
        pack.blob_sha512 = Some(blob.blob.sha512.clone());
        pack.migration_error = None;
        state
            .content_store
            .retain("synced-pack", id, std::slice::from_ref(&blob.blob.sha512))
            .await?;
    }
    io::create_dir_all(directory(state)).await?;
    let destination = directory(state).join("packs.json");
    let temporary =
        directory(state).join(format!(".packs-{}.tmp", uuid::Uuid::new_v4()));
    io::write(&temporary, serde_json::to_vec(&library)?).await?;
    tokio::fs::File::options()
        .write(true)
        .open(&temporary)
        .await?
        .sync_all()
        .await?;
    tokio::fs::rename(&temporary, &destination).await?;
    crate::state::content_store::sync_directory(&directory(state)).await?;
    for id in previous
        .packs
        .keys()
        .filter(|id| !library.packs.contains_key(*id))
    {
        state.content_store.release("synced-pack", id).await?;
    }
    for (id, pack) in &library.packs {
        if let Some(hash) = &pack.blob_sha512
            && pack.migration_error.is_none()
        {
            state
                .content_store
                .replace_retained("synced-pack", id, std::slice::from_ref(hash))
                .await?;
            state
                .content_store
                .release("synced-cache", &pack.sha1)
                .await?;
        }
    }
    Ok(())
}

pub(super) async fn cache_bytes(
    bytes: Bytes,
    state: &State,
) -> crate::Result<String> {
    let blob = state.content_store.ingest_bytes(&bytes, None).await?;
    state
        .content_store
        .retain(
            "synced-cache",
            &blob.blob.sha1,
            std::slice::from_ref(&blob.blob.sha512),
        )
        .await?;
    Ok(blob.blob.sha1.clone())
}

pub(super) async fn read_blob(
    pack: &SyncedPack,
    state: &State,
) -> crate::Result<BlobLease> {
    if let Some(blob) = state
        .content_store
        .lookup(
            pack.blob_sha512.as_deref(),
            Some(&pack.sha1),
            Some(pack.item.size),
        )
        .await?
    {
        return Ok(blob);
    }
    let library = read_library(state).await?;
    let id = library
        .packs
        .iter()
        .find(|(_, candidate)| candidate.sha1 == pack.sha1)
        .map(|(id, _)| id.as_str());
    match recover_legacy_pack(id, pack, &library, state).await {
        Ok(blob) => return Ok(blob),
        Err(error) => {
            tracing::debug!("Local synced-pack recovery deferred: {error}")
        }
    }
    let version = pack.item.version.as_ref().ok_or_else(|| {
        input("The synced pack is missing or damaged; repair or re-import it")
    })?;
    let version = CachedEntry::get_version(
        &version.id,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| {
        input("The synced pack's original version is unavailable; re-import it")
    })?;
    let file = version
        .files
        .iter()
        .find(|file| {
            file.hashes.get("sha1") == Some(&pack.sha1)
                && pack
                    .blob_sha512
                    .as_ref()
                    .is_none_or(|hash| file.hashes.get("sha512") == Some(hash))
        })
        .ok_or_else(|| {
            input(
                "The synced pack's original file is unavailable; re-import it",
            )
        })?;
    state
        .content_store
        .require_staging_space(u64::from(file.size))?;
    Ok(state
        .content_store
        .acquire(
            &[file.url.as_str()],
            file.hashes.get("sha512").map(String::as_str),
            Some(&pack.sha1),
            Some(u64::from(file.size)),
            None,
            &state.fetch_semaphore,
            None,
        )
        .await?
        .blob)
}

pub(crate) async fn migrate_store(state: &State) -> crate::Result<()> {
    let ids = {
        let _guard = state.lock_synced_options().await;
        read_library(state)
            .await?
            .packs
            .keys()
            .cloned()
            .collect::<Vec<_>>()
    };
    for id in ids {
        let _turn = state.content_store.legacy_migration_priority.read().await;
        let snapshot = {
            let _guard = state.lock_synced_options().await;
            read_library(state).await?
        };
        let Some(expected) = snapshot.packs.get(&id) else {
            continue;
        };
        let recovered =
            recover_legacy_pack(Some(&id), expected, &snapshot, state).await;
        let _guard = state.lock_synced_options().await;
        let mut library = read_library(state).await?;
        let Some(previous) = library.packs.get(&id).cloned() else {
            continue;
        };
        if previous.sha1 != expected.sha1
            || previous.blob_sha512 != expected.blob_sha512
        {
            continue;
        }
        let mut pack = previous.clone();
        match &recovered {
            Ok(blob) => {
                pack.blob_sha512 = Some(blob.blob.sha512.clone());
                pack.item.size = blob.blob.size as u64;
                pack.migration_error = None;
            }
            Err(error) => {
                pack.migration_error = Some(error.to_string());
                tracing::warn!(pack_id = %id, "Synced pack migration deferred: {error}");
            }
        }
        let mut changed = pack.blob_sha512 != previous.blob_sha512
            || pack.item.size != previous.item.size
            || pack.migration_error != previous.migration_error;
        for placements in library.instances.values_mut() {
            if let Some(placement) = placements.get_mut(&id)
                && (pack.migration_error.is_some()
                    || placement.error == previous.migration_error)
            {
                changed |= placement.error != pack.migration_error;
                placement.error.clone_from(&pack.migration_error);
            }
        }
        library.packs.insert(id, pack);
        if changed {
            write_library(&library, state).await?;
        }
    }
    let cache_complete = cleanup_legacy_cache(state).await?;
    let _guard = state.lock_synced_options().await;
    let library = read_library(state).await?;
    for owner in crate::state::content_store::catalog::retained_owners(
        &state.pool,
        "synced-pack",
    )
    .await?
    {
        if !library.packs.contains_key(&owner) {
            state.content_store.release("synced-pack", &owner).await?;
        }
    }
    for owner in crate::state::content_store::catalog::retained_owners(
        &state.pool,
        "synced-cache",
    )
    .await?
    {
        if !library.packs.values().any(|pack| pack.sha1 == owner) {
            state.content_store.release("synced-cache", &owner).await?;
        }
    }
    let complete = cache_complete
        && library.packs.values().all(|pack| {
            pack.blob_sha512.is_some() && pack.migration_error.is_none()
        });
    crate::state::content_store::catalog::set_setting(
        &state.pool,
        "store_synced_packs_migrated",
        if complete { "1" } else { "0" },
    )
    .await?;
    Ok(())
}

async fn recover_legacy_pack(
    id: Option<&str>,
    pack: &SyncedPack,
    library: &PackLibrary,
    state: &State,
) -> crate::Result<BlobLease> {
    validate_digest(&pack.sha1, 40)?;
    if let Some(blob) = state
        .content_store
        .lookup(
            pack.blob_sha512.as_deref(),
            Some(&pack.sha1),
            Some(pack.item.size),
        )
        .await?
    {
        return Ok(blob);
    }
    let legacy = directory(state).join("files").join(&pack.sha1);
    if let Some(blob) = import_matching_pack(&legacy, pack, state).await? {
        return Ok(blob);
    }
    for instance in instance_rows::list_instances(&state.pool).await? {
        validate_relative(&instance.path)?;
        let base = state.directories.instances_dir().join(&instance.path);
        if let Some(placement) = library
            .instances
            .get(&instance.id)
            .and_then(|placements| id.and_then(|id| placements.get(id)))
            && !placement.path.is_empty()
            && validate_relative(&placement.path).is_ok()
        {
            let source = base.join(
                crate::state::content_store::materialized_content_path(
                    &placement.path,
                    placement.enabled,
                ),
            );
            if let Some(blob) =
                import_matching_pack(&source, pack, state).await?
            {
                return Ok(blob);
            }
        }
        for file in
            content_rows::get_instance_files(&instance.id, &state.pool).await?
        {
            if file.sha1 == pack.sha1
                && validate_relative(&file.relative_path).is_ok()
                && let Some(blob) = import_matching_pack(
                    &base.join(content_file_path(&file)),
                    pack,
                    state,
                )
                .await?
            {
                return Ok(blob);
            }
        }
    }
    Err(input(format!(
        "Synced pack {} is missing or damaged; restore or re-import it to finish migration",
        pack.item.file_name
    )))
}

async fn import_matching_pack(
    source: &std::path::Path,
    pack: &SyncedPack,
    state: &State,
) -> crate::Result<Option<BlobLease>> {
    let Ok((sha512, sha1, _)) = hash_file(source).await else {
        return Ok(None);
    };
    if sha1 != pack.sha1
        || pack
            .blob_sha512
            .as_ref()
            .is_some_and(|expected| expected != &sha512)
    {
        return Ok(None);
    }
    let blob = state.content_store.ingest_file(source).await?;
    if blob.blob.sha512 != sha512 {
        return Err(input(
            "The synced pack changed while it was being migrated; retry after closing the instance",
        ));
    }
    Ok(Some(blob))
}

async fn cleanup_legacy_cache(state: &State) -> crate::Result<bool> {
    let legacy = directory(state).join("files");
    if !tokio::fs::try_exists(&legacy).await? {
        return Ok(true);
    }
    let mut complete = true;
    let mut entries = tokio::fs::read_dir(&legacy).await?;
    while let Some(entry) = entries.next_entry().await? {
        let _turn = state.content_store.legacy_migration_priority.read().await;
        let _guard = state.lock_synced_options().await;
        let library = read_library(state).await?;
        if !entry.file_type().await?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if validate_digest(&name, 40).is_err() {
            continue;
        }
        if library.packs.values().any(|pack| {
            pack.sha1 == name
                && (pack.migration_error.is_some()
                    || pack.blob_sha512.is_none())
        }) {
            complete = false;
            continue;
        }
        let result: crate::Result<()> = async {
            if hash_file(&entry.path()).await?.1 != name {
                let quarantine = directory(state).join("quarantine");
                tokio::fs::create_dir_all(&quarantine).await?;
                tokio::fs::rename(
                    entry.path(),
                    quarantine.join(format!("{name}-{}", uuid::Uuid::new_v4())),
                )
                .await?;
            } else {
                state.content_store.ingest_file(&entry.path()).await?;
                tokio::fs::remove_file(entry.path()).await?;
            }
            Ok(())
        }
        .await;
        if let Err(error) = result {
            complete = false;
            tracing::warn!(path = %entry.path().display(), "Preserving legacy cache file for a later retry: {error}");
        }
    }
    if tokio::fs::read_dir(&legacy)
        .await?
        .next_entry()
        .await?
        .is_none()
    {
        tokio::fs::remove_dir(&legacy).await?;
    }
    Ok(complete)
}
