use super::content_set_diff::{
    ContentSetDiffEntry, ContentSetDiffKind, ContentSetDiffOptions,
    ContentSetSnapshot, ContentSetSnapshotVersion, diff_content_sets,
};
use crate::SharedInstanceUnavailableReason;
use crate::event::InstancePayloadType;
use crate::event::emit::emit_instance;
use crate::install::{
    InstallJobSnapshot, SharedInstanceExternalFileData,
    SharedInstanceInstallData,
};
use crate::state::instances::{InstanceLink, SharedInstanceAttachment};
use crate::state::{
    AppliedContentSetPatch, CacheBehaviour, CachedEntry, ContentSetSyncStatus,
    ContentSourceKind, EditInstance, ModLoader, ModrinthCredentials,
    ProjectType, SharedInstanceRole, State,
};
use crate::util::fetch::{INSECURE_REQWEST_CLIENT, REQWEST_CLIENT};
use chrono::{DateTime, Utc};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::io::Read;

pub(crate) const CONFIG_BUNDLE_FILE_NAME: &str = "configs.zip";
pub(crate) const CONFIG_BUNDLE_FILE_TYPE: &str = "configs";
pub(crate) const CONFIG_SYNC_ENABLED: bool = true;
pub(crate) const CONFIG_DIRECTORY: &str = "config";
pub(crate) const MAX_CONFIG_BUNDLE_ENTRIES: usize = 4096;
pub(crate) const MAX_CONFIG_BUNDLE_FILE_SIZE: u64 = 16 * 1024 * 1024;
pub(crate) const MAX_CONFIG_BUNDLE_TOTAL_SIZE: u64 = 128 * 1024 * 1024;
pub(crate) const CONFIG_FILE_EXTENSIONS: [&str; 13] = [
    "json",
    "json5",
    "jsonc",
    "yml",
    "yaml",
    "css",
    "toml",
    "txt",
    "ini",
    "cfg",
    "conf",
    "properties",
    "xml",
];

pub(crate) fn read_bounded_config_bundle_entry(
    reader: impl Read,
    declared_size: u64,
    total_size: &mut u64,
) -> crate::Result<Vec<u8>> {
    let remaining = MAX_CONFIG_BUNDLE_TOTAL_SIZE.saturating_sub(*total_size);
    let entry_limit = MAX_CONFIG_BUNDLE_FILE_SIZE.min(remaining);
    if declared_size > entry_limit {
        return Err(crate::ErrorKind::InputError(
            "Shared instance config bundle exceeds the uncompressed size limit"
                .to_string(),
        )
        .into());
    }

    let capacity = usize::try_from(declared_size).map_err(|_| {
        crate::ErrorKind::InputError(
            "Shared instance config bundle entry is too large".to_string(),
        )
    })?;
    let mut bytes = Vec::with_capacity(capacity);
    reader
        .take(entry_limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| {
            crate::ErrorKind::OtherError(format!(
                "Failed to read shared instance config bundle: {error}"
            ))
        })?;
    let actual_size = bytes.len() as u64;
    if actual_size > entry_limit {
        return Err(crate::ErrorKind::InputError(
            "Shared instance config bundle exceeds the uncompressed size limit"
                .to_string(),
        )
        .into());
    }
    *total_size = total_size.checked_add(actual_size).ok_or_else(|| {
        crate::ErrorKind::InputError(
            "Shared instance config bundle size overflowed".to_string(),
        )
    })?;

    Ok(bytes)
}

mod client;
mod diff;
mod install;
mod invites;
mod publish;
mod types;

pub(crate) use self::install::check_shared_instance_availability_before_launch;
pub(crate) use self::publish::sync_shared_instance_icon;

pub use self::install::{
    accept_shared_instance_invite_for_install,
    get_shared_instance_install_preview, get_shared_instance_update_preview,
    install_shared_instance, update_shared_instance,
};
pub use self::invites::{
    accept_pending_shared_instance_invite, create_shared_instance_invite_link,
    decline_pending_shared_instance_invite, get_shared_instance_users,
    invite_shared_instance_users, remove_shared_instance_users,
};
pub use self::publish::{
    get_shared_instance_publish_preview, publish_shared_instance,
    unlink_shared_instance, unpublish_shared_instance,
};
pub use self::types::{
    SharedInstanceExternalFilePreview, SharedInstanceInstallPreview,
    SharedInstanceInviteInstallPreview, SharedInstanceInviteLink,
    SharedInstanceJoinType, SharedInstancePublishPreview,
    SharedInstanceUpdateDiff, SharedInstanceUpdateDiffType,
    SharedInstanceUpdatePreview, SharedInstanceUser, SharedInstanceUsers,
};

pub async fn can_active_user_use_shared_instances() -> crate::Result<bool> {
    let state = State::get().await?;
    let Some(credentials) =
        ModrinthCredentials::get_and_refresh(&state.pool, &state.api_semaphore)
            .await?
    else {
        return Ok(true);
    };

    let status =
        client::get_user_blacklist_status(&credentials.user_id, &state).await?;
    Ok(!status.blacklisted)
}
