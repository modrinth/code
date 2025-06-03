use crate::database::models::DBUserId;
use crate::database::models::shared_instance_item::{
    DBSharedInstance, DBSharedInstanceVersion,
};
use crate::models::ids::{SharedInstanceId, SharedInstanceVersionId};
use ariadne::ids::UserId;
use hex::ToHex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedInstance {
    pub id: SharedInstanceId,
    pub title: String,
    pub owner: UserId,
    pub current_version: Option<SharedInstanceVersion>,
    pub additional_users: Vec<UserId>,
}

impl SharedInstance {
    pub fn from_db(
        instance: DBSharedInstance,
        users: Vec<DBUserId>,
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
