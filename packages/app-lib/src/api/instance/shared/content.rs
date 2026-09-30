use super::client::InstanceVersionResponse;
use super::diff::shared_external_file_key;
use super::publish::dedupe_strings;
use super::*;
use crate::api::pack::install_from::{
    EnvType, PackDependency, PackFileHash, PackFormat,
};
use crate::state::{DependencyType, SideType};
use crate::util::fetch::{DownloadMeta, DownloadReason};
use async_zip::tokio::read::fs::ZipFileReader;
use std::collections::BTreeMap;

pub(crate) struct SharedModpackFile {
    pub relative_path: String,
    pub sha1: Option<String>,
    pub version_id: Option<String>,
    removed_path_alias: Option<String>,
}

impl SharedModpackFile {
    pub(crate) fn is_removed(
        &self,
        removed_files: &[SharedInstanceRemovedFile],
    ) -> bool {
        removed_files.iter().any(|removed| {
            removed.matches(self.version_id.as_deref(), &self.relative_path)
                || self
                    .removed_path_alias
                    .as_deref()
                    .is_some_and(|path| removed.matches(None, path))
        })
    }
}

/// Reads the client content from the cached pack archive, including unresolved overrides.
#[tracing::instrument(skip(state))]
pub(crate) async fn shared_modpack_files(
    modpack_id: &str,
    state: &State,
) -> crate::Result<Vec<SharedModpackFile>> {
    let version = CachedEntry::get_version(
        modpack_id,
        Some(CacheBehaviour::MustRevalidate),
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Shared instance modpack version was not found".to_string(),
        )
    })?;
    let file = version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "Shared instance modpack has no files".to_string(),
            )
        })?;
    let download_meta = DownloadMeta {
        reason: DownloadReason::Modpack,
        game_version: version
            .game_versions
            .first()
            .cloned()
            .unwrap_or_default(),
        loader: version.loaders.first().cloned().unwrap_or_default(),
        dependent_on: Some(modpack_id.to_string()),
    };
    let archive = crate::util::fetch::fetch_content_file(
        state,
        &[&file.url],
        file.hashes.get("sha512").map(String::as_str),
        Some(u64::from(file.size)),
        Some(&download_meta),
        None,
    )
    .await?;
    let zip = ZipFileReader::new(archive.path()).await?;
    let manifest_index = zip
        .file()
        .entries()
        .iter()
        .position(|entry| {
            matches!(entry.filename().as_str(), Ok("modrinth.index.json"))
        })
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "No modrinth.index.json found in shared modpack".to_string(),
            )
        })?;
    let mut manifest = String::new();
    zip.reader_with_entry(manifest_index)
        .await?
        .read_to_string_checked(&mut manifest)
        .await?;
    let pack: PackFormat = serde_json::from_str(&manifest)?;
    let default_parent = if pack
        .dependencies
        .keys()
        .any(|loader| *loader != PackDependency::Minecraft)
    {
        "mods"
    } else {
        "datapacks"
    };
    let unresolved_basenames = version
        .dependencies
        .iter()
        .filter(|dependency| {
            matches!(dependency.dependency_type, DependencyType::Embedded)
                && dependency.version_id.is_none()
        })
        .filter_map(|dependency| dependency.file_name.as_deref())
        .filter(|filename| !filename.contains('/'))
        .collect::<HashSet<_>>();
    let mut files = BTreeMap::new();
    for file in pack.files {
        if file.env.as_ref().is_some_and(|env| {
            env.get(&EnvType::Client) == Some(&SideType::Unsupported)
        }) {
            continue;
        }
        let path = file.path.as_str();
        if is_content_path(path) {
            files.insert(
                path.to_string(),
                file.hashes.get(&PackFileHash::Sha1).cloned(),
            );
        }
    }
    let overrides = zip
        .file()
        .entries()
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let filename = entry.filename().as_str().ok()?;
            let path = filename
                .strip_prefix("overrides/")
                .or_else(|| filename.strip_prefix("client-overrides/"))?;
            is_content_path(path).then(|| (index, path.to_string()))
        })
        .collect::<Vec<_>>();
    let mut buffer = vec![0_u8; 256 * 1024];
    for (index, path) in overrides {
        let mut reader = zip.reader_with_entry(index).await?;
        let crc32 = reader.entry().crc32();
        let mut hash = sha1_smol::Sha1::new();
        loop {
            let read =
                futures_lite::io::AsyncReadExt::read(&mut reader, &mut buffer)
                    .await?;
            if read == 0 {
                break;
            }
            hash.update(&buffer[..read]);
        }
        if reader.compute_hash() != crc32 {
            return Err(async_zip::error::ZipError::CRC32CheckError.into());
        }
        files.insert(path, Some(hash.hexdigest()));
    }
    let hashes = files
        .values()
        .filter_map(|hash| hash.as_deref())
        .collect::<Vec<_>>();
    let metadata = CachedEntry::get_file_many(
        &hashes,
        Some(CacheBehaviour::MustRevalidate),
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .into_iter()
    .map(|file| (file.hash.clone(), file))
    .collect::<HashMap<_, _>>();
    Ok(files
        .into_iter()
        .map(|(relative_path, hash)| {
            let metadata = hash.as_ref().and_then(|hash| metadata.get(hash));
            // Hosting uses the loader's default directory when Labrinth supplies only an unresolved basename.
            let removed_path_alias =
                relative_path.rsplit_once('/').and_then(|(_, filename)| {
                    unresolved_basenames
                        .contains(filename)
                        .then(|| format!("{default_parent}/{filename}"))
                });
            SharedModpackFile {
                relative_path,
                sha1: hash,
                version_id: metadata.map(|file| file.version_id.clone()),
                removed_path_alias,
            }
        })
        .collect())
}

fn is_content_path(path: &str) -> bool {
    crate::state::content_store::is_managed_content_path(path)
        && !path.ends_with(".disabled")
}

pub(super) async fn remote_shared_content(
    version: &InstanceVersionResponse,
    state: &State,
) -> crate::Result<(Vec<String>, BTreeSet<ExternalFileKey>)> {
    version.validate_removed_files()?;
    let mut version_ids = version
        .modrinth_ids
        .iter()
        .filter(|id| version.modpack_id.as_deref() != Some(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let mut external_files = version
        .external_files
        .iter()
        .filter(|file| {
            !matches!(
                file.file_type.as_str(),
                CONFIG_BUNDLE_FILE_TYPE | CONFIG_FILE_TYPE
            )
        })
        .map(|file| shared_external_file_key(&file.file_type, &file.file_name))
        .collect::<crate::Result<BTreeSet<_>>>()?;
    let explicit_paths = version
        .external_files
        .iter()
        .filter_map(|file| {
            ProjectType::from_name(&file.file_type)
                .map(|kind| format!("{}/{}", kind.get_folder(), file.file_name))
        })
        .collect::<HashSet<_>>();
    if let Some(modpack_id) =
        version.modpack_id.as_deref().filter(|id| !id.is_empty())
    {
        for file in shared_modpack_files(modpack_id, state).await? {
            if file.is_removed(&version.removed_files)
                || explicit_paths.contains(&file.relative_path)
            {
                continue;
            }
            if let Some(version_id) = file.version_id {
                version_ids.push(version_id);
            } else if let Some((parent, filename)) =
                file.relative_path.split_once('/')
            {
                external_files.insert(ExternalFileKey {
                    content_type: ProjectType::from_name(parent)
                        .expect("pack content path was validated")
                        .into(),
                    path: filename.to_string(),
                });
            }
        }
    }
    dedupe_strings(&mut version_ids);
    Ok((version_ids, external_files))
}
