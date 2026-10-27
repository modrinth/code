use super::*;
use crate::state::{
    CachedEntry, CachedSharedInstanceSummary, ModrinthCredentials,
    hidden_shared_instances,
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

#[tracing::instrument]
pub async fn get_hidden_shared_instance_ids() -> crate::Result<Vec<String>> {
    let state = State::get().await?;
    let Some(credentials) =
        ModrinthCredentials::get_active(&state.pool).await?
    else {
        return Ok(Vec::new());
    };

    hidden_shared_instances::get_hidden_shared_instance_ids(
        &credentials.user_id,
        &state.pool,
    )
    .await
}

#[tracing::instrument]
pub async fn set_shared_instance_hidden(
    shared_instance_id: &str,
    hidden: bool,
) -> crate::Result<()> {
    let state = State::get().await?;
    let Some(credentials) =
        ModrinthCredentials::get_active(&state.pool).await?
    else {
        return Ok(());
    };

    if hidden {
        hidden_shared_instances::hide_shared_instance(
            &credentials.user_id,
            shared_instance_id,
            &state.pool,
        )
        .await
    } else {
        hidden_shared_instances::unhide_shared_instance(
            &credentials.user_id,
            shared_instance_id,
            &state.pool,
        )
        .await
    }
}
