//! Theseus profile management interface
use crate::launcher::get_loader_version_from_profile;
use crate::settings::Hooks;
use crate::state::{LauncherFeatureVersion, LinkedData, ProfileInstallStage};
use crate::util::io::{self, canonicalize};
use crate::{ErrorKind, pack, profile};
pub use crate::{State, state::Profile};
use crate::{
    event::{ProfilePayloadType, emit::emit_profile},
    prelude::ModLoader,
};
use chrono::Utc;
use std::path::PathBuf;
use tracing::{info, trace};

// Creates a profile of a given name and adds it to the in-memory state
// Returns relative filepath as ProfilePathId which can be used to access it in the State
#[tracing::instrument]
#[allow(clippy::too_many_arguments)]
pub async fn profile_create(
    name: String,         // the name of the profile, and relative path
    game_version: String, // the game version of the profile
    modloader: ModLoader, // the modloader to use
    loader_version: Option<String>, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader. defaults to latest
    icon_path: Option<String>,      // the icon for the profile
    linked_data: Option<LinkedData>, // the linked project ID (mainly for modpacks)- used for updating
    skip_install_profile: Option<bool>,
) -> crate::Result<String> {
    trace!("Creating new profile. {}", name);
    let state = State::get().await?;

    let mut path = profile::sanitize_profile_name(&name);
    let mut full_path = state.directories.profiles_dir().join(&path);
    if full_path.exists() {
        let mut new_path;
        let mut new_full_path;
        let mut which = 1;

        loop {
            new_path = format!("{path} ({which})");
            new_full_path = state.directories.profiles_dir().join(&new_path);
            if !new_full_path.exists() {
                break;
            }
            which += 1;
        }

        tracing::debug!(
            "Folder collision: {}, renaming to: {}",
            full_path.display(),
            new_full_path.display()
        );

        path = new_path;
        full_path = new_full_path;
    }
    io::create_dir_all(&full_path).await?;

    info!(
        "Creating profile at path {}",
        &canonicalize(&full_path)?.display()
    );
    let loader = if modloader != ModLoader::Vanilla {
        get_loader_version_from_profile(
            &game_version,
            modloader,
            loader_version.as_deref(),
        )
        .await?
    } else {
        None
    };

    let mut profile = Profile {
        path: path.clone(),
        install_stage: ProfileInstallStage::NotInstalled,
        launcher_feature_version: LauncherFeatureVersion::MOST_RECENT,
        name,
        icon_path: None,
        game_version,
        protocol_version: None,
        loader: modloader,
        loader_version: loader.map(|x| x.id),
        groups: Vec::new(),
        linked_data,
        created: Utc::now(),
        modified: Utc::now(),
        last_played: None,
        submitted_time_played: 0,
        recent_time_played: 0,
        java_path: None,
        extra_launch_args: None,
        custom_env_vars: None,
        memory: None,
        force_fullscreen: None,
        game_resolution: None,
        hooks: Hooks {
            pre_launch: None,
            wrapper: None,
            post_exit: None,
        },
    };

    let result = async {
        if let Some(ref icon) = icon_path {
            let bytes =
                io::read(state.directories.caches_dir().join(icon)).await?;
            profile
                .set_icon(
                    &state.directories.caches_dir(),
                    &state.io_semaphore,
                    bytes::Bytes::from(bytes),
                    icon,
                )
                .await?;
        }

        crate::state::fs_watcher::watch_profile(
            &profile.path,
            &state.file_watcher,
            &state.directories,
        )
        .await;

        profile.upsert(&state.pool).await?;

        emit_profile(&profile.path, ProfilePayloadType::Created).await?;

        if !skip_install_profile.unwrap_or(false) {
            crate::launcher::install_minecraft(&profile, None, false).await?;
        }

        Ok(profile.path)
    }
    .await;

    match result {
        Ok(profile) => Ok(profile),
        Err(err) => {
            let _ = profile::remove(&path).await;

            Err(err)
        }
    }
}

pub async fn profile_create_from_duplicate(
    copy_from: &str,
) -> crate::Result<String> {
    // Original profile
    let profile = profile::get(copy_from).await?.ok_or_else(|| {
        ErrorKind::UnmanagedProfileError(copy_from.to_string())
    })?;

    let profile_path_id = profile_create(
        profile.name.clone(),
        profile.game_version.clone(),
        profile.loader,
        profile.loader_version.clone(),
        profile.icon_path.clone(),
        profile.linked_data.clone(),
        Some(true),
    )
    .await?;

    // Copy it over using the import system (essentially importing from the same profile)
    let state = State::get().await?;
    let bar = pack::import::copy_dotminecraft(
        &profile_path_id,
        profile::get_full_path(copy_from).await?,
        &state.io_semaphore,
        None,
    )
    .await?;

    let duplicated_profile =
        profile::get(&profile_path_id).await?.ok_or_else(|| {
            ErrorKind::UnmanagedProfileError(profile_path_id.to_string())
        })?;

    crate::launcher::install_minecraft(&duplicated_profile, Some(bar), false)
        .await?;

    // emit profile edited
    emit_profile(&profile.path, ProfilePayloadType::Edited).await?;
    Ok(profile_path_id)
}

#[derive(thiserror::Error, Debug)]
pub enum ProfileCreationError {
    #[error("Profile .json exists: {0}")]
    ProfileExistsError(PathBuf),
    #[error("Modloader {0} unsupported for Minecraft version {1}")]
    ModloaderUnsupported(String, String),
    #[error("Invalid version {0} for modloader {1}")]
    InvalidVersionModloader(String, String),
    #[error("Could not get manifest for loader {0}. This is a bug in the GUI")]
    NoManifest(String),
    #[error("Could not get State.")]
    NoState,

    #[error("Attempted to create project in something other than a folder.")]
    NotFolder,
    #[error("You are trying to create a profile in a non-empty directory")]
    NotEmptyFolder,

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}
