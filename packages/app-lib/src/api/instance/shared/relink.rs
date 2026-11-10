use super::client::*;
use super::install::*;
use super::publish::*;
use super::types::*;
use super::*;

/// Resolves a pasted invite link or ID to the shared instance it points to,
/// and previews relinking a local instance to it as its owner.
#[tracing::instrument(skip(input))]
pub async fn get_shared_instance_relink_preview(
    instance_id: &str,
    input: &str,
) -> crate::Result<SharedInstanceRelinkPreview> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or(crate::ErrorKind::InputError("unknown instance".to_string()))?;
    let shared_instance_id = resolve_shared_instance_id(input, &state).await?;

    Ok(relink_target(&metadata, &shared_instance_id, &state)
        .await?
        .preview)
}

/// Links a local instance back to a shared instance its user owns, such as
/// after the app unlinked it. The local content is kept, and the owner can
/// publish it as the next version.
#[tracing::instrument]
pub async fn relink_shared_instance(
    instance_id: &str,
    shared_instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let _shared_instance_lock =
        state.instance_locks.lock_shared(instance_id).await;
    let _relink_lock =
        state.instance_locks.lock_relink(shared_instance_id).await;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or(crate::ErrorKind::InputError("unknown instance".to_string()))?;
    let target = relink_target(&metadata, shared_instance_id, &state).await?;
    if target.preview.blocker.is_some() {
        bail!(crate::ErrorKind::InputError(
            "this shared instance can't be relinked to this instance"
                .to_string(),
        ));
    }

    crate::state::attach_shared_instance(
        instance_id,
        crate::state::SharedInstanceAttachmentInput {
            id: shared_instance_id.to_string(),
            role: SharedInstanceRole::Owner,
            restored_from_remote: false,
            manager_id: None,
            server_manager_name: None,
            server_manager_icon_url: None,
            linked_user_id: Some(target.user_id),
            status: ContentSetSyncStatus::Unknown,
            applied_version: Some(target.latest_version.version),
            applied_backtrack_version: target
                .latest_version
                .backtrack_version
                .clone(),
            latest_version: Some(target.latest_version.version),
        },
        &state.pool,
    )
    .await?;
    emit_instance(instance_id, InstancePayloadType::Edited).await?;

    Ok(())
}

/// Where to find the shared instance pasted to relink an instance.
enum RelinkInput {
    /// An invite link, holding the invite's ID.
    Invite(String),
    /// A bare ID, of either a shared instance or an invite.
    Id(String),
}

/// Parses an invite link (`.../share/<invite id>`), or a bare shared instance
/// or invite ID.
fn parse_relink_input(input: &str) -> crate::Result<RelinkInput> {
    let input = input.trim();
    let (id, is_invite) = match input.split_once("/share/") {
        Some((_, rest)) => {
            (rest.split(['/', '?', '#']).next().unwrap_or_default(), true)
        }
        None => (input, false),
    };
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric()) {
        bail!(crate::ErrorKind::InputError(
            "enter a shared instance ID or invite link".to_string(),
        ));
    }

    Ok(if is_invite {
        RelinkInput::Invite(id.to_string())
    } else {
        RelinkInput::Id(id.to_string())
    })
}

/// Resolves pasted input to a shared instance ID. Shared instance and invite
/// IDs look alike, so a bare ID is looked up as a shared instance first, then
/// as an invite.
async fn resolve_shared_instance_id(
    input: &str,
    state: &State,
) -> crate::Result<String> {
    let id = match parse_relink_input(input)? {
        RelinkInput::Invite(invite_id) => {
            return find_shared_instance_invite_info(&invite_id, state)
                .await?
                .map(|invite| invite.instance_id)
                .ok_or(
                    crate::ErrorKind::InputError(
                        "this invite link doesn't exist or has expired"
                            .to_string(),
                    )
                    .into(),
                );
        }
        RelinkInput::Id(id) => id,
    };

    match get_remote_instance_access(&id, state).await? {
        SharedInstanceRemoteResponse::Available(_) => return Ok(id),
        SharedInstanceRemoteResponse::Unavailable(
            SharedInstanceUnavailableReason::Deleted,
        ) => {}
        SharedInstanceRemoteResponse::Unavailable(reason) => {
            return Err(shared_instance_unavailable_error(reason));
        }
    }

    find_shared_instance_invite_info(&id, state)
        .await?
        .map(|invite| invite.instance_id)
        .ok_or(
            crate::ErrorKind::InputError(
                "no shared instance or invite has this ID".to_string(),
            )
            .into(),
        )
}

/// A shared instance a local instance could be relinked to.
struct RelinkTarget {
    preview: SharedInstanceRelinkPreview,
    latest_version: InstanceVersionResponse,
    user_id: String,
}

/// Checks that the local instance can be relinked, then fetches the shared
/// instance and what would prevent relinking the instance to it.
async fn relink_target(
    metadata: &crate::state::InstanceMetadata,
    shared_instance_id: &str,
    state: &State,
) -> crate::Result<RelinkTarget> {
    ensure_relinkable_instance(metadata)?;
    let user_id = linked_modrinth_user_id(state)
        .await?
        .ok_or(crate::ErrorKind::NotLoggedIn)?;

    let remote =
        match get_remote_instance_access(shared_instance_id, state).await? {
            SharedInstanceRemoteResponse::Available(remote) => remote,
            SharedInstanceRemoteResponse::Unavailable(reason) => {
                return Err(shared_instance_unavailable_error(reason));
            }
        };
    if remote.quarantine {
        return Err(shared_instance_unavailable_error(
            SharedInstanceUnavailableReason::Quarantined,
        ));
    }
    let latest_version =
        get_latest_remote_version(shared_instance_id, state).await?;

    let blocker = if remote.linked_server.is_some() {
        Some(SharedInstanceRelinkBlocker::ServerManaged)
    } else if !get_remote_users(shared_instance_id, state)
        .await?
        .users
        .iter()
        .any(|user| {
            user.id == user_id
                && user.join_type == SharedInstanceJoinType::Owner
        })
    {
        Some(SharedInstanceRelinkBlocker::NotOwner)
    } else {
        crate::state::list_instances(&state.pool)
            .await?
            .into_iter()
            .find(|instance| {
                instance.shared_instance.as_ref().is_some_and(|attachment| {
                    attachment.id == shared_instance_id
                })
            })
            .map(|instance| SharedInstanceRelinkBlocker::AlreadyLinked {
                instance_id: instance.instance.id,
                instance_name: instance.instance.name,
            })
    };

    let content_set = &metadata.applied_content_set;
    Ok(RelinkTarget {
        preview: SharedInstanceRelinkPreview {
            shared_instance_id: shared_instance_id.to_string(),
            name: remote.name,
            icon_url: remote.icon,
            version: latest_version.version,
            game_version: latest_version.game_version.clone(),
            loader: latest_version.loader,
            game_version_mismatch: content_set.game_version
                != latest_version.game_version,
            loader_mismatch: content_set.loader != latest_version.loader,
            blocker,
        },
        latest_version,
        user_id,
    })
}

/// Checks that a local instance is installed, unlinked and could be shared.
fn ensure_relinkable_instance(
    metadata: &crate::state::InstanceMetadata,
) -> crate::Result<()> {
    if metadata.shared_instance.is_some()
        || matches!(metadata.link, InstanceLink::SharedInstance { .. })
    {
        bail!(crate::ErrorKind::InputError(
            "this instance is already linked to a shared instance".to_string(),
        ));
    }
    if matches!(
        metadata.link,
        InstanceLink::ServerProject { .. }
            | InstanceLink::ServerProjectModpack { .. }
            | InstanceLink::ModrinthHosting { .. }
    ) {
        bail!(crate::ErrorKind::InputError(
            "server project instances can't be linked to a shared instance"
                .to_string(),
        ));
    }
    if metadata.quarantined {
        bail!(crate::ErrorKind::InputError(
            "locked instances can't be linked to a shared instance".to_string(),
        ));
    }
    if metadata.instance.install_stage
        != crate::state::InstanceInstallStage::Installed
    {
        bail!(crate::ErrorKind::InputError(
            "the instance must finish installing before it can be linked"
                .to_string(),
        ));
    }

    ensure_shareable_link(&metadata.link)
}
