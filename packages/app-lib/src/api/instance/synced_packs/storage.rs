use super::{PackLibrary, SyncedPack};
use crate::state::State;
use crate::state::content_store::input;
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
        let blob = state.content_store.lookup(pack.blob_sha512.as_deref(), Some(&pack.sha1), Some(pack.item.size)).await?
			.ok_or_else(|| input("A synced pack needs repair or re-import before its library can be saved"))?;
        pack.blob_sha512 = Some(blob.blob.sha512.clone());
        state
            .content_store
            .retain("synced-pack", id, &[blob.blob.sha512.clone()])
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
        if let Some(hash) = &pack.blob_sha512 {
            state
                .content_store
                .replace_retained("synced-pack", id, &[hash.clone()])
                .await?;
        }
        state
            .content_store
            .release("synced-cache", &pack.sha1)
            .await?;
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
        .retain("synced-cache", &blob.blob.sha1, &[blob.blob.sha512.clone()])
        .await?;
    Ok(blob.blob.sha1.clone())
}

pub(super) async fn read_bytes(
    pack: &SyncedPack,
    state: &State,
) -> crate::Result<Bytes> {
    let blob = state
        .content_store
        .lookup(
            pack.blob_sha512.as_deref(),
            Some(&pack.sha1),
            Some(pack.item.size),
        )
        .await?
        .ok_or_else(|| {
            input(
                "The synced pack is missing or damaged; repair or re-import it",
            )
        })?;
    Ok(Bytes::from(io::read(&blob.path).await?))
}

pub(super) async fn read_cached_bytes(
    sha1: &str,
    state: &State,
) -> crate::Result<Option<Bytes>> {
    let Some(blob) = state.content_store.lookup(None, Some(sha1), None).await?
    else {
        return Ok(None);
    };
    Ok(Some(Bytes::from(io::read(&blob.path).await?)))
}

pub(crate) async fn migrate_store(state: &State) -> crate::Result<()> {
    let mut library = read_library(state).await?;
    if crate::state::content_store::catalog::setting(
        &state.pool,
        "store_synced_packs_migrated",
    )
    .await?
    .as_deref()
        == Some("1")
    {
        for (id, pack) in &library.packs {
            if let Some(hash) = &pack.blob_sha512 {
                if state.content_store.catalog_blob(hash).await?.is_some() {
                    state
                        .content_store
                        .replace_retained("synced-pack", id, &[hash.clone()])
                        .await?;
                }
            }
        }
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
        return Ok(());
    }
    let legacy = directory(state).join("files");
    for (id, pack) in &mut library.packs {
        if let Some(blob) = state
            .content_store
            .lookup(
                pack.blob_sha512.as_deref(),
                Some(&pack.sha1),
                Some(pack.item.size),
            )
            .await?
        {
            pack.blob_sha512 = Some(blob.blob.sha512.clone());
            state
                .content_store
                .retain("synced-pack", id, &[blob.blob.sha512.clone()])
                .await?;
            continue;
        }
        crate::state::content_store::validate_digest(&pack.sha1, 40)?;
        let source = legacy.join(&pack.sha1);
        if !tokio::fs::try_exists(&source).await? {
            return Err(input(format!(
                "Synced pack {} is missing; restore it before finishing migration",
                pack.item.file_name
            )));
        }
        let blob = state.content_store.ingest_file(&source).await?;
        if blob.blob.sha1 != pack.sha1 {
            return Err(input("A legacy synced-pack cache file is damaged"));
        }
        pack.blob_sha512 = Some(blob.blob.sha512.clone());
        state
            .content_store
            .retain("synced-pack", id, &[blob.blob.sha512.clone()])
            .await?;
    }
    if !library.packs.is_empty() {
        write_library(&library, state).await?;
    }
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
    if tokio::fs::try_exists(&legacy).await? {
        let mut entries = tokio::fs::read_dir(&legacy).await?;
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if crate::state::content_store::validate_digest(&name, 40).is_err()
            {
                continue;
            }
            let blob = state.content_store.ingest_file(&entry.path()).await?;
            if blob.blob.sha1 != name {
                return Err(input(
                    "A legacy synced-pack cache file is damaged",
                ));
            }
            tokio::fs::remove_file(entry.path()).await?;
        }
        if tokio::fs::read_dir(&legacy)
            .await?
            .next_entry()
            .await?
            .is_none()
        {
            tokio::fs::remove_dir(&legacy).await?;
        }
    }
    crate::state::content_store::catalog::set_setting(
        &state.pool,
        "store_synced_packs_migrated",
        "1",
    )
    .await?;
    Ok(())
}
