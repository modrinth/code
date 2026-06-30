use crate::state::instances::{
    ContentSet, Instance, InstanceLaunchOverrides, InstanceLink,
    SharedInstanceAttachment,
    adapters::sqlite::instance_rows,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceMetadata {
    pub instance: Instance,
    pub applied_content_set: ContentSet,
    pub link: InstanceLink,
    pub shared_instance: Option<SharedInstanceAttachment>,
    pub groups: Vec<String>,
    pub launch_overrides: InstanceLaunchOverrides,
}

pub(crate) async fn get_instance(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<Option<InstanceMetadata>> {
    get_instance_metadata(instance_id, pool).await
}

pub(crate) async fn get_instance_metadata(
    instance_id: &str,
    pool: &SqlitePool,
) -> crate::Result<Option<InstanceMetadata>> {
    Ok(
        instance_rows::get_instance_metadata_by_id(instance_id, pool)
            .await?
            .map(Into::into),
    )
}

pub(crate) async fn get_instances_metadata(
    instance_ids: &[&str],
    pool: &SqlitePool,
) -> crate::Result<Vec<InstanceMetadata>> {
    Ok(
        instance_rows::get_instance_metadata_many(instance_ids, pool)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    )
}

pub(crate) async fn list_instances(
    pool: &SqlitePool,
) -> crate::Result<Vec<InstanceMetadata>> {
    Ok(instance_rows::list_instance_metadata(pool)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

impl From<instance_rows::InstanceMetadataRecord> for InstanceMetadata {
    fn from(record: instance_rows::InstanceMetadataRecord) -> Self {
        Self {
            instance: record.instance,
            applied_content_set: record.applied_content_set,
            link: record.link,
            shared_instance: record.shared_instance,
            groups: record.groups,
            launch_overrides: record.launch_overrides,
        }
    }
}
