use crate::state::instances::{
    ContentSet, Instance, InstanceLaunchOverrides, InstanceLink,
    SharedInstanceAttachment, adapters::sqlite::instance_rows,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceMetadata {
    pub instance: Instance,
    pub applied_content_set: ContentSet,
    pub link: InstanceLink,
    pub shared_instance: Option<SharedInstanceAttachment>,
    #[serde(default)]
    pub quarantined: bool,
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
    let Some(record) =
        instance_rows::get_instance_metadata_by_id(instance_id, pool).await?
    else {
        return Ok(None);
    };
    let quarantined =
        instance_rows::is_instance_quarantined(instance_id, pool).await?;

    Ok(Some(instance_metadata(record, quarantined)))
}

pub(crate) async fn get_instances_metadata(
    instance_ids: &[&str],
    pool: &SqlitePool,
) -> crate::Result<Vec<InstanceMetadata>> {
    let records =
        instance_rows::get_instance_metadata_many(instance_ids, pool).await?;
    let quarantined_ids =
        instance_rows::get_quarantined_instance_ids(pool).await?;

    Ok(records
        .into_iter()
        .map(|record| {
            let quarantined = quarantined_ids.contains(&record.instance.id);
            instance_metadata(record, quarantined)
        })
        .collect())
}

pub(crate) async fn list_instances(
    pool: &SqlitePool,
) -> crate::Result<Vec<InstanceMetadata>> {
    let records = instance_rows::list_instance_metadata(pool).await?;
    let quarantined_ids =
        instance_rows::get_quarantined_instance_ids(pool).await?;

    Ok(records
        .into_iter()
        .map(|record| {
            let quarantined = quarantined_ids.contains(&record.instance.id);
            instance_metadata(record, quarantined)
        })
        .collect())
}

fn instance_metadata(
    record: instance_rows::InstanceMetadataRecord,
    quarantined: bool,
) -> InstanceMetadata {
    InstanceMetadata {
        instance: record.instance,
        applied_content_set: record.applied_content_set,
        link: record.link,
        shared_instance: record.shared_instance,
        quarantined,
        groups: record.groups,
        launch_overrides: record.launch_overrides,
    }
}
