use std::{
    fmt,
    path::{Path, PathBuf},
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
    context: &crate::InvocationContext,
    instance_id: &str,
    launcher_type: ImportLauncherType,
    base_path: PathBuf,
    instance_folder: String,
    reporter: InstallProgressReporter,
) -> crate::Result<()> {
    import_instance_inner(
        context,
        instance_id,
        launcher_type,
        base_path,
        instance_folder,
        reporter,
    )
    .await
}

async fn import_instance_inner(
    context: &crate::InvocationContext,
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
                context,
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
                context,
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
                context,
                base_path.join("instances").join(instance_folder), // path to gdlauncher folder
                instance_id,
                reporter.clone(),
                details.clone(),
            )
            .await
        }
        ImportLauncherType::Curseforge => {
            curseforge::import_curseforge(
                context,
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
                        context,
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

    let bytes = tokio::fs::read(&icon_path).await;
    if let Ok(bytes) = bytes {
        let bytes = bytes::Bytes::from(bytes);
        let cache_dir = &state.directories.caches_dir();
        let semaphore = &state.io_semaphore;
        Ok(Some(
            fetch::write_cached_icon(
                &icon_path.to_string_lossy(),
                cache_dir,
                bytes,
                semaphore,
            )
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
    let instance_path =
        crate::api::instance::get_full_path(instance_id).await?;
    let subfiles = get_all_subfiles(&dotminecraft, false).await?;
    let total_subfiles = subfiles.len() as u64;

    for (index, src_child) in subfiles.into_iter().enumerate() {
        let dst_child =
            src_child.strip_prefix(&dotminecraft).map_err(|_| {
                crate::ErrorKind::InputError(format!(
                    "Invalid file: {}",
                    &src_child.display()
                ))
            })?;
        let dst_child = instance_path.join(dst_child);

        tokio::time::sleep(std::time::Duration::from_millis(1)).await;

        fetch::copy(&src_child, &dst_child, io_semaphore).await?;
        reporter
            .update(
                InstallPhaseId::PreparingInstance,
                Some(InstallProgress {
                    current: (index + 1) as u64,
                    total: total_subfiles,
                    secondary: None,
                }),
                details.clone(),
            )
            .await?;
    }

    Ok(())
}

pub(crate) async fn finish_import(
    context: &crate::InvocationContext,
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
        context,
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
    if !src.is_dir() {
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
