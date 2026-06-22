use super::get::get;
use crate::event::emit::{emit_instance, emit_loading, init_loading};
use crate::event::{InstancePayloadType, LoadingBarType};
use crate::state::instances::BulkUpdatePreview;
use crate::state::{ProjectType, State};
use crate::util::fetch;
use std::collections::HashMap;
use std::path::Path;

#[tracing::instrument]
pub async fn preview_update_all_projects(
    instance_id: &str,
) -> crate::Result<BulkUpdatePreview> {
    let state = State::get().await?;

    crate::state::instances::commands::preview_update_all_projects(
        instance_id,
        &state,
    )
    .await
}

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
) -> crate::Result<String> {
    let state = State::get().await?;
    let metadata = get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let res = crate::state::instances::commands::toggle_disable_project(
        instance_id,
        project,
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
) -> crate::Result<()> {
    let state = State::get().await?;
    crate::state::instances::commands::update_managed_modrinth_version(
        instance_id,
        version_id,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn repair_managed_modrinth(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    crate::state::instances::commands::repair_managed_modrinth(
        instance_id,
        &state,
    )
    .await
}
