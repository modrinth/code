use super::content_set_diff::{
    ContentSetDiffEntry, ContentSetDiffKind, ContentSetDiffOptions,
    ContentSetSnapshot, ContentSetSnapshotVersion, diff_content_sets,
};
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
use crate::SharedInstanceUnavailableReason;
use chrono::{DateTime, Utc};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};

mod client;
mod diff;
mod install;
mod invites;
mod publish;
mod types;

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
