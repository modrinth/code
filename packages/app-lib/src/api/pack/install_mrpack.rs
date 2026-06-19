use crate::event::LoadingBarType;
use crate::event::emit::{
    emit_loading, init_or_edit_loading, loading_try_for_each_concurrent,
};
use crate::pack::install_from::{
    EnvType, PackFile, PackFileHash, set_instance_information,
};
use crate::state::{
    CachedEntry, EditInstance, InstanceInstallStage, SideType, cache_file_hash,
};
use crate::state::instances::ContentSourceKind;
use crate::util::fetch::{DownloadMeta, DownloadReason, fetch_mirrors, write};
use crate::util::io;
use crate::State;
use async_zip::base::read::seek::ZipFileReader as SeekZipFileReader;
use async_zip::base::read::{WithEntry, ZipEntryReader};
use async_zip::tokio::read::fs::ZipFileReader as FsZipFileReader;
use futures::StreamExt;
use path_util::SafeRelativeUtf8UnixPathBuf;

use super::install_from::{
    CreatePack, CreatePackFile, CreatePackLocation, PackFormat,
    generate_pack_from_file, generate_pack_from_version_id,
};
use crate::data::ProjectType;
use std::io::{Cursor, ErrorKind};
use std::path::Path;
use tokio::io::AsyncWriteExt;

enum MrpackZipReader {
    Memory(async_zip::tokio::read::seek::ZipFileReader<Cursor<bytes::Bytes>>),
    // Local imports stay on disk so large .mrpacks do not have to fit in memory.
    File(FsZipFileReader),
}

impl MrpackZipReader {
    async fn new(file: &CreatePackFile) -> crate::Result<Self> {
        match file {
            CreatePackFile::Bytes(file) => Ok(Self::Memory(
                SeekZipFileReader::with_tokio(Cursor::new(file.clone()))
                    .await
                    .map_err(|_| {
                        crate::Error::from(crate::ErrorKind::InputError(
                            "Failed to read input modpack zip".to_string(),
                        ))
                    })?,
            )),
            CreatePackFile::Path(path) => Ok(Self::File(
                FsZipFileReader::new(path).await.map_err(|_| {
                    crate::Error::from(crate::ErrorKind::InputError(
                        "Failed to read input modpack zip".to_string(),
                    ))
                })?,
            )),
        }
    }

    fn file(&self) -> &async_zip::ZipFile {
        match self {
            Self::Memory(reader) => reader.file(),
            Self::File(reader) => reader.file(),
        }
    }

    async fn read_entry_to_string(
        &mut self,
        index: usize,
    ) -> crate::Result<String> {
        let mut value = String::new();
        match self {
            Self::Memory(reader) => {
                let mut reader = reader.reader_with_entry(index).await?;
                reader.read_to_string_checked(&mut value).await?;
            }
            Self::File(reader) => {
                let mut reader = reader.reader_with_entry(index).await?;
                reader.read_to_string_checked(&mut value).await?;
            }
        }

        Ok(value)
    }

    async fn hash_entry(
        &mut self,
        index: usize,
    ) -> crate::Result<(u64, String)> {
        match self {
            Self::Memory(reader) => {
                hash_zip_entry(reader.reader_with_entry(index).await?).await
            }
            Self::File(reader) => {
                hash_zip_entry(reader.reader_with_entry(index).await?).await
            }
        }
    }

    async fn extract_entry(
        &mut self,
        index: usize,
        path: &Path,
        semaphore: &crate::util::fetch::IoSemaphore,
    ) -> crate::Result<(u64, String)> {
        match self {
            Self::Memory(reader) => {
                extract_zip_entry(
                    reader.reader_with_entry(index).await?,
                    path,
                    semaphore,
                )
                .await
            }
            Self::File(reader) => {
                extract_zip_entry(
                    reader.reader_with_entry(index).await?,
                    path,
                    semaphore,
                )
                .await
            }
        }
    }
}

async fn hash_zip_entry<R>(
    mut reader: ZipEntryReader<'_, R, WithEntry<'_>>,
) -> crate::Result<(u64, String)>
where
    R: futures_lite::io::AsyncBufRead + Unpin,
{
    let expected_crc32 = reader.entry().crc32();
    let mut hasher = sha1_smol::Sha1::new();
    let mut size = 0;
    let mut buffer = vec![0; 262144];

    loop {
        let bytes_read =
            futures_lite::io::AsyncReadExt::read(&mut reader, &mut buffer)
                .await?;
        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
        size += bytes_read as u64;
    }

    if reader.compute_hash() != expected_crc32 {
        return Err(async_zip::error::ZipError::CRC32CheckError.into());
    }

    Ok((size, hasher.digest().to_string()))
}

async fn extract_zip_entry<R>(
    mut reader: ZipEntryReader<'_, R, WithEntry<'_>>,
    path: &Path,
    semaphore: &crate::util::fetch::IoSemaphore,
) -> crate::Result<(u64, String)>
where
    R: futures_lite::io::AsyncBufRead + Unpin,
{
    let _permit = semaphore.0.acquire().await?;

    if let Some(parent) = path.parent() {
        io::create_dir_all(parent).await?;
    }

    let parent = path.parent().ok_or_else(|| {
        io::IOError::from(std::io::Error::other(
            "could not get parent directory for temporary file",
        ))
    })?;
    let temp_path = tempfile::NamedTempFile::new_in(parent)
        .map_err(|e| io::IOError::with_path(e, parent))?
        .into_temp_path();

    // Only replace the profile file after the ZIP entry has passed its CRC check.
    let expected_crc32 = reader.entry().crc32();
    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| io::IOError::with_path(e, &temp_path))?;
    let mut hasher = sha1_smol::Sha1::new();
    let mut size = 0;
    let mut buffer = vec![0; 262144];

    loop {
        let bytes_read =
            futures_lite::io::AsyncReadExt::read(&mut reader, &mut buffer)
                .await?;
        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .await
            .map_err(|e| io::IOError::with_path(e, &temp_path))?;
        hasher.update(&buffer[..bytes_read]);
        size += bytes_read as u64;
    }

    file.flush()
        .await
        .map_err(|e| io::IOError::with_path(e, &temp_path))?;
    drop(file);

    if reader.compute_hash() != expected_crc32 {
        return Err(async_zip::error::ZipError::CRC32CheckError.into());
    }

    temp_path.persist(path).map_err(|e| {
        let tempfile::PathPersistError { error, .. } = e;
        io::IOError::with_path(error, path)
    })?;

    Ok((size, hasher.digest().to_string()))
}

/// Install a pack
/// Wrapper around install_pack_files that generates a pack creation description, and
/// attempts to install the pack files. If it fails, it will remove the profile (fail safely)
/// Install a modpack from a mrpack file (a modrinth .zip format)
pub async fn install_zipped_mrpack(
    location: CreatePackLocation,
    instance_id: String,
) -> crate::Result<String> {
    // Get file from description
    let create_pack: CreatePack = match location {
        CreatePackLocation::FromVersionId {
            project_id,
            version_id,
            title,
            icon_url,
        } => {
            generate_pack_from_version_id(
                project_id,
                version_id,
                title,
                icon_url,
                instance_id.clone(),
                None,
                DownloadReason::Modpack,
            )
            .await?
        }
        CreatePackLocation::FromFile { path } => {
            generate_pack_from_file(path, instance_id.clone()).await?
        }
    };

    // Install pack files, and if it fails, fail safely by removing the instance.
    let result = install_zipped_mrpack_files(
        create_pack,
        false,
        DownloadReason::Modpack,
    )
    .await;

    match result {
        Ok(instance_id) => Ok(instance_id),
        Err(err) => {
            let _ = crate::api::instance::remove(&instance_id).await;

            Err(err)
        }
    }
}

/// Install all pack files from a description
/// Does not remove the profile if it fails
pub async fn install_zipped_mrpack_files(
    create_pack: CreatePack,
    ignore_lock: bool,
    reason: DownloadReason,
) -> crate::Result<String> {
    let state = &State::get().await?;

    let file = create_pack.file;
    let description = create_pack.description.clone();
    let icon = create_pack.description.icon;
    let project_id = create_pack.description.project_id;
    let version_id = create_pack.description.version_id;
    let existing_loading_bar = create_pack.description.existing_loading_bar;
    let instance_id = create_pack.description.instance_id;
    let icon_exists = icon.is_some();

    let mut zip_reader = MrpackZipReader::new(&file).await?;

    // Extract index of modrinth.index.json
    let Some(manifest_idx) = zip_reader.file().entries().iter().position(|f| {
        matches!(f.filename().as_str(), Ok("modrinth.index.json"))
    }) else {
        return Err(crate::Error::from(crate::ErrorKind::InputError(
            "No pack manifest found in mrpack".to_string(),
        )));
    };

    let mut manifest = String::new();
    manifest.push_str(&zip_reader.read_entry_to_string(manifest_idx).await?);

    let pack: PackFormat = serde_json::from_str(&manifest)?;

    if &*pack.game != "minecraft" {
        return Err(crate::ErrorKind::InputError(
            "Pack does not support Minecraft".to_string(),
        )
        .into());
    }

    // Cache the modpack file hashes for later filtering of user-added content
    // Includes both manifest file hashes and computed hashes for override files
    if let Some(ref version_id) = version_id {
        let mut file_hashes: Vec<String> = pack
            .files
            .iter()
            .filter_map(|f| f.hashes.get(&PackFileHash::Sha1).cloned())
            .collect();

        // Also hash files from overrides folders (these aren't in modrinth.index.json)
        let override_entries: Vec<usize> = zip_reader
            .file()
            .entries()
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let filename = entry.filename().as_str().ok()?;
                let is_override = (filename.starts_with("overrides/")
                    || filename.starts_with("client-overrides/")
                    || filename.starts_with("server-overrides/"))
                    && !filename.ends_with('/');
                is_override.then_some(index)
            })
            .collect();

        for index in override_entries {
            let (_, hash) = zip_reader.hash_entry(index).await?;
            file_hashes.push(hash);
        }

        let project_ids: Vec<String> = pack
            .files
            .iter()
            .filter_map(|f| {
                f.downloads.iter().find_map(|url| {
                    let parts: Vec<&str> = url.split('/').collect();
                    let data_idx = parts.iter().position(|&p| p == "data")?;
                    parts.get(data_idx + 1).map(|s| s.to_string())
                })
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        tracing::info!(
            "Caching {} modpack file hashes and {} project IDs for version {}",
            file_hashes.len(),
            project_ids.len(),
            version_id
        );
        CachedEntry::cache_modpack_files(
            version_id,
            file_hashes,
            project_ids,
            &state.pool,
        )
        .await?;
    } else {
        tracing::warn!(
            "No version_id available, skipping modpack file hash caching"
        );
    }

    set_instance_information(
        instance_id.clone(),
        &description,
        &pack.name,
        &pack.dependencies,
        ignore_lock,
    )
    .await?;

    let metadata = crate::api::instance::get(&instance_id)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!("Unknown instance {instance_id}"))
        })?;
    let instance_path = metadata.instance.path.clone();
    let instance_full_path = crate::api::instance::get_full_path(&instance_id).await?;
    let loading_bar = init_or_edit_loading(
        existing_loading_bar,
        LoadingBarType::PackDownload {
            instance_id: instance_id.clone(),
            pack_name: pack.name.clone(),
            icon,
            pack_id: project_id.clone(),
            pack_version: version_id.clone(),
        },
        100.0,
        "Downloading modpack",
    )
    .await?;

    let download_meta = DownloadMeta {
        reason,
        game_version: metadata.applied_content_set.game_version.clone(),
        loader: metadata.applied_content_set.loader.as_str().to_string(),
        dependent_on: version_id.clone(),
    };

    let num_files = pack.files.len();
    loading_try_for_each_concurrent(
        futures::stream::iter(pack.files).map(Ok::<PackFile, crate::Error>),
        None,
        Some(&loading_bar),
        70.0,
        num_files,
        None,
        |project| {
            let instance_id = instance_id.clone();
            let instance_path = instance_path.clone();
            let instance_full_path = instance_full_path.clone();
            let download_meta = download_meta.clone();
            let pack_version_id = version_id.clone();
            async move {
                //TODO: Future update: prompt user for optional files in a modpack
                if let Some(env) = project.env
                    && env
                        .get(&EnvType::Client)
                        .is_some_and(|x| x == &SideType::Unsupported)
                {
                    return Ok(());
                }

                let file = fetch_mirrors(
                    &project
                        .downloads
                        .iter()
                        .map(|x| &**x)
                        .collect::<Vec<&str>>(),
                    project.hashes.get(&PackFileHash::Sha1).map(|x| &**x),
                    Some(&download_meta),
                    None,
                    &state.fetch_semaphore,
                    &state.pool,
                )
                .await?;

                let path = instance_full_path.join(project.path.as_str());

                cache_file_hash(
                    file.clone(),
                    &instance_path,
                    project.path.as_str(),
                    project.hashes.get(&PackFileHash::Sha1).map(|x| &**x),
                    ProjectType::get_from_parent_folder(&path),
                    None,
                    &state.pool,
                )
                .await?;

                write(&path, &file, &state.io_semaphore).await?;

                if let Some(project_type) =
                    ProjectType::get_from_parent_folder(project.path.as_str())
                {
                    let hash =
                        project.hashes.get(&PackFileHash::Sha1).map(|x| &**x);
                    let file_info = if let Some(hash) = hash {
                        CachedEntry::get_file_many(
                            &[hash],
                            None,
                            &state.pool,
                            &state.api_semaphore,
                        )
                        .await?
                        .into_iter()
                        .next()
                    } else {
                        None
                    };
                    if let Some(hash) = hash {
                        crate::state::instances::commands::record_project_file(
                            &instance_id,
                            project.path.as_str(),
                            hash,
                            project.file_size as u64,
                            project_type,
                            modpack_source_kind(pack_version_id.as_deref()),
                            file_info.as_ref().map(|file| file.project_id.as_str()),
                            file_info.as_ref().map(|file| file.version_id.as_str()),
                            state,
                        )
                        .await?;
                    }
                }

                Ok(())
            }
        },
    )
    .await?;

    emit_loading(&loading_bar, 0.0, Some("Extracting overrides"))?;

    let override_file_entries = zip_reader
        .file()
        .entries()
        .iter()
        .enumerate()
        .filter_map(|(index, file)| {
            let filename = file.filename().as_str().unwrap_or_default();
            ((filename.starts_with("overrides/")
                || filename.starts_with("client-overrides/"))
                && !filename.ends_with('/'))
            .then(|| (index, file.clone()))
        })
        .collect::<Vec<_>>();
    let override_file_entries_count = override_file_entries.len();

    for (i, (index, file)) in override_file_entries.into_iter().enumerate() {
        let relative_override_file_path =
            SafeRelativeUtf8UnixPathBuf::try_from(
                file.filename().as_str().unwrap().to_string(),
            )?;
        let relative_override_file_path = relative_override_file_path
            .strip_prefix("overrides")
            .or_else(|_| relative_override_file_path.strip_prefix("client-overrides"))
            .map_err(|_| {
                crate::Error::from(crate::ErrorKind::OtherError(
                    format!("Failed to strip override prefix from override file path: {relative_override_file_path}")
                ))
            })?;

        let path = instance_full_path.join(relative_override_file_path.as_str());
        let (size, hash) = zip_reader
            .extract_entry(index, &path, &state.io_semaphore)
            .await?;

        crate::state::cache_file_hash_metadata(
            &instance_path,
            relative_override_file_path.as_str(),
            size,
            hash.clone(),
            ProjectType::get_from_parent_folder(
                relative_override_file_path.as_str(),
            ),
            None,
            &state.pool,
        )
        .await?;

        if let Some(project_type) =
            ProjectType::get_from_parent_folder(relative_override_file_path.as_str())
        {
            crate::state::instances::commands::record_project_file(
                &instance_id,
                relative_override_file_path.as_str(),
                &hash,
                size,
                project_type,
                modpack_source_kind(version_id.as_deref()),
                None,
                None,
                state,
            )
            .await?;
        }

        emit_loading(
            &loading_bar,
            30.0 / override_file_entries_count as f64,
            Some(&format!(
                "Extracting override {}/{override_file_entries_count}",
                i + 1
            )),
        )?;
    }

    // If the icon doesn't exist, we expect icon.png to be a potential icon.
    // If it doesn't exist, and an override to icon.png exists, cache and use that
    let potential_icon = instance_full_path.join("icon.png");
    if !icon_exists && potential_icon.exists() {
        crate::api::instance::edit_icon(&instance_id, Some(&potential_icon)).await?;
    }

    crate::launcher::install_minecraft_for_instance_id(
        &instance_id,
        Some(loading_bar),
        false,
    )
    .await?;

    Ok::<String, crate::Error>(instance_id.clone())
}

fn modpack_source_kind(version_id: Option<&str>) -> ContentSourceKind {
    if version_id.is_some() {
        ContentSourceKind::ModrinthModpack
    } else {
        ContentSourceKind::ImportedModpack
    }
}

#[tracing::instrument(skip(mrpack_file))]

pub async fn remove_all_related_files(
    instance_id: String,
    mrpack_file: CreatePackFile,
) -> crate::Result<()> {
    // Updates can remove files from a locally imported or downloaded pack, so share the same reader path.
    let mut zip_reader = MrpackZipReader::new(&mrpack_file).await?;

    // Extract index of modrinth.index.json
    let Some(manifest_idx) = zip_reader.file().entries().iter().position(|f| {
        matches!(f.filename().as_str(), Ok("modrinth.index.json"))
    }) else {
        return Err(crate::Error::from(crate::ErrorKind::InputError(
            "No pack manifest found in mrpack".to_string(),
        )));
    };

    let manifest = zip_reader.read_entry_to_string(manifest_idx).await?;

    let pack: PackFormat = serde_json::from_str(&manifest)?;

    if &*pack.game != "minecraft" {
        return Err(crate::ErrorKind::InputError(
            "Pack does not support Minecraft".to_string(),
        )
        .into());
    }

    crate::api::instance::edit(
        &instance_id,
        EditInstance {
            install_stage: Some(InstanceInstallStage::PackInstalling),
            ..EditInstance::default()
        },
    )
    .await?;

    // First, remove all modrinth projects by their version hashes
    // Remove all modrinth projects by their version hashes
    // We need to do a fetch to get the project ids from Modrinth
    let state = State::get().await?;
    let metadata = crate::api::instance::get(&instance_id)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!("Unknown instance {instance_id}"))
        })?;
    let instance_full_path = crate::api::instance::get_full_path(&instance_id).await?;
    let all_hashes = pack
        .files
        .iter()
        .filter_map(|f| Some(f.hashes.get(&PackFileHash::Sha1)?.clone()))
        .collect::<Vec<_>>();

    // First, get project info by hash
    let file_infos = CachedEntry::get_file_many(
        &all_hashes.iter().map(|x| &**x).collect::<Vec<_>>(),
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;

    let to_remove = file_infos
        .into_iter()
        .map(|p| p.project_id)
        .collect::<Vec<_>>();

    for file in crate::state::instances::commands::list_project_files(
        &metadata.instance.id,
        &state,
    )
    .await?
    {
        if let Some(project_id) = &file.project_id
            && to_remove.contains(project_id)
        {
            crate::state::instances::commands::remove_project(
                &metadata.instance.id,
                &file.relative_path,
                &state,
            )
            .await?;
        }
    }

    // Iterate over all Modrinth project file paths in the json, and remove them
    // (There should be few, but this removes any files the .mrpack intended as Modrinth projects but were unrecognized)
    for file in pack.files {
        match io::remove_file(instance_full_path.join(file.path.as_str())).await
        {
            Ok(_) => (),
            Err(err) if err.kind() == ErrorKind::NotFound => (),
            Err(err) => return Err(err.into()),
        }
    }

    // Iterate over each 'overrides' file and remove it
    let override_file_entries =
        zip_reader.file().entries().iter().filter(|file| {
            let filename = file.filename().as_str().unwrap_or_default();
            (filename.starts_with("overrides/")
                || filename.starts_with("client-overrides/"))
                && !filename.ends_with('/')
        });

    for file in override_file_entries {
        let relative_override_file_path =
            SafeRelativeUtf8UnixPathBuf::try_from(
                file.filename().as_str().unwrap().to_string(),
            )?;
        let relative_override_file_path = relative_override_file_path
            .strip_prefix("overrides")
            .or_else(|_| relative_override_file_path.strip_prefix("client-overrides"))
            .map_err(|_| {
                crate::Error::from(crate::ErrorKind::OtherError(
                    format!("Failed to strip override prefix from override file path: {relative_override_file_path}")
                ))
            })?;

        // Remove this file if a corresponding one exists in the filesystem
        match io::remove_file(
            instance_full_path.join(relative_override_file_path.as_str()),
        )
        .await
        {
            Ok(_) => (),
            Err(err) if err.kind() == ErrorKind::NotFound => (),
            Err(err) => return Err(err.into()),
        }
    }

    Ok(())
}
