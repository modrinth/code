use crate::event::emit::init_loading;
use crate::event::{InstancePayloadType, LoadingBarType, emit::emit_instance};
use crate::pack::{self, install_from::generate_pack_from_version_id};
use crate::state::instances::InstanceLink;
use crate::state::{InstanceInstallStage, State};
use crate::util::fetch::DownloadReason;
use futures::try_join;
use std::collections::HashSet;

use super::{
    get_instance_metadata, list_project_files, remove_project,
    set_instance_install_stage, toggle_disable_project,
};

pub(crate) async fn update_managed_modrinth_version(
    instance_id: &str,
    new_version_id: &str,
    state: &State,
) -> crate::Result<()> {
    let metadata = get_instance_metadata(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let InstanceLink::ModrinthModpack {
        project_id,
        version_id,
    } = metadata.link
    else {
        return Err(unmanaged_pack_error(&metadata.instance.id).into());
    };

    replace_managed_modrinth(
        &metadata.instance.id,
        &metadata.instance.name,
        &project_id,
        &version_id,
        Some(new_version_id),
        true,
        state,
    )
    .await?;

    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(())
}

pub(crate) async fn repair_managed_modrinth(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    let metadata = get_instance_metadata(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let InstanceLink::ModrinthModpack {
        project_id,
        version_id,
    } = metadata.link
    else {
        return Err(unmanaged_pack_error(&metadata.instance.id).into());
    };

    for file in list_project_files(&metadata.instance.id, state).await? {
        remove_project(&metadata.instance.id, &file.relative_path, state)
            .await?;
    }

    replace_managed_modrinth(
        &metadata.instance.id,
        &metadata.instance.name,
        &project_id,
        &version_id,
        None,
        false,
        state,
    )
    .await?;

    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(())
}

async fn replace_managed_modrinth(
    instance_id: &str,
    instance_name: &str,
    project_id: &str,
    version_id: &str,
    new_version_id: Option<&str>,
    ignore_lock: bool,
    state: &State,
) -> crate::Result<()> {
    let metadata = get_instance_metadata(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let disabled_project_ids = list_project_files(instance_id, state)
        .await?
        .into_iter()
        .filter_map(|file| (!file.enabled).then_some(file.project_id?))
        .collect::<HashSet<_>>();

    set_instance_install_stage(
        &metadata.instance.id,
        InstanceInstallStage::MinecraftInstalling,
        &state.pool,
    )
    .await?;

    let (old_pack_creator, new_pack_creator) =
        if let Some(new_version_id) = new_version_id {
            let shared_loading_bar = init_loading(
                LoadingBarType::PackFileDownload {
                    instance_id: instance_id.to_string(),
                    pack_name: instance_name.to_string(),
                    icon: None,
                    pack_version: version_id.to_string(),
                },
                200.0,
                "Downloading pack file",
            )
            .await?;

            try_join!(
                generate_pack_from_version_id(
                    project_id.to_string(),
                    version_id.to_string(),
                    instance_name.to_string(),
                    None,
                    instance_id.to_string(),
                    Some(shared_loading_bar.clone()),
                    DownloadReason::Update,
                ),
                generate_pack_from_version_id(
                    project_id.to_string(),
                    new_version_id.to_string(),
                    instance_name.to_string(),
                    None,
                    instance_id.to_string(),
                    Some(shared_loading_bar),
                    DownloadReason::Update,
                )
            )?
        } else {
            let mut old_pack_creator = generate_pack_from_version_id(
                project_id.to_string(),
                version_id.to_string(),
                instance_name.to_string(),
                None,
                instance_id.to_string(),
                None,
                DownloadReason::Update,
            )
            .await?;
            old_pack_creator.description.existing_loading_bar = None;
            (old_pack_creator.clone(), old_pack_creator)
        };

    pack::install_mrpack::remove_all_related_files(
        instance_id.to_string(),
        old_pack_creator.file,
    )
    .await?;
    pack::install_mrpack::install_zipped_mrpack_files(
        new_pack_creator,
        ignore_lock,
        DownloadReason::Update,
    )
    .await?;

    if !disabled_project_ids.is_empty() {
        for file in list_project_files(instance_id, state).await? {
            if file.enabled
                && let Some(project_id) = &file.project_id
                && disabled_project_ids.contains(project_id)
            {
                toggle_disable_project(instance_id, &file.relative_path, state)
                    .await?;
            }
        }
    }

    Ok(())
}

fn unmanaged_pack_error(instance_id: &str) -> crate::ErrorKind {
    crate::ErrorKind::InputError(format!(
        "Instance {instance_id} is not a managed Modrinth pack, or has been disconnected."
    ))
}
