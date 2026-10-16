use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use crate::state::{DirectoryInfo, JavaVersion};
use daedalus::minecraft::{LoggingConfiguration, LoggingSide, VersionInfo};
use modrinth_sandbox::{
    SandboxStdio,
    api::{SandboxApiChild, SandboxApiLauncher, SandboxApiService},
    minecraft::{MinecraftCommand, MinecraftLoggingConfig},
};
use parking_lot::Mutex;
use tokio::sync::oneshot;
use uuid::Uuid;

const THESEUS_MINECRAFT_LAUNCHER_MAIN_CLASS: &str =
    "com.modrinth.theseus.MinecraftLaunch";
const SANDBOX_API_PORT: u16 = 28881;

static LOGIN_HANDLER: LazyLock<Arc<SimpleLoginHandler>> =
    LazyLock::new(|| Default::default());
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
    child: SandboxApiChild,
}

pub(super) fn prepare_session(uuid: Uuid, access_token: String) -> Session {
    LOGIN_HANDLER
        .sessions
        .lock()
        .insert(uuid, access_token.into());
    let child = SERVICE.clone().start(uuid);

    Session {
        access_token: child.get_dummy_access_token().to_string(),
        child,
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

    Ok(MinecraftCommand {
        sandbox_api_child: Some(session.child),
        jre_path,
        java_agent: Some(input.java_agent),
        classpath,
        natives_path: input.directories.version_natives_dir(input.version_jar),
        assets_path: input.directories.assets_dir(),
        logging_config,
        instance_path: input.instance_path,
        persistent_dir: input.directories.instance_data_dir(),
        jvm_args: jvm_args.into_iter().map(Into::into).collect(),
        main_class: THESEUS_MINECRAFT_LAUNCHER_MAIN_CLASS.into(),
        main_class_args: launcher_args,
        extra_environment: env_args
            .iter()
            .cloned()
            .map(|(key, value)| (key.into(), value.into()))
            .collect(),
        stdin: SandboxStdio::Null,
        stdout: SandboxStdio::Pipe,
        stderr: SandboxStdio::Pipe,
    })
}
