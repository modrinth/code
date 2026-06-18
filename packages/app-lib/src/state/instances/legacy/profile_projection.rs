use crate::state::{
    ContentSet, Hooks, Instance, InstanceLaunchOverrides, InstanceLink,
    LinkedData, Profile,
};
use sqlx::SqlitePool;

use super::super::adapters::sqlite::{content_rows, instance_rows};

pub(crate) async fn get_profile_projection_by_path(
    path: &str,
    pool: &SqlitePool,
) -> crate::Result<Option<Profile>> {
    let Some(instance) =
        instance_rows::get_instance_by_path(path, pool).await?
    else {
        return Ok(None);
    };

    project_profile(instance, pool).await.map(Some)
}

pub(crate) async fn get_profile_projections_by_paths(
    paths: &[&str],
    pool: &SqlitePool,
) -> crate::Result<Vec<Profile>> {
    let mut profiles = Vec::with_capacity(paths.len());

    for path in paths {
        if let Some(profile) =
            get_profile_projection_by_path(path, pool).await?
        {
            profiles.push(profile);
        }
    }

    Ok(profiles)
}

pub(crate) async fn list_profile_projections(
    pool: &SqlitePool,
) -> crate::Result<Vec<Profile>> {
    let instances = instance_rows::list_instances(pool).await?;
    let mut profiles = Vec::with_capacity(instances.len());

    for instance in instances {
        profiles.push(project_profile(instance, pool).await?);
    }

    Ok(profiles)
}

pub(crate) async fn sync_profile_metadata(
    profile: &Profile,
    pool: &SqlitePool,
) -> crate::Result<()> {
    instance_rows::upsert_instance_from_profile(profile, pool).await
}

pub(crate) async fn delete_profile_metadata_by_path(
    path: &str,
    pool: &SqlitePool,
) -> crate::Result<()> {
    instance_rows::delete_instance_by_path(path, pool).await
}

async fn project_profile(
    instance: Instance,
    pool: &SqlitePool,
) -> crate::Result<Profile> {
    let content_set = content_rows::get_applied_content_set(&instance.id, pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "Instance {} has no applied content set",
                instance.path
            ))
            .as_error()
        })?;
    let link = instance_rows::get_instance_link(&instance.id, pool).await?;
    let groups = instance_rows::get_instance_groups(&instance.id, pool).await?;
    let launch_overrides =
        instance_rows::get_instance_launch_overrides(&instance.id, pool)
            .await?
            .unwrap_or_else(|| default_launch_overrides(&instance.id));

    Ok(profile_from_parts(instance, content_set, link, groups, launch_overrides))
}

fn profile_from_parts(
    instance: Instance,
    content_set: ContentSet,
    link: InstanceLink,
    groups: Vec<String>,
    launch_overrides: InstanceLaunchOverrides,
) -> Profile {
    Profile {
        path: instance.path,
        install_stage: instance.install_stage,
        launcher_feature_version: instance.launcher_feature_version,
        name: instance.name,
        icon_path: instance.icon_path,
        game_version: content_set.game_version,
        protocol_version: content_set.protocol_version,
        loader: content_set.loader,
        loader_version: content_set.loader_version,
        groups,
        linked_data: linked_data_from_link(link),
        preferred_update_channel: instance.update_channel,
        created: instance.created,
        modified: instance.modified,
        last_played: instance.last_played,
        submitted_time_played: instance.submitted_time_played,
        recent_time_played: instance.recent_time_played,
        java_path: launch_overrides.java_path,
        extra_launch_args: launch_overrides.extra_launch_args,
        custom_env_vars: launch_overrides.custom_env_vars,
        memory: launch_overrides.memory,
        force_fullscreen: launch_overrides.force_fullscreen,
        game_resolution: launch_overrides.game_resolution,
        hooks: launch_overrides.hooks,
    }
}

fn linked_data_from_link(link: InstanceLink) -> Option<LinkedData> {
    match link {
        InstanceLink::Unmanaged => None,
        InstanceLink::ModrinthModpack {
            project_id,
            version_id,
        } => Some(LinkedData {
            project_id,
            version_id,
            locked: true,
        }),
        InstanceLink::ServerProject { project_id } => Some(LinkedData {
            project_id,
            version_id: String::new(),
            locked: false,
        }),
        InstanceLink::ServerProjectModpack {
            server_project_id,
            content_version_id,
            ..
        } => Some(LinkedData {
            project_id: server_project_id,
            version_id: content_version_id,
            locked: false,
        }),
        InstanceLink::ImportedModpack {
            project_id: Some(project_id),
            version_id: Some(version_id),
        } => Some(LinkedData {
            project_id,
            version_id,
            locked: false,
        }),
        InstanceLink::ImportedModpack { .. }
        | InstanceLink::ModrinthHosting { .. }
        | InstanceLink::SharedInstance { .. } => None,
    }
}

fn default_launch_overrides(instance_id: &str) -> InstanceLaunchOverrides {
    InstanceLaunchOverrides {
        instance_id: instance_id.to_string(),
        java_path: None,
        extra_launch_args: None,
        custom_env_vars: None,
        memory: None,
        force_fullscreen: None,
        game_resolution: None,
        hooks: Hooks {
            pre_launch: None,
            wrapper: None,
            post_exit: None,
        },
    }
}
