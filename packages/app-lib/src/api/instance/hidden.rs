use crate::state::{ModrinthCredentials, State, hidden_instances};

#[tracing::instrument]
pub async fn get_hidden_instance_ids() -> crate::Result<Vec<String>> {
    let state = State::get().await?;
    let Some(credentials) =
        ModrinthCredentials::get_active(&state.pool).await?
    else {
        return Ok(Vec::new());
    };

    hidden_instances::get_hidden_instance_ids(&credentials.user_id, &state.pool)
        .await
}

#[tracing::instrument]
pub async fn set_instance_hidden(
    instance_id: &str,
    hidden: bool,
) -> crate::Result<()> {
    let state = State::get().await?;
    let Some(credentials) =
        ModrinthCredentials::get_active(&state.pool).await?
    else {
        return Ok(());
    };

    if hidden {
        hidden_instances::hide_instance(
            &credentials.user_id,
            instance_id,
            &state.pool,
        )
        .await
    } else {
        hidden_instances::unhide_instance(
            &credentials.user_id,
            instance_id,
            &state.pool,
        )
        .await
    }
}
