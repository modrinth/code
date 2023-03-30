//! Theseus profile management interface
use crate::{prelude::ModLoader, profile};
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

const DEFAULT_NAME: &'static str = "Untitled Instance";

// Generic basic profile creation tool.
// Creates an essentially empty dummy profile with profile_create
#[tracing::instrument]
pub async fn profile_create_empty() -> crate::Result<Profile> {
    Ok(profile_create(
        String::from(DEFAULT_NAME), // the name/path of the profile
        String::from("1.8.2"),      // the game version of the profile
        ModLoader::Vanilla,         // the modloader to use
        String::from("stable"), // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader
        None,                   // the icon for the profile
    )
    .await?)
}

// Creates a profile at  the given filepath and adds it to the in-memory state
// This is reused mostly from the CLI. TODO: touch up.
// invoke('profile_add',profile)
#[tracing::instrument]
pub async fn profile_create(
    name: String,           // the name of the profile, and relative path
    game_version: String,   // the game version of the profile
    modloader: ModLoader,   // the modloader to use
    loader_version: String, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader
    icon: Option<PathBuf>,  // the icon for the profile
) -> crate::Result<Profile> {
    let state = State::get().await?;

    let path = state.directories.profiles_dir().join(&name);

    if path.exists() {
        if !path.is_dir() {
            return Err(crate::ErrorKind::ProfileCreationError(format!(
                "Attempted to create profile in something other than a folder"
            ))
            .as_error());
        }
        if path.join("profile.json").exists() {
            return Err(crate::ErrorKind::ProfileCreationError(format!(
                "Profile already exists! Perhaps you want `profile add` instead"
            ))
            .as_error());
        }

        if ReadDirStream::new(fs::read_dir(&path).await?)
            .next()
            .await
            .is_some()
        {
            return Err(crate::ErrorKind::ProfileCreationError(format!(
                "You are trying to create a profile in a non-empty directory"
            ))
            .as_error());
        }
    } else {
        fs::create_dir_all(&path).await?;
    }
    println!(
        "Creating profile at path {}",
        &canonicalize(&path)?.display()
    );

    let loader = modloader;
    let loader = if loader != ModLoader::Vanilla {
        let version = loader_version;

        let filter = |it: &LoaderVersion| match version.as_str() {
            "latest" => true,
            "stable" => it.stable,
            id => it.id == String::from(id),
        };

        let loader_data = match loader {
            ModLoader::Forge => &state.metadata.forge,
            ModLoader::Fabric => &state.metadata.fabric,
            _ => return Err(crate::ErrorKind::ProfileCreationError(format!(
                "Could not get manifest for loader {loader}. This is a bug in the GUI"
            )).as_error())
        };

        let ref loaders = loader_data.game_versions
            .iter()
            .find(|it| it.id == game_version)
            .ok_or_else(|| crate::ErrorKind::ProfileCreationError(format!(
                "Modloader {loader} unsupported for Minecraft version {game_version}!"
            )).as_error())?
            .loaders;

        let loader_version =
            loaders.iter().cloned().find(filter).ok_or_else(|| {
                crate::ErrorKind::ProfileCreationError(format!(
                    "Invalid version {version} for modloader {loader}"
                ))
                .as_error()
            })?;

        Some((loader_version, loader))
    } else {
        None
    };
    let mut profile = Profile::new(name, game_version, path.clone()).await?;
    if let Some(ref icon) = icon {
        profile.with_icon(icon).await?;
    }
    if let Some((loader_version, loader)) = loader {
        profile.with_loader(loader, Some(loader_version));
    }
    profile::add(profile.clone()).await?;
    State::sync().await?;

    Ok(profile)
}
