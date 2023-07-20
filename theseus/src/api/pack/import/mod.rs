use std::path::{Path, PathBuf};

use tokio::fs;

use crate::{util::fetch, event::LoadingBarId};

pub mod atlauncher;
pub mod curseforge;
pub mod gdlauncher;
pub mod mmc;

#[derive(Debug, Clone, Copy)]
pub enum ImportLauncherType {
    Modrinth,
    MultiMC,
    PrismLauncher,
    ATLauncher,
    GDLauncher,
    Curseforge,
    Unknown
}

pub async fn get_importable_instances(   
    launcher_type: ImportLauncherType,
    base_path: PathBuf,
) -> crate::Result<Vec<String>> {

    // Some launchers have a different folder structure for instances
    let instances_folder = match launcher_type {
        ImportLauncherType::Modrinth => {
            todo!()
        },
        ImportLauncherType::GDLauncher | ImportLauncherType::MultiMC |  ImportLauncherType::PrismLauncher | ImportLauncherType::ATLauncher => {
            base_path.join("instances")
        },
        ImportLauncherType::Curseforge => {
            base_path.join("Instances")
        },
        ImportLauncherType::Unknown => {
            todo!()
        }
    };
    println!("Searching {:?} - instances_folder: {:?}", launcher_type, instances_folder);
    let mut instances = Vec::new();
    let mut dir = fs::read_dir(instances_folder).await?;
    while let Some(entry) = dir.next_entry().await? {
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

pub async fn import_instance(
    profile_path: PathBuf,
    launcher_type: ImportLauncherType,
    base_path: PathBuf,
    instance_folder: String,
    existing_loading_bar: Option<LoadingBarId>
) -> crate::Result<()> {
    match launcher_type {
        ImportLauncherType::Modrinth => {
            todo!()
        },
        ImportLauncherType::MultiMC |  ImportLauncherType::PrismLauncher => {
            mmc::import_mmc(
                base_path,  // path to base mmc folder
                instance_folder, // instance folder in mmc_base_path
                profile_path,   // path to profile
                existing_loading_bar,
            ).await?;
        },
        ImportLauncherType::ATLauncher => {
            atlauncher::import_atlauncher(
                base_path,  // path to atlauncher folder
                instance_folder, // instance folder in atlauncher
                profile_path,   // path to profile
                existing_loading_bar,
            ).await?;
        },
        ImportLauncherType::GDLauncher => {
            gdlauncher::import_gdlauncher(
                base_path.join("instances").join(instance_folder),  // path to gdlauncher folder
                profile_path,   // path to profile
                existing_loading_bar,
            ).await?;
        },
        ImportLauncherType::Curseforge => {
        
            curseforge::import_curseforge(
                base_path.join("Instances").join(instance_folder), // path to curseforge folder
                profile_path,   // path to profile
                existing_loading_bar,
            ).await?;
        },
        ImportLauncherType::Unknown => {
            todo!()
        }
    }
    Ok(())
}

pub async fn guess_launcher(filepath : &Path) -> crate::Result<ImportLauncherType> {
    // search filepath for each launcher type
    // if found, return that launcher type
    // if not found, return unknown
    let mut found_type = ImportLauncherType::Unknown;

    // search path as string for mmc
    if filepath.to_string_lossy().to_lowercase().contains("multimc") {
        found_type = ImportLauncherType::MultiMC;
    }

    // search path as string for prism
    if filepath.to_string_lossy().to_lowercase().contains("prism") {
        found_type = ImportLauncherType::PrismLauncher;
    }

    // search path as string for atlauncher
    if filepath.to_string_lossy().to_lowercase().contains("atlauncher") {
        found_type = ImportLauncherType::ATLauncher;
    }

    // search path as string for curseforge
    if filepath.to_string_lossy().to_lowercase().contains("curseforge") {
        found_type = ImportLauncherType::Curseforge;
    }

    // search path as string for modrinth
    if filepath.to_string_lossy().to_lowercase().contains("modrinth") {
        found_type = ImportLauncherType::Modrinth;
    }

    // search path as string for gdlauncher
    if filepath.to_string_lossy().to_lowercase().contains("gdlauncher") {
        found_type = ImportLauncherType::GDLauncher;
    }

    Ok(found_type)
}

// Checks if this PathBuf is a valid instance for the given launcher type
#[theseus_macros::debug_pin]
#[tracing::instrument]
pub async fn is_valid_importable_instance(instance_path : PathBuf, r#type : ImportLauncherType) -> bool {
    match r#type {
        ImportLauncherType::Modrinth => {
            todo!()
        },
        ImportLauncherType::MultiMC |  ImportLauncherType::PrismLauncher => {
            mmc::is_valid_mmc(instance_path).await
        },
        ImportLauncherType::ATLauncher => {
            atlauncher::is_valid_atlauncher(instance_path).await
        },
        ImportLauncherType::GDLauncher => {
            gdlauncher::is_valid_gdlauncher(instance_path).await
        },
        ImportLauncherType::Curseforge => {
            curseforge::is_valid_curseforge(instance_path).await
        },
        ImportLauncherType::Unknown => {
            todo!()
        }
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
        dbg!(&cache_dir);
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
    profile_path: PathBuf,
    dotminecraft: PathBuf,
) -> crate::Result<()> {
    // std fs copy every file in dotminecraft to profile_path
    for entry in std::fs::read_dir(dotminecraft)? {
        let entry = entry?;
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
        fs::copy(src, dst).await?;
        return Ok(());
    }

    // Create the destination directory
    fs::create_dir_all(&dst).await?;

    // Iterate over the directory
    let mut dir = fs::read_dir(src).await?;
    while let Some(child) = dir.next_entry().await? {
        let src_child = child.path();
        let dst_child = dst.join(src_child.file_name().ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Invalid file: {}",
                &src_child.display()
            ))
        })?);

        if child.metadata().await?.is_dir() {
            // Recurse into sub-directory
            copy_dir_to(&src_child, &dst_child).await?;
        } else {
            // Copy file
            fs::copy(&src_child, &dst_child).await?;
        }
    }

    Ok(())
}
