use super::client::*;
use super::diff::*;
use super::publish::*;
use super::types::*;
use super::*;

#[tracing::instrument]
pub async fn install_shared_instance(
    shared_instance_id: &str,
    name: String,
    manager_id: Option<String>,
    server_manager_name: Option<String>,
    server_manager_icon_url: Option<String>,
    instance_icon_url: Option<String>,
) -> crate::Result<InstallJobSnapshot> {
    let state = State::get().await?;
    let version = get_latest_remote_version(shared_instance_id, &state).await?;
    let data = shared_instance_install_data(
        shared_instance_id,
        manager_id,
        server_manager_name,
        server_manager_icon_url,
        instance_icon_url,
        name,
        version,
        &state,
    )
    .await?;

    crate::install::create_shared_instance(data).await
}

pub(super) fn shared_instance_invite_install_name(
    shared_instance_id: &str,
    name: String,
) -> String {
    let name = name.trim();
    if name.is_empty() || name == "Shared instance" {
        return format!("Shared instance {shared_instance_id}");
    }

    name.to_string()
}

#[tracing::instrument]
pub async fn get_shared_instance_install_preview(
    shared_instance_id: &str,
    name: String,
) -> crate::Result<SharedInstanceInstallPreview> {
    let state = State::get().await?;
    let version = get_latest_remote_version(shared_instance_id, &state).await?;
    shared_instance_install_preview_from_version(name, version, &state).await
}

#[tracing::instrument]
pub async fn accept_shared_instance_invite_for_install(
    invite_id: &str,
) -> crate::Result<SharedInstanceInviteInstallPreview> {
    let state = State::get().await?;
    let invite = get_shared_instance_invite_info(invite_id, &state).await?;
    let shared_instance_id = invite.instance_id;
    let instance_icon_url = invite.instance_icon;
    let (manager_id, server_manager_name, server_manager_icon_url) =
        shared_instance_invite_manager(invite.managers);
    let name = shared_instance_invite_install_name(
        &shared_instance_id,
        invite.instance_name,
    );
    accept_shared_instance_invite(&shared_instance_id, invite_id, &state)
        .await?;
    let version =
        get_latest_remote_version(&shared_instance_id, &state).await?;
    let mut preview =
        shared_instance_install_preview_from_version(name, version, &state)
            .await?;
    if instance_icon_url.is_some() {
        preview.icon_url.clone_from(&instance_icon_url);
    }

    Ok(SharedInstanceInviteInstallPreview {
        shared_instance_id,
        manager_id,
        server_manager_name,
        server_manager_icon_url,
        instance_icon_url,
        preview,
    })
}

pub(super) fn shared_instance_invite_manager(
    managers: Vec<InstanceInviteManagerResponse>,
) -> (Option<String>, Option<String>, Option<String>) {
    match managers.into_iter().next() {
        Some(InstanceInviteManagerResponse::User { id }) => {
            (Some(id), None, None)
        }
        Some(InstanceInviteManagerResponse::Server { name, icon }) => {
            (None, Some(name), icon)
        }
        None => (None, None, None),
    }
}

pub(super) async fn shared_instance_install_preview_from_version(
    name: String,
    version: InstanceVersionResponse,
    state: &State,
) -> crate::Result<SharedInstanceInstallPreview> {
    let name = match name.trim() {
        "" => "Shared instance".to_string(),
        name => name.to_string(),
    };
    let mut content_version_ids = version.modrinth_ids.clone();
    let mut seen_content_version_ids = HashSet::new();
    content_version_ids
        .retain(|id| seen_content_version_ids.insert(id.clone()));
    let modpack = shared_instance_install_modpack(&version, state).await?;
    let modpack_dependency_count = modpack
        .as_ref()
        .map(|modpack| modpack.dependency_count)
        .unwrap_or_default();
    let modpack_version_id =
        modpack.as_ref().map(|modpack| modpack.version_id.clone());
    if let Some(modpack_version_id) = modpack_version_id.as_deref() {
        content_version_ids.retain(|id| id != modpack_version_id);
    }
    let icon_url = modpack
        .as_ref()
        .and_then(|modpack| modpack.icon_url.clone());

    let external_files = version
        .external_files
        .iter()
        .map(|file| SharedInstanceExternalFilePreview {
            file_name: file.file_name.clone(),
            file_type: file.file_type.clone(),
        })
        .collect::<Vec<_>>();
    let external_file_count = external_files.len();

    Ok(SharedInstanceInstallPreview {
        name,
        icon_url,
        game_version: version.game_version,
        loader: version.loader,
        mod_count: modpack_dependency_count
            + content_version_ids.len()
            + external_file_count,
        external_file_count,
        modpack_version_id,
        content_version_ids,
        external_files,
    })
}

#[tracing::instrument]
pub async fn get_shared_instance_update_preview(
    instance_id: &str,
) -> crate::Result<Option<SharedInstanceUpdatePreview>> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let Some(attachment) = metadata.shared_instance.clone() else {
        return Ok(None);
    };
    if !attachment.role.is_member() {
        return Ok(None);
    }

    let version =
        get_latest_remote_member_version(instance_id, &attachment, &state)
            .await?;
    if !version.ready {
        return Ok(Some(SharedInstanceUpdatePreview {
            shared_instance_id: attachment.id,
            current_version: attachment.applied_version,
            latest_version: version.version,
            update_available: false,
            diffs: Vec::new(),
        }));
    }

    let update_available = attachment
        .applied_version
        .is_none_or(|current| current < version.version);
    let diffs = if update_available {
        shared_instance_update_diffs(&metadata, &version, &state).await?
    } else {
        Vec::new()
    };

    Ok(Some(SharedInstanceUpdatePreview {
        shared_instance_id: attachment.id,
        current_version: attachment.applied_version,
        latest_version: version.version,
        update_available,
        diffs,
    }))
}

#[tracing::instrument]
pub async fn update_shared_instance(
    instance_id: &str,
) -> crate::Result<InstallJobSnapshot> {
    let state = State::get().await?;
    let metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;
    let attachment = metadata.shared_instance.clone().ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Instance is not attached to a shared instance".to_string(),
        )
    })?;
    if !attachment.role.is_member() {
        return Err(crate::ErrorKind::InputError(
            "Only shared instance members can update from shared instances"
                .to_string(),
        )
        .into());
    }

    let version =
        get_latest_remote_member_version(instance_id, &attachment, &state)
            .await?;
    let data = shared_instance_install_data(
        &attachment.id,
        attachment.manager_id.clone(),
        attachment.server_manager_name.clone(),
        attachment.server_manager_icon_url.clone(),
        None,
        metadata.instance.name,
        version,
        &state,
    )
    .await?;
    crate::install::update_shared_instance(instance_id.to_string(), data).await
}

pub(super) async fn ensure_ready_remote_version_for_invite(
    instance_id: &str,
    attachment: &SharedInstanceAttachment,
    state: &State,
) -> crate::Result<()> {
    match get_latest_remote_version_optional_unavailable(&attachment.id, state)
        .await?
    {
        SharedInstanceRemoteResponse::Available(_) => Ok(()),
        SharedInstanceRemoteResponse::Unavailable(
            SharedInstanceUnavailableReason::Deleted,
        ) => {
            tracing::info!(
                instance_id,
                shared_instance_id = %attachment.id,
                "Shared instance has no ready version before invite; publishing current content"
            );
            publish_shared_instance_inner(instance_id, &[], state).await
        }
        SharedInstanceRemoteResponse::Unavailable(reason) => {
            clear_shared_instance_if_current_user(
                instance_id,
                attachment,
                state,
            )
            .await?;
            Err(shared_instance_unavailable_error(reason))
        }
    }
}

pub(super) async fn get_latest_remote_member_version(
    instance_id: &str,
    attachment: &SharedInstanceAttachment,
    state: &State,
) -> crate::Result<InstanceVersionResponse> {
    if let SharedInstanceRemoteResponse::Unavailable(reason) =
        get_remote_instance_access(&attachment.id, state).await?
    {
        if shared_attachment_matches_current_user(attachment, state).await? {
            clear_shared_instance_if_current_user(
                instance_id,
                attachment,
                state,
            )
            .await?;
            return Err(shared_instance_unavailable_error(reason));
        }

        return Err(crate::ErrorKind::OtherError(format!(
            "{}: {}",
            shared_instance_unavailable_message(reason),
            attachment.id
        ))
        .into());
    }

    let version = match get_latest_remote_version_optional_unavailable(
        &attachment.id,
        state,
    )
    .await?
    {
        SharedInstanceRemoteResponse::Available(version) => version,
        SharedInstanceRemoteResponse::Unavailable(reason) => {
            if shared_attachment_matches_current_user(attachment, state).await?
            {
                clear_shared_instance_if_current_user(
                    instance_id,
                    attachment,
                    state,
                )
                .await?;
                return Err(shared_instance_unavailable_error(reason));
            }

            return Err(crate::ErrorKind::OtherError(format!(
                "{}: {}",
                shared_instance_unavailable_message(reason),
                attachment.id
            ))
            .into());
        }
    };

    Ok(version)
}

pub(super) async fn shared_attachment_matches_current_user(
    attachment: &SharedInstanceAttachment,
    state: &State,
) -> crate::Result<bool> {
    let Some(linked_user_id) = attachment.linked_user_id.as_deref() else {
        return Ok(false);
    };

    Ok(linked_modrinth_user_id(state)
        .await?
        .as_deref()
        .is_some_and(|current_user_id| current_user_id == linked_user_id))
}

pub(super) async fn has_shared_instance_recipients(
    users: &SharedInstanceUsers,
    attachment: &SharedInstanceAttachment,
    state: &State,
) -> crate::Result<bool> {
    if users.tokens > 0 {
        return Ok(true);
    }

    let current_user_id = linked_modrinth_user_id(state).await?;

    Ok(users.user_ids.iter().any(|user_id| {
        Some(user_id.as_str()) != attachment.linked_user_id.as_deref()
            && Some(user_id.as_str()) != attachment.manager_id.as_deref()
            && Some(user_id.as_str()) != current_user_id.as_deref()
    }))
}

pub(super) async fn clear_shared_instance_if_current_user(
    instance_id: &str,
    attachment: &SharedInstanceAttachment,
    state: &State,
) -> crate::Result<()> {
    if shared_attachment_matches_current_user(attachment, state).await? {
        crate::state::clear_shared_instance(instance_id, &state.pool).await?;
        emit_instance(instance_id, InstancePayloadType::Edited).await?;
    }

    Ok(())
}

pub(super) fn shared_instance_unavailable_error(
    reason: SharedInstanceUnavailableReason,
) -> crate::Error {
    crate::ErrorKind::SharedInstanceUnavailable(reason).into()
}

pub(super) fn shared_instance_unavailable_message(
    reason: SharedInstanceUnavailableReason,
) -> &'static str {
    match reason {
        SharedInstanceUnavailableReason::Deleted => {
            "Shared instance was deleted"
        }
        SharedInstanceUnavailableReason::AccessRevoked => {
            "Shared instance access was revoked"
        }
    }
}

fn shared_instance_external_file_data(
    file: ExternalFileResponse,
) -> crate::Result<SharedInstanceExternalFileData> {
    let file_size = file.file_size.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Shared instance external file {} is missing its size",
            file.file_name
        ))
    })?;
    let file_size = u64::try_from(file_size).map_err(|_| {
        crate::ErrorKind::InputError(format!(
            "Shared instance external file {} has an invalid size",
            file.file_name
        ))
    })?;

    Ok(SharedInstanceExternalFileData {
        file_name: file.file_name,
        file_type: file.file_type,
        url: file.url,
        file_size,
    })
}

pub(super) async fn shared_instance_install_data(
    shared_instance_id: &str,
    manager_id: Option<String>,
    server_manager_name: Option<String>,
    server_manager_icon_url: Option<String>,
    instance_icon_url: Option<String>,
    name: String,
    version: InstanceVersionResponse,
    state: &State,
) -> crate::Result<SharedInstanceInstallData> {
    if !version.ready {
        return Err(crate::ErrorKind::InputError(
            "Shared instance version is not ready to install".to_string(),
        )
        .into());
    }

    let name = shared_instance_name(name);
    let linked_user_id = linked_modrinth_user_id(state).await?;
    let modpack = shared_instance_install_modpack(&version, state).await?;
    let modpack_version_id =
        modpack.as_ref().map(|modpack| modpack.version_id.as_str());
    let modrinth_ids = version
        .modrinth_ids
        .into_iter()
        .filter(|id| Some(id.as_str()) != modpack_version_id)
        .collect();

    Ok(SharedInstanceInstallData {
        shared_instance_id: shared_instance_id.to_string(),
        manager_id,
        server_manager_name,
        server_manager_icon_url,
        instance_icon_url,
        linked_user_id,
        name,
        version: version.version,
        modrinth_ids,
        external_files: version
            .external_files
            .into_iter()
            .map(shared_instance_external_file_data)
            .collect::<crate::Result<Vec<_>>>()?,
        modpack,
        game_version: version.game_version,
        loader: version.loader,
        loader_version: shared_instance_loader_version(version.loader_version),
    })
}

pub(super) async fn linked_modrinth_user_id(
    state: &State,
) -> crate::Result<Option<String>> {
    Ok(
        ModrinthCredentials::get_and_refresh(&state.pool, &state.api_semaphore)
            .await?
            .map(|credentials| credentials.user_id),
    )
}
