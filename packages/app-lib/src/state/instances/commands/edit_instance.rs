use crate::state::instances::{
	Instance, InstanceLaunchOverrides, InstanceLink,
	adapters::sqlite::{content_rows, instance_rows},
};
use crate::state::{
	Hooks, LauncherFeatureVersion, MemorySettings, ModLoader,
	InstanceInstallStage, ReleaseChannel, WindowSize,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EditInstance {
	pub install_stage: Option<InstanceInstallStage>,
	pub launcher_feature_version: Option<LauncherFeatureVersion>,
	pub name: Option<String>,
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "serde_with::rust::double_option"
	)]
	pub icon_path: Option<Option<String>>,
	pub update_channel: Option<ReleaseChannel>,
	pub groups: Option<Vec<String>>,
	pub link: Option<InstanceLink>,
	pub launch_overrides: Option<InstanceLaunchOverridesPatch>,
	pub content_set_patch: Option<AppliedContentSetPatch>,
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "serde_with::rust::double_option"
	)]
	pub last_played: Option<Option<DateTime<Utc>>>,
	pub submitted_time_played: Option<u64>,
	pub recent_time_played: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InstanceLaunchOverridesPatch {
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "serde_with::rust::double_option"
	)]
	pub java_path: Option<Option<String>>,
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "serde_with::rust::double_option"
	)]
	pub extra_launch_args: Option<Option<Vec<String>>>,
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "serde_with::rust::double_option"
	)]
	pub custom_env_vars: Option<Option<Vec<(String, String)>>>,
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "serde_with::rust::double_option"
	)]
	pub memory: Option<Option<MemorySettings>>,
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "serde_with::rust::double_option"
	)]
	pub force_fullscreen: Option<Option<bool>>,
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "serde_with::rust::double_option"
	)]
	pub game_resolution: Option<Option<WindowSize>>,
	pub hooks: Option<Hooks>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppliedContentSetPatch {
	pub game_version: Option<String>,
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "serde_with::rust::double_option"
	)]
	pub protocol_version: Option<Option<u32>>,
	pub loader: Option<ModLoader>,
	#[serde(
		default,
		skip_serializing_if = "Option::is_none",
		with = "serde_with::rust::double_option"
	)]
	pub loader_version: Option<Option<String>>,
}

pub(crate) async fn edit_instance(
	instance_id: &str,
	patch: EditInstance,
	pool: &SqlitePool,
) -> crate::Result<Instance> {
	let mut instance = instance_rows::get_instance_by_id(instance_id, pool)
		.await?
		.ok_or_else(|| {
			crate::ErrorKind::InputError("Unknown instance".to_string())
		})?;
	let now = Utc::now();

	apply_instance_patch(&mut instance, &patch, now);

	let mut content_set = match patch.content_set_patch {
		Some(content_set_patch) => {
			let applied_content_set =
				content_rows::get_applied_content_set(&instance.id, pool)
					.await?
					.ok_or_else(|| {
						crate::ErrorKind::InputError(format!(
							"Instance {} has no applied content set",
							instance.id
						))
					})?;
			Some(apply_content_set_patch(
				applied_content_set,
				content_set_patch,
				now,
			))
		}
		None => None,
	};
	let mut launch_overrides = match patch.launch_overrides {
		Some(launch_patch) => {
			let current = instance_rows::get_instance_launch_overrides(
				&instance.id,
				pool,
			)
			.await?
			.unwrap_or_else(|| {
				InstanceLaunchOverrides::empty(instance.id.clone())
			});
			Some(apply_launch_overrides_patch(current, launch_patch))
		}
		None => None,
	};

	let mut tx = pool.begin().await?;
	instance_rows::update_instance(&instance, &mut tx).await?;

	if let Some(content_set) = content_set.as_mut() {
		content_rows::update_content_set(content_set, &mut tx).await?;
	}

	if let Some(link) = &patch.link {
		instance_rows::upsert_instance_link(&instance.id, link, &mut tx)
			.await?;
	}

	if let Some(groups) = &patch.groups {
		instance_rows::replace_instance_groups(&instance.id, groups, &mut tx)
			.await?;
	}

	if let Some(overrides) = launch_overrides.as_mut() {
		instance_rows::upsert_instance_launch_overrides(overrides, &mut tx)
			.await?;
	}

	tx.commit().await?;

	Ok(instance)
}

fn apply_instance_patch(
	instance: &mut Instance,
	patch: &EditInstance,
	now: DateTime<Utc>,
) {
	if let Some(install_stage) = patch.install_stage {
		instance.install_stage = install_stage;
	}
	if let Some(launcher_feature_version) = patch.launcher_feature_version {
		instance.launcher_feature_version = launcher_feature_version;
	}
	if let Some(name) = &patch.name {
		instance.name = name.clone();
	}
	if let Some(icon_path) = &patch.icon_path {
		instance.icon_path = icon_path.clone();
	}
	if let Some(update_channel) = patch.update_channel {
		instance.update_channel = update_channel;
	}
	if let Some(last_played) = &patch.last_played {
		instance.last_played = *last_played;
	}
	if let Some(submitted_time_played) = patch.submitted_time_played {
		instance.submitted_time_played = submitted_time_played;
	}
	if let Some(recent_time_played) = patch.recent_time_played {
		instance.recent_time_played = recent_time_played;
	}

	instance.modified = now;
}

fn apply_content_set_patch(
	mut content_set: crate::state::instances::ContentSet,
	patch: AppliedContentSetPatch,
	now: DateTime<Utc>,
) -> crate::state::instances::ContentSet {
	if let Some(game_version) = patch.game_version {
		content_set.game_version = game_version;
	}
	if let Some(protocol_version) = patch.protocol_version {
		content_set.protocol_version = protocol_version;
	}
	if let Some(loader) = patch.loader {
		content_set.loader = loader;
	}
	if let Some(loader_version) = patch.loader_version {
		content_set.loader_version = loader_version;
	}

	content_set.modified = now;
	content_set
}

fn apply_launch_overrides_patch(
	mut overrides: InstanceLaunchOverrides,
	patch: InstanceLaunchOverridesPatch,
) -> InstanceLaunchOverrides {
	if let Some(java_path) = patch.java_path {
		overrides.java_path = java_path;
	}
	if let Some(extra_launch_args) = patch.extra_launch_args {
		overrides.extra_launch_args = extra_launch_args;
	}
	if let Some(custom_env_vars) = patch.custom_env_vars {
		overrides.custom_env_vars = custom_env_vars;
	}
	if let Some(memory) = patch.memory {
		overrides.memory = memory;
	}
	if let Some(force_fullscreen) = patch.force_fullscreen {
		overrides.force_fullscreen = force_fullscreen;
	}
	if let Some(game_resolution) = patch.game_resolution {
		overrides.game_resolution = game_resolution;
	}
	if let Some(hooks) = patch.hooks {
		overrides.hooks = hooks;
	}

	overrides
}
