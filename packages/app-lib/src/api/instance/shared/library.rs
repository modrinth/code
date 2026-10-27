use super::*;
use crate::state::{
    CachedEntry, CachedSharedInstanceSummary, ModrinthCredentials,
};

#[tracing::instrument]
pub async fn get_cached_shared_instance_library()
-> crate::Result<Vec<CachedSharedInstanceSummary>> {
    let state = State::get().await?;
    let Some(credentials) =
        ModrinthCredentials::get_active(&state.pool).await?
    else {
        return Ok(Vec::new());
    };

    CachedEntry::get_shared_instance_library(
        &credentials.user_id,
        &state.pool,
        &state.api_semaphore,
    )
    .await
}

#[tracing::instrument(skip(instances))]
pub async fn cache_shared_instance_library(
    user_id: &str,
    instances: Vec<CachedSharedInstanceSummary>,
) -> crate::Result<()> {
    let state = State::get().await?;
    CachedEntry::cache_shared_instance_library(user_id, instances, &state.pool)
        .await
}
