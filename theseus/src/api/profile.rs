//! Theseus profile management interface
use crate::state::MinecraftChild;
pub use crate::{
    state::{JavaSettings, Profile},
    State,
};
use daedalus as d;
use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{process::Command, sync::RwLock};

/// Add a profile to the in-memory state
#[tracing::instrument]
pub async fn add(profile: Profile) -> crate::Result<()> {
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;
    profiles.insert(profile)?;

    Ok(())
}

/// Add a path as a profile in-memory
#[tracing::instrument]
pub async fn add_path(path: &Path) -> crate::Result<()> {
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;
    profiles.insert_from(path).await?;

    Ok(())
}

/// Remove a profile
#[tracing::instrument]
pub async fn remove(path: &Path) -> crate::Result<()> {
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;
    profiles.remove(path)?;

    Ok(())
}

/// Get a profile by path,
#[tracing::instrument]
pub async fn get(path: &Path) -> crate::Result<Option<Profile>> {
    let state = State::get().await?;
    let profiles = state.profiles.read().await;

    profiles.0.get(path).map_or(Ok(None), |prof| match prof {
        Some(prof) => Ok(Some(prof.clone())),
        None => Err(crate::ErrorKind::UnloadedProfileError(
            path.display().to_string(),
        )
        .as_error()),
    })
}

/// Check if a profile is already managed by Theseus
#[tracing::instrument]
pub async fn is_managed(profile: &Path) -> crate::Result<bool> {
    let state = State::get().await?;
    let profiles = state.profiles.read().await;
    Ok(profiles.0.contains_key(profile))
}

/// Check if a profile is loaded
#[tracing::instrument]
pub async fn is_loaded(profile: &Path) -> crate::Result<bool> {
    let state = State::get().await?;
    let profiles = state.profiles.read().await;
    Ok(profiles.0.get(profile).and_then(Option::as_ref).is_some())
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
        Some(&mut Some(ref mut profile)) => action(profile).await,
        Some(&mut None) => Err(crate::ErrorKind::UnloadedProfileError(
            path.display().to_string(),
        )
        .as_error()),
        None => Err(crate::ErrorKind::UnmanagedProfileError(
            path.display().to_string(),
        )
        .as_error()),
    }
}

/// Get a copy of the profile set
#[tracing::instrument]
pub async fn list(
) -> crate::Result<std::collections::HashMap<PathBuf, Option<Profile>>> {
    let state = State::get().await?;
    let profiles = state.profiles.read().await;
    Ok(profiles.0.clone())
}

/// Run Minecraft using a profile
/// Returns Arc pointer to RwLock to Child
#[tracing::instrument(skip_all)]
pub async fn run(
    path: &Path,
    credentials: &crate::auth::Credentials,
) -> crate::Result<Arc<RwLock<MinecraftChild>>> {
    let state = State::get().await.unwrap();
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
        .find(|it| it.id == profile.metadata.game_version.as_ref())
        .ok_or_else(|| {
            crate::ErrorKind::LauncherError(format!(
                "Invalid or unknown Minecraft version: {}",
                profile.metadata.game_version
            ))
        })?;
    let version_info = d::minecraft::fetch_version_info(version).await?;

    let pre_launch_hooks =
        &profile.hooks.as_ref().unwrap_or(&settings.hooks).pre_launch;
    for hook in pre_launch_hooks.iter() {
        // TODO: hook parameters
        let mut cmd = hook.split(' ');
        let result = Command::new(cmd.next().unwrap())
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

    dbg!("Profile java settings: {:?}", &profile.java);
    let java_version = match profile.java {
        // Load profile-specific Java implementation choice
        // (This defaults to Daedalus-decided key on init, but can be changed by the user)
        Some(JavaSettings {
            jre_key: Some(ref jre_key),
            ..
        }) => settings.java_globals.get(jre_key),
        // Fall back to Daedalus-decided key if no profile-specific key is set
        _ => if version_info
            .java_version
            .as_ref()
            .filter(|it| it.major_version >= 17)
            .is_some()
        {
            settings.java_globals.get(&crate::jre::JAVA_17PLUS_KEY.to_string())
        } else {
            settings.java_globals.get(&crate::jre::JAVA_8_KEY.to_string())
        }
    };
    dbg!("Intermediate java version: {:?}", &java_version);
    let java_version = java_version.as_ref().ok_or_else(|| {
        crate::ErrorKind::LauncherError(format!(
            "No Java stored for version {}",
            version_info.java_version.map_or(8, |it| it.major_version),
        ))
    })?;

    // Get the path to the Java executable from the chosen Java implementation key
    let java_install: &Path = &PathBuf::from(&java_version.path);
    dbg!("Java install: {}", &java_install.display());
    if !java_install.exists() {
        return Err(crate::ErrorKind::LauncherError(format!(
            "Could not find Java install: {}",
            java_install.display()
        ))
        .as_error());
    }
    dbg!("Sccess. java!");
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

    let mc_process = crate::launcher::launch_minecraft(
        &profile.metadata.game_version,
        &profile.metadata.loader_version,
        &profile.path,
        &java_install,
        java_args,
        env_args,
        wrapper,
        &memory,
        &resolution,
        credentials,
    )
    .await?;

    // Insert child into state
    let mut state_children = state.children.write().await;
    let pid = mc_process.id().ok_or_else(|| {
        crate::ErrorKind::LauncherError(
            "Process failed to stay open.".to_string(),
        )
    })?;
    let mchild_arc =
        state_children.insert_process(pid, path.to_path_buf(), mc_process);

    Ok(mchild_arc)
}
