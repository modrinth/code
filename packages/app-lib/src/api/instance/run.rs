use super::content::get_projects;
use crate::server_address::ServerAddress;
use crate::state::{
    Credentials, InstanceLink, ProcessMetadata, Settings, State,
};
use crate::util::fetch;
use crate::util::io::IOError;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub enum QuickPlayType {
    None,
    Singleplayer(String),
    Server(ServerAddress),
}

#[tracing::instrument]
pub async fn run(
    instance_id: &str,
    quick_play_type: QuickPlayType,
) -> crate::Result<ProcessMetadata> {
    let state = State::get().await?;
    if crate::state::instances::adapters::sqlite::instance_rows::is_instance_quarantined(
        instance_id,
        &state.pool,
    )
    .await?
    {
        return Err(crate::ErrorKind::InputError(
            "This instance has been quarantined".to_string(),
        )
        .into());
    }
    super::shared::check_shared_instance_availability_before_launch(
        instance_id,
        &state,
    )
    .await?;

    let default_account = Credentials::get_default_credential(&state.pool)
        .await?
        .ok_or_else(|| crate::ErrorKind::NoCredentialsError.as_error())?;

    run_credentials(instance_id, &default_account, quick_play_type).await
}

#[tracing::instrument(skip(credentials))]
async fn run_credentials(
    instance_id: &str,
    credentials: &Credentials,
    quick_play_type: QuickPlayType,
) -> crate::Result<ProcessMetadata> {
    let state = State::get().await?;
    let settings = Settings::get(&state.pool).await?;
    let context =
        crate::state::instances::commands::get_instance_launch_context(
            instance_id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "Tried to run a nonexistent instance {instance_id}!"
            ))
        })?;
    if crate::state::instances::adapters::sqlite::instance_rows::is_instance_quarantined(
        instance_id,
        &state.pool,
    )
    .await?
    {
        return Err(crate::ErrorKind::InputError(
            "This instance has been quarantined".to_string(),
        )
        .into());
    }

    let pre_launch_hook = context
        .launch_overrides
        .hooks
        .pre_launch
        .as_ref()
        .or(settings.hooks.pre_launch.as_ref())
        .filter(|hook_command| !hook_command.is_empty());

    let java_args = context
        .launch_overrides
        .extra_launch_args
        .clone()
        .unwrap_or(settings.extra_launch_args);

    let wrapper = context
        .launch_overrides
        .hooks
        .wrapper
        .clone()
        .or(settings.hooks.wrapper)
        .filter(|hook_command| !hook_command.is_empty());

    let env_args = context
        .launch_overrides
        .custom_env_vars
        .clone()
        .unwrap_or(settings.custom_env_vars);

    let post_exit_hook = context
        .launch_overrides
        .hooks
        .post_exit
        .clone()
        .or(settings.hooks.post_exit)
        .filter(|hook_command| !hook_command.is_empty());

    let memory = context.launch_overrides.memory.unwrap_or(settings.memory);
    let resolution = context
        .launch_overrides
        .game_resolution
        .unwrap_or(settings.game_resolution);
    let has_hook_commands = pre_launch_hook.is_some()
        || wrapper.is_some()
        || post_exit_hook.is_some();
    let full_path = if has_hook_commands {
        Some(crate::util::io::canonicalize(
            state
                .directories
                .instances_dir()
                .join(&context.instance.path),
        )?)
    } else {
        None
    };
    let hook_environment = if has_hook_commands {
        let full_path = full_path
            .as_ref()
            .expect("hooked launches always resolve their instance path");
        let java_version =
            crate::launcher::resolve_java_for_launch(&context).await?;

        Some(crate::launcher::hooks::HookEnvironment::from_current_env(
            &env_args,
            crate::launcher::hooks::HookVariables {
                instance_name: context.instance.name.clone(),
                instance_id: context.instance.path.clone(),
                instance_dir: full_path.to_string_lossy().to_string(),
                java_path: java_version.path.clone(),
                java_args: crate::launcher::hooks::build_hook_java_args(
                    &java_args,
                    memory,
                    &java_version,
                ),
            },
        ))
    } else {
        None
    };
    let launch_env_args = hook_environment
        .as_ref()
        .map_or_else(|| env_args.clone(), |env| env.injected_envs());

    if let (Some(hook), Some(hook_environment), Some(full_path)) = (
        pre_launch_hook,
        hook_environment.as_ref(),
        full_path.as_ref(),
    ) {
        let expanded_hook = hook_environment.expand(hook);
        let mut cmd = shlex::split(&expanded_hook)
            .ok_or_else(|| {
                crate::ErrorKind::LauncherError(format!(
                    "Invalid pre-launch command: {hook}",
                ))
            })?
            .into_iter();

        if let Some(command) = cmd.next() {
            let result = Command::new(command)
                .args(cmd)
                .envs(launch_env_args.iter().cloned())
                .current_dir(full_path)
                .spawn()
                .map_err(|e| IOError::with_path(e, full_path))?
                .wait()
                .await
                .map_err(IOError::from)?;

            if !result.success() {
                return Err(crate::ErrorKind::LauncherError(format!(
                    "Non-zero exit code for pre-launch hook: {}",
                    result.code().unwrap_or(-1)
                ))
                .as_error());
            }
        }
    }

    let wrapper = wrapper
        .map(|hook| {
            hook_environment
                .as_ref()
                .map_or(hook.clone(), |env| env.expand(&hook))
        })
        .filter(|hook_command| !hook_command.is_empty());
    let post_exit_hook = post_exit_hook
        .map(|hook| {
            hook_environment
                .as_ref()
                .map_or(hook.clone(), |env| env.expand(&hook))
        })
        .filter(|hook_command| !hook_command.is_empty());

    let mut mc_set_options: Vec<(String, String)> = vec![];
    if let Some(fullscreen) = context.launch_overrides.force_fullscreen {
        mc_set_options.push(("fullscreen".to_string(), fullscreen.to_string()));
    } else if settings.force_fullscreen {
        mc_set_options.push(("fullscreen".to_string(), "true".to_string()));
    }

    if let Some(project_id) = server_play_project_id(&context.link)
        && !project_id.trim().is_empty()
    {
        let server_id = uuid::Uuid::new_v4().to_string();
        let join_result = fetch::INSECURE_REQWEST_CLIENT
			.post("https://sessionserver.mojang.com/session/minecraft/join")
			.json(&json!({
				"accessToken": &credentials.access_token,
				"selectedProfile": credentials.offline_profile.id.simple().to_string(),
				"serverId": &server_id,
			}))
			.timeout(Duration::from_secs(5))
			.send()
			.await;

        match join_result {
            Ok(resp) if resp.status().is_success() => {
                let result = fetch::post_json(
                    concat!(
                        env!("MODRINTH_API_BASE_URL"),
                        "analytics/minecraft-server-play"
                    ),
                    json!({
                        "project_id": project_id,
                        "username": &credentials.offline_profile.name,
                        "server_id": &server_id,
                    }),
                    &state.api_semaphore,
                    &state.pool,
                )
                .await;

                match result {
                    Ok(()) => {
                        info!(
                            "Tracked server play for '{project_id}' in analytics"
                        )
                    }
                    Err(err) => warn!("Failed to report server play: {err:?}"),
                }
            }
            Ok(resp) => warn!(
                "Failed to join Mojang session server: HTTP {}",
                resp.status()
            ),
            Err(err) => warn!("Failed to join Mojang session server: {err:?}"),
        }
    }

    crate::minecraft_skins::flush_pending_skin_change().await?;
    crate::launcher::launch_minecraft(
        &java_args,
        &launch_env_args,
        &mc_set_options,
        &wrapper,
        &memory,
        &resolution,
        credentials,
        post_exit_hook,
        &context,
        quick_play_type,
    )
    .await
}

fn server_play_project_id(link: &InstanceLink) -> Option<&String> {
    match link {
        InstanceLink::ServerProject { project_id }
        | InstanceLink::ServerProjectModpack {
            server_project_id: project_id,
            ..
        } => Some(project_id),
        InstanceLink::Unmanaged
        | InstanceLink::ModrinthModpack { .. }
        | InstanceLink::ModrinthHosting { .. }
        | InstanceLink::ImportedModpack { .. }
        | InstanceLink::SharedInstance { .. } => None,
    }
}

pub async fn kill(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let processes =
        crate::api::process::get_by_instance_id(instance_id).await?;

    for process in processes {
        state.process_manager.kill(process.uuid).await?;
    }

    Ok(())
}

#[tracing::instrument]
pub async fn try_update_playtime_by_instance_id(
    instance_id: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let context =
        crate::state::instances::commands::get_instance_launch_context(
            instance_id,
            &state.pool,
        )
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(format!(
                "Tried to update playtime for nonexistent instance {instance_id}!"
            ))
        })?;
    let updated_recent_playtime = context.instance.recent_time_played;
    let res = if updated_recent_playtime > 0 {
        let modrinth_pack_version_id = match &context.link {
            InstanceLink::ModrinthModpack { version_id, .. }
            | InstanceLink::ServerProjectModpack {
                content_version_id: version_id,
                ..
            }
            | InstanceLink::ImportedModpack {
                version_id: Some(version_id),
                ..
            } => Some(version_id.clone()),
            InstanceLink::Unmanaged
            | InstanceLink::ServerProject { .. }
            | InstanceLink::ModrinthHosting { .. }
            | InstanceLink::ImportedModpack { .. }
            | InstanceLink::SharedInstance { .. } => None,
        };
        let playtime_update_json = json!({
            "seconds": updated_recent_playtime,
            "loader": context.applied_content_set.loader.as_str(),
            "game_version": &context.applied_content_set.game_version,
            "parent": modrinth_pack_version_id,
        });
        let mut hashmap: HashMap<String, serde_json::Value> = HashMap::new();

        for (_, project) in get_projects(instance_id, None).await? {
            if let Some(metadata) = project.metadata {
                hashmap
                    .insert(metadata.version_id, playtime_update_json.clone());
            }
        }

        fetch::post_json(
            concat!(env!("MODRINTH_API_BASE_URL"), "analytics/playtime"),
            serde_json::to_value(hashmap)?,
            &state.api_semaphore,
            &state.pool,
        )
        .await
    } else {
        Ok(())
    };

    if res.is_ok() {
        crate::state::instances::commands::mark_instance_playtime_submitted(
            &context.instance.id,
            updated_recent_playtime,
            &state.pool,
        )
        .await?;
    }

    res
}
