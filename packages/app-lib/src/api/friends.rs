use crate::state::{FriendsSocket, UserFriend};
use ariadne::users::UserStatus;

#[tracing::instrument]
pub async fn friends() -> crate::Result<Vec<UserFriend>> {
    let state = crate::State::get().await?;
    let friends =
        FriendsSocket::friends(&state.pool, &state.api_semaphore).await?;

    Ok(friends)
}

pub async fn friend_statuses() -> crate::Result<Vec<UserStatus>> {
    let state = crate::State::get().await?;
    let statuses = state.friends_socket.friend_statuses();

    Ok(statuses)
}

#[tracing::instrument]
pub async fn add_friend(user_id: &str) -> crate::Result<()> {
    let state = crate::State::get().await?;
    FriendsSocket::add_friend(user_id, &state.pool, &state.api_semaphore)
        .await?;

    Ok(())
}

#[tracing::instrument]
pub async fn remove_friend(user_id: &str) -> crate::Result<()> {
    let state = crate::State::get().await?;
    FriendsSocket::remove_friend(user_id, &state.pool, &state.api_semaphore)
        .await?;

    Ok(())
}
