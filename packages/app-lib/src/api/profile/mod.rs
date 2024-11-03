//! Theseus profile management interface

use crate::event::emit::{
    emit_loading, init_loading, loading_try_for_each_concurrent,
};
use crate::event::LoadingBarType;
use crate::pack::install_from::{
    EnvType, PackDependency, PackFile, PackFileHash, PackFormat,
};
use crate::state::{
    CacheBehaviour, CachedEntry, Credentials, JavaVersion, ProcessMetadata,
    ProfileFile, ProjectType, SideType,
};

use crate::event::{emit::emit_profile, ProfilePayloadType};
use crate::util::fetch;
use crate::util::io::{self, IOError};
pub use crate::{state::Profile, State};
use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use serde_json::json;

use std::collections::{HashMap, HashSet};

use crate::data::Settings;
use dashmap::DashMap;
use std::iter::FromIterator;
use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::io::AsyncReadExt;
use tokio::{fs::File, process::Command, sync::RwLock};

pub mod create;
pub mod update;

/// Remove a profile
#[tracing::instrument]
pub async fn remove(path: &str) -> crate::Result<()> {
    let state = State::get().await?;
    Profile::remove(path, &state.pool).await?;

    emit_profile(path, ProfilePayloadType::Removed).await?;

    Ok(())
}

/// Get a profile by relative path (or, name)
#[tracing::instrument]
pub async fn get(path: &str) -> crate::Result<Option<Profile>> {
    let state = State::get().await?;
    let profile = Profile::get(path, &state.pool).await?;

    Ok(profile)
}

#[tracing::instrument]
pub async fn get_many(paths: &[&str]) -> crate::Result<Vec<Profile>> {
    let state = State::get().await?;
    let profiles = Profile::get_many(paths, &state.pool).await?;
    Ok(profiles)
}

#[tracing::instrument]
pub async fn get_projects(
    path: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<DashMap<String, ProfileFile>> {
    let state = State::get().await?;

    if let Some(profile) = get(path).await? {
        let files = profile
            .get_projects(cache_behaviour, &state.pool, &state.api_semaphore)
            .await?;

        Ok(files)
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(path.to_string())
            .as_error())
    }
}

/// Get profile's full path in the filesystem
#[tracing::instrument]
pub async fn get_full_path(path: &str) -> crate::Result<PathBuf> {
    let state = State::get().await?;
    let profiles_dir = state.directories.profiles_dir();

    let full_path = io::canonicalize(profiles_dir.join(path))?;
    Ok(full_path)
}

/// Get mod's full path in the filesystem
#[tracing::instrument]
pub async fn get_mod_full_path(
    profile_path: &str,
    project_path: &str,
) -> crate::Result<PathBuf> {
    let path = get_full_path(profile_path).await?;

    Ok(path.join(project_path))
}

/// Edit a profile using a given asynchronous closure
pub async fn edit<Fut>(
    path: &str,
    action: impl Fn(&mut Profile) -> Fut,
) -> crate::Result<()>
where
    Fut: Future<Output = crate::Result<()>>,
{
    let state = State::get().await?;

    if let Some(mut profile) = get(path).await? {
        action(&mut profile).await?;
        profile.upsert(&state.pool).await?;

        emit_profile(path, ProfilePayloadType::Edited).await?;

        Ok(())
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(path.to_string())
            .as_error())
    }
}

/// Edits a profile's icon
pub async fn edit_icon(
    path: &str,
    icon_path: Option<&Path>,
) -> crate::Result<()> {
    let state = State::get().await?;

    if let Some(mut profile) = get(path).await? {
        if let Some(icon) = icon_path {
            let bytes = io::read(icon).await?;

            profile
                .set_icon(
                    &state.directories.caches_dir(),
                    &state.io_semaphore,
                    bytes::Bytes::from(bytes),
                    &icon.to_string_lossy(),
                )
                .await?;
        } else {
            profile.icon_path = None;
        }

        profile.upsert(&state.pool).await?;

        emit_profile(path, ProfilePayloadType::Edited).await?;

        Ok(())
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(path.to_string())
            .as_error())
    }
}

// Gets the optimal JRE key for the given profile, using Daedalus
// Generally this would be used for profile_create, to get the optimal JRE key
// this can be overwritten by the user a profile-by-profile basis
pub async fn get_optimal_jre_key(
    path: &str,
) -> crate::Result<Option<JavaVersion>> {
    let state = State::get().await?;

    if let Some(profile) = get(path).await? {
        let minecraft = crate::api::metadata::get_minecraft_versions().await?;

        // Fetch version info from stored profile game_version
        let version = minecraft
            .versions
            .iter()
            .find(|it| it.id == profile.game_version)
            .ok_or_else(|| {
                crate::ErrorKind::LauncherError(format!(
                    "Invalid or unknown Minecraft version: {}",
                    profile.game_version
                ))
            })?;

        let loader_version = crate::launcher::get_loader_version_from_profile(
            &profile.game_version,
            profile.loader,
            profile.loader_version.as_deref(),
        )
        .await?;

        // Get detailed manifest info from Daedalus
        let version_info = crate::launcher::download::download_version_info(
            &state,
            version,
            loader_version.as_ref(),
            None,
            None,
        )
        .await?;

        let version = crate::launcher::get_java_version_from_profile(
            &profile,
            &version_info,
        )
        .await?;

        Ok(version)
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(path.to_string())
            .as_error())
    }
}

/// Get a copy of the profile set
#[tracing::instrument]
pub async fn list() -> crate::Result<Vec<Profile>> {
    let state = State::get().await?;
    let profiles = Profile::get_all(&state.pool).await?;
    Ok(profiles)
}

/// Installs/Repairs a profile
#[tracing::instrument]
pub async fn install(path: &str, force: bool) -> crate::Result<()> {
    if let Some(profile) = get(path).await? {
        crate::launcher::install_minecraft(&profile, None, force).await?;
    } else {
        return Err(crate::ErrorKind::UnmanagedProfileError(path.to_string())
            .as_error());
    }
    Ok(())
}

#[tracing::instrument]
pub async fn update_all_projects(
    profile_path: &str,
) -> crate::Result<HashMap<String, String>> {
    if let Some(profile) = get(profile_path).await? {
        let loading_bar = init_loading(
            LoadingBarType::ProfileUpdate {
                profile_path: profile.path.clone(),
                profile_name: profile.name.clone(),
            },
            100.0,
            "Updating profile",
        )
        .await?;

        let state = State::get().await?;
        let keys = profile
            .get_projects(
                Some(CacheBehaviour::MustRevalidate),
                &state.pool,
                &state.api_semaphore,
            )
            .await?
            .into_iter()
            .filter(|(_, project)| project.update_version_id.is_some())
            .map(|x| x.0)
            .collect::<Vec<_>>();
        let len = keys.len();

        let map = Arc::new(RwLock::new(HashMap::new()));

        use futures::StreamExt;
        loading_try_for_each_concurrent(
            futures::stream::iter(keys).map(Ok::<String, crate::Error>),
            None,
            Some(&loading_bar),
            100.0,
            len,
            None,
            |project| async {
                let map = map.clone();

                async move {
                    let new_path =
                        update_project(profile_path, &project, Some(true))
                            .await?;

                    map.write().await.insert(project, new_path);

                    Ok(())
                }
                .await
            },
        )
        .await?;

        emit_profile(profile_path, ProfilePayloadType::Edited).await?;

        Ok(Arc::try_unwrap(map).unwrap().into_inner())
    } else {
        Err(
            crate::ErrorKind::UnmanagedProfileError(profile_path.to_string())
                .as_error(),
        )
    }
}

/// Updates a project to the latest version
/// Uses and returns the relative path to the project
#[tracing::instrument]
pub async fn update_project(
    profile_path: &str,
    project_path: &str,
    skip_send_event: Option<bool>,
) -> crate::Result<String> {
    if let Some(profile) = get(profile_path).await? {
        let state = State::get().await?;
        if let Some((_, file)) = profile
            .get_projects(
                Some(CacheBehaviour::MustRevalidate),
                &state.pool,
                &state.api_semaphore,
            )
            .await?
            .remove(project_path)
        {
            if let Some(update_version) = &file.update_version_id {
                let path = Profile::add_project_version(
                    profile_path,
                    update_version,
                    &state.pool,
                    &state.fetch_semaphore,
                    &state.io_semaphore,
                )
                .await?;

                if path != project_path {
                    Profile::remove_project(profile_path, project_path).await?;
                }

                if !skip_send_event.unwrap_or(false) {
                    emit_profile(profile_path, ProfilePayloadType::Edited)
                        .await?;
                }

                return Ok(path);
            }
        }

        Err(crate::ErrorKind::InputError(
            "This project cannot be updated!".to_string(),
        )
        .as_error())
    } else {
        Err(
            crate::ErrorKind::UnmanagedProfileError(profile_path.to_string())
                .as_error(),
        )
    }
}

/// Add a project from a version
/// Returns the relative path to the project as a ProjectPathId
#[tracing::instrument]
pub async fn add_project_from_version(
    profile_path: &str,
    version_id: &str,
) -> crate::Result<String> {
    let state = State::get().await?;
    let project_path = Profile::add_project_version(
        profile_path,
        version_id,
        &state.pool,
        &state.fetch_semaphore,
        &state.io_semaphore,
    )
    .await?;

    emit_profile(profile_path, ProfilePayloadType::Edited).await?;

    Ok(project_path)
}

/// Add a project from an FS path
/// Uses and returns the relative path to the project as a ProjectPathId
#[tracing::instrument]
pub async fn add_project_from_path(
    profile_path: &str,
    path: &Path,
    project_type: Option<ProjectType>,
) -> crate::Result<String> {
    let state = State::get().await?;

    let file = io::read(path).await?;
    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let path = Profile::add_project_bytes(
        profile_path,
        &file_name,
        bytes::Bytes::from(file),
        None,
        project_type,
        &state.io_semaphore,
        &state.pool,
    )
    .await?;

    Ok(path)
}

/// Toggle whether a project is disabled or not
/// Project path should be relative to the profile
/// returns the new state, relative to the profile
#[tracing::instrument]
pub async fn toggle_disable_project(
    profile_path: &str,
    project: &str,
) -> crate::Result<String> {
    let res = Profile::toggle_disable_project(profile_path, project).await?;

    emit_profile(profile_path, ProfilePayloadType::Edited).await?;

    Ok(res)
}

/// Remove a project from a profile
/// Uses and returns the relative path to the project
#[tracing::instrument]
pub async fn remove_project(
    profile_path: &str,
    project: &str,
) -> crate::Result<()> {
    Profile::remove_project(profile_path, project).await?;

    emit_profile(profile_path, ProfilePayloadType::Edited).await?;

    Ok(())
}

/// Exports the profile to a Modrinth-formatted .mrpack file
// Version ID of uploaded version (ie 1.1.5), not the unique identifying ID of the version (nvrqJg44)
#[tracing::instrument(skip_all)]
pub async fn export_mrpack(
    profile_path: &str,
    export_path: PathBuf,
    included_export_candidates: Vec<String>, // which folders/files to include in the export
    version_id: Option<String>,
    description: Option<String>,
    _name: Option<String>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _permit: tokio::sync::SemaphorePermit =
        state.io_semaphore.0.acquire().await?;
    let profile = get(profile_path).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to export a nonexistent or unloaded profile at path {}!",
            profile_path
        ))
    })?;

    // remove .DS_Store files from included_export_candidates
    let included_export_candidates = included_export_candidates
        .into_iter()
        .filter(|x| {
            if let Some(f) = PathBuf::from(x).file_name() {
                if f.to_string_lossy().starts_with(".DS_Store") {
                    return false;
                }
            }
            true
        })
        .collect::<Vec<_>>();

    let profile_base_path = get_full_path(profile_path).await?;

    let mut file = File::create(&export_path)
        .await
        .map_err(|e| IOError::with_path(e, &export_path))?;
    let mut writer = ZipFileWriter::with_tokio(&mut file);

    // Create mrpack json configuration file
    let version_id = version_id.unwrap_or("1.0.0".to_string());
    let mut packfile =
        create_mrpack_json(&profile, version_id, description).await?;
    let included_candidates_set =
        HashSet::<_>::from_iter(included_export_candidates.iter());
    packfile
        .files
        .retain(|f| included_candidates_set.contains(&f.path));

    // Build vec of all files in the folder
    let mut path_list = Vec::new();
    add_all_recursive_folder_paths(&profile_base_path, &mut path_list).await?;

    // Initialize loading bar
    let loading_bar = init_loading(
        LoadingBarType::ZipExtract {
            profile_path: profile.path.clone(),
            profile_name: profile.name.clone(),
        },
        path_list.len() as f64,
        "Exporting profile to .mrpack",
    )
    .await?;

    // Iterate over every file in the folder
    // Every file that is NOT in the config file is added to the zip, in overrides
    for path in path_list {
        emit_loading(&loading_bar, 1.0, None)?;

        let relative_path = pack_get_relative_path(&profile_base_path, &path)?;

        if packfile.files.iter().any(|f| f.path == relative_path)
            || !included_candidates_set
                .iter()
                .any(|x| relative_path.starts_with(&**x))
        {
            continue;
        }

        // File is not in the config file, add it to the .mrpack zip
        if path.is_file() {
            let mut file = File::open(&path)
                .await
                .map_err(|e| IOError::with_path(e, &path))?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .await
                .map_err(|e| IOError::with_path(e, &path))?;
            let builder = ZipEntryBuilder::new(
                format!("overrides/{relative_path}").into(),
                Compression::Deflate,
            );
            writer.write_entry_whole(builder, &data).await?;
        }
    }

    // Add modrinth json to the zip
    let data = serde_json::to_vec_pretty(&packfile)?;
    let builder = ZipEntryBuilder::new(
        "modrinth.index.json".to_string().into(),
        Compression::Deflate,
    );
    writer.write_entry_whole(builder, &data).await?;

    writer.close().await?;

    Ok(())
}

// Given a folder path, populate a Vec of all the subfolders and files, at most 2 layers deep
// profile
// -- folder1
// -- folder2
//    -- innerfolder
//       -- innerfile
//    -- folder2file
// -- file1
// => [folder1, folder2/innerfolder, folder2/folder2file, file1]
#[tracing::instrument]
pub async fn get_pack_export_candidates(
    profile_path: &str,
) -> crate::Result<Vec<String>> {
    let mut path_list: Vec<String> = Vec::new();

    let profile_base_dir = get_full_path(profile_path).await?;
    let mut read_dir = io::read_dir(&profile_base_dir).await?;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, &profile_base_dir))?
    {
        let path: PathBuf = entry.path();
        if path.is_dir() {
            // Two layers of files/folders if its a folder
            let mut read_dir = io::read_dir(&path).await?;
            while let Some(entry) = read_dir
                .next_entry()
                .await
                .map_err(|e| IOError::with_path(e, &profile_base_dir))?
            {
                let path: PathBuf = entry.path();

                path_list
                    .push(pack_get_relative_path(&profile_base_dir, &path)?);
            }
        } else {
            // One layer of files/folders if its a file
            path_list.push(pack_get_relative_path(&profile_base_dir, &path)?);
        }
    }
    Ok(path_list)
}

fn pack_get_relative_path(
    profile_path: &PathBuf,
    path: &PathBuf,
) -> crate::Result<String> {
    Ok(path
        .strip_prefix(profile_path)
        .map_err(|_| {
            crate::ErrorKind::FSError(format!(
                "Path {path:?} does not correspond to a profile",
                path = path
            ))
        })?
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/"))
}

/// Run Minecraft using a profile and the default credentials, logged in credentials,
/// failing with an error if no credentials are available
#[tracing::instrument]
pub async fn run(path: &str) -> crate::Result<ProcessMetadata> {
    let state = State::get().await?;

    let default_account = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or_else(|| crate::ErrorKind::NoCredentialsError.as_error())?;

    run_credentials(path, &default_account).await
}

/// Run Minecraft using a profile, and credentials for authentication
/// Returns Arc pointer to RwLock to Child
#[tracing::instrument(skip(credentials))]

pub async fn run_credentials(
    path: &str,
    credentials: &Credentials,
) -> crate::Result<ProcessMetadata> {
    let state = State::get().await?;
    let settings = Settings::get(&state.pool).await?;
    let profile = get(path).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to run a nonexistent or unloaded profile at path {}!",
            path
        ))
    })?;

    let pre_launch_hooks = profile
        .hooks
        .pre_launch
        .as_ref()
        .or(settings.hooks.pre_launch.as_ref());
    if let Some(hook) = pre_launch_hooks {
        // TODO: hook parameters
        let mut cmd = hook.split(' ');
        if let Some(command) = cmd.next() {
            let full_path = get_full_path(&profile.path).await?;
            let result = Command::new(command)
                .args(cmd.collect::<Vec<&str>>())
                .current_dir(&full_path)
                .spawn()
                .map_err(|e| IOError::with_path(e, &full_path))?
                .wait()
                .await
                .map_err(IOError::from)?;

            if !result.success() {
                return Err(crate::ErrorKind::LauncherError(format!(
                    "Non-zero exit code for pre-launch hook: {}",
                    result.code().unwrap_or(-1)
                ))
                .as_error());
            }
        }
    }

    let java_args = profile
        .extra_launch_args
        .clone()
        .unwrap_or(settings.extra_launch_args);

    let wrapper = profile.hooks.wrapper.clone().or(settings.hooks.wrapper);

    let memory = profile.memory.unwrap_or(settings.memory);
    let resolution =
        profile.game_resolution.unwrap_or(settings.game_resolution);

    let env_args = profile
        .custom_env_vars
        .clone()
        .unwrap_or(settings.custom_env_vars);

    // Post post exit hooks
    let post_exit_hook =
        profile.hooks.post_exit.clone().or(settings.hooks.post_exit);

    // Any options.txt settings that we want set, add here
    let mut mc_set_options: Vec<(String, String)> = vec![];
    if let Some(fullscreen) = profile.force_fullscreen {
        // Profile fullscreen setting takes priority
        mc_set_options.push(("fullscreen".to_string(), fullscreen.to_string()));
    } else if settings.force_fullscreen {
        // If global settings wants to force a fullscreen, do it
        mc_set_options.push(("fullscreen".to_string(), "true".to_string()));
    }

    crate::launcher::launch_minecraft(
        &java_args,
        &env_args,
        &mc_set_options,
        &wrapper,
        &memory,
        &resolution,
        credentials,
        post_exit_hook,
        &profile,
    )
    .await
}

pub async fn kill(path: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let processes = crate::api::process::get_by_profile_path(path).await?;

    for process in processes {
        state.process_manager.kill(process.uuid).await?;
    }

    Ok(())
}

/// Update playtime- sending a request to the server to update the playtime
#[tracing::instrument]
pub async fn try_update_playtime(path: &str) -> crate::Result<()> {
    let state = State::get().await?;

    let profile = get(path).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to update playtime for a nonexistent or unloaded profile at path {}!",
            path
        ))
    })?;
    let updated_recent_playtime = profile.recent_time_played;

    let res = if updated_recent_playtime > 0 {
        // Create update struct to send to Labrinth
        let modrinth_pack_version_id =
            profile.linked_data.as_ref().map(|l| l.version_id.clone());
        let playtime_update_json = json!({
            "seconds": updated_recent_playtime,
            "loader": profile.loader.as_str(),
            "game_version": profile.game_version,
            "parent": modrinth_pack_version_id,
        });
        // Copy this struct for every Modrinth project in the profile
        let mut hashmap: HashMap<String, serde_json::Value> = HashMap::new();

        for (_, project) in profile
            .get_projects(None, &state.pool, &state.api_semaphore)
            .await?
        {
            if let Some(metadata) = project.metadata {
                hashmap
                    .insert(metadata.version_id, playtime_update_json.clone());
            }
        }

        fetch::post_json(
            "https://api.modrinth.com/analytics/playtime",
            serde_json::to_value(hashmap)?,
            &state.api_semaphore,
            &state.pool,
        )
        .await
    } else {
        Ok(())
    };

    // If successful, update the profile metadata to match submitted
    if res.is_ok() {
        edit(&profile.path, |prof| {
            prof.submitted_time_played += updated_recent_playtime;
            prof.recent_time_played = 0;

            async { Ok(()) }
        })
        .await?;
    }

    res
}

/// Creates a json configuration for a .mrpack zipped file
// Version ID of uploaded version (ie 1.1.5), not the unique identifying ID of the version (nvrqJg44)
#[tracing::instrument(skip_all)]
pub async fn create_mrpack_json(
    profile: &Profile,
    version_id: String,
    description: Option<String>,
) -> crate::Result<PackFormat> {
    // Add loader version to dependencies
    let mut dependencies = HashMap::new();
    match (profile.loader, profile.loader_version.clone()) {
        (crate::prelude::ModLoader::Forge, Some(v)) => {
            dependencies.insert(PackDependency::Forge, v)
        }
        (crate::prelude::ModLoader::NeoForge, Some(v)) => {
            dependencies.insert(PackDependency::NeoForge, v)
        }
        (crate::prelude::ModLoader::Fabric, Some(v)) => {
            dependencies.insert(PackDependency::FabricLoader, v)
        }
        (crate::prelude::ModLoader::Quilt, Some(v)) => {
            dependencies.insert(PackDependency::QuiltLoader, v)
        }
        (crate::prelude::ModLoader::Vanilla, _) => None,
        _ => {
            return Err(crate::ErrorKind::OtherError(
                "Loader version mismatch".to_string(),
            )
            .into())
        }
    };
    dependencies
        .insert(PackDependency::Minecraft, profile.game_version.clone());

    let state = State::get().await?;
    let projects = profile
        .get_projects(
            Some(CacheBehaviour::MustRevalidate),
            &state.pool,
            &state.api_semaphore,
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
                // TODO: envtype should be a controllable option (in general or at least .mrpack exporting)
                // For now, assume required.
                // env.insert(EnvType::Client, project.client_side.clone());
                // env.insert(EnvType::Server, project.server_side.clone());
                env.insert(EnvType::Client, SideType::Required);
                env.insert(EnvType::Server, SideType::Required);

                let primary_file =
                    if let Some(primary_file) = version.files.first() {
                        primary_file
                    } else {
                        return Some(Err(crate::ErrorKind::OtherError(
                            format!("No primary file found for mod at: {path}"),
                        )
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
                    path,
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
        name: profile.name.clone(),
        summary: description,
        files,
        dependencies,
    })
}

// Given a folder path, populate a Vec of all the files in the folder, recursively
#[async_recursion::async_recursion]
pub async fn add_all_recursive_folder_paths(
    path: &Path,
    path_list: &mut Vec<PathBuf>,
) -> crate::Result<()> {
    let mut read_dir = io::read_dir(path).await?;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, path))?
    {
        let path = entry.path();
        if path.is_dir() {
            add_all_recursive_folder_paths(&path, path_list).await?;
        } else {
            path_list.push(path);
        }
    }
    Ok(())
}

pub fn sanitize_profile_name(input: &str) -> String {
    input.replace(
        ['/', '\\', '?', '*', ':', '\'', '\"', '|', '<', '>', '!'],
        "_",
    )
}
