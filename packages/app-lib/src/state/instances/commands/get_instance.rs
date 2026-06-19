use crate::state::instances::{
	ContentSet, Instance, InstanceLaunchOverrides, InstanceLink,
	adapters::sqlite::{content_rows, instance_rows},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceMetadata {
	pub instance: Instance,
	pub applied_content_set: ContentSet,
	pub link: InstanceLink,
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
	let Some(instance) = instance_rows::get_instance_by_id(instance_id, pool).await?
	else {
		return Ok(None);
	};

	instance_metadata(instance, pool).await.map(Some)
}

pub(crate) async fn list_instances(
	pool: &SqlitePool,
) -> crate::Result<Vec<InstanceMetadata>> {
	let instances = instance_rows::list_instances(pool).await?;
	let mut metadata = Vec::with_capacity(instances.len());

	for instance in instances {
		metadata.push(instance_metadata(instance, pool).await?);
	}

	Ok(metadata)
}

async fn instance_metadata(
	instance: Instance,
	pool: &SqlitePool,
) -> crate::Result<InstanceMetadata> {
	let applied_content_set =
		content_rows::get_applied_content_set(&instance.id, pool)
			.await?
			.ok_or_else(|| {
				crate::ErrorKind::InputError(format!(
					"Instance {} has no applied content set",
					instance.id
				))
			})?;
	let link = instance_rows::get_instance_link(&instance.id, pool).await?;
	let groups =
		instance_rows::get_instance_groups(&instance.id, pool).await?;
	let launch_overrides =
		instance_rows::get_instance_launch_overrides(&instance.id, pool)
			.await?
			.unwrap_or_else(|| {
				InstanceLaunchOverrides::empty(instance.id.clone())
			});

	Ok(InstanceMetadata {
		instance,
		applied_content_set,
		link,
		groups,
		launch_overrides,
	})
}
