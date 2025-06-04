use crate::bitflags_serde_impl;
use crate::database::models::shared_instance_item::{
    DBSharedInstance, DBSharedInstanceUser, DBSharedInstanceVersion,
};
use crate::models::ids::{SharedInstanceId, SharedInstanceVersionId};
use ariadne::ids::UserId;
use bitflags::bitflags;
use hex::ToHex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedInstance {
    pub id: SharedInstanceId,
    pub title: String,
    pub owner: UserId,
    pub current_version: Option<SharedInstanceVersion>,
    pub additional_users: Vec<SharedInstanceUser>,
}

impl SharedInstance {
    pub fn from_db(
        instance: DBSharedInstance,
        users: Vec<DBSharedInstanceUser>,
        current_version: Option<DBSharedInstanceVersion>,
        cdn_url: &str,
    ) -> Self {
        SharedInstance {
            id: instance.id.into(),
            title: instance.title,
            owner: instance.owner_id.into(),
            current_version: current_version
                .map(|x| SharedInstanceVersion::from_db(x, cdn_url)),
            additional_users: users.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedInstanceVersion {
    pub id: SharedInstanceVersionId,
    pub shared_instance: SharedInstanceId,
    pub size: u64,
    pub sha512: String,
    pub url: String,
}

impl SharedInstanceVersion {
    pub fn from_db(version: DBSharedInstanceVersion, cdn_url: &str) -> Self {
        let version_id = version.id.into();
        let shared_instance_id = version.shared_instance_id.into();
        SharedInstanceVersion {
            id: version_id,
            shared_instance: shared_instance_id,
            size: version.size,
            sha512: version.sha512.encode_hex(),
            url: format!(
                "{cdn_url}/shared_instance/{shared_instance_id}/{version_id}.mrpack"
            ),
        }
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct SharedInstanceUserPermissions: u64 {
        const EDIT = 1 << 0;
        const DELETE = 1 << 1;
        const UPLOAD_VERSION = 1 << 2;
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
