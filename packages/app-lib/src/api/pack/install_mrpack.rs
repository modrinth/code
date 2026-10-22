use crate::State;
use crate::api::instance::CONFIG_FILE_EXTENSIONS;
use crate::api::instance::GameOptionsPackSource;
use crate::api::instance::synced_servers::MODPACK_SERVER_PATHS;
use crate::event::emit::loading_try_for_each_concurrent;
use crate::install::{
    InstallErrorContext, InstallJobEventKind, InstallPhaseDetails,
    InstallPhaseId, InstallProgress, InstallProgressReporter,
    InstallProgressSecondary,
};
use crate::pack::install_from::{
    EnvType, PackFile, PackFileHash, set_instance_information,
};
use crate::state::instances::ContentSourceKind;
use crate::state::instances::commands::{
    ContentOrigin, InstallContent, install_stored_file,
};
use crate::state::{
    CachedEntry, CachedFile, EditInstance, InstanceInstallStage, SideType,
};
use crate::util::fetch::{
    DownloadMeta, DownloadReason, FetchProgressFn, fetch_content_file,
};
use crate::util::io;
use anyhow::{Context, anyhow, bail};
use futures::StreamExt;
use path_util::SafeRelativeUtf8UnixPathBuf;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use super::install_from::{CreatePack, CreatePackFile, PackFormat};
use crate::data::ProjectType;
use std::io::{BufReader, Cursor, ErrorKind, Read, Seek, Write};
use std::path::{Path, PathBuf};

use tokio::sync::Mutex;

type ExtractProgressFn<'a> = dyn FnMut(u64) -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send + 'a>>
    + Send
    + 'a;
type HashProgressFn<'a> = dyn FnMut(u64) -> crate::Result<()> + Send + 'a;
const MODPACK_CONTENT_DOWNLOAD_CONCURRENCY: usize = 4;
const MRPACK_WARNING_IGNORED_EXTENSIONS: &[&str] = &["rpo"];

fn is_ignored_mrpack_warning_file(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            CONFIG_FILE_EXTENSIONS
                .iter()
                .chain(MRPACK_WARNING_IGNORED_EXTENSIONS)
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

#[derive(Clone)]
struct ModpackContentInstallContext {
    instance_id: String,
    instance_path: String,
    instance_full_path: PathBuf,
    download_meta: DownloadMeta,
    pack_version_id: Option<String>,
    pack_project_id: Option<String>,
    reporter: InstallProgressReporter,
    modpack_details: InstallPhaseDetails,
    content_progress: Arc<AtomicU64>,
    content_bytes_progress: Arc<AtomicU64>,
    active_download_bytes: Arc<Mutex<HashMap<String, u64>>>,
    file_infos_by_hash: Arc<HashMap<String, CachedFile>>,
    num_files: usize,
    content_total_bytes: u64,
}

impl ModpackContentInstallContext {
    async fn mark_downloaded(
        &self,
        file_size: u64,
        event: InstallJobEventKind,
    ) -> crate::Result<()> {
        let current = self.content_progress.fetch_add(1, Ordering::Relaxed) + 1;
        let current_bytes = self
            .content_bytes_progress
            .fetch_add(file_size, Ordering::Relaxed)
            + file_size;

        self.reporter
            .update_with_events(
                InstallPhaseId::DownloadingContent,
                Some(InstallProgress {
                    current,
                    total: self.num_files as u64,
                    secondary: (self.content_total_bytes > 0).then_some(
                        InstallProgressSecondary {
                            current: current_bytes
                                .min(self.content_total_bytes),
                            total: self.content_total_bytes,
                        },
                    ),
                }),
                self.modpack_details.clone(),
                vec![event],
            )
            .await
    }

    async fn remove_active_download(&self, path: &str) {
        let mut active_download_bytes = self.active_download_bytes.lock().await;
        active_download_bytes.remove(path);
    }

    async fn update_active_download(
        &self,
        path: String,
        downloaded: u64,
    ) -> u64 {
        let mut active_download_bytes = self.active_download_bytes.lock().await;
        active_download_bytes.insert(path, downloaded);
        active_download_bytes.values().sum::<u64>()
    }
}

trait ZipReadSeek: Read + Seek + Send {}

impl<T: Read + Seek + Send> ZipReadSeek for T {}

type MrpackZipArchive = zip::ZipArchive<Box<dyn ZipReadSeek>>;

#[derive(Clone)]
struct MrpackZipEntry {
    name: String,
    size: u64,
}

struct MrpackZipReader {
    archive: Arc<std::sync::Mutex<MrpackZipArchive>>,
    entries: Vec<MrpackZipEntry>,
}

impl MrpackZipReader {
    async fn new(file: &CreatePackFile) -> crate::Result<Self> {
        let source = match file {
            CreatePackFile::Bytes(bytes) => either::Either::Left(bytes.clone()),
            CreatePackFile::Downloaded(file) => {
                either::Either::Right(file.path().to_path_buf())
            }
            CreatePackFile::Path(path) => either::Either::Right(path.clone()),
        };
        tokio::task::spawn_blocking(move || -> crate::Result<Self> {
            let source: Box<dyn ZipReadSeek> = match source {
                either::Either::Left(bytes) => Box::new(Cursor::new(bytes)),
                either::Either::Right(path) => {
                    Box::new(BufReader::new(std::fs::File::open(path)?))
                }
            };
            let mut archive = zip::ZipArchive::new(source).context(
                crate::ErrorKind::InputError(
                    "failed to read input modpack zip".to_string(),
                ),
            )?;
            let entries = (0..archive.len())
                .map(|index| -> crate::Result<MrpackZipEntry> {
                    let entry = archive.by_index_raw(index)?;
                    Ok(MrpackZipEntry {
                        name: entry.name()?.into_owned(),
                        size: entry.size(),
                    })
                })
                .collect::<crate::Result<Vec<_>>>()?;
            Ok(Self {
                archive: Arc::new(std::sync::Mutex::new(archive)),
                entries,
            })
        })
        .await?
    }

    fn entries(&self) -> &[MrpackZipEntry] {
        &self.entries
    }

    async fn read_entry_to_string(
        &mut self,
        index: usize,
    ) -> crate::Result<String> {
        let archive = Arc::clone(&self.archive);
        tokio::task::spawn_blocking(move || -> crate::Result<String> {
            let mut archive = archive
                .lock()
                .map_err(|_| anyhow!("modpack ZIP reader lock poisoned"))?;
            let mut entry = archive.by_index(index)?;
            let mut value = String::new();
            entry.read_to_string(&mut value)?;
            Ok(value)
        })
        .await?
    }

    async fn hash_entry(
        &mut self,
        index: usize,
        mut progress: Option<&mut HashProgressFn<'_>>,
    ) -> crate::Result<(u64, String)> {
        let archive = Arc::clone(&self.archive);
        let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
        let task = tokio::task::spawn_blocking(move || -> crate::Result<_> {
            let mut archive = archive
                .lock()
                .map_err(|_| anyhow!("modpack ZIP reader lock poisoned"))?;
            hash_zip_entry(archive.by_index(index)?, |bytes| {
                sender
                    .blocking_send(bytes)
                    .map_err(|_| anyhow!("modpack hashing cancelled"))
            })
        });
        while let Some(bytes) = receiver.recv().await {
            if let Some(progress) = progress.as_mut() {
                progress(bytes)?;
            }
        }
        task.await?
    }

    async fn extract_entry(
        &mut self,
        index: usize,
        path: &Path,
        semaphore: &crate::util::fetch::IoSemaphore,
        mut progress: Option<&mut ExtractProgressFn<'_>>,
    ) -> crate::Result<(u64, String)> {
        let _permit = semaphore.0.acquire().await?;
        let archive = Arc::clone(&self.archive);
        let destination = path.to_path_buf();
        let path = destination.clone();
        let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
        let task = tokio::task::spawn_blocking(move || -> crate::Result<_> {
            let mut archive = archive
                .lock()
                .map_err(|_| anyhow!("modpack ZIP reader lock poisoned"))?;
            extract_zip_entry(archive.by_index(index)?, &path, |bytes| {
                sender
                    .blocking_send(bytes)
                    .map_err(|_| anyhow!("modpack extraction cancelled"))
            })
        });
        while let Some(bytes) = receiver.recv().await {
            if let Some(progress) = progress.as_mut() {
                progress(bytes).await?;
            }
        }
        let (temporary, size, hash) = task.await??;
        // Keep publication in this task so cancellation cannot detach a pending rename.
        temporary.persist(&destination).map_err(|error| {
            io::IOError::with_path(error.error, &destination)
        })?;
        Ok((size, hash))
    }
}

fn hash_zip_entry(
    mut reader: impl Read,
    mut progress: impl FnMut(u64) -> crate::Result<()>,
) -> crate::Result<(u64, String)> {
    let mut hasher = sha1_smol::Sha1::new();
    let mut size = 0;
    let mut buffer = vec![0; 262144];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
        size += bytes_read as u64;
        progress(bytes_read as u64)?;
    }
    Ok((size, hasher.digest().to_string()))
}

pub(crate) async fn get_external_files_from_mrpack(
    file: &CreatePackFile,
    mut progress: impl FnMut(u64, u64) -> crate::Result<()> + Send,
) -> crate::Result<Vec<String>> {
    let mut zip_reader = MrpackZipReader::new(file).await?;
    let Some(manifest_idx) = zip_reader
        .entries()
        .iter()
        .position(|entry| matches!(entry.name.as_str(), "modrinth.index.json"))
    else {
        bail!(crate::ErrorKind::InputError(
            "no pack manifest found in mrpack".to_string(),
        ));
    };

    let manifest = zip_reader.read_entry_to_string(manifest_idx).await?;
    let pack: PackFormat = serde_json::from_str(&manifest)?;
    let mut destinations = HashMap::<String, String>::new();
    for file in &pack.files {
        if file.env.as_ref().is_some_and(|env| {
            env.get(&EnvType::Client) == Some(&SideType::Unsupported)
        }) {
            continue;
        }
        let path = file.path.as_str();
        if crate::state::content_store::is_managed_content_path(path) {
            let canonical = path.trim_end_matches(".disabled").to_string();
            if let Some(previous) =
                destinations.insert(canonical, path.to_string())
            {
                bail!(crate::ErrorKind::InputError(format!(
                    "modpack content paths {previous} and {path} refer to the same file"
                )));
            }
        }
    }
    for entry in zip_reader.entries() {
        let name = entry.name.as_str();
        let Some(path) = name
            .strip_prefix("overrides/")
            .or_else(|| name.strip_prefix("client-overrides/"))
        else {
            continue;
        };
        if crate::state::content_store::is_managed_content_path(path) {
            let canonical = path.trim_end_matches(".disabled").to_string();
            if let Some(previous) =
                destinations.insert(canonical, path.to_string())
            {
                // Overrides may replace the same physical path, but must not change its enabled form.
                if previous != path {
                    bail!(crate::ErrorKind::InputError(format!(
                        "modpack content paths {previous} and {path} refer to the same file"
                    )));
                }
            }
        }
    }

    let mut candidates = pack
        .files
        .into_iter()
        .filter_map(|file| {
            let path = file.path.as_str();
            if is_ignored_mrpack_warning_file(path) {
                return None;
            }
            let hash = file.hashes.get(&PackFileHash::Sha1)?.clone();
            let file_name = path.rsplit('/').next()?.to_string();
            Some((file_name, hash))
        })
        .collect::<Vec<_>>();

    let override_entries = zip_reader
        .entries()
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let path = entry.name.as_str();
            let relative_path = external_override_relative_path(path)?;
            let file_name = relative_path.rsplit('/').next()?.to_string();
            Some((index, file_name, entry.size))
        })
        .collect::<Vec<_>>();

    let total_bytes = override_entries
        .iter()
        .map(|(_, _, size)| size)
        .sum::<u64>();
    let min_delta = (total_bytes / 200).max(256 * 1024);
    let mut current_bytes = 0_u64;
    let mut last_reported_bytes = 0_u64;
    let mut report_progress = |bytes_read: u64| {
        current_bytes = current_bytes.saturating_add(bytes_read);
        if current_bytes >= total_bytes
            || current_bytes.saturating_sub(last_reported_bytes) < min_delta
        {
            return Ok(());
        }

        last_reported_bytes = current_bytes;
        progress(current_bytes.min(total_bytes), total_bytes)
    };

    for (index, file_name, _) in override_entries {
        let (_, hash) = zip_reader
            .hash_entry(index, Some(&mut report_progress))
            .await?;
        candidates.push((file_name, hash));
    }

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let state = State::get().await?;
    let hashes = candidates
        .iter()
        .map(|(_, hash)| hash.as_str())
        .collect::<Vec<_>>();
    let recognized_hashes = match CachedEntry::get_file_many(
        &hashes,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await
    {
        Ok(files) => files
            .into_iter()
            .map(|file| file.hash)
            .collect::<HashSet<_>>(),
        Err(err) => {
            tracing::warn!("Failed to look up files in imported mrpack: {err}");
            HashSet::new()
        }
    };

    let mut external_files = candidates
        .into_iter()
        .filter_map(|(file_name, hash)| {
            (!recognized_hashes.contains(&hash)).then_some(file_name)
        })
        .collect::<Vec<_>>();
    external_files.sort();
    external_files.dedup();
    Ok(external_files)
}

pub(crate) async fn get_external_file_hashing_size_from_mrpack(
    file: &CreatePackFile,
) -> crate::Result<u64> {
    let zip_reader = MrpackZipReader::new(file).await?;
    Ok(zip_reader
        .entries()
        .iter()
        .filter_map(|entry| {
            let path = entry.name.as_str();
            external_override_relative_path(path).map(|_| entry.size)
        })
        .sum())
}

fn external_override_relative_path(path: &str) -> Option<&str> {
    let relative_path = path
        .strip_prefix("overrides/")
        .or_else(|| path.strip_prefix("client-overrides/"))?;
    (!path.ends_with('/')
        && !is_ignored_mrpack_warning_file(relative_path)
        && ProjectType::get_from_parent_folder(relative_path).is_some())
    .then_some(relative_path)
}

fn extract_zip_entry(
    mut reader: impl Read,
    path: &Path,
    mut progress: impl FnMut(u64) -> crate::Result<()>,
) -> crate::Result<(tempfile::TempPath, u64, String)> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| io::IOError::with_path(error, parent))?;
    }
    let parent = path.parent().ok_or_else(|| {
        io::IOError::from(std::io::Error::other(
            "could not get parent directory for temporary file",
        ))
    })?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| io::IOError::with_path(error, parent))?;
    let mut hasher = sha1_smol::Sha1::new();
    let mut size = 0;
    let mut buffer = vec![0; 262144];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        temporary
            .write_all(&buffer[..bytes_read])
            .map_err(|error| io::IOError::with_path(error, temporary.path()))?;
        hasher.update(&buffer[..bytes_read]);
        size += bytes_read as u64;
        progress(bytes_read as u64)?;
    }
    temporary
        .flush()
        .map_err(|error| io::IOError::with_path(error, temporary.path()))?;
    Ok((
        temporary.into_temp_path(),
        size,
        hasher.digest().to_string(),
    ))
}

pub(crate) fn install_zipped_mrpack_files_with_reporter(
    create_pack: CreatePack,
    ignore_lock: bool,
    reason: DownloadReason,
    reporter: InstallProgressReporter,
) -> impl Future<Output = crate::Result<String>> + Send + 'static {
    Box::pin(install_zipped_mrpack_files_with_reporter_inner(
        create_pack,
        ignore_lock,
        reason,
        reporter,
    ))
}

async fn install_zipped_mrpack_files_with_reporter_inner(
    create_pack: CreatePack,
    ignore_lock: bool,
    reason: DownloadReason,
    reporter: InstallProgressReporter,
) -> crate::Result<String> {
    let state = &State::get().await?;

    let file = create_pack.file;
    let description = create_pack.description.clone();
    let icon = create_pack.description.icon;
    let project_id = create_pack.description.project_id;
    let version_id = create_pack.description.version_id;
    let instance_id = create_pack.description.instance_id;
    let mut icon_exists = icon.is_some();
    let source_path = pack_source_path(&file);

    reporter
        .set_context(
            InstallErrorContext::new("read modpack archive")
                .maybe_project_id(project_id.clone())
                .maybe_version_id(version_id.clone())
                .source_path(source_path.clone())
                .build(),
        )
        .await?;
    let mut zip_reader = MrpackZipReader::new(&file).await?;
    let instance_full_path =
        crate::api::instance::get_full_path(&instance_id).await?;
    let modpack_details = InstallPhaseDetails::Modpack {
        project_id: project_id.clone(),
        version_id: version_id.clone(),
        title: description.override_title.clone(),
    };
    reporter
        .update(
            InstallPhaseId::ReadingPackManifest,
            None,
            modpack_details.clone(),
        )
        .await?;
    reporter
        .set_context(
            InstallErrorContext::new("read modpack manifest")
                .maybe_project_id(project_id.clone())
                .maybe_version_id(version_id.clone())
                .source_path(source_path.clone())
                .entry_path("modrinth.index.json")
                .build(),
        )
        .await?;

    // Extract index of modrinth.index.json
    let Some(manifest_idx) = zip_reader
        .entries()
        .iter()
        .position(|f| matches!(f.name.as_str(), "modrinth.index.json"))
    else {
        bail!(crate::ErrorKind::InputError(
            "no pack manifest found in mrpack".to_string(),
        ));
    };

    let mut manifest = String::new();
    manifest.push_str(&zip_reader.read_entry_to_string(manifest_idx).await?);

    let pack: PackFormat = serde_json::from_str(&manifest)?;
    if &*pack.game != "minecraft" {
        bail!(crate::ErrorKind::InputError(
            "pack does not support Minecraft".to_string(),
        ));
    }

    reporter
        .update(InstallPhaseId::ResolvingPack, None, modpack_details.clone())
        .await?;

    if !icon_exists {
        let icon_entry = zip_reader.entries().iter().enumerate().find_map(
            |(index, entry)| {
                matches!(
                    entry.name.as_str(),
                    "icon.png"
                        | "overrides/icon.png"
                        | "client-overrides/icon.png"
                )
                .then_some(index)
            },
        );

        if let Some(icon_entry) = icon_entry {
            let icon_path = instance_full_path.join("icon.png");
            zip_reader
                .extract_entry(
                    icon_entry,
                    &icon_path,
                    &state.io_semaphore,
                    None,
                )
                .await?;
            crate::api::instance::edit_icon(&instance_id, Some(&icon_path))
                .await?;
            icon_exists = true;
        }
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
            .entries()
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let filename = entry.name.as_str();
                let is_override = (filename.starts_with("overrides/")
                    || filename.starts_with("client-overrides/")
                    || filename.starts_with("server-overrides/"))
                    && !filename.ends_with('/');
                is_override.then_some(index)
            })
            .collect();

        for index in override_entries {
            let (_, hash) = zip_reader.hash_entry(index, None).await?;
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
        Some(&pack.version_id),
        &pack.dependencies,
        ignore_lock,
        &state.directories,
    )
    .await?;

    let metadata = crate::api::instance::get(&instance_id).await?.ok_or(
        crate::ErrorKind::InputError(format!("unknown instance {instance_id}")),
    )?;
    let instance_path = metadata.instance.path.clone();
    let download_meta = DownloadMeta {
        reason,
        game_version: metadata.applied_content_set.game_version.clone(),
        loader: metadata.applied_content_set.loader.as_str().to_string(),
        dependent_on: version_id.clone(),
    };

    let num_files = pack.files.len();
    let content_total_bytes = pack
        .files
        .iter()
        .map(|file| file.file_size as u64)
        .sum::<u64>();
    reporter
        .update_with_events(
            InstallPhaseId::DownloadingContent,
            Some(InstallProgress {
                current: 0,
                total: num_files as u64,
                secondary: (content_total_bytes > 0).then_some(
                    InstallProgressSecondary {
                        current: 0,
                        total: content_total_bytes,
                    },
                ),
            }),
            modpack_details.clone(),
            vec![InstallJobEventKind::ContentDownloadStarted {
                files: num_files as u64,
                bytes: (content_total_bytes > 0).then_some(content_total_bytes),
            }],
        )
        .await?;
    let content_progress = Arc::new(AtomicU64::new(0));
    let content_bytes_progress = Arc::new(AtomicU64::new(0));
    let active_download_bytes =
        Arc::new(Mutex::new(HashMap::<String, u64>::new()));
    let file_info_hashes = pack
        .files
        .iter()
        .filter_map(|file| {
            file.hashes.get(&PackFileHash::Sha1).map(String::as_str)
        })
        .collect::<Vec<_>>();
    let file_infos_by_hash = Arc::new(
        CachedEntry::get_file_many(
            &file_info_hashes,
            None,
            &state.pool,
            &state.api_semaphore,
        )
        .await?
        .into_iter()
        .map(|file| (file.hash.clone(), file))
        .collect::<HashMap<_, _>>(),
    );
    let content_context = ModpackContentInstallContext {
        instance_id: instance_id.clone(),
        instance_path: instance_path.clone(),
        instance_full_path: instance_full_path.clone(),
        download_meta,
        pack_version_id: version_id.clone(),
        pack_project_id: project_id.clone(),
        reporter: reporter.clone(),
        modpack_details: modpack_details.clone(),
        content_progress,
        content_bytes_progress,
        active_download_bytes,
        file_infos_by_hash,
        num_files,
        content_total_bytes,
    };
    loading_try_for_each_concurrent(
        futures::stream::iter(pack.files).map(Ok::<PackFile, crate::Error>),
        Some(MODPACK_CONTENT_DOWNLOAD_CONCURRENCY),
        None,
        70.0,
        num_files,
        None,
        |project| {
            let content_context = content_context.clone();
            async move {
                let project_size = project.file_size as u64;
                let project_path = project.path.as_str().to_string();
                let target_path = content_context
                    .instance_full_path
                    .join(project.path.as_str());

                //TODO: Future update: prompt user for optional files in a modpack
                if let Some(env) = project.env.as_ref()
                    && env
                        .get(&EnvType::Client)
                        .is_some_and(|x| x == &SideType::Unsupported)
                {
                    content_context
                        .mark_downloaded(
                            project_size,
                            InstallJobEventKind::ContentFileSkipped {
                                path: project_path,
                                reason: "unsupported on client".to_string(),
                            },
                        )
                        .await?;
                    return Ok(());
                }

                let context =
                    InstallErrorContext::new("download modpack content file")
                        .maybe_project_id(
                            content_context.pack_project_id.clone(),
                        )
                        .maybe_version_id(
                            content_context.pack_version_id.clone(),
                        )
                        .file_path(project_path.clone())
                        .target_path(target_path.display().to_string())
                        .urls(project.downloads.clone())
                        .maybe_expected_hash(
                            project.hashes.get(&PackFileHash::Sha1).cloned(),
                        )
                        .expected_size(project_size)
                        .build();
                content_context
                    .reporter
                    .set_transient_context(context.clone())
                    .await?;

                let progress_key = project_path.clone();
                let progress_context = content_context.clone();
                let min_download_progress_delta =
                    (project_size / 200).max(256 * 1024);
                let mut last_reported_downloaded = 0_u64;
                let mut report_download_progress = move |downloaded: u64,
                                                         _total_size: u64|
                      -> Pin<
                    Box<dyn Future<Output = crate::Result<()>> + Send>,
                > {
                    if downloaded < project_size
                        && downloaded.saturating_sub(last_reported_downloaded)
                            < min_download_progress_delta
                    {
                        return Box::pin(async { Ok(()) });
                    }

                    last_reported_downloaded = downloaded;
                    let progress_context = progress_context.clone();
                    let progress_key = progress_key.clone();
                    Box::pin(async move {
                        let active_bytes = progress_context
                            .update_active_download(progress_key, downloaded)
                            .await;
                        let current_bytes = progress_context
                            .content_bytes_progress
                            .load(Ordering::Relaxed)
                            .saturating_add(active_bytes)
                            .min(progress_context.content_total_bytes);
                        progress_context
                            .reporter
                            .update(
                                InstallPhaseId::DownloadingContent,
                                Some(InstallProgress {
                                    current: progress_context
                                        .content_progress
                                        .load(Ordering::Relaxed),
                                    total: progress_context.num_files as u64,
                                    secondary: (progress_context
                                        .content_total_bytes
                                        > 0)
                                    .then_some(InstallProgressSecondary {
                                        current: current_bytes,
                                        total: progress_context
                                            .content_total_bytes,
                                    }),
                                }),
                                progress_context.modpack_details.clone(),
                            )
                            .await?;
                        Ok(())
                    })
                };
                let progress =
                    &mut report_download_progress as &mut FetchProgressFn<'_>;
                let file = match fetch_content_file(
                    state,
                    &project
                        .downloads
                        .iter()
                        .map(|x| &**x)
                        .collect::<Vec<&str>>(),
                    project
                        .hashes
                        .get(&PackFileHash::Sha512)
                        .map(String::as_str),
                    Some(project_size),
                    Some(&content_context.download_meta),
                    Some(progress),
                )
                .await
                {
                    Ok(file) => {
                        content_context
                            .remove_active_download(&project_path)
                            .await;
                        file
                    }
                    Err(error) => {
                        content_context
                            .remove_active_download(&project_path)
                            .await;
                        content_context
                            .reporter
                            .persist_failure_context(context)
                            .await;
                        return Err(error);
                    }
                };
                let downloaded_bytes = if file.reused { 0 } else { file.size };

                if crate::state::content_store::is_managed_content_path(
                    &project_path,
                ) {
                    let project_type =
                        ProjectType::get_from_parent_folder(&project_path)
                            .ok_or(crate::ErrorKind::InputError(
                                "unsupported content path".to_string(),
                            ))?;
                    let file_info =
                        project.hashes.get(&PackFileHash::Sha1).and_then(
                            |hash| content_context.file_infos_by_hash.get(hash),
                        );
                    let stored_file =
                        file.store_file(&state.content_store).await?;
                    content_context
                        .reporter
                        .preserve_failure_context(
                            context.clone(),
                            install_stored_file(
                                &content_context.instance_id,
                                InstallContent {
                                    requested_path: &project_path,
                                    stored_file: &stored_file,
                                    project_type,
                                    source_kind: modpack_source_kind(
                                        content_context
                                            .pack_version_id
                                            .as_deref(),
                                    ),
                                    origin: file_info.map(|file| {
                                        ContentOrigin {
                                            project_id: &file.project_id,
                                            version_id: &file.version_id,
                                        }
                                    }),
                                    enabled_override: None,
                                    previous_path: None,
                                },
                                state,
                            )
                            .await,
                        )
                        .await?;
                } else {
                    let destination = state
                        .content_store
                        .instance_path(
                            &content_context.instance_path,
                            &project_path,
                        )
                        .await?;
                    content_context
                        .reporter
                        .preserve_failure_context(
                            context.clone(),
                            file.copy_to(&destination, &state.io_semaphore)
                                .await,
                        )
                        .await?;
                }

                content_context
                    .mark_downloaded(
                        project_size,
                        InstallJobEventKind::ContentFileCompleted {
                            path: project_path,
                            bytes: downloaded_bytes,
                            reused: file.reused,
                        },
                    )
                    .await?;
                Ok(())
            }
        },
    )
    .await?;

    let has_override = |path: &str| {
        zip_reader
            .entries()
            .iter()
            .any(|file| file.name.as_str() == path)
    };
    let has_client_options_override =
        has_override("client-overrides/options.txt");
    let has_options_override = has_override("overrides/options.txt");
    let has_client_yosbr_options_override =
        has_override("client-overrides/config/yosbr/options.txt");
    let has_yosbr_options_override =
        has_override("overrides/config/yosbr/options.txt");
    let has_client_default_options_override =
        has_override("client-overrides/config/defaultoptions/options.txt")
            || has_override(
                "client-overrides/config/defaultoptions/keybindings.txt",
            );
    let has_default_options_override =
        has_override("overrides/config/defaultoptions/options.txt")
            || has_override("overrides/config/defaultoptions/keybindings.txt");
    let override_file_entries = zip_reader
        .entries()
        .iter()
        .enumerate()
        .filter_map(|(index, file)| {
            let filename = file.name.as_str();
            let is_override = (filename.starts_with("overrides/")
                || filename.starts_with("client-overrides/"))
                && !filename.ends_with('/');
            let shadowed_sync_override =
                filename.strip_prefix("overrides/").is_some_and(|path| {
                    (MODPACK_SERVER_PATHS.contains(&path)
                        || matches!(
                            path,
                            "options.txt"
                                | "config/yosbr/options.txt"
                                | "config/defaultoptions/options.txt"
                                | "config/defaultoptions/keybindings.txt"
                        ))
                        && has_override(&format!("client-overrides/{path}"))
                });
            (is_override && !shadowed_sync_override)
                .then(|| (index, file.clone()))
        })
        .collect::<Vec<_>>();
    let override_total_bytes = override_file_entries
        .iter()
        .map(|(_, file)| file.size)
        .sum::<u64>();
    let servers_override_path = MODPACK_SERVER_PATHS.into_iter().find(|path| {
        has_override(&format!("client-overrides/{path}"))
            || has_override(&format!("overrides/{path}"))
    });
    let game_options_override_source = if has_client_options_override {
        Some(GameOptionsPackSource::ClientOverrides)
    } else if has_options_override {
        Some(GameOptionsPackSource::Overrides)
    } else if has_client_yosbr_options_override {
        Some(GameOptionsPackSource::ClientOverridesYosbr)
    } else if has_yosbr_options_override {
        Some(GameOptionsPackSource::OverridesYosbr)
    } else if has_client_default_options_override {
        Some(GameOptionsPackSource::ClientOverridesDefaultOptions)
    } else if has_default_options_override {
        Some(GameOptionsPackSource::OverridesDefaultOptions)
    } else {
        None
    };
    let progress = (override_total_bytes > 0).then_some(InstallProgress {
        current: 0,
        total: override_total_bytes,
        secondary: None,
    });
    reporter
        .update(
            InstallPhaseId::ExtractingOverrides,
            progress,
            modpack_details.clone(),
        )
        .await?;

    let extracted_override_bytes = Arc::new(AtomicU64::new(0));
    let mut last_reported_override_bytes = 0_u64;
    let reporter_for_overrides = reporter.clone();
    let details_for_overrides = modpack_details.clone();
    let mut report_override_progress = |bytes_read: u64| -> Pin<
        Box<dyn Future<Output = crate::Result<()>> + Send>,
    > {
        let current = extracted_override_bytes
            .fetch_add(bytes_read, Ordering::Relaxed)
            + bytes_read;
        let min_delta = (override_total_bytes / 200).max(256 * 1024);
        if current < override_total_bytes
            && current.saturating_sub(last_reported_override_bytes) < min_delta
        {
            return Box::pin(async { Ok(()) });
        }

        last_reported_override_bytes = current;
        let reporter = reporter_for_overrides.clone();
        let details = details_for_overrides.clone();
        Box::pin(async move {
            reporter
                .update(
                    InstallPhaseId::ExtractingOverrides,
                    Some(InstallProgress {
                        current: current.min(override_total_bytes),
                        total: override_total_bytes,
                        secondary: None,
                    }),
                    details,
                )
                .await?;
            Ok(())
        })
    };

    for (index, file) in override_file_entries {
        let relative_override_file_path =
            SafeRelativeUtf8UnixPathBuf::try_from(file.name.to_string())?;
        let relative_override_file_path = relative_override_file_path
			.strip_prefix("overrides")
			.or_else(|_| relative_override_file_path.strip_prefix("client-overrides"))
            .context(format!("failed to strip override prefix from override file path {relative_override_file_path}"))?;

        let managed = crate::state::content_store::is_managed_content_path(
            relative_override_file_path.as_str(),
        );
        let temporary = if managed {
            let (file, path) = state.content_store.temporary().await?;
            drop(file);
            Some(path)
        } else {
            None
        };
        let path = match &temporary {
            Some(path) => path.to_path_buf(),
            None => {
                state
                    .content_store
                    .instance_path(
                        &instance_path,
                        relative_override_file_path.as_str(),
                    )
                    .await?
            }
        };
        let override_context =
            InstallErrorContext::new("extract modpack override")
                .maybe_project_id(project_id.clone())
                .maybe_version_id(version_id.clone())
                .source_path(source_path.clone())
                .entry_path(file.name.as_str())
                .target_path(path.display().to_string())
                .build();
        reporter
            .set_transient_context(override_context.clone())
            .await?;
        let extract_result = if override_total_bytes > 0 {
            let progress =
                &mut report_override_progress as &mut ExtractProgressFn<'_>;
            zip_reader
                .extract_entry(
                    index,
                    &path,
                    &state.io_semaphore,
                    Some(progress),
                )
                .await
        } else {
            zip_reader
                .extract_entry(index, &path, &state.io_semaphore, None)
                .await
        };
        let (size, hash) = reporter
            .preserve_failure_context(override_context, extract_result)
            .await?;
        let modified_at_ns =
            crate::state::file_modified_at_ns(&io::metadata(&path).await?)?;

        {
            let _permit = state.installs.db_semaphore.acquire().await?;
            let record_context =
                InstallErrorContext::new("record modpack override")
                    .maybe_project_id(project_id.clone())
                    .maybe_version_id(version_id.clone())
                    .source_path(source_path.clone())
                    .entry_path(file.name.as_str())
                    .target_path(path.display().to_string())
                    .build();
            reporter
                .preserve_failure_context(
                    record_context.clone(),
                    crate::state::cache_file_hash_metadata(
                        &instance_path,
                        relative_override_file_path.as_str(),
                        size,
                        modified_at_ns,
                        hash.clone(),
                        ProjectType::get_from_parent_folder(
                            relative_override_file_path.as_str(),
                        ),
                        None,
                        &state.pool,
                    )
                    .await,
                )
                .await?;

            if managed
                && let Some(project_type) = ProjectType::get_from_parent_folder(
                    relative_override_file_path.as_str(),
                )
            {
                let stored_file = state.content_store.store_file(&path).await?;
                reporter
                    .preserve_failure_context(
                        record_context,
                        install_stored_file(
                            &instance_id,
                            InstallContent {
                                requested_path: relative_override_file_path
                                    .as_str(),
                                stored_file: &stored_file,
                                project_type,
                                source_kind: modpack_source_kind(
                                    version_id.as_deref(),
                                ),
                                origin: None,
                                enabled_override: None,
                                previous_path: None,
                            },
                            state,
                        )
                        .await,
                    )
                    .await?;
            }
        }
    }

    crate::api::instance::synced_servers::capture_modpack_server_override(
        &instance_id,
        servers_override_path,
    )
    .await?;
    if let Err(error) = crate::api::instance::capture_game_options_pack_base(
        &instance_id,
        game_options_override_source,
    )
    .await
    {
        tracing::warn!(
            "The modpack was installed, but its options.txt could not be captured for game-settings sync: {error}"
        );
    }
    crate::api::instance::reconcile_instance_after_pack_update(&instance_id)
        .await?;

    // If the icon doesn't exist, we expect icon.png to be a potential icon.
    // If it doesn't exist, and an override to icon.png exists, cache and use that
    let potential_icon = instance_full_path.join("icon.png");
    if !icon_exists && potential_icon.exists() {
        crate::api::instance::edit_icon(&instance_id, Some(&potential_icon))
            .await?;
    }

    crate::launcher::install_minecraft_for_instance_id_with_reporter(
        &instance_id,
        false,
        Some(reporter.clone()),
        state,
    )
    .await?;
    reporter.clear_context().await?;

    Ok::<String, crate::Error>(instance_id.clone())
}

fn pack_source_path(file: &CreatePackFile) -> String {
    match file {
        CreatePackFile::Bytes(_) => "downloaded mrpack bytes".to_string(),
        CreatePackFile::Downloaded(_) => "downloaded mrpack file".to_string(),
        CreatePackFile::Path(path) => path.display().to_string(),
    }
}

fn modpack_source_kind(version_id: Option<&str>) -> ContentSourceKind {
    if version_id.is_some() {
        ContentSourceKind::ModrinthModpack
    } else {
        ContentSourceKind::ImportedModpack
    }
}

/// Instance-relative paths installing or removing this pack can write: its
/// manifest files, its overrides and the instance icon.
pub(crate) async fn mrpack_instance_paths(
    mrpack_file: &CreatePackFile,
) -> crate::Result<Vec<String>> {
    let mut zip_reader = MrpackZipReader::new(mrpack_file).await?;
    let Some(manifest_idx) = zip_reader
        .entries()
        .iter()
        .position(|f| matches!(f.name.as_str(), "modrinth.index.json"))
    else {
        return Err(crate::Error::from(crate::ErrorKind::InputError(
            "No pack manifest found in mrpack".to_string(),
        )));
    };
    let manifest = zip_reader.read_entry_to_string(manifest_idx).await?;
    let pack: PackFormat = serde_json::from_str(&manifest)?;

    let mut paths = vec!["icon.png".to_string()];
    paths.extend(pack.files.iter().map(|file| file.path.to_string()));
    for entry in zip_reader.entries() {
        let filename = entry.name.as_str();
        if filename.ends_with('/') {
            continue;
        }
        let Some(relative) = filename
            .strip_prefix("overrides/")
            .or_else(|| filename.strip_prefix("client-overrides/"))
        else {
            continue;
        };
        if let Ok(relative) =
            SafeRelativeUtf8UnixPathBuf::try_from(relative.to_string())
        {
            paths.push(relative.to_string());
        }
    }

    Ok(paths)
}

#[tracing::instrument(skip(mrpack_file))]

pub async fn remove_all_related_files(
    instance_id: String,
    mrpack_file: CreatePackFile,
) -> crate::Result<()> {
    // Updates can remove files from a locally imported or downloaded pack, so share the same reader path.
    let mut zip_reader = MrpackZipReader::new(&mrpack_file).await?;

    // Extract index of modrinth.index.json
    let Some(manifest_idx) = zip_reader
        .entries()
        .iter()
        .position(|f| matches!(f.name.as_str(), "modrinth.index.json"))
    else {
        bail!(crate::ErrorKind::InputError(
            "no pack manifest found in mrpack".to_string(),
        ));
    };

    let manifest = zip_reader.read_entry_to_string(manifest_idx).await?;

    let pack: PackFormat = serde_json::from_str(&manifest)?;

    if &*pack.game != "minecraft" {
        bail!(crate::ErrorKind::InputError(
            "pack does not support Minecraft".to_string(),
        ));
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
    let metadata = crate::api::instance::get(&instance_id).await?.ok_or(
        crate::ErrorKind::InputError(format!("unknown instance {instance_id}")),
    )?;
    let instance_full_path =
        crate::api::instance::get_full_path(&instance_id).await?;
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
        if crate::state::content_store::is_managed_content_path(
            file.path.as_str(),
        ) {
            crate::state::instances::commands::remove_project(
                &instance_id,
                file.path.as_str(),
                &state,
            )
            .await?;
            continue;
        }
        match io::remove_file(instance_full_path.join(file.path.as_str())).await
        {
            Ok(_) => (),
            Err(err) if err.kind() == ErrorKind::NotFound => (),
            Err(err) => return Err(err.into()),
        }
    }

    // Iterate over each 'overrides' file and remove it
    let override_file_entries = zip_reader.entries().iter().filter(|file| {
        let filename = file.name.as_str();
        (filename.starts_with("overrides/")
            || filename.starts_with("client-overrides/"))
            && !filename.ends_with('/')
    });

    for file in override_file_entries {
        let relative_override_file_path =
            SafeRelativeUtf8UnixPathBuf::try_from(file.name.to_string())?;
        let relative_override_file_path = relative_override_file_path
            .strip_prefix("overrides")
            .or_else(|_| relative_override_file_path.strip_prefix("client-overrides"))
            .context(format!("failed to strip override prefix from override file path {relative_override_file_path}"))?;
        if relative_override_file_path.as_str() == "options.txt" {
            continue;
        }
        if crate::state::content_store::is_managed_content_path(
            relative_override_file_path.as_str(),
        ) {
            crate::state::instances::commands::remove_project(
                &instance_id,
                relative_override_file_path.as_str(),
                &state,
            )
            .await?;
            continue;
        }

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

#[cfg(test)]
mod zip_reader_tests {
    use super::*;

    fn archive_bytes(contents: &[u8]) -> bytes::Bytes {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(
                "overrides/mods/example.jar",
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(contents).unwrap();
        writer.finish().unwrap().into_inner().into()
    }

    #[tokio::test]
    async fn resolves_unicode_override_names() {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let mut options = zip::write::FullFileOptions::default();
        let mut extra = vec![1];
        extra.extend_from_slice(&0x4b8e39ef_u32.to_le_bytes());
        extra.extend_from_slice("overrides/mods/日本語.jar".as_bytes());
        options.add_extra_field(0x7075, extra, false).unwrap();
        writer.start_file("abcdef", options).unwrap();
        writer.write_all(b"payload").unwrap();
        let source =
            CreatePackFile::Bytes(writer.finish().unwrap().into_inner().into());
        let reader = MrpackZipReader::new(&source).await.unwrap();
        assert_eq!(reader.entries()[0].name, "overrides/mods/日本語.jar");
    }

    #[tokio::test]
    async fn hashes_and_extracts_memory_and_disk_archives() {
        let contents = vec![b'a'; 600_000];
        let bytes = archive_bytes(&contents);
        let directory = tempfile::tempdir().unwrap();
        let archive_path = directory.path().join("pack.mrpack");
        std::fs::write(&archive_path, &bytes).unwrap();
        let expected_hash =
            sha1_smol::Sha1::from(&contents).digest().to_string();
        for source in [
            CreatePackFile::Bytes(bytes),
            CreatePackFile::Path(archive_path),
        ] {
            let mut reader = MrpackZipReader::new(&source).await.unwrap();
            assert_eq!(reader.entries()[0].name, "overrides/mods/example.jar");
            let mut reported = 0;
            let mut progress = |count| {
                reported += count;
                Ok(())
            };
            let (size, hash) =
                reader.hash_entry(0, Some(&mut progress)).await.unwrap();
            assert_eq!(size, contents.len() as u64);
            assert_eq!(reported, size);
            assert_eq!(hash, expected_hash);

            let destination = directory.path().join("output.jar");
            std::fs::write(&destination, b"previous contents").unwrap();
            let semaphore =
                crate::util::fetch::IoSemaphore(tokio::sync::Semaphore::new(1));
            assert_eq!(
                reader
                    .extract_entry(0, &destination, &semaphore, None)
                    .await
                    .unwrap(),
                (size, hash)
            );
            assert_eq!(std::fs::read(&destination).unwrap(), contents);
        }
    }

    #[tokio::test]
    async fn corrupt_entries_do_not_replace_destination() {
        let contents = b"unique stored payload for CRC validation";
        let mut bytes = archive_bytes(contents).to_vec();
        let offset = bytes
            .windows(contents.len())
            .position(|window| window == contents)
            .unwrap();
        bytes[offset] ^= 1;
        let source = CreatePackFile::Bytes(bytes.into());
        let mut reader = MrpackZipReader::new(&source).await.unwrap();
        assert!(reader.hash_entry(0, None).await.is_err());

        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("output.jar");
        std::fs::write(&destination, b"previous contents").unwrap();
        let semaphore =
            crate::util::fetch::IoSemaphore(tokio::sync::Semaphore::new(1));
        assert!(
            reader
                .extract_entry(0, &destination, &semaphore, None)
                .await
                .is_err()
        );
        assert_eq!(std::fs::read(&destination).unwrap(), b"previous contents");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[tokio::test]
    async fn failed_progress_does_not_publish_extracted_entry() {
        let source = CreatePackFile::Bytes(archive_bytes(&vec![b'a'; 600_000]));
        let mut reader = MrpackZipReader::new(&source).await.unwrap();
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("output.jar");
        std::fs::write(&destination, b"previous contents").unwrap();
        let semaphore =
            crate::util::fetch::IoSemaphore(tokio::sync::Semaphore::new(1));
        let mut progress: Box<ExtractProgressFn<'_>> = Box::new(|_| {
            Box::pin(async { bail!("progress reporting stopped") })
        });
        assert!(
            reader
                .extract_entry(
                    0,
                    &destination,
                    &semaphore,
                    Some(&mut *progress)
                )
                .await
                .is_err()
        );
        assert_eq!(std::fs::read(&destination).unwrap(), b"previous contents");
    }
}
