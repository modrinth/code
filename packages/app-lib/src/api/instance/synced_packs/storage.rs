use super::{PackLibrary, SyncedPack};
use crate::state::State;
use crate::util::{fetch, io};
use bytes::Bytes;
use std::path::PathBuf;

fn directory(state: &State) -> PathBuf {
    state.directories.synced_options_dir().join("packs")
}

pub(super) async fn read_library(state: &State) -> crate::Result<PackLibrary> {
    match io::read(directory(state).join("packs.json")).await {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
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
    io::create_dir_all(directory(state)).await?;
    let path = directory(state).join("packs.json");
    let bytes = serde_json::to_vec(library)?;
    if io::read(&path).await.is_ok_and(|current| current == bytes) {
        return Ok(());
    }
    io::write(path, bytes).await?;
    state
        .pack_sync_worker
        .revision
        .fetch_add(1, std::sync::atomic::Ordering::Release);
    Ok(())
}

pub(super) async fn cache_bytes(
    bytes: Bytes,
    state: &State,
) -> crate::Result<String> {
    let sha1 = fetch::sha1_async(bytes.clone()).await?;
    let folder = directory(state).join("files");
    io::create_dir_all(&folder).await?;
    let path = folder.join(&sha1);
    if !path.exists() {
        io::write(path, &bytes).await?;
    }
    Ok(sha1)
}

pub(super) async fn read_bytes(
    pack: &SyncedPack,
    state: &State,
) -> crate::Result<Bytes> {
    if pack.sha1.len() != 40
        || !pack.sha1.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(crate::ErrorKind::InputError(
            "Invalid synced pack hash.".to_string(),
        )
        .into());
    }
    let bytes = Bytes::from(
        io::read(directory(state).join("files").join(&pack.sha1)).await?,
    );
    if fetch::sha1_async(bytes.clone()).await? != pack.sha1 {
        return Err(crate::ErrorKind::InputError(
            "The synced pack file is damaged.".to_string(),
        )
        .into());
    }
    Ok(bytes)
}

pub(super) async fn read_cached_bytes(
    sha1: &str,
    state: &State,
) -> crate::Result<Option<Bytes>> {
    if sha1.len() != 40 || !sha1.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(None);
    }
    let bytes = match io::read(directory(state).join("files").join(sha1)).await
    {
        Ok(bytes) => Bytes::from(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(error) => return Err(error.into()),
    };
    if fetch::sha1_async(bytes.clone()).await? != sha1 {
        io::remove_file(directory(state).join("files").join(sha1)).await?;
        return Ok(None);
    }
    Ok(Some(bytes))
}
