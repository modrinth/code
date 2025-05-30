use crate::database::models::DBUserId;
use crate::database::models::shared_instance_item::DBSharedInstance;
use crate::models::ids::{FileId, SharedInstanceId, SharedInstanceVersionId};
use ariadne::ids::UserId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedInstance {
    pub id: SharedInstanceId,
    pub title: String,
    pub owner: UserId,
    pub current_version: Option<SharedInstanceVersionId>,
    pub additional_users: Vec<UserId>,
}

impl SharedInstance {
    pub fn from_db(instance: DBSharedInstance, users: Vec<DBUserId>) -> Self {
        SharedInstance {
            id: instance.id.into(),
            title: instance.title,
            owner: instance.owner_id.into(),
            current_version: instance.current_version_id.map(Into::into),
            additional_users: users.into_iter().map(Into::into).collect(),
        }
    }
}

pub struct SharedInstanceVersion {
    pub id: SharedInstanceVersionId,
    pub file: FileId,
    pub instance: SharedInstanceId,
}
