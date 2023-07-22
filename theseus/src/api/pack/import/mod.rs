use std::path::{Path, PathBuf};

use io::IOError;
use serde::{Deserialize, Serialize};

use crate::{
    prelude::ProfilePathId,
    util::{fetch, io},
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

// Return a list of importable instances from a launcher type and base path, by iterating through the folder and checking
pub async fn get_importable_instances(
    launcher_type: ImportLauncherType,
    base_path: PathBuf,
) -> crate::Result<Vec<String>> {
    // Some launchers have a different folder structure for instances
    let instances_folder = match launcher_type {
        ImportLauncherType::GDLauncher
        | ImportLauncherType::MultiMC
        | ImportLauncherType::PrismLauncher
        | ImportLauncherType::ATLauncher => base_path.join("instances"),
        ImportLauncherType::Curseforge => base_path.join("Instances"),
        ImportLauncherType::Unknown => {
            return Err(crate::ErrorKind::InputError(
                "Launcher type Unknown".to_string(),
            )
            .into())
        }
    };
    let mut instances = Vec::new();
    let mut dir = io::read_dir(&instances_folder).await?;
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

// Import an instance from a launcher type and base path
#[theseus_macros::debug_pin]
#[tracing::instrument]
pub async fn import_instance(
    profile_path: ProfilePathId,
    launcher_type: ImportLauncherType,
    base_path: PathBuf,
    instance_folder: String,
) -> crate::Result<()> {
    tracing::debug!("Importing instance from {instance_folder}");
    match launcher_type {
        ImportLauncherType::MultiMC | ImportLauncherType::PrismLauncher => {
            mmc::import_mmc(
                base_path,       // path to base mmc folder
                instance_folder, // instance folder in mmc_base_path
                profile_path,    // path to profile
            )
            .await?;
        }
        ImportLauncherType::ATLauncher => {
            atlauncher::import_atlauncher(
                base_path,       // path to atlauncher folder
                instance_folder, // instance folder in atlauncher
                profile_path,    // path to profile
            )
            .await?;
        }
        ImportLauncherType::GDLauncher => {
            gdlauncher::import_gdlauncher(
                base_path.join("instances").join(instance_folder), // path to gdlauncher folder
                profile_path, // path to profile
            )
            .await?;
        }
        ImportLauncherType::Curseforge => {
            curseforge::import_curseforge(
                base_path.join("Instances").join(instance_folder), // path to curseforge folder
                profile_path, // path to profile
            )
            .await?;
        }
        ImportLauncherType::Unknown => {
            return Err(crate::ErrorKind::InputError(
                "Launcher type Unknown".to_string(),
            )
            .into());
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
        ImportLauncherType::MultiMC => None, // multimc data is *in* app dir
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
            Some(dirs::home_dir()?.join("curseforge").join("minecraft"))
        }
        ImportLauncherType::Unknown => None,
    };
    let path = path?;
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Checks if this PathBuf is a valid instance for the given launcher type
#[theseus_macros::debug_pin]
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
#[theseus_macros::debug_pin]
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

async fn copy_dotminecraft(
    profile_path: ProfilePathId,
    dotminecraft: PathBuf,
) -> crate::Result<()> {
    // Get full path to profile
    let profile_path = profile_path.get_full_path().await?;

    // std fs copy every file in dotminecraft to profile_path
    let mut dir = io::read_dir(&dotminecraft).await?;
    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, &dotminecraft))?
    {
        let path = entry.path();
        copy_dir_to(
            &path,
            &profile_path.join(path.file_name().ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Invalid file: {}",
                    &path.display()
                ))
            })?),
        )
        .await?;
    }
    Ok(())
}

/// Recursively fs::copy every file in src to dest
/// uses async recursion
#[theseus_macros::debug_pin]
#[async_recursion::async_recursion]
#[tracing::instrument]
async fn copy_dir_to(src: &Path, dst: &Path) -> crate::Result<()> {
    if !src.is_dir() {
        io::copy(src, dst).await?;
        return Ok(());
    }

    // Create the destination directory
    io::create_dir_all(&dst).await?;

    // Iterate over the directory
    let mut dir = io::read_dir(&src).await?;
    while let Some(child) = dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, src))?
    {
        let src_child = child.path();
        let dst_child = dst.join(src_child.file_name().ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Invalid file: {}",
                &src_child.display()
            ))
        })?);

        if src_child.is_dir() {
            // Recurse into sub-directory
            copy_dir_to(&src_child, &dst_child).await?;
        } else {
            // Copy file
            io::copy(&src_child, &dst_child).await?;
        }
    }

    Ok(())
}
