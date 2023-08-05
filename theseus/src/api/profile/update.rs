use crate::{
    event::{
        emit::{emit_profile, loading_try_for_each_concurrent},
        ProfilePayloadType,
    },
    pack::{self, install_from::generate_pack_from_version_id},
    prelude::{ProfilePathId, ProjectPathId},
    profile::get,
    state::Project,
    State,
};
use futures::try_join;

/// Updates a managed modrinth pack to the cached latest version found in 'modrinth_update_version'
#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn update_managed_modrinth(
    profile_path: &ProfilePathId,
) -> crate::Result<()> {
    let profile = get(profile_path, None).await?.ok_or_else(|| {
        crate::ErrorKind::UnmanagedProfileError(profile_path.to_string())
            .as_error()
    })?;

    let unmanaged_err = || {
        crate::ErrorKind::InputError(
            format!("Profile at {} is not a managed modrinth pack, or has been disconnected.", profile_path),
        )
    };

    // Extract modrinth pack information, if appropriate
    let linked_data = profile
        .metadata
        .linked_data
        .as_ref()
        .ok_or_else(unmanaged_err)?;
    let project_id: &String =
        linked_data.project_id.as_ref().ok_or_else(unmanaged_err)?;
    let version_id =
        linked_data.version_id.as_ref().ok_or_else(unmanaged_err)?;

    // extract modrinth_update_version, returning Ok(()) if it is none
    let modrinth_update_version = match profile.modrinth_update_version {
        Some(ref x) if x != version_id => x,
        _ => return Ok(()), // No update version, or no update needed, return Ok(())
    };

    // Replace the pack with the new version
    replace_managed_modrinth(
        profile_path,
        &profile,
        project_id,
        version_id,
        Some(modrinth_update_version),
    )
    .await?;

    emit_profile(
        profile.uuid,
        profile_path,
        &profile.metadata.name,
        ProfilePayloadType::Edited,
    )
    .await?;

    State::sync().await?;
    Ok(())
}

/// Repair a managed modrinth pack by 'updating' it to the current version
#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn repair_managed_modrinth(
    profile_path: &ProfilePathId,
) -> crate::Result<()> {
    let profile = get(profile_path, None).await?.ok_or_else(|| {
        crate::ErrorKind::UnmanagedProfileError(profile_path.to_string())
            .as_error()
    })?;

    let unmanaged_err = || {
        crate::ErrorKind::InputError(
            format!("Profile at {} is not a managed modrinth pack, or has been disconnected.", profile_path),
        )
    };

    // For repairing specifically, first we remove all installed projects (to ensure we do remove ones that aren't in the pack)
    // We do a project removal followed by removing everything in the .mrpack, to ensure we only
    // remove relevant projects and not things like save files
    let projects_map = profile.projects.clone();
    let stream = futures::stream::iter(
        projects_map
            .into_iter()
            .map(Ok::<(ProjectPathId, Project), crate::Error>),
    );
    loading_try_for_each_concurrent(
        stream,
        None,
        None,
        0.0,
        0,
        None,
        |(project_id, _)| {
            let profile = profile.clone();
            async move {
                profile.remove_project(&project_id, Some(true)).await?;
                Ok(())
            }
        },
    )
    .await?;

    // Extract modrinth pack information, if appropriate
    let linked_data = profile
        .metadata
        .linked_data
        .as_ref()
        .ok_or_else(unmanaged_err)?;
    let project_id: &String =
        linked_data.project_id.as_ref().ok_or_else(unmanaged_err)?;
    let version_id =
        linked_data.version_id.as_ref().ok_or_else(unmanaged_err)?;

    // Replace the pack with the same version
    replace_managed_modrinth(
        profile_path,
        &profile,
        project_id,
        version_id,
        None,
    )
    .await?;

    emit_profile(
        profile.uuid,
        profile_path,
        &profile.metadata.name,
        ProfilePayloadType::Edited,
    )
    .await?;

    State::sync().await?;
    Ok(())
}

/// Replace a managed modrinth pack with a new version
/// If new_version_id is None, the pack is 'reinstalled' in-place
#[tracing::instrument(skip(profile))]
#[theseus_macros::debug_pin]
async fn replace_managed_modrinth(
    profile_path: &ProfilePathId,
    profile: &crate::state::Profile,
    project_id: &String,
    version_id: &String,
    new_version_id: Option<&String>,
) -> crate::Result<()> {
    // Fetch .mrpacks for both old and new versions
    // TODO: this will need to be updated if we revert the hacky pack method we needed for compiler speed
    let old_pack_creator = generate_pack_from_version_id(
        project_id.clone(),
        version_id.clone(),
        profile.metadata.name.clone(),
        None,
        profile_path.clone(),
    );

    // download in parallel, then join. If new_version_id is None, we don't need to download the new pack, so we clone the old one
    let (old_pack_creator, new_pack_creator) =
        if let Some(new_version_id) = new_version_id {
            try_join!(
                old_pack_creator,
                generate_pack_from_version_id(
                    project_id.clone(),
                    new_version_id.clone(),
                    profile.metadata.name.clone(),
                    None,
                    profile_path.clone()
                )
            )?
        } else {
            let mut old_pack_creator = old_pack_creator.await?;
            old_pack_creator.description.existing_loading_bar = None;
            (old_pack_creator.clone(), old_pack_creator)
        };

    // Removal - remove all files that were added by the old pack
    // - remove all installed projects
    // - remove all overrides
    pack::install_mrpack::remove_all_related_files(
        profile_path.clone(),
        old_pack_creator.file,
    )
    .await?;

    // Reinstallation - install all files that are added by the new pack
    // - install all projects
    // - install all overrides
    // - edits the profile to update the new data
    // - (functionals almost identically to rteinstalling the pack 'in-place')
    pack::install_mrpack::install_zipped_mrpack_files(new_pack_creator).await?;

    Ok(())
}
