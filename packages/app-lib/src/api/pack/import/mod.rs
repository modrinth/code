use std::{
    fmt,
    path::{Path, PathBuf},
};

use crate::state::content_store::{FileContent, content_file_path};
use crate::state::instances::commands::{
    ContentOrigin, InstallContent, install_stored_file,
};
use io::IOError;
use serde::{Deserialize, Serialize};

use crate::{
    install::{
        InstallPhaseDetails, InstallPhaseId, InstallProgress,
        InstallProgressReporter,
    },
    util::{
        fetch::{self, IoSemaphore},
        io,
    },
};

pub mod atlauncher;
pub mod curseforge;
pub mod gdlauncher;
pub mod mmc;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
pub enum ImportLauncherType {
    MultiMC,
    PrismLauncher,
    ATLauncher,
    GDLauncher,
    Curseforge,
    #[serde(other)]
    Unknown,
}
// impl display
impl fmt::Display for ImportLauncherType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportLauncherType::MultiMC => write!(f, "MultiMC"),
            ImportLauncherType::PrismLauncher => write!(f, "PrismLauncher"),
            ImportLauncherType::ATLauncher => write!(f, "ATLauncher"),
            ImportLauncherType::GDLauncher => write!(f, "GDLauncher"),
            ImportLauncherType::Curseforge => write!(f, "Curseforge"),
            ImportLauncherType::Unknown => write!(f, "Unknown"),
        }
    }
}

// Return a list of importable instances from a launcher type and base path, by iterating through the folder and checking
pub async fn get_importable_instances(
    launcher_type: ImportLauncherType,
    base_path: PathBuf,
) -> crate::Result<Vec<String>> {
    // Some launchers have a different folder structure for instances
    let instances_subfolder = match launcher_type {
        ImportLauncherType::GDLauncher | ImportLauncherType::ATLauncher => {
            "instances".to_string()
        }
        ImportLauncherType::Curseforge => "Instances".to_string(),
        ImportLauncherType::MultiMC => {
            mmc::get_instances_subpath(base_path.clone().join("multimc.cfg"))
                .await
                .unwrap_or_else(|| "instances".to_string())
        }
        ImportLauncherType::PrismLauncher => mmc::get_instances_subpath(
            base_path.clone().join("prismlauncher.cfg"),
        )
        .await
        .unwrap_or_else(|| "instances".to_string()),
        ImportLauncherType::Unknown => {
            let types = [
                ImportLauncherType::MultiMC,
                ImportLauncherType::PrismLauncher,
                ImportLauncherType::ATLauncher,
                ImportLauncherType::GDLauncher,
                ImportLauncherType::Curseforge,
            ];
            for lt in types {
                if let Ok(instances) =
                    Box::pin(get_importable_instances(lt, base_path.clone()))
                        .await
                    && !instances.is_empty()
                {
                    return Ok(instances);
                }
            }
            return Ok(Vec::new());
        }
    };

    let instances_folder = base_path.join(&instances_subfolder);
    let mut instances = Vec::new();
    let mut dir = io::read_dir(&instances_folder).await.map_err(| _ | {
        crate::ErrorKind::InputError(format!(
            "Invalid {launcher_type} launcher path, could not find '{instances_subfolder}' subfolder."
        ))
    })?;
    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, &instances_folder))?
    {
        let path = entry.path();
        if path.is_dir() {
            // Check instance is valid of this launcher type
            if is_valid_importable_instance(path.clone(), launcher_type).await {
                let name = path.file_name();
                if let Some(name) = name {
                    instances.push(name.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(instances)
}

pub(crate) async fn import_instance_with_reporter(
    instance_id: &str,
    launcher_type: ImportLauncherType,
    base_path: PathBuf,
    instance_folder: String,
    reporter: InstallProgressReporter,
) -> crate::Result<()> {
    import_instance_inner(
        instance_id,
        launcher_type,
        base_path,
        instance_folder,
        reporter,
    )
    .await
}

async fn import_instance_inner(
    instance_id: &str,
    launcher_type: ImportLauncherType,
    base_path: PathBuf,
    instance_folder: String,
    reporter: InstallProgressReporter,
) -> crate::Result<()> {
    tracing::debug!("Importing instance from {instance_folder}");
    let details = InstallPhaseDetails::Import {
        launcher_type,
        instance_folder: instance_folder.clone(),
    };
    let res = match launcher_type {
        ImportLauncherType::MultiMC | ImportLauncherType::PrismLauncher => {
            mmc::import_mmc(
                base_path,       // path to base mmc folder
                instance_folder, // instance folder in mmc_base_path
                instance_id,
                reporter.clone(),
                details.clone(),
            )
            .await
        }
        ImportLauncherType::ATLauncher => {
            atlauncher::import_atlauncher(
                base_path,       // path to atlauncher folder
                instance_folder, // instance folder in atlauncher
                instance_id,
                reporter.clone(),
                details.clone(),
            )
            .await
        }
        ImportLauncherType::GDLauncher => {
            gdlauncher::import_gdlauncher(
                base_path.join("instances").join(instance_folder), // path to gdlauncher folder
                instance_id,
                reporter.clone(),
                details.clone(),
            )
            .await
        }
        ImportLauncherType::Curseforge => {
            curseforge::import_curseforge(
                base_path.join("Instances").join(instance_folder), // path to curseforge folder
                instance_id,
                reporter.clone(),
                details.clone(),
            )
            .await
        }
        ImportLauncherType::Unknown => {
            let types = [
                ImportLauncherType::MultiMC,
                ImportLauncherType::PrismLauncher,
                ImportLauncherType::ATLauncher,
                ImportLauncherType::GDLauncher,
                ImportLauncherType::Curseforge,
            ];
            let mut matched = false;
            for lt in types {
                if let Ok(instances) =
                    Box::pin(get_importable_instances(lt, base_path.clone()))
                        .await
                    && instances.contains(&instance_folder)
                {
                    matched = true;
                    Box::pin(import_instance_inner(
                        instance_id,
                        lt,
                        base_path,
                        instance_folder,
                        reporter.clone(),
                    ))
                    .await?;
                    break;
                }
            }
            if !matched {
                return Err(crate::ErrorKind::InputError(
                    "Could not determine launcher type for the given path"
                        .to_string(),
                )
                .into());
            }
            return Ok(());
        }
    };

    // If import failed, delete the profile
    match res {
        Ok(_) => {}
        Err(e) => {
            tracing::warn!("Import failed: {:?}", e);
            let _ = crate::api::instance::remove(instance_id).await;
            return Err(e);
        }
    }

    tracing::debug!("Completed import.");
    Ok(())
}

/// Returns the default path for the given launcher type
/// None if it can't be found or doesn't exist
pub fn get_default_launcher_path(
    r#type: ImportLauncherType,
) -> Option<PathBuf> {
    let path = match r#type {
        ImportLauncherType::MultiMC => {
            return find_multimc_path();
        }
        ImportLauncherType::PrismLauncher => {
            Some(dirs::data_dir()?.join("PrismLauncher"))
        }
        ImportLauncherType::ATLauncher => {
            Some(dirs::data_dir()?.join("ATLauncher"))
        }
        ImportLauncherType::GDLauncher => {
            Some(dirs::data_dir()?.join("gdlauncher_next"))
        }
        ImportLauncherType::Curseforge => {
            let home = dirs::home_dir()?;
            let primary = home.join("curseforge").join("minecraft");
            if primary.exists() {
                return Some(primary);
            }
            Some(dirs::document_dir()?.join("curseforge").join("minecraft"))
        }
        ImportLauncherType::Unknown => None,
    };
    let path = path?;
    if path.exists() { Some(path) } else { None }
}

/// Searches common locations for a MultiMC installation.
/// MultiMC stores data in its own application directory (not a standard data dir)
fn find_multimc_path() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // Linux/macOS: ~/.local/share/multimc is the typical location
    if let Some(data_dir) = dirs::data_dir() {
        candidates.push(data_dir.join("multimc"));
        candidates.push(data_dir.join("MultiMC"));
    }

    // Windows: check common extraction locations
    #[cfg(target_os = "windows")]
    {
        if let Some(home) = dirs::home_dir() {
            candidates.push(home.join("MultiMC"));
            candidates.push(home.join("Desktop").join("MultiMC"));
            candidates.push(home.join("Downloads").join("MultiMC"));
        }
        candidates.push(PathBuf::from("C:\\MultiMC"));
        if let Some(program_files) =
            std::env::var_os("ProgramFiles").map(PathBuf::from)
        {
            candidates.push(program_files.join("MultiMC"));
        }
        if let Some(program_files_x86) =
            std::env::var_os("ProgramFiles(x86)").map(PathBuf::from)
        {
            candidates.push(program_files_x86.join("MultiMC"));
        }
    }

    // macOS: MultiMC is a .app bundle with data inside MultiMC.app/Data/
    #[cfg(target_os = "macos")]
    {
        candidates.push(PathBuf::from("/Applications/MultiMC.app/Data"));
        if let Some(home) = dirs::home_dir() {
            candidates.push(
                home.join("Applications").join("MultiMC.app").join("Data"),
            );
        }
    }

    candidates
        .into_iter()
        .find(|p| p.join("multimc.cfg").exists())
}

/// Checks if this PathBuf is a valid instance for the given launcher type

#[tracing::instrument]
pub async fn is_valid_importable_instance(
    instance_path: PathBuf,
    r#type: ImportLauncherType,
) -> bool {
    match r#type {
        ImportLauncherType::MultiMC | ImportLauncherType::PrismLauncher => {
            mmc::is_valid_mmc(instance_path).await
        }
        ImportLauncherType::ATLauncher => {
            atlauncher::is_valid_atlauncher(instance_path).await
        }
        ImportLauncherType::GDLauncher => {
            gdlauncher::is_valid_gdlauncher(instance_path).await
        }
        ImportLauncherType::Curseforge => {
            curseforge::is_valid_curseforge(instance_path).await
        }
        ImportLauncherType::Unknown => false,
    }
}

/// Caches an image file in the filesystem into the cache directory, and returns the path to the cached file.

#[tracing::instrument]
pub async fn recache_icon(
    icon_path: PathBuf,
) -> crate::Result<Option<PathBuf>> {
    let state = crate::State::get().await?;

    if tokio::fs::try_exists(&icon_path).await.unwrap_or(false) {
        Ok(Some(
            crate::api::instance::cache_icon_from_path(&icon_path, &state)
                .await?,
        ))
    } else {
        // could not find icon (for instance, prism default icon, etc)
        Ok(None)
    }
}

pub(crate) async fn copy_dotminecraft_with_reporter(
    instance_id: &str,
    dotminecraft: PathBuf,
    io_semaphore: &IoSemaphore,
    reporter: InstallProgressReporter,
    details: InstallPhaseDetails,
) -> crate::Result<()> {
    let state = crate::State::get().await?;
    let _lease = state.content_store.lease().await;
    let dotminecraft = tokio::fs::canonicalize(&dotminecraft).await?;
    let subfiles = get_all_subfiles(&dotminecraft, false).await?;
    let mut content_paths = std::collections::HashSet::new();
    let mut duplicate_content_paths = std::collections::HashSet::new();
    for source in &subfiles {
        let relative = source
            .strip_prefix(&dotminecraft)?
            .components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        if crate::state::content_store::eligible(&relative) {
            let canonical = relative.trim_end_matches(".disabled").to_string();
            if !content_paths.insert(canonical.clone()) {
                duplicate_content_paths.insert(canonical);
            }
        }
    }
    let profiles =
        tokio::fs::canonicalize(state.directories.instances_dir()).await?;
    let source_instance = if let Ok(path) = dotminecraft.strip_prefix(&profiles)
    {
        crate::state::instances::adapters::sqlite::instance_rows::get_instance_by_path(&path.to_string_lossy(), &state.pool).await?
    } else {
        None
    };
    let mut managed_paths = std::collections::HashSet::new();
    if let Some(source) = &source_instance {
        if crate::state::instance_has_running_process(&source.id, &state)
            .await?
        {
            return Err(crate::state::content_store::input(
                "Stop the source instance before duplicating it",
            ));
        }
        let files = crate::state::instances::commands::sync_content_files(
            &source.id, &state,
        )
        .await?;
        let entries = if let Some(content_set) = &source.applied_content_set_id
        {
            crate::state::instances::adapters::sqlite::content_rows::get_content_entries(content_set, &state.pool).await?
        } else {
            Vec::new()
        };
        for file in files {
            if file.missing {
                return Err(crate::state::content_store::input(format!(
                    "Restore or repair {} before duplicating this instance",
                    file.relative_path
                )));
            }
            let stored_file =
                match state.content_store.file_content(&file).await? {
                    FileContent::Stored { stored_file, .. } => stored_file,
                    FileContent::Unmanaged => continue,
                    FileContent::Damaged(_) => {
                        return Err(crate::state::content_store::input(
                            format!(
                                "Repair {} before duplicating this instance",
                                file.relative_path
                            ),
                        ));
                    }
                };
            managed_paths.insert(content_file_path(&file));
            let project_type =
                crate::state::ProjectType::get_from_parent_folder(
                    &file.relative_path,
                )
                .ok_or_else(|| {
                    crate::state::content_store::input("Invalid content path")
                })?;
            let entry = entries.iter().find(|entry| {
                entry.file_id.as_deref() == Some(file.id.as_str())
            });
            install_stored_file(
                instance_id,
                InstallContent {
                    requested_path: &file.relative_path,
                    stored_file: &stored_file,
                    project_type,
                    source_kind: entry.map_or(
                        crate::state::ContentSourceKind::Local,
                        |entry| entry.source_kind,
                    ),
                    origin: entry
                        .and_then(|entry| {
                            entry
                                .project_id
                                .as_deref()
                                .zip(entry.version_id.as_deref())
                        })
                        .map(|(project_id, version_id)| ContentOrigin {
                            project_id,
                            version_id,
                        }),
                    enabled_override: Some(file.enabled),
                    previous_path: None,
                },
                &state,
            )
            .await?;
            if crate::state::instances::commands::is_project_locked(
                &source.id,
                &file.relative_path,
                &state,
            )
            .await?
            {
                crate::state::instances::commands::set_project_locked(
                    instance_id,
                    &file.relative_path,
                    true,
                    &state,
                )
                .await?;
            }
        }
    }
    let total = subfiles.len() as u64;
    for (index, source) in subfiles.into_iter().enumerate() {
        let relative = source
            .strip_prefix(&dotminecraft)?
            .components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        if managed_paths.contains(&relative) {
            continue;
        }
        if tokio::fs::symlink_metadata(&source)
            .await?
            .file_type()
            .is_symlink()
        {
            tracing::warn!(path = %source.display(), "Skipping an unmanaged symlink while importing an instance");
            continue;
        }
        if crate::state::content_store::eligible(&relative)
            && !duplicate_content_paths
                .contains(relative.trim_end_matches(".disabled"))
        {
            let stored_file = state.content_store.store_file(&source).await?;
            let project_type =
                crate::state::ProjectType::get_from_parent_folder(&relative)
                    .ok_or_else(|| {
                        crate::state::content_store::input(
                            "Invalid imported content path",
                        )
                    })?;
            install_stored_file(
                instance_id,
                InstallContent {
                    requested_path: &relative,
                    stored_file: &stored_file,
                    project_type,
                    source_kind: crate::state::ContentSourceKind::Local,
                    origin: None,
                    enabled_override: Some(!relative.ends_with(".disabled")),
                    previous_path: None,
                },
                &state,
            )
            .await?;
        } else {
            let target_instance = crate::state::instances::adapters::sqlite::instance_rows::get_instance_by_id(instance_id, &state.pool).await?
				.ok_or_else(|| crate::state::content_store::input("Unknown destination instance"))?;
            let target = state
                .content_store
                .instance_path(&target_instance.path, &relative)
                .await?;
            if tokio::fs::symlink_metadata(&target)
                .await
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
            {
                return Err(crate::state::content_store::input(
                    "Import cannot overwrite a symbolic link",
                ));
            }
            fetch::copy(&source, &target, io_semaphore).await?;
        }
        reporter
            .update(
                InstallPhaseId::PreparingInstance,
                Some(InstallProgress {
                    current: (index + 1) as u64,
                    total,
                    secondary: None,
                }),
                details.clone(),
            )
            .await?;
    }
    reporter
        .update(
            InstallPhaseId::PreparingInstance,
            Some(InstallProgress {
                current: total,
                total,
                secondary: None,
            }),
            details,
        )
        .await?;
    Ok(())
}

pub(crate) async fn finish_import(
    instance_id: &str,
    dotminecraft: PathBuf,
    io_semaphore: &IoSemaphore,
    reporter: InstallProgressReporter,
    details: InstallPhaseDetails,
) -> crate::Result<()> {
    copy_dotminecraft_with_reporter(
        instance_id,
        dotminecraft,
        io_semaphore,
        reporter.clone(),
        details,
    )
    .await?;

    crate::launcher::install_minecraft_for_instance_id_with_reporter(
        instance_id,
        false,
        Some(reporter),
    )
    .await?;

    Ok(())
}

/// Recursively get a list of all subfiles in src
/// uses async recursion

#[async_recursion::async_recursion]
#[tracing::instrument]
pub async fn get_all_subfiles(
    src: &Path,
    include_empty_dirs: bool,
) -> crate::Result<Vec<PathBuf>> {
    let metadata = tokio::fs::symlink_metadata(src).await?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Ok(vec![src.to_path_buf()]);
    }

    let mut files = Vec::new();
    let mut dir = io::read_dir(&src).await?;

    let mut has_files = false;
    while let Some(child) = dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, src))?
    {
        has_files = true;
        let src_child = child.path();
        files.append(
            &mut get_all_subfiles(&src_child, include_empty_dirs).await?,
        );
    }

    if !has_files && include_empty_dirs {
        files.push(src.to_path_buf());
    }

    Ok(files)
}
