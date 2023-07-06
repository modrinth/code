//! Theseus profile management interface
use crate::state::{LinkedData, ProfilePathId};
use crate::{
    event::{emit::emit_profile, ProfilePayloadType},
    prelude::ModLoader,
};
pub use crate::{
    state::{JavaSettings, Profile},
    State,
};
use daedalus::modded::LoaderVersion;
use dunce::canonicalize;
use futures::prelude::*;

use std::path::PathBuf;
use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;
use tracing::{info, trace};
use uuid::Uuid;

// Creates a profile of a given name and adds it to the in-memory state
// Returns relative filepath as ProfilePathId which can be used to access it in the State
#[tracing::instrument]
#[theseus_macros::debug_pin]
#[allow(clippy::too_many_arguments)]
pub async fn profile_create(
    name: String,         // the name of the profile, and relative path
    game_version: String, // the game version of the profile
    modloader: ModLoader, // the modloader to use
    loader_version: Option<String>, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader. defaults to latest
    icon: Option<PathBuf>,          // the icon for the profile
    icon_url: Option<String>, // the URL icon for a profile (ONLY USED FOR TEMPORARY PROFILES)
    linked_data: Option<LinkedData>, // the linked project ID (mainly for modpacks)- used for updating
    skip_install_profile: Option<bool>,
) -> crate::Result<ProfilePathId> {
    trace!("Creating new profile. {}", name);
    let state = State::get().await?;
    let uuid = Uuid::new_v4();
    let path = state.directories.profiles_dir().await.join(&name);
    if path.exists() {
        if !path.is_dir() {
            return Err(ProfileCreationError::NotFolder.into());
        }
        if path.join("profile.json").exists() {
            return Err(ProfileCreationError::ProfileExistsError(
                path.join("profile.json"),
            )
            .into());
        }

        if ReadDirStream::new(fs::read_dir(&path).await?)
            .next()
            .await
            .is_some()
        {
            return Err(ProfileCreationError::NotEmptyFolder.into());
        }
    } else {
        fs::create_dir_all(&path).await?;
    }

    info!(
        "Creating profile at path {}",
        &canonicalize(&path)?.display()
    );
    let loader = if modloader != ModLoader::Vanilla {
        get_loader_version_from_loader(
            game_version.clone(),
            modloader,
            loader_version,
        )
        .await?
    } else {
        None
    };

    let mut profile = Profile::new(uuid, name, game_version).await?;
    let result = async {
        if let Some(ref icon) = icon {
            let bytes =
                tokio::fs::read(state.directories.caches_dir().join(icon))
                    .await?;
            profile
                .set_icon(
                    &state.directories.caches_dir(),
                    &state.io_semaphore,
                    bytes::Bytes::from(bytes),
                    &icon.to_string_lossy(),
                )
                .await?;
        }

        profile.metadata.icon_url = icon_url;
        if let Some(loader_version) = loader {
            profile.metadata.loader = modloader;
            profile.metadata.loader_version = Some(loader_version);
        }

        profile.metadata.linked_data = linked_data;

        emit_profile(
            uuid,
            profile.get_profile_full_path().await?,
            &profile.metadata.name,
            ProfilePayloadType::Created,
        )
        .await?;

        {
            let mut profiles = state.profiles.write().await;
            profiles.insert(profile.clone()).await?;
        }

        if !skip_install_profile.unwrap_or(false) {
            crate::launcher::install_minecraft(&profile, None).await?;
        }
        State::sync().await?;

        Ok(profile.name_as_path_id())
    }
    .await;

    match result {
        Ok(profile) => Ok(profile),
        Err(err) => {
            let _ =
                crate::api::profile::remove(&profile.name_as_path_id()).await;

            Err(err)
        }
    }
}

#[tracing::instrument]
#[theseus_macros::debug_pin]
pub(crate) async fn get_loader_version_from_loader(
    game_version: String,
    loader: ModLoader,
    loader_version: Option<String>,
) -> crate::Result<Option<LoaderVersion>> {
    let state = State::get().await?;
    let metadata = state.metadata.read().await;

    let version = loader_version.unwrap_or_else(|| "latest".to_string());

    let filter = |it: &LoaderVersion| match version.as_str() {
        "latest" => true,
        "stable" => it.stable,
        id => {
            it.id == *id
                || format!("{}-{}", game_version, id) == it.id
                || format!("{}-{}-{}", game_version, id, game_version) == it.id
        }
    };

    let loader_data = match loader {
        ModLoader::Forge => &metadata.forge,
        ModLoader::Fabric => &metadata.fabric,
        ModLoader::Quilt => &metadata.quilt,
        _ => {
            return Err(
                ProfileCreationError::NoManifest(loader.to_string()).into()
            )
        }
    };

    let loaders = &loader_data
        .game_versions
        .iter()
        .find(|it| {
            it.id
                .replace(daedalus::modded::DUMMY_REPLACE_STRING, &game_version)
                == game_version
        })
        .ok_or_else(|| {
            ProfileCreationError::ModloaderUnsupported(
                loader.to_string(),
                game_version.clone(),
            )
        })?
        .loaders;

    let loader_version = loaders
        .iter()
        .cloned()
        .find(filter)
        .or(
            // If stable was searched for but not found, return latest by default
            if version == "stable" {
                loaders.iter().next().cloned()
            } else {
                None
            },
        )
        .ok_or_else(|| {
            ProfileCreationError::InvalidVersionModloader(
                version,
                loader.to_string(),
            )
        })?;

    Ok(Some(loader_version))
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
