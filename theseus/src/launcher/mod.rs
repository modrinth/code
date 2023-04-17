//! Logic for launching Minecraft
use crate::{
    process,
    state::{self as st, MinecraftChild},
};
use daedalus as d;
use dunce::canonicalize;
use st::Profile;
use std::{path::Path, process::Stdio, sync::Arc};
use tokio::process::Command;

mod args;

pub mod auth;
pub mod download;

#[tracing::instrument]
pub fn parse_rule(rule: &d::minecraft::Rule) -> bool {
    use d::minecraft::{Rule, RuleAction};

    let res = match rule {
        Rule {
            os: Some(ref os), ..
        } => crate::util::platform::os_rule(os),
        Rule {
            features: Some(ref features),
            ..
        } => features.has_demo_resolution.unwrap_or(false),
        _ => true,
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

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip_all, fields(path = ?instance_path))]
pub async fn launch_minecraft(
    game_version: &str,
    loader_version: &Option<d::modded::LoaderVersion>,
    instance_path: &Path,
    java_install: &Path,
    java_args: &[String],
    env_args: &[(String, String)],
    wrapper: &Option<String>,
    memory: &st::MemorySettings,
    resolution: &st::WindowSize,
    credentials: &auth::Credentials,
    post_exit_hook: Option<Command>,
    profile: &Profile, // optional ref to Profile for event tracking
) -> crate::Result<Arc<tokio::sync::RwLock<MinecraftChild>>> {
    let state = st::State::get().await?;
    let instance_path = &canonicalize(instance_path)?;

    let version = state
        .metadata
        .minecraft
        .versions
        .iter()
        .find(|it| it.id == game_version)
        .ok_or(crate::ErrorKind::LauncherError(format!(
            "Invalid game version: {game_version}"
        )))?;

    let version_jar =
        loader_version.as_ref().map_or(version.id.clone(), |it| {
            format!("{}-{}", version.id.clone(), it.id.clone())
        });

    let mut version_info = download::download_version_info(
        &state,
        version,
        loader_version.as_ref(),
    )
    .await?;

    let client_path = state
        .directories
        .version_dir(&version_jar)
        .join(format!("{version_jar}.jar"));

    download::download_minecraft(&state, &version_info, profile).await?;
    st::State::sync().await?;

    if let Some(processors) = &version_info.processors {
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
                    client => game_version,
                    server => "";
                "ROOT":
                    client => instance_path.to_string_lossy(),
                    server => "";
                "LIBRARY_DIR":
                    client => state.directories.libraries_dir().to_string_lossy(),
                    server => "";
            }

            for processor in processors {
                if let Some(sides) = &processor.sides {
                    if !sides.contains(&String::from("client")) {
                        continue;
                    }
                }

                let cp = wrap_ref_builder!(cp = processor.classpath.clone() => {
                    cp.push(processor.jar.clone())
                });

                let child = Command::new("java")
                    .arg("-cp")
                    .arg(args::get_class_paths_jar(
                        &state.directories.libraries_dir(),
                        &cp,
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
            }
        }
    }

    let args = version_info.arguments.clone().unwrap_or_default();
    let mut command = match wrapper {
        Some(hook) => {
            wrap_ref_builder!(it = Command::new(hook) => {it.arg(java_install)})
        }
        None => Command::new(String::from(java_install.to_string_lossy())),
    };

    let env_args = Vec::from(env_args);

    // Check if profile has a running profile, and reject running the command if it does
    // Done late so a quick double call doesn't launch two instances
    let existing_processes =
        process::get_uuids_by_profile_path(instance_path).await?;
    if let Some(pid) = existing_processes.first() {
        return Err(crate::ErrorKind::LauncherError(format!(
            "Profile {} is already running at PID: {pid}",
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
                )?,
                &version_jar,
                *memory,
                Vec::from(java_args),
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
            )?
            .into_iter()
            .collect::<Vec<_>>(),
        )
        .current_dir(instance_path.clone())
        .env_clear()
        .envs(env_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Create Minecraft child by inserting it into the state
    // This also spawns the process and prepares the subsequent processes
    let mut state_children = state.children.write().await;
    state_children.insert_process(
        uuid::Uuid::new_v4(),
        instance_path.to_path_buf(),
        command,
        post_exit_hook,
    )
}
