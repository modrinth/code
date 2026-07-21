use crate::OperationContext;
use crate::state::{
    CacheBehaviour, ContentFile, ContentItem, ContentSet, Dependency,
    InstanceInstallCandidate, InstanceInstallTarget, LinkedModpackInfo,
    ProjectType, State,
};
use dashmap::DashMap;

#[tracing::instrument]
pub async fn sync_content_files(
    context: &OperationContext,
    instance_id: &str,
) -> crate::Result<Vec<crate::state::instances::InstanceFile>> {
    let state = State::get().await?;
    crate::state::sync_content_files(context, instance_id, &state).await
}

#[tracing::instrument]
pub async fn list_content_sets(
    instance_id: &str,
) -> crate::Result<Vec<ContentSet>> {
    let state = State::get().await?;
    crate::state::list_content_sets(instance_id, &state.pool).await
}

#[tracing::instrument]
pub async fn get_projects(
    context: &OperationContext,
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<DashMap<String, ContentFile>> {
    let state = State::get().await?;
    crate::state::get_content_projects(
        context,
        instance_id,
        None,
        cache_behaviour,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn get_installed_project_ids(
    context: &OperationContext,
    instance_id: &str,
) -> crate::Result<Vec<String>> {
    let state = State::get().await?;
    crate::state::get_installed_project_ids_for_instance(
        context,
        instance_id,
        None,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn get_install_candidates(
    project_id: &str,
    project_type: ProjectType,
    targets: Vec<InstanceInstallTarget>,
) -> crate::Result<Vec<InstanceInstallCandidate>> {
    let state = State::get().await?;
    crate::state::get_instance_install_candidates(
        project_id,
        project_type,
        &targets,
        &state.pool,
    )
    .await
}

#[tracing::instrument]
pub async fn get_content_items(
    context: &OperationContext,
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<Vec<ContentItem>> {
    let state = State::get().await?;
    crate::state::list_content(
        context,
        instance_id,
        None,
        cache_behaviour,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn get_linked_modpack_content(
    context: &OperationContext,
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<Vec<ContentItem>> {
    let state = State::get().await?;
    crate::state::list_linked_modpack_content(
        context,
        instance_id,
        None,
        cache_behaviour,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn get_dependencies_as_content_items(
    context: &OperationContext,
    dependencies: Vec<Dependency>,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<Vec<ContentItem>> {
    let state = State::get().await?;
    crate::state::dependencies_to_content_items(
        context,
        &dependencies,
        cache_behaviour,
        &state.pool,
        &state.api_semaphore,
    )
    .await
}

#[tracing::instrument]
pub async fn get_linked_modpack_info(
    context: &OperationContext,
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<Option<LinkedModpackInfo>> {
    let state = State::get().await?;
    crate::state::get_linked_modpack_info(
        context,
        instance_id,
        None,
        cache_behaviour,
        &state,
    )
    .await
}
