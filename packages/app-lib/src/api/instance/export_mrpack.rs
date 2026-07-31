use super::content::get_projects;
use super::get::get;
use super::paths::get_full_path;
use crate::event::LoadingBarType;
use crate::event::emit::{emit_loading, init_loading};
use crate::pack::install_from::{
    EnvType, PackDependency, PackFile, PackFileHash, PackFormat,
};
use crate::state::{
    CacheBehaviour, CachedEntry, InstanceMetadata, ModLoader, SideType, State,
};
use crate::util::io::{self, IOError};
use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use futures::{StreamExt, stream};
use path_util::SafeRelativeUtf8UnixPathBuf;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use tokio::fs::File;
use tokio_util::compat::FuturesAsyncWriteCompatExt;

const DEFAULT_SELECTED_EXPORT_PATH_PREFIXES: &[&str] =
    &["mods", "datapacks", "resourcepacks", "shaderpacks", "config"];
const EXPORT_CANDIDATE_METADATA_CONCURRENCY: usize = 32;

const NEVER_EXPORTABLE_PATH_PREFIXES: &[&str] = &[
    "profile.json",
    "modrinth_logs",
    "mods/.connector",
    ".sable/natives",
    "local/crash_assistant",
    "mods/mcef-libraries",
    "mods/mcef-cache",
    "config/super_resolution/libraries",
    "config/Veinminer/update",
    "config/epicfight/native",
    "essential",
    ".mixin.out",
    ".fabric",
    "__MACOSX",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackExportCandidate {
    pub path: SafeRelativeUtf8UnixPathBuf,
    #[serde(rename = "type")]
    pub kind: PackExportCandidateType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    pub disabled: bool,
    pub default_selected: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PackExportCandidateType {
    Directory,
    File,
}

#[derive(Default)]
struct ExportSelectionNode {
    selected: Option<bool>,
    has_included_rule: bool,
    children: HashMap<String, ExportSelectionNode>,
}

#[derive(Default)]
struct ExportSelection {
    root: ExportSelectionNode,
}

impl ExportSelection {
    fn new(included_paths: Vec<String>, excluded_paths: Vec<String>) -> Self {
        let mut rules = HashMap::new();

        for (paths, selected) in
            [(included_paths, true), (excluded_paths, false)]
        {
            for path in paths {
                let Ok(path) = SafeRelativeUtf8UnixPathBuf::try_from(path)
                else {
                    continue;
                };
                if path.as_str().is_empty() || !is_path_exportable(&path) {
                    continue;
                }
                rules.insert(path.as_str().to_string(), selected);
            }
        }

        let mut selection = Self::default();
        for (path, selected) in rules {
            selection.root.insert(&path, selected);
        }
        selection
    }

    fn is_included(&self, path: &SafeRelativeUtf8UnixPathBuf) -> bool {
        self.resolve(path).0
    }

    fn should_visit_directory(
        &self,
        path: &SafeRelativeUtf8UnixPathBuf,
    ) -> bool {
        let (selected, node) = self.resolve(path);
        selected || node.is_some_and(|node| node.has_included_rule)
    }

    fn resolve(
        &self,
        path: &SafeRelativeUtf8UnixPathBuf,
    ) -> (bool, Option<&ExportSelectionNode>) {
        let mut node = &self.root;
        let mut selected = node.selected.unwrap_or(false);

        for segment in path.as_str().split('/') {
            let Some(child) = node.children.get(segment) else {
                return (selected, None);
            };
            node = child;
            selected = node.selected.unwrap_or(selected);
        }

        (selected, Some(node))
    }
}

impl ExportSelectionNode {
    fn insert(&mut self, path: &str, selected: bool) {
        if selected {
            self.has_included_rule = true;
        }

        let mut node = self;
        for segment in path.split('/') {
            node = node.children.entry(segment.to_string()).or_default();
            if selected {
                node.has_included_rule = true;
            }
        }
        node.selected = Some(selected);
    }
}

#[tracing::instrument(skip_all)]
pub async fn export_mrpack(
    instance_id: &str,
    export_path: PathBuf,
    included_export_candidates: Vec<String>,
    excluded_export_candidates: Vec<String>,
    version_id: Option<String>,
    description: Option<String>,
    _name: Option<String>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _permit: tokio::sync::SemaphorePermit =
        state.io_semaphore.0.acquire().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to export a nonexistent instance {instance_id}!"
        ))
    })?;
    let export_selection = ExportSelection::new(
        included_export_candidates,
        excluded_export_candidates,
    );

    let instance_base_path = get_full_path(instance_id).await?;
    let mut file = File::create(&export_path)
        .await
        .map_err(|e| IOError::with_path(e, &export_path))?;
    let mut writer = ZipFileWriter::with_tokio(&mut file);
    let version_id = version_id.unwrap_or("1.0.0".to_string());
    let mut packfile =
        create_mrpack_json(&metadata, version_id, description).await?;
    packfile.files.retain(|f| {
        is_path_exportable(&f.path) && export_selection.is_included(&f.path)
    });
    let packfile_paths = packfile
        .files
        .iter()
        .map(|file| file.path.as_str().to_string())
        .collect::<HashSet<_>>();
    let loading_bar = init_loading(
        LoadingBarType::ZipExtract {
            instance_id: metadata.instance.id.clone(),
            instance_name: metadata.instance.name.clone(),
        },
        1.0,
        "Exporting instance to .mrpack",
    )
    .await?;

    let mut directories = vec![instance_base_path.clone()];
    while let Some(directory) = directories.pop() {
        let mut read_dir = io::read_dir(&directory).await?;
        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| IOError::with_path(e, &directory))?
        {
            let path = entry.path();
            let relative_path =
                pack_get_relative_path(&instance_base_path, &path)?;
            if !is_path_exportable(&relative_path) {
                continue;
            }

            let file_type = entry
                .file_type()
                .await
                .map_err(|e| IOError::with_path(e, &path))?;
            if file_type.is_dir() {
                if export_selection.should_visit_directory(&relative_path) {
                    directories.push(path);
                }
                continue;
            }
            if !file_type.is_file()
                || !export_selection.is_included(&relative_path)
                || packfile_paths.contains(relative_path.as_str())
            {
                continue;
            }

            let mut stream = writer
                .write_entry_stream(
                    ZipEntryBuilder::new(
                        format!("overrides/{relative_path}").into(),
                        Compression::Deflate,
                    )
                    .build(),
                )
                .await?
                .compat_write();
            let mut source = File::open(&path)
                .await
                .map_err(|e| IOError::with_path(e, &path))?;
            tokio::io::copy(&mut source, &mut stream)
                .await
                .map_err(IOError::from)?;
            stream.into_inner().close().await?;
        }
    }

    let data = serde_json::to_vec_pretty(&packfile)?;
    let builder = ZipEntryBuilder::new(
        "modrinth.index.json".to_string().into(),
        Compression::Deflate,
    );
    writer.write_entry_whole(builder, &data).await?;
    writer.close().await?;
    emit_loading(&loading_bar, 1.0, None)?;

    Ok(())
}

fn is_path_exportable(relative_path: &SafeRelativeUtf8UnixPathBuf) -> bool {
    let path = relative_path.as_str();

    if path.ends_with(".DS_Store") {
        return false;
    }

    !NEVER_EXPORTABLE_PATH_PREFIXES.iter().any(|prefix| {
        path == *prefix
            || path
                .strip_prefix(prefix)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

#[tracing::instrument]
pub async fn get_pack_export_candidates(
    instance_id: &str,
) -> crate::Result<Vec<PackExportCandidate>> {
    get_pack_export_candidates_for_parent(instance_id, None).await
}

#[tracing::instrument]
pub async fn get_pack_export_candidates_for_parent(
    instance_id: &str,
    parent: Option<SafeRelativeUtf8UnixPathBuf>,
) -> crate::Result<Vec<PackExportCandidate>> {
    let instance_base_dir = get_full_path(instance_id).await?;
    let parent_dir = if let Some(parent) = parent {
        if parent.as_str().is_empty() || !is_path_exportable(&parent) {
            return Ok(Vec::new());
        }

        instance_base_dir.join(parent.as_str())
    } else {
        instance_base_dir.clone()
    };
    let parent_dir = io::canonicalize(parent_dir)?;
    if !parent_dir.starts_with(&instance_base_dir) {
        return Ok(Vec::new());
    }

    let mut paths = Vec::new();
    let mut read_dir = io::read_dir(&parent_dir).await?;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, &parent_dir))?
    {
        paths.push(entry.path());
    }

    let candidates = stream::iter(paths)
        .map(|path| {
            let instance_base_dir = &instance_base_dir;
            async move {
                build_pack_export_candidate(instance_base_dir, &path).await
            }
        })
        .buffer_unordered(EXPORT_CANDIDATE_METADATA_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;
    let mut path_list = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if let Some(candidate) = candidate? {
            path_list.push(candidate);
        }
    }

    Ok(path_list)
}

async fn build_pack_export_candidate(
    instance_base_dir: &PathBuf,
    path: &PathBuf,
) -> crate::Result<Option<PackExportCandidate>> {
    let relative_path = pack_get_relative_path(instance_base_dir, path)?;
    if !is_path_exportable(&relative_path) {
        return Ok(None);
    }

    let metadata = tokio::fs::symlink_metadata(path)
        .await
        .map_err(|error| IOError::with_path(error, path))?;
    if metadata.file_type().is_symlink()
        || (!metadata.is_dir() && !metadata.is_file())
    {
        return Ok(None);
    }

    let kind = if metadata.is_dir() {
        PackExportCandidateType::Directory
    } else {
        PackExportCandidateType::File
    };
    let size = metadata.is_file().then_some(metadata.len());
    let modified = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());
    let default_selected = is_default_selected_export_candidate(&relative_path);

    Ok(Some(PackExportCandidate {
        path: relative_path,
        kind,
        size,
        modified,
        count: None,
        disabled: false,
        default_selected,
    }))
}

fn is_default_selected_export_candidate(
    relative_path: &SafeRelativeUtf8UnixPathBuf,
) -> bool {
    let path = relative_path.as_str();

    DEFAULT_SELECTED_EXPORT_PATH_PREFIXES.iter().any(|prefix| {
        path == *prefix
            || path
                .strip_prefix(prefix)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

fn pack_get_relative_path(
    instance_path: &PathBuf,
    path: &PathBuf,
) -> crate::Result<SafeRelativeUtf8UnixPathBuf> {
    Ok(SafeRelativeUtf8UnixPathBuf::try_from(
        path.strip_prefix(instance_path)
            .map_err(|_| {
                crate::ErrorKind::FSError(format!(
                    "Path {path:?} does not correspond to an instance"
                ))
            })?
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/"),
    )?)
}

#[tracing::instrument(skip_all)]
pub async fn create_mrpack_json(
    metadata: &InstanceMetadata,
    version_id: String,
    description: Option<String>,
) -> crate::Result<PackFormat> {
    let mut dependencies = HashMap::new();
    match (
        metadata.applied_content_set.loader,
        metadata.applied_content_set.loader_version.clone(),
    ) {
        (ModLoader::Forge, Some(v)) => {
            dependencies.insert(PackDependency::Forge, v)
        }
        (ModLoader::NeoForge, Some(v)) => {
            dependencies.insert(PackDependency::NeoForge, v)
        }
        (ModLoader::Fabric, Some(v)) => {
            dependencies.insert(PackDependency::FabricLoader, v)
        }
        (ModLoader::Quilt, Some(v)) => {
            dependencies.insert(PackDependency::QuiltLoader, v)
        }
        (ModLoader::Vanilla, _) => None,
        _ => {
            return Err(crate::ErrorKind::OtherError(
                "Loader version mismatch".to_string(),
            )
            .into());
        }
    };
    dependencies.insert(
        PackDependency::Minecraft,
        metadata.applied_content_set.game_version.clone(),
    );

    let state = State::get().await?;
    let projects = get_projects(
        &metadata.instance.id,
        Some(CacheBehaviour::MustRevalidate),
    )
    .await?
    .into_iter()
    .filter_map(|(path, file)| match file.metadata {
        Some(metadata) => Some((path, metadata.version_id)),
        _ => None,
    })
    .collect::<Vec<_>>();
    let versions = CachedEntry::get_version_many(
        &projects.iter().map(|x| &*x.1).collect::<Vec<_>>(),
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;
    let files = projects
        .into_iter()
        .filter_map(|(path, version_id)| {
            if let Some(version) = versions.iter().find(|x| x.id == version_id)
            {
                let mut env = HashMap::new();
                env.insert(EnvType::Client, SideType::Required);
                env.insert(EnvType::Server, SideType::Required);
                let Some(primary_file) = version.files.first() else {
                    return Some(Err(crate::ErrorKind::OtherError(format!(
                        "No primary file found for mod at: {path}"
                    ))
                    .as_error()));
                };
                let file_size = primary_file.size;
                let downloads = vec![primary_file.url.clone()];
                let hashes = primary_file
                    .hashes
                    .clone()
                    .into_iter()
                    .map(|(h1, h2)| (PackFileHash::from(h1), h2))
                    .collect();

                Some(Ok(PackFile {
                    path: match path.try_into() {
                        Ok(path) => path,
                        Err(_) => {
                            return Some(Err(crate::ErrorKind::OtherError(
                                "Invalid file path in project".into(),
                            )
                            .as_error()));
                        }
                    },
                    hashes,
                    env: Some(env),
                    downloads,
                    file_size,
                }))
            } else {
                None
            }
        })
        .collect::<crate::Result<Vec<PackFile>>>()?;

    Ok(PackFormat {
        game: "minecraft".to_string(),
        format_version: 1,
        version_id,
        name: metadata.instance.name.clone(),
        summary: description,
        files,
        dependencies,
    })
}
