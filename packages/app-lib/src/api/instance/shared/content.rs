use super::client::{
    ExternalFileResponse, InstanceVersionResponse, get_remote_version,
};
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

const CONFIG_VERSION_FETCH_CONCURRENCY: usize = 4;

/// Config files are published as deltas, so a member needs every config
/// uploaded after `applied_version`, keeping the newest upload of each path.
pub(super) async fn remote_shared_config_files(
    shared_instance_id: &str,
    applied_version: Option<i32>,
    latest: &InstanceVersionResponse,
    state: &State,
) -> crate::Result<Vec<ExternalFileResponse>> {
    use futures::{StreamExt, TryStreamExt};

    let first = applied_version.map_or(1, |version| version.saturating_add(1));
    let intermediate = futures::stream::iter(first..latest.version)
        .map(|version| get_remote_version(shared_instance_id, version, state))
        .buffered(CONFIG_VERSION_FETCH_CONCURRENCY)
        .try_collect::<Vec<_>>()
        .await?;

    newest_config_files(
        intermediate
            .iter()
            .flatten()
            .chain((first <= latest.version).then_some(latest))
            .map(|version| version.external_files.as_slice()),
    )
}

/// Takes each version's external files in ascending version order. A legacy
/// config bundle is a full snapshot, so it replaces every older config. The
/// bundle comes first so newer individual files are written over it.
///
/// Limits apply per version, matching what a single publish may upload.
fn newest_config_files<'a>(
    versions: impl IntoIterator<Item = &'a [ExternalFileResponse]>,
) -> crate::Result<Vec<ExternalFileResponse>> {
    let mut bundle = None;
    let mut files = BTreeMap::new();
    for external_files in versions {
        ensure_config_limits(external_files)?;
        if let Some(newer) = external_files
            .iter()
            .find(|file| file.file_type == CONFIG_BUNDLE_FILE_TYPE)
        {
            bundle = Some(newer.clone());
            files.clear();
        }
        for file in external_files
            .iter()
            .filter(|file| file.file_type == CONFIG_FILE_TYPE)
        {
            files.insert(file.file_name.clone(), file.clone());
        }
    }

    Ok(bundle.into_iter().chain(files.into_values()).collect())
}

fn ensure_config_limits(
    external_files: &[ExternalFileResponse],
) -> crate::Result<()> {
    let mut count = 0;
    let mut size = 0_u64;
    for file in external_files
        .iter()
        .filter(|file| file.file_type == CONFIG_FILE_TYPE)
    {
        count += 1;
        size = size.saturating_add(
            file.file_size
                .and_then(|size| u64::try_from(size).ok())
                .unwrap_or(u64::MAX),
        );
    }
    if count > MAX_CONFIG_BUNDLE_ENTRIES || size > MAX_CONFIG_BUNDLE_TOTAL_SIZE
    {
        return Err(crate::ErrorKind::InputError(
            "Shared instance config files exceed the size or file count limit"
                .to_string(),
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(
        file_type: &str,
        file_name: &str,
        url: &str,
    ) -> ExternalFileResponse {
        ExternalFileResponse {
            file_name: file_name.to_string(),
            file_type: file_type.to_string(),
            url: Some(url.to_string()),
            file_size: Some(1),
        }
    }

    fn names(files: &[ExternalFileResponse]) -> Vec<(&str, &str, &str)> {
        files
            .iter()
            .map(|file| {
                (
                    file.file_type.as_str(),
                    file.file_name.as_str(),
                    file.url.as_deref().unwrap_or_default(),
                )
            })
            .collect()
    }

    #[test]
    fn newest_upload_of_each_path_wins() {
        let v1 = [
            file(CONFIG_FILE_TYPE, "config/a.toml", "v1"),
            file(CONFIG_FILE_TYPE, "config/b.toml", "v1"),
        ];
        let v2 = [file("mod", "extra.jar", "v2")];
        let v3 = [file(CONFIG_FILE_TYPE, "config/a.toml", "v3")];

        let files = newest_config_files([&v1[..], &v2[..], &v3[..]]).unwrap();

        assert_eq!(
            names(&files),
            [
                (CONFIG_FILE_TYPE, "config/a.toml", "v3"),
                (CONFIG_FILE_TYPE, "config/b.toml", "v1"),
            ]
        );
    }

    #[test]
    fn bundle_replaces_older_configs_and_is_written_first() {
        let v1 = [file(CONFIG_FILE_TYPE, "config/old.toml", "v1")];
        let v2 = [
            file(CONFIG_FILE_TYPE, "config/same.toml", "v2"),
            file(CONFIG_BUNDLE_FILE_TYPE, "configs.zip", "v2"),
        ];
        let v3 = [file(CONFIG_FILE_TYPE, "config/new.toml", "v3")];

        let files = newest_config_files([&v1[..], &v2[..], &v3[..]]).unwrap();

        assert_eq!(
            names(&files),
            [
                (CONFIG_BUNDLE_FILE_TYPE, "configs.zip", "v2"),
                (CONFIG_FILE_TYPE, "config/new.toml", "v3"),
                (CONFIG_FILE_TYPE, "config/same.toml", "v2"),
            ]
        );
    }

    #[test]
    fn limits_apply_per_version() {
        let mut half = file(CONFIG_FILE_TYPE, "config/a.toml", "v1");
        half.file_size = Some((MAX_CONFIG_BUNDLE_TOTAL_SIZE / 2 + 1) as i64);
        let mut other_half = half.clone();
        other_half.file_name = "config/b.toml".to_string();
        let mut too_large = half.clone();
        too_large.file_name = "config/c.toml".to_string();

        assert!(
            newest_config_files([
                &[half.clone()][..],
                &[other_half.clone()][..]
            ])
            .is_ok()
        );
        assert!(
            newest_config_files([&[half, other_half, too_large][..]]).is_err()
        );
    }

    #[test]
    fn newest_bundle_wins() {
        let v1 = [file(CONFIG_BUNDLE_FILE_TYPE, "configs.zip", "v1")];
        let v2 = [file(CONFIG_BUNDLE_FILE_TYPE, "configs.zip", "v2")];

        let files = newest_config_files([&v1[..], &v2[..]]).unwrap();

        assert_eq!(
            names(&files),
            [(CONFIG_BUNDLE_FILE_TYPE, "configs.zip", "v2")]
        );
    }
}
