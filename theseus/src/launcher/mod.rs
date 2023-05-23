//! Logic for launching Minecraft
use crate::event::emit::{emit_loading, init_or_edit_loading};
use crate::event::{LoadingBarId, LoadingBarType};
use crate::jre::{JAVA_17_KEY, JAVA_18PLUS_KEY, JAVA_8_KEY};
use crate::prelude::JavaVersion;
use crate::state::ProfileInstallStage;
use crate::{
    process,
    state::{self as st, MinecraftChild},
    State,
};
use daedalus as d;
use daedalus::minecraft::VersionInfo;
use dunce::canonicalize;
use st::Profile;
use std::fs;
use std::{process::Stdio, sync::Arc};
use tokio::process::Command;
use uuid::Uuid;

mod args;

pub mod auth;
pub mod download;

#[tracing::instrument]
pub fn parse_rule(rule: &d::minecraft::Rule, java_version: &str) -> bool {
    use d::minecraft::{Rule, RuleAction};

    let res = match rule {
        Rule {
            os: Some(ref os), ..
        } => crate::util::platform::os_rule(os, java_version),
        Rule {
            features: Some(ref features),
            ..
        } => features.has_demo_resolution.unwrap_or(false),
        _ => false,
    };

    match rule.action {
        RuleAction::Allow => res,
        RuleAction::Disallow => !res,
    }
}

macro_rules! processor_rules {
    ($dest:expr; $($name:literal : client => $client:expr, server => $server:expr;)+) => {
        $(std::collections::HashMap::insert(
            $dest,
            String::from($name),
            daedalus::modded::SidedDataEntry {
                client: String::from($client),
                server: String::from($server),
            },
        );)+
    }
}

pub async fn get_java_version_from_profile(
    profile: &Profile,
    version_info: &VersionInfo,
) -> crate::Result<Option<JavaVersion>> {
    if let Some(java) = profile.java.clone().and_then(|x| x.override_version) {
        Ok(Some(java))
    } else {
        let optimal_keys = match version_info
            .java_version
            .as_ref()
            .map(|it| it.major_version)
            .unwrap_or(8)
        {
            0..=15 => vec![JAVA_8_KEY, JAVA_17_KEY, JAVA_18PLUS_KEY],
            16..=17 => vec![JAVA_17_KEY, JAVA_18PLUS_KEY],
            _ => vec![JAVA_18PLUS_KEY],
        };

        let state = State::get().await?;
        let settings = state.settings.read().await;

        for key in optimal_keys {
            if let Some(java) = settings.java_globals.get(&key.to_string()) {
                return Ok(Some(java.clone()));
            }
        }

        Ok(None)
    }
}

#[tracing::instrument(skip(profile))]
#[theseus_macros::debug_pin]
pub async fn install_minecraft(
    profile: &Profile,
    existing_loading_bar: Option<LoadingBarId>,
) -> crate::Result<()> {
    let loading_bar = init_or_edit_loading(
        existing_loading_bar,
        LoadingBarType::MinecraftDownload {
            // If we are downloading minecraft for a profile, provide its name and uuid
            profile_name: profile.metadata.name.clone(),
            profile_path: profile.path.clone(),
        },
        100.0,
        "Downloading Minecraft",
    )
    .await?;

    crate::api::profile::edit(&profile.path, |prof| {
        prof.install_stage = ProfileInstallStage::Installing;

        async { Ok(()) }
    })
    .await?;
    State::sync().await?;

    let state = State::get().await?;
    let instance_path = &canonicalize(&profile.path)?;
    let metadata = state.metadata.read().await;

    let version = metadata
        .minecraft
        .versions
        .iter()
        .find(|it| it.id == profile.metadata.game_version)
        .ok_or(crate::ErrorKind::LauncherError(format!(
            "Invalid game version: {}",
            profile.metadata.game_version
        )))?;

    let version_jar = profile
        .metadata
        .loader_version
        .as_ref()
        .map_or(version.id.clone(), |it| {
            format!("{}-{}", version.id.clone(), it.id.clone())
        });

    // Download version info (5)
    let mut version_info = download::download_version_info(
        &state,
        version,
        profile.metadata.loader_version.as_ref(),
        None,
        Some(&loading_bar),
    )
    .await?;

    let java_version = get_java_version_from_profile(profile, &version_info)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::OtherError(
                "No available java installation".to_string(),
            )
        })?;

    // Download minecraft (5-90)
    download::download_minecraft(
        &state,
        &version_info,
        &loading_bar,
        &java_version.architecture,
    )
    .await?;

    if let Some(processors) = &version_info.processors {
        let client_path = state
            .directories
            .version_dir(&version_jar)
            .join(format!("{version_jar}.jar"));

        if let Some(ref mut data) = version_info.data {
            processor_rules! {
                data;
                "SIDE":
                    client => "client",
                    server => "";
                "MINECRAFT_JAR" :
                    client => client_path.to_string_lossy(),
                    server => "";
                "MINECRAFT_VERSION":
                    client => profile.metadata.game_version.clone(),
                    server => "";
                "ROOT":
                    client => instance_path.to_string_lossy(),
                    server => "";
                "LIBRARY_DIR":
                    client => state.directories.libraries_dir().to_string_lossy(),
                    server => "";
            }

            emit_loading(&loading_bar, 0.0, Some("Running forge processors"))
                .await?;
            let total_length = processors.len();

            // Forge processors (90-100)
            for (index, processor) in processors.iter().enumerate() {
                if let Some(sides) = &processor.sides {
                    if !sides.contains(&String::from("client")) {
                        continue;
                    }
                }

                let cp = wrap_ref_builder!(cp = processor.classpath.clone() => {
                    cp.push(processor.jar.clone())
                });

                let child = Command::new(&java_version.path)
                    .arg("-cp")
                    .arg(args::get_class_paths_jar(
                        &state.directories.libraries_dir(),
                        &cp,
                        &java_version.architecture,
                    )?)
                    .arg(
                        args::get_processor_main_class(args::get_lib_path(
                            &state.directories.libraries_dir(),
                            &processor.jar,
                            false,
                        )?)
                        .await?
                        .ok_or_else(|| {
                            crate::ErrorKind::LauncherError(format!(
                                "Could not find processor main class for {}",
                                processor.jar
                            ))
                        })?,
                    )
                    .args(args::get_processor_arguments(
                        &state.directories.libraries_dir(),
                        &processor.args,
                        data,
                    )?)
                    .output()
                    .await
                    .map_err(|err| {
                        crate::ErrorKind::LauncherError(format!(
                            "Error running processor: {err}",
                        ))
                    })?;

                if !child.status.success() {
                    return Err(crate::ErrorKind::LauncherError(format!(
                        "Processor error: {}",
                        String::from_utf8_lossy(&child.stderr)
                    ))
                    .as_error());
                }

                emit_loading(
                    &loading_bar,
                    30.0 / total_length as f64,
                    Some(&format!(
                        "Running forge processor {}/{}",
                        index, total_length
                    )),
                )
                .await?;
            }
        }
    }

    crate::api::profile::edit(&profile.path, |prof| {
        prof.install_stage = ProfileInstallStage::Installed;

        async { Ok(()) }
    })
    .await?;
    State::sync().await?;
    emit_loading(&loading_bar, 1.0, Some("Finished installing")).await?;

    Ok(())
}

#[tracing::instrument]
#[theseus_macros::debug_pin]
#[allow(clippy::too_many_arguments)]
pub async fn launch_minecraft(
    java_args: &[String],
    env_args: &[(String, String)],
    wrapper: &Option<String>,
    memory: &st::MemorySettings,
    resolution: &st::WindowSize,
    credentials: &auth::Credentials,
    post_exit_hook: Option<Command>,
    profile: &Profile,
) -> crate::Result<Arc<tokio::sync::RwLock<MinecraftChild>>> {
    if profile.install_stage == ProfileInstallStage::PackInstalling
        || profile.install_stage == ProfileInstallStage::Installing
    {
        return Err(crate::ErrorKind::LauncherError(
            "Profile is still installing".to_string(),
        )
        .into());
    }

    if profile.install_stage != ProfileInstallStage::Installed {
        install_minecraft(profile, None).await?;
    }

    let state = State::get().await?;
    let metadata = state.metadata.read().await;
    let instance_path = &canonicalize(&profile.path)?;

    let version = metadata
        .minecraft
        .versions
        .iter()
        .find(|it| it.id == profile.metadata.game_version)
        .ok_or(crate::ErrorKind::LauncherError(format!(
            "Invalid game version: {}",
            profile.metadata.game_version
        )))?;

    let version_jar = profile
        .metadata
        .loader_version
        .as_ref()
        .map_or(version.id.clone(), |it| {
            format!("{}-{}", version.id.clone(), it.id.clone())
        });

    let version_info = download::download_version_info(
        &state,
        version,
        profile.metadata.loader_version.as_ref(),
        None,
        None,
    )
    .await?;

    let java_version = get_java_version_from_profile(profile, &version_info)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::LauncherError(
                "No available java installation".to_string(),
            )
        })?;

    let client_path = state
        .directories
        .version_dir(&version_jar)
        .join(format!("{version_jar}.jar"));

    let args = version_info.arguments.clone().unwrap_or_default();
    let mut command = match wrapper {
        Some(hook) => {
            wrap_ref_builder!(it = Command::new(hook) => {it.arg(&java_version.path)})
        }
        None => Command::new(&java_version.path),
    };

    let env_args = Vec::from(env_args);

    // Check if profile has a running profile, and reject running the command if it does
    // Done late so a quick double call doesn't launch two instances
    let existing_processes =
        process::get_uuids_by_profile_path(instance_path).await?;
    if let Some(uuid) = existing_processes.first() {
        return Err(crate::ErrorKind::LauncherError(format!(
            "Profile {} is already running at UUID: {uuid}",
            instance_path.display()
        ))
        .as_error());
    }

    command
        .args(
            args::get_jvm_arguments(
                args.get(&d::minecraft::ArgumentType::Jvm)
                    .map(|x| x.as_slice()),
                &state.directories.version_natives_dir(&version_jar),
                &state.directories.libraries_dir(),
                &args::get_class_paths(
                    &state.directories.libraries_dir(),
                    version_info.libraries.as_slice(),
                    &client_path,
                    &java_version.architecture,
                )?,
                &version_jar,
                *memory,
                Vec::from(java_args),
                &java_version.architecture,
            )?
            .into_iter()
            .collect::<Vec<_>>(),
        )
        .arg(version_info.main_class.clone())
        .args(
            args::get_minecraft_arguments(
                args.get(&d::minecraft::ArgumentType::Game)
                    .map(|x| x.as_slice()),
                version_info.minecraft_arguments.as_deref(),
                credentials,
                &version.id,
                &version_info.asset_index.id,
                instance_path,
                &state.directories.assets_dir(),
                &version.type_,
                *resolution,
                &java_version.architecture,
            )?
            .into_iter()
            .collect::<Vec<_>>(),
        )
        .current_dir(instance_path.clone())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // CARGO-set DYLD_LIBRARY_PATH breaks Minecraft on macOS during testing on playground
    #[cfg(target_os = "macos")]
    if std::env::var("CARGO").is_ok() {
        command.env_remove("DYLD_FALLBACK_LIBRARY_PATH");
    }
    command.envs(env_args);

    // Get Modrinth logs directories
    let datetime_string =
        chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let logs_dir = {
        let st = State::get().await?;
        st.directories
            .profile_logs_dir(profile.uuid)
            .join(&datetime_string)
    };
    fs::create_dir_all(&logs_dir)?;

    let stdout_log_path = logs_dir.join("stdout.log");
    let stderr_log_path = logs_dir.join("stderr.log");

    // Create Minecraft child by inserting it into the state
    // This also spawns the process and prepares the subsequent processes
    let mut state_children = state.children.write().await;
    state_children
        .insert_process(
            Uuid::new_v4(),
            instance_path.to_path_buf(),
            stdout_log_path,
            stderr_log_path,
            command,
            post_exit_hook,
        )
        .await
}
