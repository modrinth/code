use crate::State;
use crate::data::ModLoader;
use crate::event::LoadingBarType;
use crate::event::emit::{emit_loading, init_loading};
use crate::install::{
    InstallErrorContext, InstallPhaseDetails, InstallPhaseId, InstallProgress,
    InstallProgressReporter,
};
use crate::state::{
    AppliedContentSetPatch, CacheBehaviour, CachedEntry, ContentSourceKind,
    EditInstance, InstanceInstallStage, InstanceLink, SideType,
};
use crate::util::fetch::{
    DownloadMeta, DownloadReason, FetchProgressFn, fetch,
    fetch_advanced_with_progress, sha1_file_async_with_progress,
};
use path_util::SafeRelativeUtf8UnixPathBuf;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;

use std::path::{Path, PathBuf};
use std::pin::Pin;

#[derive(Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PackFormat {
    pub game: String,
    pub format_version: i32,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<PackFile>,
    pub dependencies: HashMap<PackDependency, String>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PackFile {
    pub path: SafeRelativeUtf8UnixPathBuf,
    pub hashes: HashMap<PackFileHash, String>,
    pub env: Option<HashMap<EnvType, SideType>>,
    pub downloads: Vec<String>,
    pub file_size: u32,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase", from = "String")]
pub enum PackFileHash {
    Sha1,
    Sha512,
    Unknown(String),
}

impl From<String> for PackFileHash {
    fn from(s: String) -> Self {
        match s.as_str() {
            "sha1" => PackFileHash::Sha1,
            "sha512" => PackFileHash::Sha512,
            _ => PackFileHash::Unknown(s),
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum EnvType {
    Client,
    Server,
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum PackDependency {
    #[serde(rename = "forge")]
    Forge,

    #[serde(rename = "neoforge")]
    #[serde(alias = "neo-forge")]
    NeoForge,

    #[serde(rename = "fabric-loader")]
    FabricLoader,

    #[serde(rename = "quilt-loader")]
    QuiltLoader,

    #[serde(rename = "minecraft")]
    Minecraft,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CreatePackLocation {
    // Create a pack from a modrinth version ID (such as a modpack)
    FromVersionId {
        project_id: String,
        version_id: String,
        title: String,
        icon_url: Option<String>,
    },
    // Create a pack from a file (such as an .mrpack for installing from a file, or a folder name for importing)
    FromFile {
        path: PathBuf,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePackInstance {
    pub name: String, // the name of the instance and relative path
    pub game_version: String, // the game version of the instance
    pub modloader: ModLoader, // the modloader to use
    pub loader_version: Option<String>, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader. defaults to latest
    pub icon: Option<PathBuf>,          // the icon for the instance
    pub icon_url: Option<String>, // the URL icon for an instance during import
    pub link: Option<InstanceLink>,
    pub unknown_file: bool, // true when the mrpack archive isn't found on Modrinth via hash lookup
    pub external_files_in_modpack: Vec<String>,
    pub skip_install_profile: Option<bool>,
    pub no_watch: Option<bool>,
}

// default
impl Default for CreatePackInstance {
    fn default() -> Self {
        CreatePackInstance {
            name: "Untitled".to_string(),
            game_version: "1.19.4".to_string(),
            modloader: ModLoader::Vanilla,
            loader_version: None,
            icon: None,
            icon_url: None,
            link: None,
            unknown_file: false,
            external_files_in_modpack: Vec::new(),
            skip_install_profile: Some(true),
            no_watch: Some(false),
        }
    }
}

#[derive(Clone)]
pub enum CreatePackFile {
    Bytes(bytes::Bytes),
    // Local packs can be larger than available memory, so keep them file-backed.
    Path(PathBuf),
}

#[derive(Clone)]
pub struct CreatePack {
    pub file: CreatePackFile,
    pub description: CreatePackDescription,
}

const MAX_LOCAL_FILE_HASH_LOOKUP_SIZE: u64 = 1024 * 1024 * 1024;

pub(crate) fn get_local_pack_instance(path: &Path) -> CreatePackInstance {
    CreatePackInstance {
        name: path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        ..Default::default()
    }
}

#[derive(Clone, Debug)]
pub struct CreatePackDescription {
    pub icon: Option<PathBuf>,
    pub override_title: Option<String>,
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub instance_id: String,
    pub source_filename: Option<String>,
}

pub async fn get_instance_from_pack(
    location: CreatePackLocation,
) -> crate::Result<CreatePackInstance> {
    match location {
        CreatePackLocation::FromVersionId {
            project_id,
            version_id,
            title,
            icon_url,
        } => Ok(CreatePackInstance {
            name: title,
            icon_url,
            link: Some(InstanceLink::ModrinthModpack {
                project_id,
                version_id,
            }),
            ..Default::default()
        }),
        CreatePackLocation::FromFile { path } => {
            let mut instance = get_local_pack_instance(&path);
            let file_size = tokio::fs::metadata(&path).await?.len();
            let hashes_archive = file_size <= MAX_LOCAL_FILE_HASH_LOOKUP_SIZE;
            let archive_hashing_bytes =
                if hashes_archive { file_size } else { 0 };
            let pack_file = CreatePackFile::Path(path.clone());
            let external_file_hashing_bytes =
                super::install_mrpack::get_external_file_hashing_size_from_mrpack(
                    &pack_file,
                )
                .await?;
            let inspection_total_bytes = archive_hashing_bytes
                .saturating_add(external_file_hashing_bytes)
                .max(1);
            let inspection = init_loading(
                LoadingBarType::PackImport {
                    pack_name: instance.name.clone(),
                },
                inspection_total_bytes as f64,
                "Inspecting modpack",
            )
            .await
            .ok();
            let min_delta = (inspection_total_bytes / 200).max(256 * 1024);
            let mut reported_bytes = 0_u64;
            let mut report_progress =
                |current: u64, offset: u64, message: &str| {
                    let target = offset
                        .saturating_add(current)
                        .min(inspection_total_bytes);
                    let increment = target.saturating_sub(reported_bytes);
                    if target < inspection_total_bytes && increment < min_delta
                    {
                        return;
                    }

                    if let Some(inspection) = &inspection {
                        let _ = emit_loading(
                            inspection,
                            increment as f64,
                            Some(message),
                        );
                    }
                    reported_bytes = target;
                };

            let is_known_file = if hashes_archive {
                let state = State::get().await?;
                let (_, hash) =
                    sha1_file_async_with_progress(&path, |current, _| {
                        report_progress(current, 0, "Hashing local modpack");
                        Ok(())
                    })
                    .await?;
                match CachedEntry::get_file_many(
                    &[&hash],
                    Some(CacheBehaviour::StaleWhileRevalidateSkipOffline),
                    &state.pool,
                    &state.api_semaphore,
                )
                .await
                {
                    Ok(files) => !files.is_empty(),
                    Err(err) => {
                        tracing::warn!(
                            "Failed to check Modrinth file hash for {}: {}",
                            path.display(),
                            err
                        );
                        false
                    }
                }
            } else {
                false
            };

            let external_files_in_modpack =
                super::install_mrpack::get_external_files_from_mrpack(
                    &pack_file,
                    |current, _| {
                        report_progress(
                            current,
                            archive_hashing_bytes,
                            "Inspecting modpack files",
                        );
                        Ok(())
                    },
                )
                .await?;
            report_progress(
                inspection_total_bytes,
                0,
                "Finished inspecting modpack",
            );

            instance.unknown_file = !is_known_file;
            instance.external_files_in_modpack = external_files_in_modpack;
            Ok(instance)
        }
    }
}

#[tracing::instrument(skip(reporter))]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn generate_pack_from_version_id_with_reporter(
    project_id: String,
    version_id: String,
    title: String,
    icon_url: Option<String>,
    instance_id: String,
    reason: DownloadReason,
    reporter: InstallProgressReporter,
) -> crate::Result<CreatePack> {
    let state = State::get().await?;

    let version = CachedEntry::get_version(
        &version_id,
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Invalid version ID specified!".to_string(),
        )
    })?;

    // Update instance with correct loader and game version from the API version metadata,
    // so the UI shows accurate info while the pack file is still downloading.
    if let Some(game_version) = version.game_versions.first() {
        let loader = version
            .loaders
            .first()
            .map(|l| ModLoader::from_string(l))
            .unwrap_or(ModLoader::Vanilla);
        let game_version = game_version.clone();
        crate::api::instance::edit(
            &instance_id,
            EditInstance {
                content_set_patch: Some(AppliedContentSetPatch {
                    source_kind: None,
                    game_version: Some(game_version),
                    protocol_version: Some(None),
                    loader: Some(loader),
                    loader_version: None,
                }),
                ..EditInstance::default()
            },
        )
        .await?;
    }

    let (url, hash) =
        if let Some(file) = version.files.iter().find(|x| x.primary) {
            Some((file.url.clone(), file.hashes.get("sha1")))
        } else {
            version
                .files
                .first()
                .map(|file| (file.url.clone(), file.hashes.get("sha1")))
        }
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "Specified version has no files".to_string(),
            )
        })?;

    let metadata =
        crate::api::instance::get(&instance_id)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Unknown instance {instance_id}"
                ))
            })?;

    let download_meta = DownloadMeta {
        reason,
        game_version: metadata.applied_content_set.game_version.clone(),
        loader: metadata.applied_content_set.loader.as_str().to_string(),
        dependent_on: Some(version_id.clone()),
    };

    let details = InstallPhaseDetails::Modpack {
        project_id: Some(project_id.clone()),
        version_id: Some(version_id.clone()),
        title: Some(title.clone()),
    };
    let mut last_reported_bytes = 0_u64;
    let mut progress =
        |current: u64,
         total: u64|
         -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send>> {
            let min_delta = (total / 200).max(256 * 1024);
            if current < total
                && current.saturating_sub(last_reported_bytes) < min_delta
            {
                return Box::pin(async { Ok(()) });
            }

            last_reported_bytes = current;
            let reporter = reporter.clone();
            let details = details.clone();
            Box::pin(async move {
                reporter
                    .update(
                        InstallPhaseId::DownloadingPackFile,
                        Some(InstallProgress {
                            current,
                            total,
                            secondary: None,
                        }),
                        details,
                    )
                    .await?;
                Ok(())
            })
        };
    let progress = Some(&mut progress as &mut FetchProgressFn<'_>);

    let context = InstallErrorContext::new("download modpack file")
        .urls(vec![url.clone()])
        .maybe_expected_hash(hash.cloned())
        .project_id(project_id.clone())
        .version_id(version_id.clone())
        .build();
    reporter.set_context(context).await?;
    let file = fetch_advanced_with_progress(
        Method::GET,
        &url,
        hash.map(|x| &**x),
        None,
        None,
        Some(&download_meta),
        None,
        None,
        &state.fetch_semaphore,
        &state.pool,
        progress,
    )
    .await?;

    reporter
        .update(InstallPhaseId::ResolvingPack, None, details.clone())
        .await?;

    // When no icon URL is supplied, preserve the instance's existing icon.
    let icon = if let Some(icon_url) = icon_url {
        reporter
            .set_context(
                InstallErrorContext::new("download modpack icon")
                    .urls(vec![icon_url.clone()])
                    .project_id(project_id.clone())
                    .version_id(version_id.clone())
                    .build(),
            )
            .await?;
        let icon_bytes = fetch(
            &icon_url,
            None,
            None,
            None,
            &state.fetch_semaphore,
            &state.pool,
        )
        .await?;

        Some(crate::api::instance::cache_icon(icon_bytes, &state).await?)
    } else {
        None
    };

    // Set the icon immediately so the UI shows it during download.
    if let Some(ref icon_path) = icon {
        let _ = crate::api::instance::edit_icon(
            &instance_id,
            Some(icon_path.as_path()),
        )
        .await;
    }

    Ok(CreatePack {
        file: CreatePackFile::Bytes(file),
        description: CreatePackDescription {
            icon,
            override_title: Some(title),
            project_id: Some(project_id),
            version_id: Some(version_id),
            instance_id,
            source_filename: None,
        },
    })
}

#[tracing::instrument]

pub async fn generate_pack_from_file(
    path: PathBuf,
    instance_id: String,
) -> crate::Result<CreatePack> {
    let source_filename =
        path.file_name().map(|x| x.to_string_lossy().to_string());

    Ok(CreatePack {
        file: CreatePackFile::Path(path),
        description: CreatePackDescription {
            icon: None,
            override_title: None,
            project_id: None,
            version_id: None,
            instance_id,
            source_filename,
        },
    })
}

/// Sets generated instance attributes to the pack ones.
/// This includes the pack name, icon, game version, loader version, and loader
pub async fn set_instance_information(
    instance_id: String,
    description: &CreatePackDescription,
    backup_name: &str,
    pack_version_id: Option<&str>,
    dependencies: &HashMap<PackDependency, String>,
    _ignore_lock: bool,
) -> crate::Result<()> {
    let mut game_version: Option<&String> = None;
    let mut mod_loader = None;
    let mut loader_version = None;

    for (key, value) in dependencies {
        match key {
            PackDependency::Forge => {
                mod_loader = Some(ModLoader::Forge);
                loader_version = Some(value);
            }
            PackDependency::NeoForge => {
                mod_loader = Some(ModLoader::NeoForge);
                loader_version = Some(value);
            }
            PackDependency::FabricLoader => {
                mod_loader = Some(ModLoader::Fabric);
                loader_version = Some(value);
            }
            PackDependency::QuiltLoader => {
                mod_loader = Some(ModLoader::Quilt);
                loader_version = Some(value);
            }
            PackDependency::Minecraft => game_version = Some(value),
        }
    }

    let Some(game_version) = game_version else {
        return Err(crate::ErrorKind::InputError(
            "Pack did not specify Minecraft version".to_string(),
        )
        .into());
    };

    let mod_loader = mod_loader.unwrap_or(ModLoader::Vanilla);
    let loader_version = if mod_loader != ModLoader::Vanilla {
        crate::launcher::get_loader_version_from_profile(
            game_version,
            mod_loader,
            loader_version.cloned().as_deref(),
        )
        .await?
    } else {
        None
    };
    let pack_link = match (&description.project_id, &description.version_id) {
        (Some(project_id), Some(version_id)) => {
            Some(InstanceLink::ModrinthModpack {
                project_id: project_id.clone(),
                version_id: version_id.clone(),
            })
        }
        _ if description.source_filename.is_some() => {
            Some(InstanceLink::ImportedModpack {
                project_id: None,
                version_id: None,
                name: Some(backup_name.to_string()),
                version_number: pack_version_id.map(ToString::to_string),
                filename: description.source_filename.clone(),
            })
        }
        _ => None,
    };
    let existing_link = crate::api::instance::get(&instance_id)
        .await?
        .map(|metadata| metadata.link);
    let link = match existing_link {
        Some(
            link @ (InstanceLink::ServerProject { .. }
            | InstanceLink::ServerProjectModpack { .. }
            | InstanceLink::ModrinthHosting { .. }
            | InstanceLink::SharedInstance { .. }),
        ) => Some(link),
        _ => pack_link,
    };
    let source_kind = match &link {
        Some(InstanceLink::ModrinthModpack { .. }) => {
            Some(ContentSourceKind::ModrinthModpack)
        }
        Some(
            InstanceLink::ServerProject { .. }
            | InstanceLink::ServerProjectModpack { .. },
        ) => Some(ContentSourceKind::ServerProject),
        Some(InstanceLink::ModrinthHosting { .. }) => {
            Some(ContentSourceKind::ModrinthHosting)
        }
        Some(InstanceLink::ImportedModpack { .. }) => {
            Some(ContentSourceKind::ImportedModpack)
        }
        Some(InstanceLink::SharedInstance { .. }) => {
            Some(ContentSourceKind::SharedInstance)
        }
        _ => None,
    };
    crate::api::instance::edit(
        &instance_id,
        EditInstance {
            install_stage: Some(InstanceInstallStage::PackInstalling),
            name: Some(
                description
                    .override_title
                    .clone()
                    .unwrap_or_else(|| backup_name.to_string()),
            ),
            icon_path: description
                .icon
                .as_ref()
                .map(|icon| Some(icon.to_string_lossy().to_string())),
            link,
            content_set_patch: Some(AppliedContentSetPatch {
                source_kind,
                game_version: Some(game_version.clone()),
                protocol_version: Some(None),
                loader: Some(mod_loader),
                loader_version: Some(loader_version.clone().map(|x| x.id)),
            }),
            ..EditInstance::default()
        },
    )
    .await?;
    Ok(())
}
