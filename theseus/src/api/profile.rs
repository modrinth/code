//! Theseus profile management interface
use crate::event::emit::{
    emit_loading, init_loading, loading_try_for_each_concurrent,
};
use crate::event::LoadingBarType;
use crate::pack::install_from::{
    EnvType, PackDependency, PackFile, PackFileHash, PackFormat,
};
use crate::prelude::JavaVersion;
use crate::state::{ProfilePathId, ProjectMetadata, ProjectPathId};

use crate::util::io::{self, IOError};
use crate::{
    auth::{self, refresh},
    event::{emit::emit_profile, ProfilePayloadType},
    state::MinecraftChild,
};
pub use crate::{
    state::{JavaSettings, Profile},
    State,
};
use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use std::collections::HashMap;
use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::io::AsyncReadExt;
use tokio::{fs::File, process::Command, sync::RwLock};

/// Remove a profile
#[tracing::instrument]
pub async fn remove(path: &ProfilePathId) -> crate::Result<()> {
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;

    if let Some(profile) = profiles.remove(path).await? {
        emit_profile(
            profile.uuid,
            profile.get_profile_full_path().await?,
            &profile.metadata.name,
            ProfilePayloadType::Removed,
        )
        .await?;
    }

    Ok(())
}

/// Get a profile by relative path (or, name)
#[tracing::instrument]
pub async fn get(
    path: &ProfilePathId,
    clear_projects: Option<bool>,
) -> crate::Result<Option<Profile>> {
    let state = State::get().await?;

    let profiles = state.profiles.read().await;
    let mut profile = profiles.0.get(path).cloned();

    if clear_projects.unwrap_or(false) {
        if let Some(profile) = &mut profile {
            profile.projects = HashMap::new();
        }
    }

    Ok(profile)
}

/// Get a profile by uuid
#[tracing::instrument]
pub async fn get_by_uuid(
    uuid: uuid::Uuid,
    clear_projects: Option<bool>,
) -> crate::Result<Option<Profile>> {
    let state = State::get().await?;

    let profiles = state.profiles.read().await;
    let mut profile = profiles.0.values().find(|x| x.uuid == uuid).cloned();

    if clear_projects.unwrap_or(false) {
        if let Some(profile) = &mut profile {
            profile.projects = HashMap::new();
        }
    }

    Ok(profile)
}

/// Edit a profile using a given asynchronous closure
pub async fn edit<Fut>(
    path: &ProfilePathId,
    action: impl Fn(&mut Profile) -> Fut,
) -> crate::Result<()>
where
    Fut: Future<Output = crate::Result<()>>,
{
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;

    match profiles.0.get_mut(path) {
        Some(ref mut profile) => {
            action(profile).await?;

            emit_profile(
                profile.uuid,
                profile.get_profile_full_path().await?,
                &profile.metadata.name,
                ProfilePayloadType::Edited,
            )
            .await?;

            Ok(())
        }
        None => Err(crate::ErrorKind::UnmanagedProfileError(path.to_string())
            .as_error()),
    }
}

/// Edits a profile's icon
pub async fn edit_icon(
    path: &ProfilePathId,
    icon_path: Option<&Path>,
) -> crate::Result<()> {
    let state = State::get().await?;

    let res = if let Some(icon) = icon_path {
        let bytes = io::read(icon).await?;

        let mut profiles = state.profiles.write().await;

        match profiles.0.get_mut(path) {
            Some(ref mut profile) => {
                profile
                    .set_icon(
                        &state.directories.caches_dir(),
                        &state.io_semaphore,
                        bytes::Bytes::from(bytes),
                        &icon.to_string_lossy(),
                    )
                    .await?;

                emit_profile(
                    profile.uuid,
                    profile.get_profile_full_path().await?,
                    &profile.metadata.name,
                    ProfilePayloadType::Edited,
                )
                .await?;
                Ok(())
            }
            None => {
                Err(crate::ErrorKind::UnmanagedProfileError(path.to_string())
                    .as_error())
            }
        }
    } else {
        edit(path, |profile| {
            profile.metadata.icon = None;
            async { Ok(()) }
        })
        .await?;
        State::sync().await?;

        Ok(())
    };
    State::sync().await?;
    res
}

// Gets the optimal JRE key for the given profile, using Daedalus
// Generally this would be used for profile_create, to get the optimal JRE key
// this can be overwritten by the user a profile-by-profile basis
pub async fn get_optimal_jre_key(
    path: &ProfilePathId,
) -> crate::Result<Option<JavaVersion>> {
    let state = State::get().await?;

    if let Some(profile) = get(path, None).await? {
        let metadata = state.metadata.read().await;

        // Fetch version info from stored profile game_version
        let version = metadata
            .minecraft
            .versions
            .iter()
            .find(|it| it.id == profile.metadata.game_version)
            .ok_or_else(|| {
                crate::ErrorKind::LauncherError(format!(
                    "Invalid or unknown Minecraft version: {}",
                    profile.metadata.game_version
                ))
            })?;

        // Get detailed manifest info from Daedalus
        let version_info = crate::launcher::download::download_version_info(
            &state,
            version,
            profile.metadata.loader_version.as_ref(),
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
pub async fn list(
    clear_projects: Option<bool>,
) -> crate::Result<HashMap<ProfilePathId, Profile>> {
    let state = State::get().await?;
    let profiles = state.profiles.read().await;
    Ok(profiles
        .0
        .clone()
        .into_iter()
        .map(|mut x| {
            if clear_projects.unwrap_or(false) {
                x.1.projects = HashMap::new();
            }

            x
        })
        .collect())
}

/// Installs/Repairs a profile
#[tracing::instrument]
pub async fn install(path: &ProfilePathId) -> crate::Result<()> {
    if let Some(profile) = get(path, None).await? {
        crate::launcher::install_minecraft(&profile, None).await?;
    } else {
        return Err(crate::ErrorKind::UnmanagedProfileError(path.to_string())
            .as_error());
    }
    State::sync().await?;
    Ok(())
}

#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn update_all(
    profile_path: &ProfilePathId,
) -> crate::Result<HashMap<ProjectPathId, ProjectPathId>> {
    if let Some(profile) = get(profile_path, None).await? {
        let loading_bar = init_loading(
            LoadingBarType::ProfileUpdate {
                profile_path: profile.get_profile_full_path().await?,
                profile_name: profile.metadata.name.clone(),
            },
            100.0,
            "Updating profile",
        )
        .await?;

        let profile_base_path = profile.get_profile_full_path().await?;
        let keys = profile
            .projects
            .into_iter()
            .filter(|(_, project)| {
                matches!(
                    &project.metadata,
                    ProjectMetadata::Modrinth {
                        update_version: Some(_),
                        ..
                    }
                )
            })
            .map(|x| x.0)
            .collect::<Vec<_>>();
        let len = keys.len();

        let map = Arc::new(RwLock::new(HashMap::new()));

        use futures::StreamExt;
        loading_try_for_each_concurrent(
            futures::stream::iter(keys).map(Ok::<ProjectPathId, crate::Error>),
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

        emit_profile(
            profile.uuid,
            profile_base_path,
            &profile.metadata.name,
            ProfilePayloadType::Edited,
        )
        .await?;
        State::sync().await?;

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
#[theseus_macros::debug_pin]
pub async fn update_project(
    profile_path: &ProfilePathId,
    project_path: &ProjectPathId,
    skip_send_event: Option<bool>,
) -> crate::Result<ProjectPathId> {
    if let Some(profile) = get(profile_path, None).await? {
        if let Some(project) = profile.projects.get(project_path) {
            if let ProjectMetadata::Modrinth {
                update_version: Some(update_version),
                ..
            } = &project.metadata
            {
                let (path, new_version) = profile
                    .add_project_version(update_version.id.clone())
                    .await?;

                if path != project_path.clone() {
                    profile.remove_project(project_path, Some(true)).await?;
                }

                let state = State::get().await?;
                let mut profiles = state.profiles.write().await;
                if let Some(profile) = profiles.0.get_mut(profile_path) {
                    let value = profile.projects.remove(project_path);
                    if let Some(mut project) = value {
                        if let ProjectMetadata::Modrinth {
                            ref mut version,
                            ..
                        } = project.metadata
                        {
                            *version = Box::new(new_version);
                        }
                        profile.projects.insert(path.clone(), project);
                    }
                }

                if !skip_send_event.unwrap_or(false) {
                    emit_profile(
                        profile.uuid,
                        profile.get_profile_full_path().await?,
                        &profile.metadata.name,
                        ProfilePayloadType::Edited,
                    )
                    .await?;
                    State::sync().await?;
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
    profile_path: &ProfilePathId,
    version_id: String,
) -> crate::Result<ProjectPathId> {
    if let Some(profile) = get(profile_path, None).await? {
        let (path, _) = profile.add_project_version(version_id).await?;

        emit_profile(
            profile.uuid,
            profile.get_profile_full_path().await?,
            &profile.metadata.name,
            ProfilePayloadType::Edited,
        )
        .await?;
        State::sync().await?;

        Ok(path)
    } else {
        Err(
            crate::ErrorKind::UnmanagedProfileError(profile_path.to_string())
                .as_error(),
        )
    }
}

/// Add a project from an FS path
/// Uses and returns the relative path to the project as a ProjectPathId
#[tracing::instrument]
pub async fn add_project_from_path(
    profile_path: &ProfilePathId,
    path: &Path,
    project_type: Option<String>,
) -> crate::Result<ProjectPathId> {
    if let Some(profile) = get(profile_path, None).await? {
        let file = io::read(path).await?;
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let path = profile
            .add_project_bytes(
                &file_name,
                bytes::Bytes::from(file),
                project_type.and_then(|x| serde_json::from_str(&x).ok()),
            )
            .await?;

        emit_profile(
            profile.uuid,
            profile.get_profile_full_path().await?,
            &profile.metadata.name,
            ProfilePayloadType::Edited,
        )
        .await?;
        State::sync().await?;

        Ok(path)
    } else {
        Err(
            crate::ErrorKind::UnmanagedProfileError(profile_path.to_string())
                .as_error(),
        )
    }
}

/// Toggle whether a project is disabled or not
/// Project path should be relative to the profile
/// returns the new state, relative to the profile
#[tracing::instrument]
pub async fn toggle_disable_project(
    profile: &ProfilePathId,
    project: &ProjectPathId,
) -> crate::Result<ProjectPathId> {
    if let Some(profile) = get(profile, None).await? {
        let res = profile.toggle_disable_project(project).await?;

        emit_profile(
            profile.uuid,
            profile.get_profile_full_path().await?,
            &profile.metadata.name,
            ProfilePayloadType::Edited,
        )
        .await?;
        State::sync().await?;

        Ok(res)
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(profile.to_string())
            .as_error())
    }
}

/// Remove a project from a profile
/// Uses and returns the relative path to the project
#[tracing::instrument]
pub async fn remove_project(
    profile: &ProfilePathId,
    project: &ProjectPathId,
) -> crate::Result<()> {
    if let Some(profile) = get(profile, None).await? {
        profile.remove_project(project, None).await?;

        emit_profile(
            profile.uuid,
            profile.get_profile_full_path().await?,
            &profile.metadata.name,
            ProfilePayloadType::Edited,
        )
        .await?;
        State::sync().await?;

        Ok(())
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(profile.to_string())
            .as_error())
    }
}

/// Exports the profile to a Modrinth-formatted .mrpack file
// Version ID of uploaded version (ie 1.1.5), not the unique identifying ID of the version (nvrqJg44)
#[tracing::instrument(skip_all)]
#[theseus_macros::debug_pin]
pub async fn export_mrpack(
    profile_path: &ProfilePathId,
    export_path: PathBuf,
    included_overrides: Vec<String>, // which folders to include in the overrides
    version_id: Option<String>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let io_semaphore = state.io_semaphore.0.read().await;
    let _permit: tokio::sync::SemaphorePermit = io_semaphore.acquire().await?;
    let profile = get(profile_path, None).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to export a nonexistent or unloaded profile at path {}!",
            profile_path
        ))
    })?;

    // remove .DS_Store files from included_overrides
    let included_overrides = included_overrides
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

    let profile_base_path = &profile.get_profile_full_path().await?;

    let mut file = File::create(&export_path)
        .await
        .map_err(|e| IOError::with_path(e, &export_path))?;
    let mut writer = ZipFileWriter::new(&mut file);

    // Create mrpack json configuration file
    let version_id = version_id.unwrap_or("1.0.0".to_string());
    let packfile = create_mrpack_json(&profile, version_id).await?;
    let modrinth_path_list = get_modrinth_pack_list(&packfile);

    // Build vec of all files in the folder
    let mut path_list = Vec::new();
    build_folder(profile_base_path, &mut path_list).await?;

    // Initialize loading bar
    let loading_bar = init_loading(
        LoadingBarType::ZipExtract {
            profile_path: profile.get_profile_full_path().await?,
            profile_name: profile.metadata.name.clone(),
        },
        path_list.len() as f64,
        "Exporting profile to .mrpack",
    )
    .await?;

    // Iterate over every file in the folder
    // Every file that is NOT in the config file is added to the zip, in overrides
    for path in path_list {
        emit_loading(&loading_bar, 1.0, None).await?;

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
            let mut file = File::open(&path)
                .await
                .map_err(|e| IOError::with_path(e, &path))?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .await
                .map_err(|e| IOError::with_path(e, &path))?;
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

// Given a folder path, populate a Vec of all the subfolders
// Intended to be used for finding potential override folders
// profile
// -- folder1
// -- folder2
// -- file1
// => [folder1, folder2]
#[tracing::instrument]
pub async fn get_potential_override_folders(
    profile_path: ProfilePathId,
) -> crate::Result<Vec<PathBuf>> {
    // First, get a dummy mrpack json for the files within
    let profile: Profile =
        get(&profile_path, None).await?.ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "Tried to export a nonexistent or unloaded profile at path {}!",
                profile_path
            ))
        })?;
    // dummy mrpack to get pack list
    let mrpack = create_mrpack_json(&profile, "0".to_string()).await?;
    let mrpack_files = get_modrinth_pack_list(&mrpack);

    let mut path_list: Vec<PathBuf> = Vec::new();

    let profile_base_dir = profile.get_profile_full_path().await?;
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
                let name = path.strip_prefix(&profile_base_dir)?.to_path_buf();
                if !mrpack_files.contains(&name.to_string_lossy().to_string()) {
                    path_list.push(name);
                }
            }
        } else {
            // One layer of files/folders if its a file
            let name = path.strip_prefix(&profile_base_dir)?.to_path_buf();
            if !mrpack_files.contains(&name.to_string_lossy().to_string()) {
                path_list.push(name);
            }
        }
    }
    Ok(path_list)
}

/// Run Minecraft using a profile and the default credentials, logged in credentials,
/// failing with an error if no credentials are available
#[tracing::instrument]
pub async fn run(
    path: &ProfilePathId,
) -> crate::Result<Arc<RwLock<MinecraftChild>>> {
    let state = State::get().await?;

    // Get default account and refresh credentials (preferred way to log in)
    let default_account = state.settings.read().await.default_user;
    let credentials = if let Some(default_account) = default_account {
        refresh(default_account).await?
    } else {
        // If no default account, try to use a logged in account
        let users = auth::users().await?;
        let last_account = users.first();
        if let Some(last_account) = last_account {
            refresh(last_account.id).await?
        } else {
            return Err(crate::ErrorKind::NoCredentialsError.as_error());
        }
    };
    run_credentials(path, &credentials).await
}

/// Run Minecraft using a profile, and credentials for authentication
/// Returns Arc pointer to RwLock to Child
#[tracing::instrument(skip(credentials))]
#[theseus_macros::debug_pin]
pub async fn run_credentials(
    path: &ProfilePathId,
    credentials: &auth::Credentials,
) -> crate::Result<Arc<RwLock<MinecraftChild>>> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    let profile = get(path, None).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to run a nonexistent or unloaded profile at path {}!",
            path
        ))
    })?;

    let pre_launch_hooks =
        &profile.hooks.as_ref().unwrap_or(&settings.hooks).pre_launch;
    if let Some(hook) = pre_launch_hooks {
        // TODO: hook parameters
        let mut cmd = hook.split(' ');
        if let Some(command) = cmd.next() {
            let full_path = path.get_full_path().await?;
            let result = Command::new(command)
                .args(&cmd.collect::<Vec<&str>>())
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
        .java
        .as_ref()
        .and_then(|it| it.extra_arguments.as_ref())
        .unwrap_or(&settings.custom_java_args);

    let wrapper = profile
        .hooks
        .as_ref()
        .map_or(&settings.hooks.wrapper, |it| &it.wrapper);

    let memory = profile.memory.unwrap_or(settings.memory);
    let resolution = profile.resolution.unwrap_or(settings.game_resolution);

    let env_args = profile
        .java
        .as_ref()
        .and_then(|x| x.custom_env_args.as_ref())
        .unwrap_or(&settings.custom_env_args);

    // Post post exit hooks
    let post_exit_hook =
        &profile.hooks.as_ref().unwrap_or(&settings.hooks).post_exit;

    let post_exit_hook = if let Some(hook) = post_exit_hook {
        let mut cmd = hook.split(' ');
        if let Some(command) = cmd.next() {
            let mut command = Command::new(command);
            command
                .args(&cmd.collect::<Vec<&str>>())
                .current_dir(path.get_full_path().await?);
            Some(command)
        } else {
            None
        }
    } else {
        None
    };

    let mc_process = crate::launcher::launch_minecraft(
        java_args,
        env_args,
        wrapper,
        &memory,
        &resolution,
        credentials,
        post_exit_hook,
        &profile,
    )
    .await?;
    Ok(mc_process)
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
#[tracing::instrument(skip_all)]
pub async fn create_mrpack_json(
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

    let profile_base_path = profile.get_profile_full_path().await?;
    let files: Result<Vec<PackFile>, crate::ErrorKind> = profile
        .projects
        .iter()
        .filter_map(|(mod_path, project)| {
            let path: String = profile_base_path
                .join(mod_path.0.clone())
                .to_string_lossy()
                .to_string();

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
    let mut read_dir = io::read_dir(path).await?;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, path))?
    {
        let path = entry.path();
        if path.is_dir() {
            build_folder(&path, path_list).await?;
        } else {
            path_list.push(path);
        }
    }
    Ok(())
}
