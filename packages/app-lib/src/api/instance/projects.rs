use crate::event::emit::{emit_instance, emit_loading, init_loading};
use crate::event::{InstancePayloadType, LoadingBarType};
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{CacheBehaviour, CachedEntry, ProjectType, State};
use crate::util::fetch;
use modrinth_content_management::{
    ContentType, ResolutionPreferences, ResolveContentPlan,
};
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct InstallProjectWithDependenciesRequest {
    pub project_id: String,
    pub version_id: Option<String>,
    pub content_type: ContentType,
    #[serde(default)]
    pub selected: ResolutionPreferences,
}

#[tracing::instrument]
pub async fn update_all_projects(
    instance_id: &str,
) -> crate::Result<HashMap<String, String>> {
    let state = State::get().await?;
    let instance = get_instance_display_info(instance_id, &state).await?;
    let loading_bar = init_loading(
        LoadingBarType::InstanceUpdate {
            instance_id: instance.id.clone(),
            instance_name: instance.name.clone(),
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
    emit_instance(&instance.id, InstancePayloadType::Edited).await?;

    Ok(map)
}

#[tracing::instrument]
pub async fn update_project(
    instance_id: &str,
    project_path: &str,
    skip_send_event: Option<bool>,
) -> crate::Result<String> {
    let state = State::get().await?;
    let path = crate::state::instances::commands::update_project(
        instance_id,
        project_path,
        &state,
    )
    .await?;

    if !skip_send_event.unwrap_or(false) {
        emit_instance(instance_id, InstancePayloadType::Edited).await?;
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
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(project_path)
}

#[tracing::instrument]
pub async fn install_project_with_dependencies(
    instance_id: &str,
    request: InstallProjectWithDependenciesRequest,
) -> crate::Result<ResolveContentPlan> {
    let state = State::get().await?;
    let metadata = super::get::get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let plan = crate::state::instances::commands::resolve_install_plan(
        instance_id,
        crate::state::instances::commands::InstanceInstallProjectRequest {
            project_id: request.project_id,
            version_id: request.version_id,
            content_type: request.content_type,
            selected: request.selected,
        },
        &state,
    )
    .await?;

    let instance_id = metadata.instance.id;
    let project_ids = plan_project_ids(&plan);
    let install_plan = plan.clone();
    tokio::spawn(async move {
        match crate::state::instances::commands::install_resolved_content_plan(
            &instance_id,
            &install_plan,
            &state,
        )
        .await
        {
            Ok(()) => {
                if let Err(error) = emit_instance(
                    &instance_id,
                    InstancePayloadType::ContentInstallFinished {
                        project_ids: project_ids.clone(),
                    },
                )
                .await
                {
                    tracing::error!(
                        "Failed to emit content install finished event: {error}"
                    );
                }
                if let Err(error) =
                    emit_instance(&instance_id, InstancePayloadType::Edited)
                        .await
                {
                    tracing::error!(
                        "Failed to emit instance edited event after content install: {error}"
                    );
                }
            }
            Err(error) => {
                if let Err(emit_error) = emit_instance(
                    &instance_id,
                    InstancePayloadType::ContentInstallFailed {
                        project_ids,
                        message: error.to_string(),
                    },
                )
                .await
                {
                    tracing::error!(
                        "Failed to emit content install failed event: {emit_error}"
                    );
                }
            }
        }
    });

    Ok(plan)
}

fn plan_project_ids(plan: &ResolveContentPlan) -> Vec<String> {
    let mut project_ids = Vec::with_capacity(plan.dependencies.len() + 1);
    project_ids.push(plan.primary.project_id.clone());
    project_ids.extend(
        plan.dependencies
            .iter()
            .map(|dependency| dependency.project_id.clone()),
    );
    project_ids
}

#[tracing::instrument]
pub async fn switch_project_version_with_dependencies(
    instance_id: &str,
    project_path: &str,
    version_id: &str,
) -> crate::Result<String> {
    let state = State::get().await?;
    let metadata = super::get::get(instance_id).await?.ok_or_else(|| {
        crate::ErrorKind::InputError("Unknown instance".to_string())
    })?;
    let path =
        crate::state::instances::commands::switch_project_version_with_dependencies(
            instance_id,
            project_path,
            version_id,
            &state,
        )
        .await?;
    emit_instance(&metadata.instance.id, InstancePayloadType::Edited).await?;

    Ok(path)
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
pub async fn is_file_on_modrinth(path: &Path) -> crate::Result<bool> {
    let state = State::get().await?;
    let (_, hash) = fetch::sha1_file_async(path).await?;
    let files = CachedEntry::get_file_many(
        &[&hash],
        Some(CacheBehaviour::Bypass),
        &state.pool,
        &state.api_semaphore,
    )
    .await?;

    Ok(!files.is_empty())
}

#[tracing::instrument]
pub async fn toggle_disable_project(
    instance_id: &str,
    project: &str,
    desired_enabled: Option<bool>,
) -> crate::Result<String> {
    let state = State::get().await?;
    let res = crate::state::instances::commands::toggle_disable_project(
        instance_id,
        project,
        desired_enabled,
        &state,
    )
    .await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(res)
}

#[tracing::instrument]
pub async fn remove_project(
    instance_id: &str,
    project: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    crate::state::instances::commands::remove_project(
        instance_id,
        project,
        &state,
    )
    .await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn update_managed_modrinth_version(
    instance_id: &str,
    version_id: &str,
) -> crate::Result<crate::install::InstallJobSnapshot> {
    let state = State::get().await?;
    let metadata = crate::state::instances::commands::get_instance_metadata(
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
    let metadata = crate::state::instances::commands::get_instance_metadata(
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

async fn get_instance_display_info(
    instance_id: &str,
    state: &State,
) -> crate::Result<instance_rows::InstanceDisplayInfo> {
    instance_rows::get_instance_display_info(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string()).into()
        })
}
