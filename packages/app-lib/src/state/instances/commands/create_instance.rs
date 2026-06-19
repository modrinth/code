use crate::launcher::get_loader_version_from_profile;
use crate::state::instances::{
	ContentSet, ContentSetStatus, ContentSourceKind, Instance,
	InstanceLaunchOverrides, InstanceLink,
	adapters::sqlite::{content_rows, instance_rows},
};
use crate::state::{
	LauncherFeatureVersion, ModLoader, InstanceInstallStage, ReleaseChannel,
	State,
};
use crate::util::fetch::{self, write_cached_icon};
use crate::util::io;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{info, trace};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateInstance {
	pub name: String,
	pub path: Option<String>,
	pub game_version: String,
	pub loader: ModLoader,
	pub loader_version: Option<String>,
	pub icon_path: Option<String>,
	pub link: InstanceLink,
}

pub(crate) async fn create_instance(
	input: CreateInstance,
	state: &State,
) -> crate::Result<Instance> {
	trace!("Creating new instance. {}", input.name);

	let (path, full_path) =
		resolve_instance_path(&input.name, input.path.as_deref(), state).await?;
	io::create_dir_all(&full_path).await?;

	let result = async {
		info!(
			"Creating instance at path {}",
			&io::canonicalize(&full_path)?.display()
		);

		let loader_version = if input.loader != ModLoader::Vanilla {
			get_loader_version_from_profile(
				&input.game_version,
				input.loader,
				input.loader_version.as_deref(),
			)
			.await?
			.map(|value| value.id)
		} else {
			None
		};

		let icon_path = resolve_icon_path(input.icon_path.as_deref(), state)
			.await?;
		let now = Utc::now();
		let instance_id = format!("local:{}", Uuid::new_v4());
		let content_set_id = format!("content-set:{}", Uuid::new_v4());
		let instance = Instance {
			id: instance_id.clone(),
			path: path.clone(),
			applied_content_set_id: Some(content_set_id.clone()),
			install_stage: InstanceInstallStage::NotInstalled,
			launcher_feature_version: LauncherFeatureVersion::MOST_RECENT,
			update_channel: ReleaseChannel::Release,
			name: input.name,
			icon_path,
			created: now,
			modified: now,
			last_played: None,
			submitted_time_played: 0,
			recent_time_played: 0,
		};
		let content_set = ContentSet {
			id: content_set_id,
			instance_id: instance_id.clone(),
			name: "Default".to_string(),
			source_kind: content_source_kind(&input.link),
			status: ContentSetStatus::Available,
			game_version: input.game_version,
			protocol_version: None,
			loader: input.loader,
			loader_version,
			created: now,
			modified: now,
		};
		let launch_overrides =
			InstanceLaunchOverrides::empty(instance_id.clone());

		let mut tx = state.pool.begin().await?;
		instance_rows::insert_instance(&instance, &mut tx).await?;
		content_rows::insert_content_set(&content_set, &mut tx).await?;
		instance_rows::upsert_instance_link(&instance_id, &input.link, &mut tx)
			.await?;
		instance_rows::replace_instance_groups(&instance_id, &[], &mut tx)
			.await?;
		instance_rows::upsert_instance_launch_overrides(
			&launch_overrides,
			&mut tx,
		)
		.await?;
		tx.commit().await?;

		crate::state::instances::watcher::watch_instance_folder(
			&instance.path,
			&state.file_watcher,
			&state.directories,
		)
		.await;

		Ok(instance)
	}
	.await;

	if result.is_err() {
		let _ = io::remove_dir_all(&full_path).await;
	}

	result
}

async fn resolve_instance_path(
	name: &str,
	path: Option<&str>,
	state: &State,
) -> crate::Result<(String, std::path::PathBuf)> {
	let base_path = path
		.map(ToOwned::to_owned)
		.unwrap_or_else(|| sanitize_instance_name(name));
	let mut path = base_path.clone();
	let mut full_path = state.directories.instances_dir().join(&path);

	if path_available(&path, &full_path, state).await? {
		return Ok((path, full_path));
	}

	let mut which = 1;
	loop {
		path = format!("{base_path} ({which})");
		full_path = state.directories.instances_dir().join(&path);

		if path_available(&path, &full_path, state).await? {
			return Ok((path, full_path));
		}

		which += 1;
	}
}

async fn path_available(
	path: &str,
	full_path: &std::path::Path,
	state: &State,
) -> crate::Result<bool> {
	if full_path.exists() {
		return Ok(false);
	}

	Ok(instance_rows::get_instance_by_path(path, &state.pool)
		.await?
		.is_none())
}

async fn resolve_icon_path(
	icon_path: Option<&str>,
	state: &State,
) -> crate::Result<Option<String>> {
	let Some(icon) = icon_path else {
		return Ok(None);
	};

	let (bytes, file_name) =
		if icon.starts_with("https://") || icon.starts_with("http://") {
			let fetched = fetch::fetch(
				icon,
				None,
				None,
				&state.fetch_semaphore,
				&state.pool,
			)
			.await?;
			let name = icon.rsplit('/').next().unwrap_or("icon").to_string();
			(fetched, name)
		} else {
			let data = io::read(state.directories.caches_dir().join(icon)).await?;
			(bytes::Bytes::from(data), icon.to_string())
		};

	let file = write_cached_icon(
		&file_name,
		&state.directories.caches_dir(),
		bytes,
		&state.io_semaphore,
	)
	.await?;

	Ok(Some(file.to_string_lossy().to_string()))
}

fn content_source_kind(link: &InstanceLink) -> ContentSourceKind {
	match link {
		InstanceLink::Unmanaged => ContentSourceKind::Local,
		InstanceLink::ModrinthModpack { .. } => {
			ContentSourceKind::ModrinthModpack
		}
		InstanceLink::ServerProject { .. }
		| InstanceLink::ServerProjectModpack { .. } => {
			ContentSourceKind::ServerProject
		}
		InstanceLink::ModrinthHosting { .. } => {
			ContentSourceKind::ModrinthHosting
		}
		InstanceLink::ImportedModpack { .. } => {
			ContentSourceKind::ImportedModpack
		}
		InstanceLink::SharedInstance { .. } => {
			ContentSourceKind::SharedInstance
		}
	}
}

fn sanitize_instance_name(input: &str) -> String {
	input.replace(
		['/', '\\', '?', '*', ':', '\'', '\"', '|', '<', '>', '!'],
		"_",
	)
}
