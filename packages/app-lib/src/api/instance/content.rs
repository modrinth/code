use crate::state::{
    CacheBehaviour, ContentFile, ContentItem, ContentSet, Dependency,
    LinkedModpackInfo, State,
};
use dashmap::DashMap;

#[tracing::instrument]
pub async fn sync_content_files(
    instance_id: &str,
) -> crate::Result<Vec<crate::state::instances::InstanceFile>> {
    let state = State::get().await?;
    crate::state::sync_content_files(instance_id, &state).await
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
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<DashMap<String, ContentFile>> {
    let state = State::get().await?;
    crate::state::get_content_projects(
        instance_id,
        None,
        cache_behaviour,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn get_installed_project_ids(
    instance_id: &str,
) -> crate::Result<Vec<String>> {
    let state = State::get().await?;
    crate::state::get_installed_project_ids_for_instance(
        instance_id,
        None,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn get_content_items(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<Vec<ContentItem>> {
    let state = State::get().await?;
    crate::state::list_content(instance_id, None, cache_behaviour, &state).await
}

#[tracing::instrument]
pub async fn get_linked_modpack_content(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<Vec<ContentItem>> {
    let state = State::get().await?;
    crate::state::list_linked_modpack_content(
        instance_id,
        None,
        cache_behaviour,
        &state,
    )
    .await
}

#[tracing::instrument]
pub async fn get_dependencies_as_content_items(
    dependencies: Vec<Dependency>,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<Vec<ContentItem>> {
    let state = State::get().await?;
    crate::state::dependencies_to_content_items(
        &dependencies,
        cache_behaviour,
        &state.pool,
        &state.api_semaphore,
    )
    .await
}

#[tracing::instrument]
pub async fn get_linked_modpack_info(
    instance_id: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> crate::Result<Option<LinkedModpackInfo>> {
    let state = State::get().await?;
    crate::state::get_linked_modpack_info(
        instance_id,
        None,
        cache_behaviour,
        &state,
    )
    .await
}
