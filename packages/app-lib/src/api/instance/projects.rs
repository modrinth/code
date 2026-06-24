use super::get::get;
use crate::event::emit::{emit_instance, emit_loading, init_loading};
use crate::event::{InstancePayloadType, LoadingBarType};
use crate::state::{ProjectType, State};
use crate::util::fetch;
use std::collections::HashMap;
use std::path::Path;

#[tracing::instrument]
pub async fn update_all_projects(
    instance_id: &str,
) -> crate::Result<HashMap<String, String>> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let loading_bar = init_loading(
        LoadingBarType::InstanceUpdate {
            instance_id: metadata.instance.id.clone(),
            instance_name: metadata.instance.name.clone(),
        },
        100.0,
        "Updating instance",
    )
    .await?;
    let map = crate::state::instances::commands::update_all_projects(
        instance_id,
        &state,
    )
    .await?;
    emit_loading(&loading_bar, 100.0, Some("Updated instance"))?;
    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(map)
}

#[tracing::instrument]
pub async fn update_project(
    instance_id: &str,
    project_path: &str,
    skip_send_event: Option<bool>,
) -> crate::Result<String> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let path = crate::state::instances::commands::update_project(
        instance_id,
        project_path,
        &state,
    )
    .await?;

    if !skip_send_event.unwrap_or(false) {
        emit_instance(&metadata.instance.id, InstancePayloadType::Edited)
            .await?;
    }

    Ok(path)
}

#[tracing::instrument]
pub async fn add_project_from_version(
    instance_id: &str,
    version_id: &str,
    reason: fetch::DownloadReason,
    dependent_on_version_id: Option<String>,
) -> crate::Result<String> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let project_path =
        crate::state::instances::commands::add_project_from_version(
            instance_id,
            version_id,
            reason,
            dependent_on_version_id,
            crate::state::ContentSourceKind::Local,
            &state,
        )
        .await?;
    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(project_path)
}

#[tracing::instrument]
pub async fn add_project_from_path(
    instance_id: &str,
    path: &Path,
    project_type: Option<ProjectType>,
) -> crate::Result<String> {
    let state = State::get().await?;
    crate::state::instances::commands::add_project_from_path(
        instance_id,
        path,
        project_type,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn toggle_disable_project(
    instance_id: &str,
    project: &str,
    desired_enabled: Option<bool>,
) -> crate::Result<String> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let res = crate::state::instances::commands::toggle_disable_project(
        instance_id,
        project,
        desired_enabled,
        &state,
    )
    .await?;
    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(res)
}

#[tracing::instrument]
pub async fn remove_project(
    instance_id: &str,
    project: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    crate::state::instances::commands::remove_project(
        instance_id,
        project,
        &state,
    )
    .await?;
    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn update_managed_modrinth_version(
    instance_id: &str,
    version_id: &str,
) -> crate::Result<crate::install::InstallJobSnapshot> {
    let state = State::get().await?;
    let metadata =
        crate::state::instances::commands::get_instance_metadata(
            instance_id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;

    let post_install_edit = match &metadata.link {
        crate::state::InstanceLink::ServerProjectModpack {
            server_project_id,
            content_project_id,
            ..
        } => Some(crate::install::InstallPostInstallEdit {
            name: Some(metadata.instance.name.clone()),
            icon_path: Some(metadata.instance.icon_path.clone()),
            link: Some(crate::state::InstanceLink::ServerProjectModpack {
                server_project_id: server_project_id.clone(),
                content_project_id: content_project_id.clone(),
                content_version_id: version_id.to_string(),
            }),
        }),
        _ => None,
    };

    let project_id = match &metadata.link {
        crate::state::InstanceLink::ModrinthModpack { project_id, .. } => {
            project_id.clone()
        }
        crate::state::InstanceLink::ServerProjectModpack {
            content_project_id,
            ..
        } => content_project_id.clone(),
        _ => {
            return Err(unmanaged_pack_error(&metadata.instance.id).into());
        }
    };

    crate::install::install_pack_to_existing_instance(
        metadata.instance.id,
        crate::api::pack::install_from::CreatePackLocation::FromVersionId {
            project_id,
            version_id: version_id.to_string(),
            title: metadata.instance.name.clone(),
            icon_url: None,
        },
        post_install_edit,
    )
    .await
}

#[tracing::instrument]
pub async fn repair_managed_modrinth(
    instance_id: &str,
) -> crate::Result<crate::install::InstallJobSnapshot> {
    let state = State::get().await?;
    let metadata =
        crate::state::instances::commands::get_instance_metadata(
            instance_id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;

    let post_install_edit = match &metadata.link {
        crate::state::InstanceLink::ServerProjectModpack { .. } => {
            Some(crate::install::InstallPostInstallEdit {
                name: Some(metadata.instance.name.clone()),
                icon_path: Some(metadata.instance.icon_path.clone()),
                link: Some(metadata.link.clone()),
            })
        }
        _ => None,
    };

    let (project_id, version_id) = match &metadata.link {
        crate::state::InstanceLink::ModrinthModpack {
            project_id,
            version_id,
        } => (project_id.clone(), version_id.clone()),
        crate::state::InstanceLink::ServerProjectModpack {
            content_project_id,
            content_version_id,
            ..
        } => (content_project_id.clone(), content_version_id.clone()),
        _ => {
            return Err(unmanaged_pack_error(&metadata.instance.id).into());
        }
    };

    crate::install::install_pack_to_existing_instance(
        metadata.instance.id,
        crate::api::pack::install_from::CreatePackLocation::FromVersionId {
            project_id,
            version_id,
            title: metadata.instance.name.clone(),
            icon_url: None,
        },
        post_install_edit,
    )
    .await
}

fn unmanaged_pack_error(instance_id: &str) -> crate::ErrorKind {
    crate::ErrorKind::InputError(format!(
        "Instance {instance_id} is not a managed Modrinth pack, or has been disconnected."
    ))
}
