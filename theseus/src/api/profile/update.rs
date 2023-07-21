use std::{path::{Path, PathBuf}, collections::HashMap, io::Cursor};

use reqwest::Method;

use crate::{State, profile::{get, edit}, prelude::{ModrinthVersion, ProjectMetadata}, util::{fetch::fetch_json, io}, config::MODRINTH_API_URL, event::{emit::{loading_try_for_each_concurrent, emit_profile}, ProfilePayloadType}, pack::{install_from::{generate_pack_from_version_id, CreatePackDescription, PackFormat, PackFile}, self}, state::ProfileInstallStage};
use async_zip::tokio::read::seek::ZipFileReader;


/// Updates a managed modrinth pack to the cached latest version found in 'modrinth_update_version'
#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn update_managed_modrinth(
    profile_path: &Path,
) -> crate::Result<()> {

    let profile = get(&profile_path, None).await?.ok_or_else    (||        crate::ErrorKind::UnmanagedProfileError(
        profile_path.display().to_string(),
        )
        .as_error())?;

    let unmanaged_err = || crate::ErrorKind::InputError(
            format!("Profile at {} is not a managed modrinth pack, or has been disconnected.", profile_path.display()),
        );

    // Extract modrinth pack information, if appropriate
    let linked_data = profile.metadata.linked_data.as_ref().ok_or_else(unmanaged_err)?;
    let project_id: &String = linked_data.project_id.as_ref().ok_or_else(unmanaged_err)?;
    let version_id = linked_data.version_id.as_ref().ok_or_else(unmanaged_err)?;

    // extract modrinth_update_version, returning Ok(()) if it is none
    let modrinth_update_version = match profile.modrinth_update_version {
        Some(ref x) if x != version_id =>  x,
        _ => return Ok(()), // No update version, or no update needed, return Ok(())
    };

    // Replace the pack with the new version
    replace_managed_modrinth(profile_path, &profile, project_id, version_id, modrinth_update_version).await?;

    emit_profile(
        profile.uuid,
        profile.path,
        &profile.metadata.name,
        ProfilePayloadType::Edited,
    ).await?;

    State::sync().await?;
    Ok(())
}

/// Repair a managed modrinth pack by 'updating' it to the current version
#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn repair_managed_modrinth(
    profile_path: &Path,
) -> crate::Result<()> {

    let profile = get(&profile_path, None).await?.ok_or_else    (||        crate::ErrorKind::UnmanagedProfileError(
        profile_path.display().to_string(),
        )
        .as_error())?;

    let unmanaged_err = || crate::ErrorKind::InputError(
            format!("Profile at {} is not a managed modrinth pack, or has been disconnected.", profile_path.display()),
        );

    // Extract modrinth pack information, if appropriate
    let linked_data = profile.metadata.linked_data.as_ref().ok_or_else(unmanaged_err)?;
    let project_id: &String = linked_data.project_id.as_ref().ok_or_else(unmanaged_err)?;
    let version_id = linked_data.version_id.as_ref().ok_or_else(unmanaged_err)?;

    // Replace the pack with the same version
    replace_managed_modrinth(profile_path, &profile, project_id, version_id, version_id).await?;

    emit_profile(
        profile.uuid,
        profile.path,
        &profile.metadata.name,
        ProfilePayloadType::Edited,
    ).await?;

    State::sync().await?;
    Ok(())
}

/// Replace a managed modrinth pack with a new version
#[tracing::instrument(skip(profile))]
#[theseus_macros::debug_pin]
async fn replace_managed_modrinth(
    profile_path: &Path,
    profile : &crate::state::Profile,
    project_id : &String,
    version_id : &String,
    new_version_id : &String,
) -> crate::Result<()> {
    // Fetch .mrpacks for both old and new versions
    // TODO: this will need to be updated if we revert the hacky pack method we needed for compiler speed
    let old_pack_creator = generate_pack_from_version_id(
        project_id.clone(),
        version_id.clone(),
        profile.metadata.name.clone(),
        None,
        profile_path.to_path_buf()
    ).await?;

    let new_pack_creator = generate_pack_from_version_id(
        project_id.clone(),
        new_version_id.clone(),
        profile.metadata.name.clone(),
        None,
        profile_path.to_path_buf()
    ).await?;

    // Removal - remove all files that were added by the old pack
    // - remove all installed projects
    // - remove all overrides
    pack::install::remove_all_related_files(profile_path.to_path_buf(), old_pack_creator.file).await?;

    // Reinstallation - install all files that are added by the new pack
    // - install all projects
    // - install all overrides
    // - edits the profile to update the new data
    // - (functionals almost identically to rteinstalling the pack 'in-place')
    pack::install::install_pack_files(new_pack_creator).await?;

    Ok(())
}

