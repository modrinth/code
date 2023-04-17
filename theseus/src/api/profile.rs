//! Theseus profile management interface
use crate::{
    auth::{self, refresh},
    event::{emit::emit_profile, ProfilePayloadType},
    launcher::download,
    state::MinecraftChild,
};
pub use crate::{
    state::{JavaSettings, Profile},
    State,
};
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

    if let Some(profile) = profiles.0.get(path) {
        emit_profile(
            profile.uuid,
            profile.path.clone(),
            &profile.metadata.name,
            ProfilePayloadType::Removed,
        )
        .await?;
    }

    profiles.remove(path).await?;

    Ok(())
}

/// Get a profile by path,
#[tracing::instrument]
pub async fn get(path: &Path) -> crate::Result<Option<Profile>> {
    let state = State::get().await?;
    let profiles = state.profiles.read().await;

    Ok(profiles.0.get(path).cloned())
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

/// Get a copy of the profile set
#[tracing::instrument]
pub async fn list() -> crate::Result<std::collections::HashMap<PathBuf, Profile>>
{
    let state = State::get().await?;
    let profiles = state.profiles.read().await;
    Ok(profiles.0.clone())
}

/// Query + sync profile's projects with the UI from the FS
#[tracing::instrument]
pub async fn sync(path: &Path) -> crate::Result<()> {
    let state = State::get().await?;
    let result ={
    let mut profiles: tokio::sync::RwLockWriteGuard<crate::state::Profiles> = state.profiles.write().await;

    if let Some(profile) = profiles.0.get_mut(path) {
        profile.sync().await?;
        Ok(())
    } else {
        Err(
            crate::ErrorKind::UnmanagedProfileError(path.display().to_string())
                .as_error(),
        )
    }
    };
    State::sync().await?;
    result
}

/// Add a project from a version
#[tracing::instrument]
pub async fn add_project_from_version(
    profile: &Path,
    version_id: String,
) -> crate::Result<PathBuf> {
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;

    if let Some(profile) = profiles.0.get_mut(profile) {
        profile.add_project_version(version_id).await
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(
            profile.display().to_string(),
        )
        .as_error())
    }
}

/// Add a project from an FS path
#[tracing::instrument]
pub async fn add_project_from_path(
    profile: &Path,
    path: &Path,
    project_type: Option<String>,
) -> crate::Result<PathBuf> {
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;

    if let Some(profile) = profiles.0.get_mut(profile) {
        let file = fs::read(path).await?;
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        profile
            .add_project_bytes(
                &file_name,
                bytes::Bytes::from(file),
                project_type.and_then(|x| serde_json::from_str(&x).ok()),
            )
            .await
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(
            profile.display().to_string(),
        )
        .as_error())
    }
}

/// Toggle whether a project is disabled or not
#[tracing::instrument]
pub async fn toggle_disable_project(
    profile: &Path,
    project: &Path,
) -> crate::Result<()> {
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;

    if let Some(profile) = profiles.0.get_mut(profile) {
        profile.toggle_disable_project(project).await?;

        Ok(())
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
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;

    if let Some(profile) = profiles.0.get_mut(profile) {
        profile.remove_project(project).await?;

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
#[tracing::instrument(skip_all)]
pub async fn run(path: &Path) -> crate::Result<Arc<RwLock<MinecraftChild>>> {
    let state = State::get().await?;

    // Get default account and refresh credentials (preferred way to log in)
    let default_account = state.settings.read().await.default_user;
    let credentials = if let Some(default_account) = default_account {
        refresh(default_account).await?
    } else {
        // If no default account, try to use a logged in account
        let users = auth::users().await?;
        let last_account = users.iter().next();
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
#[tracing::instrument(skip_all)]
pub async fn run_credentials(
    path: &Path,
    credentials: &auth::Credentials,
) -> crate::Result<Arc<RwLock<MinecraftChild>>> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    let profile = get(path).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to run a nonexistent or unloaded profile at path {}!",
            path.display()
        ))
    })?;

    let version = state
        .metadata
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
    let version_info = download::download_version_info(
        &state,
        version,
        profile.metadata.loader_version.as_ref(),
    )
    .await?;
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

    let java_version = match profile.java {
        // Load profile-specific Java implementation choice
        // (This defaults to Daedalus-decided key on init, but can be changed by the user)
        Some(JavaSettings {
            jre_key: Some(ref jre_key),
            ..
        }) => settings.java_globals.get(jre_key),
        // Fall back to Daedalus-decided key if no profile-specific key is set
        _ => {
            match version_info
                .java_version
                .as_ref()
                .map(|it| it.major_version)
                .unwrap_or(0)
            {
                0..=16 => settings
                    .java_globals
                    .get(&crate::jre::JAVA_8_KEY.to_string()),
                17 => settings
                    .java_globals
                    .get(&crate::jre::JAVA_17_KEY.to_string()),
                _ => settings
                    .java_globals
                    .get(&crate::jre::JAVA_18PLUS_KEY.to_string()),
            }
        }
    };
    let java_version = java_version.as_ref().ok_or_else(|| {
        crate::ErrorKind::LauncherError(format!(
            "No Java stored for version {}",
            version_info.java_version.map_or(8, |it| it.major_version),
        ))
    })?;

    // Get the path to the Java executable from the chosen Java implementation key
    let java_install: &Path = &PathBuf::from(&java_version.path);
    if !java_install.exists() {
        return Err(crate::ErrorKind::LauncherError(format!(
            "Could not find Java install: {}",
            java_install.display()
        ))
        .as_error());
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

    let env_args = &settings.custom_env_args;

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
        &profile.metadata.game_version,
        &profile.metadata.loader_version,
        &profile.path,
        java_install,
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
