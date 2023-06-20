//! Functions for fetching infromation from the Internet
use crate::event::emit::{emit_loading, init_loading};
use crate::pack::{
    EnvType, PackDependency, PackFile, PackFileHash, PackFormat,
};
use crate::process::Profile;
use crate::profile::get;
use crate::LoadingBarType;
use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use tokio::sync::SemaphorePermit;

/// Creates a .mrpack (Modrinth zip file) for a given modpack
// Version ID of uploaded version (ie 1.1.5), not the unique identifying ID of the version (nvrqJg44)
#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn export_mrpack(
    profile: &Profile,
    export_location: &Path,
    version_id: String,
    included_overrides: Vec<String>, // which folders to include in the overrides
    loading_bar: bool,
    _semaphore: &SemaphorePermit<'_>,
) -> crate::Result<()> {
    let profile_base_path = &profile.path;

    let mut file = File::create(export_location).await?;
    let mut writer = ZipFileWriter::new(&mut file);

    // Create mrpack json configuration file
    let packfile = create_mrpack_json(profile, version_id)?;
    let modrinth_path_list = get_modrinth_pack_list(&packfile);

    // Build vec of all files in the folder
    let mut path_list = Vec::new();
    build_folder(profile_base_path, &mut path_list).await?;

    // Initialize loading bar
    let loading_bar = if loading_bar {
        Some(
            init_loading(
                LoadingBarType::ZipExtract {
                    profile_path: profile.path.to_path_buf(),
                    profile_name: profile.metadata.name.clone(),
                },
                path_list.len() as f64,
                "Exporting profile to .mrpack",
            )
            .await?,
        )
    } else {
        None
    };

    // Iterate over every file in the folder
    // Every file that is NOT in the config file is added to the zip, in overrides
    for path in path_list {
        if let Some(ref loading_bar) = loading_bar {
            emit_loading(loading_bar, 1.0, None).await?;
        }

        // Get local path of file, relative to profile folder
        let relative_path = path.strip_prefix(profile_base_path)?;

        // Get highest level folder pair ('a/b' in 'a/b/c', 'a' in 'a')
        // We only go one layer deep for the sake of not having a huge list of overrides
        let topmost_two = relative_path
            .iter()
            .take(2)
            .map(|os| os.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        // a,b => a/b
        // a => a
        let topmost = match topmost_two.len() {
            2 => topmost_two.join("/"),
            1 => topmost_two[0].clone(),
            _ => {
                return Err(crate::ErrorKind::OtherError(
                    "No topmost folder found".to_string(),
                )
                .into())
            }
        };

        if !included_overrides.contains(&topmost) {
            continue;
        }

        let relative_path: std::borrow::Cow<str> =
            relative_path.to_string_lossy();
        let relative_path = relative_path.replace('\\', "/");
        let relative_path = relative_path.trim_start_matches('/').to_string();

        if modrinth_path_list.contains(&relative_path) {
            continue;
        }

        // File is not in the config file, add it to the .mrpack zip
        if path.is_file() {
            let mut file = File::open(&path).await?;
            let mut data = Vec::new();
            file.read_to_end(&mut data).await?;
            let builder = ZipEntryBuilder::new(
                format!("overrides/{relative_path}"),
                Compression::Deflate,
            );
            writer.write_entry_whole(builder, &data).await?;
        }
    }

    // Add modrinth json to the zip
    let data = serde_json::to_vec_pretty(&packfile)?;
    let builder = ZipEntryBuilder::new(
        "modrinth.index.json".to_string(),
        Compression::Deflate,
    );
    writer.write_entry_whole(builder, &data).await?;

    writer.close().await?;
    Ok(())
}

fn get_modrinth_pack_list(packfile: &PackFormat) -> Vec<String> {
    packfile
        .files
        .iter()
        .map(|f| {
            let path = PathBuf::from(f.path.clone());
            let name = path.to_string_lossy();
            let name = name.replace('\\', "/");
            name.trim_start_matches('/').to_string()
        })
        .collect::<Vec<String>>()
}

/// Creates a json configuration for a .mrpack zipped file
// Version ID of uploaded version (ie 1.1.5), not the unique identifying ID of the version (nvrqJg44)
#[tracing::instrument]
pub fn create_mrpack_json(
    profile: &Profile,
    version_id: String,
) -> crate::Result<PackFormat> {
    // Add loader version to dependencies
    let mut dependencies = HashMap::new();
    match (
        profile.metadata.loader,
        profile.metadata.loader_version.clone(),
    ) {
        (crate::prelude::ModLoader::Forge, Some(v)) => {
            dependencies.insert(PackDependency::Forge, v.id)
        }
        (crate::prelude::ModLoader::Fabric, Some(v)) => {
            dependencies.insert(PackDependency::FabricLoader, v.id)
        }
        (crate::prelude::ModLoader::Quilt, Some(v)) => {
            dependencies.insert(PackDependency::QuiltLoader, v.id)
        }
        (crate::prelude::ModLoader::Vanilla, _) => None,
        _ => {
            return Err(crate::ErrorKind::OtherError(
                "Loader version mismatch".to_string(),
            )
            .into())
        }
    };
    dependencies.insert(
        PackDependency::Minecraft,
        profile.metadata.game_version.clone(),
    );

    // Converts a HashMap<String, String> to a HashMap<String, String>
    // But the values are sanitized to only include the version number
    let dependencies = dependencies
        .into_iter()
        .map(|(k, v)| (k, sanitize_loader_version_string(&v).to_string()))
        .collect::<HashMap<_, _>>();

    let base_path = &profile.path;
    let files: Result<Vec<PackFile>, crate::ErrorKind> = profile
        .projects
        .iter()
        .filter_map(|(mod_path, project)| {
            let path = match mod_path.strip_prefix(base_path) {
                Ok(path) => path.to_string_lossy().to_string(),
                Err(e) => {
                    return Some(Err(e.into()));
                }
            };

            // Only Modrinth projects have a modrinth metadata field for the modrinth.json
            Some(Ok(match project.metadata {
                crate::prelude::ProjectMetadata::Modrinth {
                    ref project,
                    ref version,
                    ..
                } => {
                    let mut env = HashMap::new();
                    env.insert(EnvType::Client, project.client_side.clone());
                    env.insert(EnvType::Server, project.server_side.clone());

                    let primary_file = if let Some(primary_file) =
                        version.files.first()
                    {
                        primary_file
                    } else {
                        return Some(Err(crate::ErrorKind::OtherError(
                            format!("No primary file found for mod at: {path}"),
                        )));
                    };

                    let file_size = primary_file.size;
                    let downloads = vec![primary_file.url.clone()];
                    let hashes = primary_file
                        .hashes
                        .clone()
                        .into_iter()
                        .map(|(h1, h2)| (PackFileHash::from(h1), h2))
                        .collect();

                    PackFile {
                        path,
                        hashes,
                        env: Some(env),
                        downloads,
                        file_size,
                    }
                }
                // Inferred files are skipped for the modrinth.json
                crate::prelude::ProjectMetadata::Inferred { .. } => {
                    return None
                }
                // Unknown projects are skipped for the modrinth.json
                crate::prelude::ProjectMetadata::Unknown => return None,
            }))
        })
        .collect();
    let files = files?;

    Ok(PackFormat {
        game: "minecraft".to_string(),
        format_version: 1,
        version_id,
        name: profile.metadata.name.clone(),
        summary: None,
        files,
        dependencies,
    })
}

fn sanitize_loader_version_string(s: &str) -> &str {
    // Split on '-'
    // If two or more, take the second
    // If one, take the first
    // If none, take the whole thing
    let mut split: std::str::Split<'_, char> = s.split('-');
    match split.next() {
        Some(first) => match split.next() {
            Some(second) => second,
            None => first,
        },
        None => s,
    }
}

// Given a folder path, populate a Vec of all the files in the folder, recursively
#[async_recursion::async_recursion]
pub async fn build_folder(
    path: &Path,
    path_list: &mut Vec<PathBuf>,
) -> crate::Result<()> {
    let mut read_dir = fs::read_dir(path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            build_folder(&path, path_list).await?;
        } else {
            path_list.push(path);
        }
    }
    Ok(())
}

// Given a folder path, populate a Vec of all the subfolders
// Intended to be used for finding potential override folders
// profile
// -- folder1
// -- folder2
// ----- file2
// ----- folder3
// ------- folder4
// -- file1
// => [folder1, folder2, fil2, folder3, file1]
pub async fn get_potential_override_folders(
    profile_path: PathBuf,
) -> crate::Result<Vec<PathBuf>> {

    // Force sync the profile before export functions
    Profile::sync_projects_inner(profile_path.clone()).await?;


    // First, get a dummy mrpack json for the files within
    let profile: Profile =
        get(&profile_path, None).await?.ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "Tried to export a nonexistent or unloaded profile at path {}!",
                profile_path.display()
            ))
        })?;
    let mrpack = create_mrpack_json(&profile, "0".to_string())?;
    let mrpack_files = get_modrinth_pack_list(&mrpack);

    let mut path_list: Vec<PathBuf> = Vec::new();
    let mut read_dir = fs::read_dir(&profile_path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let path: PathBuf = entry.path();
        if path.is_dir() {
            // Two layers of files/folders if its a folder
            let mut read_dir = fs::read_dir(&path).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let path: PathBuf = entry.path();
                let name = path.strip_prefix(&profile_path)?.to_path_buf();
                if !mrpack_files.contains(&name.to_string_lossy().to_string()) {
                    path_list.push(name);
                }
            }
        } else {
            // One layer of files/folders if its a file
            let name = path.strip_prefix(&profile_path)?.to_path_buf();
            if !mrpack_files.contains(&name.to_string_lossy().to_string()) {
                path_list.push(name);
            }
        }
    }
    Ok(path_list)
}
