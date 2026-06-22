use crate::state::{InstanceMetadata, State};

#[tracing::instrument]
pub async fn get(instance_id: &str) -> crate::Result<Option<InstanceMetadata>> {
    let state = State::get().await?;
    crate::state::get_instance(instance_id, &state.pool).await
}

#[tracing::instrument]
pub async fn get_many(
    instance_ids: &[&str],
) -> crate::Result<Vec<InstanceMetadata>> {
    let state = State::get().await?;
    let mut instances = Vec::with_capacity(instance_ids.len());

    for instance_id in instance_ids {
        if let Some(instance) =
            crate::state::get_instance(instance_id, &state.pool).await?
        {
            instances.push(instance);
        }
    }

    Ok(instances)
}

#[tracing::instrument]
pub async fn list() -> crate::Result<Vec<InstanceMetadata>> {
    let state = State::get().await?;
    crate::state::list_instances(&state.pool).await
}
