use crate::ErrorKind;
use crate::state::{ModrinthCredentials, State, hidden_instances};

#[tracing::instrument]
pub async fn get_hidden_instance_ids() -> crate::Result<Vec<String>> {
    let state = State::get().await?;
    let Some(user_id) =
        ModrinthCredentials::get_active_user_id(&state.pool).await?
    else {
        return Ok(Vec::new());
    };

    hidden_instances::get_hidden_instance_ids(&user_id, &state.pool).await
}

#[tracing::instrument]
pub async fn set_instance_hidden(
    instance_id: &str,
    hidden: bool,
) -> crate::Result<()> {
    let state = State::get().await?;
    let user_id = ModrinthCredentials::get_active_user_id(&state.pool)
        .await?
        .ok_or(ErrorKind::NotLoggedIn)?;

    hidden_instances::set_instance_hidden(
        &user_id,
        instance_id,
        hidden,
        &state.pool,
    )
    .await
}
