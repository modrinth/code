const SHARED_INSTANCE_INVITE_MAX_AGE_SECONDS: i32 = 604800;
const SHARED_INSTANCE_INVITE_MAX_USES: i32 = 10;

use super::client::*;
use super::types::*;
use super::*;
use super::install::*;
use super::publish::*;

#[tracing::instrument]
pub async fn get_shared_instance_users(
    instance_id: &str,
) -> crate::Result<SharedInstanceUsers> {
    let state = State::get().await?;
    let Some(attachment) = shared_attachment(instance_id, &state).await? else {
        return Ok(SharedInstanceUsers::empty());
    };

    get_remote_users(&attachment.id, &state).await
}

#[tracing::instrument]
pub async fn invite_shared_instance_users(
    instance_id: &str,
    user_ids: Vec<String>,
) -> crate::Result<SharedInstanceUsers> {
    let state = State::get().await?;
    let (metadata, attachment) =
        shared_instance_for_invites(instance_id, user_ids.len(), &state)
            .await?;

    ensure_owner(&attachment)?;
    if !user_ids.is_empty() {
        ensure_ready_remote_version_for_invite(
            instance_id,
            &attachment,
            &state,
        )
        .await?;
        update_remote_instance(
            &attachment.id,
            shared_instance_name(metadata.instance.name.clone()),
            &state,
        )
        .await?;
        tracing::info!(
            instance_id,
            shared_instance_id = %attachment.id,
            user_count = user_ids.len(),
            "Adding users to shared instance"
        );
        add_remote_users(&attachment.id, user_ids.clone(), &state).await?;
    }
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    let mut users = get_remote_users(&attachment.id, &state).await?;
    users.include_pending_invites(user_ids);
    Ok(users)
}

#[tracing::instrument]
pub async fn create_shared_instance_invite_link(
    instance_id: &str,
    max_age_seconds: Option<i32>,
    max_uses: Option<i32>,
    replace_invite_id: Option<String>,
) -> crate::Result<SharedInstanceInviteLink> {
    let state = State::get().await?;
    let (metadata, attachment) =
        shared_instance_for_invites(instance_id, 0, &state).await?;
    ensure_owner(&attachment)?;
    ensure_ready_remote_version_for_invite(instance_id, &attachment, &state)
        .await?;
    update_remote_instance(
        &attachment.id,
        shared_instance_name(metadata.instance.name),
        &state,
    )
    .await?;

    let max_age_seconds =
        max_age_seconds.unwrap_or(SHARED_INSTANCE_INVITE_MAX_AGE_SECONDS);
    let max_uses = max_uses.unwrap_or(SHARED_INSTANCE_INVITE_MAX_USES);
    if max_age_seconds <= 0
        || max_age_seconds > SHARED_INSTANCE_INVITE_MAX_AGE_SECONDS
    {
        return Err(crate::ErrorKind::InputError(
            "Invite expiry must be between now and seven days".to_string(),
        )
        .into());
    }
    if max_uses <= 0 {
        return Err(crate::ErrorKind::InputError(
            "Invite max uses must be greater than zero".to_string(),
        )
        .into());
    }

    if let Some(invite_id) = replace_invite_id {
        request_empty(
            "delete_instance_invite",
            Method::DELETE,
            &format!("/instances/{}/invites/{invite_id}", attachment.id),
            None,
            &state,
        )
        .await?;
    }

    let created_at = Utc::now();

    let response = request_json::<CreateInstanceInviteResponse>(
        "create_instance_invite",
        Method::POST,
        &format!("/instances/{}/invites", attachment.id),
        Some(json!({
            "max_age": max_age_seconds,
            "max_uses": max_uses,
        })),
        &state,
    )
    .await?;

    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(SharedInstanceInviteLink {
        invite_id: response.id,
        expires_at: created_at
            + chrono::Duration::seconds(i64::from(max_age_seconds)),
        max_uses,
    })
}

#[tracing::instrument]
pub async fn remove_shared_instance_users(
    instance_id: &str,
    user_ids: Vec<String>,
) -> crate::Result<SharedInstanceUsers> {
    let state = State::get().await?;
    let Some(attachment) = shared_attachment(instance_id, &state).await? else {
        return Ok(SharedInstanceUsers::empty());
    };
    ensure_owner(&attachment)?;

    if !user_ids.is_empty() {
        remove_remote_users(&attachment.id, user_ids, &state).await?;
    }

    let remaining_users = get_remote_users(&attachment.id, &state).await?;
    if !has_shared_instance_recipients(&remaining_users, &attachment, &state)
        .await?
    {
        delete_remote_instance(&attachment.id, &state).await?;
        crate::state::clear_shared_instance(instance_id, &state.pool).await?;
        emit_instance(instance_id, InstancePayloadType::Edited).await?;

        return Ok(SharedInstanceUsers::empty());
    }

    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(remaining_users)
}

#[tracing::instrument]
pub async fn accept_pending_shared_instance_invite(
    shared_instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;

    if accept_pending_remote_invite(shared_instance_id, &state).await? {
        return Ok(());
    }

    Err(crate::ErrorKind::InputError(
        "No pending invite found for shared instance".to_string(),
    )
    .into())
}

#[tracing::instrument]
pub async fn decline_pending_shared_instance_invite(
    shared_instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    decline_pending_remote_invite(shared_instance_id, &state).await
}


