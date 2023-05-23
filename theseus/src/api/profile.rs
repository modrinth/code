//! Theseus profile management interface
use crate::event::emit::{init_loading, loading_try_for_each_concurrent};
use crate::event::LoadingBarType;
use crate::prelude::JavaVersion;
use crate::state::ProjectMetadata;
use crate::{
    auth::{self, refresh},
    event::{emit::emit_profile, ProfilePayloadType},
    state::MinecraftChild,
};
pub use crate::{
    state::{JavaSettings, Profile},
    State,
};
use std::collections::HashMap;
use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{fs, process::Command, sync::RwLock};

/// Remove a profile
#[tracing::instrument]
pub async fn remove(path: &Path) -> crate::Result<()> {
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;

    if let Some(profile) = profiles.remove(path).await? {
        emit_profile(
            profile.uuid,
            profile.path.clone(),
            &profile.metadata.name,
            ProfilePayloadType::Removed,
        )
        .await?;
    }

    Ok(())
}

/// Get a profile by path,
#[tracing::instrument]
pub async fn get(
    path: &Path,
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

/// Edit a profile using a given asynchronous closure
pub async fn edit<Fut>(
    path: &Path,
    action: impl Fn(&mut Profile) -> Fut,
) -> crate::Result<()>
where
    Fut: Future<Output = crate::Result<()>>,
{
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;

    match profiles.0.get_mut(path) {
        Some(ref mut profile) => {
            emit_profile(
                profile.uuid,
                profile.path.clone(),
                &profile.metadata.name,
                ProfilePayloadType::Edited,
            )
            .await?;

            action(profile).await
        }
        None => Err(crate::ErrorKind::UnmanagedProfileError(
            path.display().to_string(),
        )
        .as_error()),
    }
}

/// Edits a profile's icon
pub async fn edit_icon(
    path: &Path,
    icon_path: Option<&Path>,
) -> crate::Result<()> {
    let state = State::get().await?;

    if let Some(icon) = icon_path {
        let bytes = tokio::fs::read(icon).await?;

        let mut profiles = state.profiles.write().await;

        match profiles.0.get_mut(path) {
            Some(ref mut profile) => {
                emit_profile(
                    profile.uuid,
                    profile.path.clone(),
                    &profile.metadata.name,
                    ProfilePayloadType::Edited,
                )
                .await?;

                profile
                    .set_icon(
                        &state.directories.caches_dir(),
                        &state.io_semaphore,
                        bytes::Bytes::from(bytes),
                        &icon.to_string_lossy(),
                    )
                    .await
            }
            None => Err(crate::ErrorKind::UnmanagedProfileError(
                path.display().to_string(),
            )
            .as_error()),
        }
    } else {
        edit(path, |profile| {
            profile.metadata.icon = None;
            async { Ok(()) }
        })
        .await
    }
}

// Gets the optimal JRE key for the given profile, using Daedalus
// Generally this would be used for profile_create, to get the optimal JRE key
// this can be overwritten by the user a profile-by-profile basis
pub async fn get_optimal_jre_key(
    path: &Path,
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
        Err(
            crate::ErrorKind::UnmanagedProfileError(path.display().to_string())
                .as_error(),
        )
    }
}

/// Get a copy of the profile set
#[tracing::instrument]
pub async fn list(
    clear_projects: Option<bool>,
) -> crate::Result<HashMap<PathBuf, Profile>> {
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
pub async fn install(path: &Path) -> crate::Result<()> {
    if let Some(profile) = get(path, None).await? {
        crate::launcher::install_minecraft(&profile, None).await?;
    } else {
        return Err(crate::ErrorKind::UnmanagedProfileError(
            path.display().to_string(),
        )
        .as_error());
    }
    State::sync().await?;
    Ok(())
}

#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn update_all(
    profile_path: &Path,
) -> crate::Result<HashMap<PathBuf, PathBuf>> {
    if let Some(profile) = get(profile_path, None).await? {
        let loading_bar = init_loading(
            LoadingBarType::ProfileUpdate {
                profile_path: profile.path.clone(),
                profile_name: profile.metadata.name.clone(),
            },
            100.0,
            "Updating profile",
        )
        .await?;

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
            futures::stream::iter(keys).map(Ok::<PathBuf, crate::Error>),
            None,
            Some(&loading_bar),
            100.0,
            len,
            None,
            |project| async {
                let map = map.clone();

                async move {
                    let new_path =
                        update_project(profile_path, &project).await?;

                    map.write().await.insert(project, new_path);

                    Ok(())
                }
                .await
            },
        )
        .await?;

        Ok(Arc::try_unwrap(map).unwrap().into_inner())
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(
            profile_path.display().to_string(),
        )
        .as_error())
    }
}

#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn update_project(
    profile_path: &Path,
    project_path: &Path,
) -> crate::Result<PathBuf> {
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

                if path != project_path {
                    profile.remove_project(project_path, Some(true)).await?;
                }

                let state = State::get().await?;
                let mut profiles = state.profiles.write().await;
                if let Some(profile) = profiles.0.get_mut(project_path) {
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

                return Ok(path);
            }
        }

        Err(crate::ErrorKind::InputError(
            "This project cannot be updated!".to_string(),
        )
        .as_error())
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(
            profile_path.display().to_string(),
        )
        .as_error())
    }
}

/// Add a project from a version
#[tracing::instrument]
pub async fn add_project_from_version(
    profile_path: &Path,
    version_id: String,
) -> crate::Result<PathBuf> {
    if let Some(profile) = get(profile_path, None).await? {
        let (path, _) = profile.add_project_version(version_id).await?;

        Ok(path)
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(
            profile_path.display().to_string(),
        )
        .as_error())
    }
}

/// Add a project from an FS path
#[tracing::instrument]
pub async fn add_project_from_path(
    profile_path: &Path,
    path: &Path,
    project_type: Option<String>,
) -> crate::Result<PathBuf> {
    if let Some(profile) = get(profile_path, None).await? {
        let file = fs::read(path).await?;
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

        Ok(path)
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(
            profile_path.display().to_string(),
        )
        .as_error())
    }
}

/// Toggle whether a project is disabled or not
#[tracing::instrument]
pub async fn toggle_disable_project(
    profile: &Path,
    project: &Path,
) -> crate::Result<PathBuf> {
    if let Some(profile) = get(profile, None).await? {
        Ok(profile.toggle_disable_project(project).await?)
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(
            profile.display().to_string(),
        )
        .as_error())
    }
}

/// Remove a project from a profile
#[tracing::instrument]
pub async fn remove_project(
    profile: &Path,
    project: &Path,
) -> crate::Result<()> {
    if let Some(profile) = get(profile, None).await? {
        profile.remove_project(project, None).await?;

        Ok(())
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(
            profile.display().to_string(),
        )
        .as_error())
    }
}

/// Run Minecraft using a profile and the default credentials, logged in credentials,
/// failing with an error if no credentials are available
#[tracing::instrument]
pub async fn run(path: &Path) -> crate::Result<Arc<RwLock<MinecraftChild>>> {
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
    path: &Path,
    credentials: &auth::Credentials,
) -> crate::Result<Arc<RwLock<MinecraftChild>>> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    let profile = get(path, None).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to run a nonexistent or unloaded profile at path {}!",
            path.display()
        ))
    })?;

    let pre_launch_hooks =
        &profile.hooks.as_ref().unwrap_or(&settings.hooks).pre_launch;
    if let Some(hook) = pre_launch_hooks {
        // TODO: hook parameters
        let mut cmd = hook.split(' ');
        if let Some(command) = cmd.next() {
            let result = Command::new(command)
                .args(&cmd.collect::<Vec<&str>>())
                .current_dir(path)
                .spawn()?
                .wait()
                .await?;

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
            command.args(&cmd.collect::<Vec<&str>>()).current_dir(path);
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
