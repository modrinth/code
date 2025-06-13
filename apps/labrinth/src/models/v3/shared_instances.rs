use crate::bitflags_serde_impl;
use crate::database::models::shared_instance_item::{
    DBSharedInstance, DBSharedInstanceUser, DBSharedInstanceVersion,
};
use crate::models::ids::{SharedInstanceId, SharedInstanceVersionId};
use ariadne::ids::UserId;
use bitflags::bitflags;
use chrono::{DateTime, Utc};
use hex::ToHex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedInstance {
    pub id: SharedInstanceId,
    pub title: String,
    pub owner: UserId,
    pub public: bool,
    pub current_version: Option<SharedInstanceVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_users: Option<Vec<SharedInstanceUser>>,
}

impl SharedInstance {
    pub fn from_db(
        instance: DBSharedInstance,
        users: Option<Vec<DBSharedInstanceUser>>,
        current_version: Option<DBSharedInstanceVersion>,
    ) -> Self {
        SharedInstance {
            id: instance.id.into(),
            title: instance.title,
            owner: instance.owner_id.into(),
            public: instance.public,
            current_version: current_version.map(Into::into),
            additional_users: users
                .map(|x| x.into_iter().map(Into::into).collect()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedInstanceVersion {
    pub id: SharedInstanceVersionId,
    pub shared_instance: SharedInstanceId,
    pub size: u64,
    pub sha512: String,
    pub created: DateTime<Utc>,
}

impl From<DBSharedInstanceVersion> for SharedInstanceVersion {
    fn from(value: DBSharedInstanceVersion) -> Self {
        let version_id = value.id.into();
        let shared_instance_id = value.shared_instance_id.into();
        SharedInstanceVersion {
            id: version_id,
            shared_instance: shared_instance_id,
            size: value.size,
            sha512: value.sha512.encode_hex(),
            created: value.created,
        }
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct SharedInstanceUserPermissions: u64 {
        const EDIT = 1 << 0;
        const DELETE = 1 << 1;
        const UPLOAD_VERSION = 1 << 2;
        const DELETE_VERSION = 1 << 3;
    }
}

bitflags_serde_impl!(SharedInstanceUserPermissions, u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedInstanceUser {
    pub user: UserId,
    pub permissions: SharedInstanceUserPermissions,
}

impl From<DBSharedInstanceUser> for SharedInstanceUser {
    fn from(user: DBSharedInstanceUser) -> Self {
        SharedInstanceUser {
            user: user.user_id.into(),
            permissions: user.permissions,
        }
    }
}
