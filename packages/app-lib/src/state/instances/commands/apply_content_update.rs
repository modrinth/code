use crate::state::instances::ContentSourceKind;
use crate::state::{CacheBehaviour, State};
use crate::util::fetch::DownloadReason;
use std::collections::HashMap;

use super::apply_content_install::{
	add_project_from_version, remove_project, toggle_disable_project,
};
use super::check_content_updates::check_content_updates;

pub(crate) async fn update_project(
	instance_id: &str,
	project_path: &str,
	state: &State,
) -> crate::Result<String> {
	let updates =
		check_content_updates(instance_id, Some(CacheBehaviour::MustRevalidate), state)
			.await?;
	let update = updates
		.into_iter()
		.find(|update| update.relative_path == project_path)
		.ok_or_else(|| {
			crate::ErrorKind::InputError(
				"This project cannot be updated!".to_string(),
			)
		})?;
	let mut new_path = add_project_from_version(
		instance_id,
		&update.update_version_id,
		DownloadReason::Update,
		Some(update.current_version_id),
		ContentSourceKind::Local,
		state,
	)
	.await?;

	if project_path.ends_with(".disabled") {
		new_path = toggle_disable_project(instance_id, &new_path, state).await?;
	}

	if new_path != project_path {
		remove_project(instance_id, project_path, state).await?;
	}

	Ok(new_path)
}

pub(crate) async fn update_all_projects(
	instance_id: &str,
	state: &State,
) -> crate::Result<HashMap<String, String>> {
	let updates =
		check_content_updates(instance_id, Some(CacheBehaviour::MustRevalidate), state)
			.await?;
	let mut changed = HashMap::new();

	for update in updates {
		let new_path = update_project(instance_id, &update.relative_path, state).await?;
		changed.insert(update.relative_path, new_path);
	}

	Ok(changed)
}
