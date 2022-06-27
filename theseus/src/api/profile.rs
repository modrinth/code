//! Theseus profile management interface

pub use crate::{
    state::{JavaSettings, Profile},
    State,
};
use daedalus as d;
use std::{future::Future, path::Path};
use tokio::process::{Child, Command};

/// Add a profile to the in-memory state
pub async fn add(profile: Profile) -> crate::Result<()> {
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;
    profiles.insert(profile)?;

    Ok(())
}

/// Add a path as a profile in-memory
pub async fn add_path(path: &Path) -> crate::Result<()> {
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;
    profiles.insert_from(path).await?;

    Ok(())
}

/// Remove a profile
pub async fn remove(path: &Path) -> crate::Result<()> {
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;
    profiles.remove(path)?;

    Ok(())
}

/// Get a profile by path,
pub async fn get(path: &Path) -> crate::Result<Option<Profile>> {
    let state = State::get().await?;
    let profiles = state.profiles.read().await;

    profiles.0.get(path).map_or(Ok(None), |prof| match prof {
        Some(prof) => Ok(Some(prof.clone())),
        None => Err(crate::Error::UnloadedProfileError(
            path.display().to_string(),
        )),
    })
}

/// Check if a profile is already managed by Theseus
pub async fn is_managed(profile: &Path) -> crate::Result<bool> {
    let state = State::get().await?;
    let profiles = state.profiles.read().await;
    Ok(profiles.0.contains_key(profile))
}

/// Check if a profile is loaded
pub async fn is_loaded(profile: &Path) -> crate::Result<bool> {
    let state = State::get().await?;
    let profiles = state.profiles.read().await;
    Ok(profiles
        .0
        .get(profile)
        .map(Option::as_ref)
        .flatten()
        .is_some())
}

/// Edit a profile using a given asynchronous closure
pub async fn edit<Fut>(
    path: &Path,
    action: impl Fn(&mut Profile) -> Fut,
) -> crate::Result<()>
where
    Fut: Future<Output = crate::Result<()>>,
{
    let state = State::get().await.unwrap();
    let mut profiles = state.profiles.write().await;

    match profiles.0.get_mut(path) {
        Some(&mut Some(ref mut profile)) => action(profile).await,
        Some(&mut None) => Err(crate::Error::UnloadedProfileError(
            path.display().to_string(),
        )),
        None => Err(crate::Error::UnmanagedProfileError(
            path.display().to_string(),
        )),
    }
}

/// Run Minecraft using a profile
pub async fn run(
    path: &Path,
    credentials: &crate::launcher::Credentials,
) -> crate::Result<Child> {
    let state = State::get().await.unwrap();
    let settings = state.settings.read().await;
    let profile = get(path).await?.ok_or_else(|| {
        crate::Error::OtherError(format!(
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
            crate::Error::LauncherError(format!(
                "Invalid or unknown Minecraft version: {}",
                profile.metadata.game_version
            ))
        })?;
    let version_info = d::minecraft::fetch_version_info(version).await?;

    let ref pre_launch_hooks =
        profile.hooks.as_ref().unwrap_or(&settings.hooks).pre_launch;
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
            return Err(crate::Error::LauncherError(format!(
                "Non-zero exit code for pre-launch hook: {}",
                result.code().unwrap_or(-1)
            )));
        }
    }

    let java_install = match profile.java {
        Some(JavaSettings {
            install: Some(ref install),
            ..
        }) => install,
        _ => if version_info
            .java_version
            .as_ref()
            .filter(|it| it.major_version >= 16)
            .is_some()
        {
            settings.java_17_path.as_ref()
        } else {
            settings.java_8_path.as_ref()
        }
        .ok_or_else(|| {
            crate::Error::LauncherError(format!(
                "No Java installed for version {}",
                version_info.java_version.map_or(8, |it| it.major_version),
            ))
        })?,
    };

    if !java_install.exists() {
        return Err(crate::Error::LauncherError(format!(
            "Could not find Java install: {}",
            java_install.display()
        )));
    }

    let ref java_args = profile
        .java
        .as_ref()
        .and_then(|it| it.extra_arguments.as_ref())
        .unwrap_or(&settings.custom_java_args);

    let wrapper = profile
        .hooks
        .as_ref()
        .map_or(&settings.hooks.wrapper, |it| &it.wrapper);

    let ref memory = profile.memory.unwrap_or(settings.memory);
    let ref resolution = profile.resolution.unwrap_or(settings.game_resolution);

    crate::launcher::launch_minecraft(
        &profile.metadata.game_version,
        &profile.metadata.loader_version,
        &profile.path,
        &java_install,
        &java_args,
        &wrapper,
        memory,
        resolution,
        credentials,
    )
    .await
}

pub async fn kill(running: &mut Child) -> crate::Result<()> {
    running.kill().await?;
    wait_for(running).await
}

pub async fn wait_for(running: &mut Child) -> crate::Result<()> {
    let result = running.wait().await.map_err(|err| {
        crate::Error::LauncherError(format!("Error running minecraft: {err}"))
    })?;

    match result.success() {
        false => Err(crate::Error::LauncherError(format!(
            "Minecraft exited with non-zero code {}",
            result.code().unwrap_or(-1)
        ))),
        true => Ok(()),
    }
}
