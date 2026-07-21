use crate::InvocationContext;
use crate::state::{FriendsSocket, UserFriend};
use ariadne::users::UserStatus;

#[tracing::instrument]
pub async fn friends(
    context: &InvocationContext,
) -> crate::Result<Vec<UserFriend>> {
    let state = crate::State::get().await?;
    let friends =
        FriendsSocket::friends(context, &state.pool, &state.api_semaphore)
            .await?;

    Ok(friends)
}

pub async fn friend_statuses() -> crate::Result<Vec<UserStatus>> {
    let state = crate::State::get().await?;
    let statuses = state.friends_socket.friend_statuses();

    Ok(statuses)
}

#[tracing::instrument]
pub async fn add_friend(
    context: &InvocationContext,
    user_id: &str,
) -> crate::Result<()> {
    let state = crate::State::get().await?;
    FriendsSocket::add_friend(
        context,
        user_id,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;

    Ok(())
}

#[tracing::instrument]
pub async fn remove_friend(
    context: &InvocationContext,
    user_id: &str,
) -> crate::Result<()> {
    let state = crate::State::get().await?;
    FriendsSocket::remove_friend(
        context,
        user_id,
        &state.pool,
        &state.api_semaphore,
    )
    .await?;

    Ok(())
}
