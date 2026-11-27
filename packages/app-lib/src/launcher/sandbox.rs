use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use super::profile::ResolvedVersion as VersionInfo;
use crate::state::instances::adapters::sqlite::content_rows;
use crate::state::{
    DirectoryInfo, JavaVersion, SandboxMode, SandboxSettings, State,
};
use minecraft_meta_manifest::minecraft::{LoggingConfiguration, LoggingSide};
use modrinth_sandbox::{
    PathRule, SandboxStdio,
    api::{SandboxApiChild, SandboxApiLauncher, SandboxApiService},
    minecraft::{MinecraftCommand, MinecraftLoggingConfig},
};
use parking_lot::Mutex;
use sqlx::SqlitePool;
use tokio::sync::oneshot;
use uuid::Uuid;

const THESEUS_MINECRAFT_LAUNCHER_MAIN_CLASS: &str =
    "com.modrinth.theseus.MinecraftLaunch";
const SANDBOX_API_PORT: u16 = 28881;

static LOGIN_HANDLER: LazyLock<Arc<SimpleLoginHandler>> =
    LazyLock::new(Default::default);
static SERVICE: LazyLock<SandboxApiService> = LazyLock::new(|| {
    SandboxApiService::new(SANDBOX_API_PORT, LOGIN_HANDLER.clone())
});

#[derive(Default)]
struct SimpleLoginHandler {
    sessions: Mutex<HashMap<Uuid, Arc<str>>>,
}

impl SandboxApiLauncher for SimpleLoginHandler {
    fn login(&self, uuid: Uuid, token: oneshot::Sender<Arc<str>>) {
        if let Some(access_token) = self.sessions.lock().get(&uuid) {
            token.send(access_token.clone()).ok();
        }
    }
}

pub(super) struct Session {
    pub access_token: String,
    child: Option<SandboxApiChild>,
}

pub(super) fn prepare_session(
    uuid: Uuid,
    access_token: String,
    sandboxed: bool,
) -> Session {
    if !sandboxed {
        return Session {
            access_token,
            child: None,
        };
    }

    LOGIN_HANDLER
        .sessions
        .lock()
        .insert(uuid, access_token.into());
    let child = SERVICE.clone().start(uuid);

    Session {
        access_token: child.get_dummy_access_token().to_string(),
        child: Some(child),
    }
}

/// Whether a launch should run in the sandbox under `settings`.
///
/// In [`SandboxMode::Unrecognized`], an instance is sandboxed when any
/// enabled content entry isn't resolved to a Modrinth project version.
pub(super) async fn should_sandbox(
    settings: &SandboxSettings,
    content_set_id: &str,
    pool: &SqlitePool,
) -> crate::Result<bool> {
    Ok(match settings.mode {
        SandboxMode::Never => false,
        SandboxMode::Always => true,
        SandboxMode::Unrecognized => {
            content_rows::get_content_entries(content_set_id, pool)
                .await?
                .iter()
                .any(|entry| {
                    entry.enabled
                        && (entry.project_id.is_none()
                            || entry.version_id.is_none())
                })
        }
    })
}

/// Host folders to expose to a sandboxed instance.
#[derive(Default)]
pub(super) struct GrantedPaths {
    pub read_only: Vec<PathBuf>,
    pub read_write: Vec<PathBuf>,
}

pub(super) fn granted_paths(settings: &SandboxSettings) -> GrantedPaths {
    // Read-only to prevent mods from modifying files that the user later runs
    let mut read_only = Vec::new();
    if settings.allow_downloads
        && let Some(dir) = dirs::download_dir()
    {
        read_only.push(dir);
    }
    if settings.allow_documents
        && let Some(dir) = dirs::document_dir()
    {
        read_only.push(dir);
    }
    let mut read_write = settings
        .extra_paths
        .iter()
        .map(|path| path.trim())
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    for paths in [&mut read_only, &mut read_write] {
        paths.retain(|path| {
            let exists = path.is_dir();
            if !exists {
                tracing::warn!(
                    "Skipping sandbox folder grant for missing directory {}",
                    path.display()
                );
            }
            exists
        });
        paths.sort_unstable();
        paths.dedup();
    }
    GrantedPaths {
        read_only,
        read_write,
    }
}

/// Picks the sandbox environment for a launch, failing if sandboxing was
/// requested but couldn't be set up on this system.
pub(super) fn select_env(
    state: &State,
    sandboxed: bool,
) -> crate::Result<modrinth_sandbox::SandboxEnv> {
    if !sandboxed {
        return Ok(state.sandbox_env_noop.clone());
    }

    match &state.sandbox_env {
		Ok(env) => Ok(env.clone()),
		Err(reason) => Err(crate::ErrorKind::LauncherError(format!(
			"This instance needs to run in the sandbox, but sandboxing is unavailable on this system: {reason}"
		))
		.into()),
	}
}

pub(super) struct CommandInput<'a> {
    pub directories: &'a DirectoryInfo,
    pub java: &'a JavaVersion,
    pub version_info: &'a VersionInfo,
    pub version_jar: &'a str,
    pub java_agent: PathBuf,
    pub classpath: Vec<PathBuf>,
    pub rpc_address: SocketAddr,
    pub instance_path: PathBuf,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub granted_paths: GrantedPaths,
    pub allow_network: bool,
}

pub(super) fn prepare_command(
    session: Session,
    input: CommandInput<'_>,
    env_args: &[(String, String)],
) -> crate::Result<MinecraftCommand> {
    let jre_path = Path::new(&input.java.path)
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| {
            crate::ErrorKind::LauncherError(format!(
                "Could not determine Java runtime directory from {}",
                input.java.path
            ))
        })?
        .to_path_buf();

    let mut classpath = Vec::with_capacity(input.classpath.len() + 1);
    classpath.push(input.java_agent.clone());
    classpath.extend(input.classpath);

    let mut jvm_args = input.jvm_args;
    jvm_args.extend([
        format!("-Dmodrinth.internal.ipc.host={}", input.rpc_address.ip()),
        format!("-Dmodrinth.internal.ipc.port={}", input.rpc_address.port()),
    ]);

    // Theseus needs reflective access to the game's main class across modules.
    if input.java.parsed_version >= 9 {
        jvm_args.push(
            "--add-opens=java.base/java.lang.reflect=ALL-UNNAMED".to_owned(),
        );
    }
    // Theseus uses internal JDK APIs to support JEP 512 on Java 25+.
    if input.java.parsed_version >= 25 {
        jvm_args.push(
            "--add-opens=jdk.internal/jdk.internal.misc=ALL-UNNAMED".to_owned(),
        );
    }

    let mut launcher_args = Vec::with_capacity(input.game_args.len() + 1);
    launcher_args.push(input.version_info.main_class.clone().into());
    launcher_args.extend(input.game_args.into_iter().map(Into::into));

    let logging_config = input
        .version_info
        .logging
        .as_ref()
        .and_then(|logging| logging.get(&LoggingSide::Client))
        .map(|logging| match logging {
            LoggingConfiguration::Log4j2Xml { argument, file } => {
                MinecraftLoggingConfig {
                    path: input.directories.log_configs_dir().join(&file.id),
                    argument: argument.clone(),
                }
            }
        });

    let mut command = MinecraftCommand::default();
    command.sandbox_api_child = session.child;
    command.jre_path = jre_path;
    command.java_agent = Some(input.java_agent);
    command.classpath = classpath;
    command.natives_path =
        input.directories.version_natives_dir(input.version_jar);
    command.assets_path = input.directories.assets_dir();
    command.logging_config = logging_config;
    command.instance_path = input.instance_path;
    command.persistent_dir = Some(input.directories.instance_data_dir());
    command.launcher_files_dir = Some(input.directories.launcher_files_dir());
    command.read_only_paths = input.granted_paths.read_only;
    command
        .read_only_paths
        .push(input.directories.libraries_dir());
    command.read_write_paths = input.granted_paths.read_write;
    command.allow_network = input.allow_network;
    command.path_rules = vec![
        PathRule::SoftBlock(input.directories.config_dir.clone()),
        PathRule::HardBlock(input.directories.settings_dir.clone()),
        // These can be inside of the settings directory, we need to downgrade
        // the hard block into a soft block so our binds are accepted
        PathRule::SoftBlock(input.directories.metadata_dir()),
        PathRule::SoftBlock(input.directories.instance_data_dir()),
        PathRule::SoftBlock(input.directories.synced_options_dir()),
        PathRule::SoftBlock(input.directories.instances_dir()),
        // We might want to drag-and-drop a resourcepack from one instance to another,
        // so we unblock the other instance directories
        PathRule::UnblockRead(input.directories.instances_dir()),
    ];
    command.jvm_args = jvm_args.into_iter().map(Into::into).collect();
    command.main_class = THESEUS_MINECRAFT_LAUNCHER_MAIN_CLASS.into();
    command.main_class_args = launcher_args;
    command.extra_environment = env_args
        .iter()
        .cloned()
        .map(|(key, value)| (key.into(), value.into()))
        .collect();
    command.stdin = SandboxStdio::Null;
    command.stdout = SandboxStdio::Pipe;
    command.stderr = SandboxStdio::Pipe;
    Ok(command)
}
