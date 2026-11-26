use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use anyhow::{Context, Result, ensure};
use rand::Rng;
use sha1::{Digest, Sha1};

use crate::{
    PathPermission, PathRule, SandboxChild, SandboxCommand, SandboxStdio,
    api::SandboxApiChild,
    default_path_rules::{hard_path_rules, soft_path_rules},
    path_access::PathRules,
    util::SandboxArg,
};

#[derive(Debug)]
#[non_exhaustive]
pub struct MinecraftCommand {
    /// Child reference to the sandbox API service, created using
    /// [`SandboxApiService::start`]
    ///
    /// [`SandboxApiService::start`]: crate::api::SandboxApiService::start
    pub sandbox_api_child: Option<SandboxApiChild>,
    /// Path to the Java Runtime Environment on the host.
    pub jre_path: PathBuf,
    /// Optional Java agent JAR on the host.
    pub java_agent: Option<PathBuf>,
    /// Complete Java classpath on the host, in classpath order.
    pub classpath: Vec<PathBuf>,
    /// Path to the extracted native libraries on the host.
    pub natives_path: PathBuf,
    /// Path to the Minecraft assets directory on the host.
    pub assets_path: PathBuf,
    /// Optional version-specific logging configuration.
    pub logging_config: Option<MinecraftLoggingConfig>,
    /// Path to the Minecraft instance directory on the host.
    pub instance_path: PathBuf,
    /// Directory for persistent home, data, configuration, cache, and state,
    /// shared across all game instances. If this isn't set, the game doesn't have a
    /// home directory.
    pub persistent_dir: Option<PathBuf>,
    /// Directory for files that the launcher uses to run the game, such as JARs created
    /// by [`create_launcher_jar`]. Required if [`MinecraftCommand::sandbox_api_child`] is
    /// set.
    ///
    /// The game can't be given write access to this directory, so it must not overlap
    /// with [`instance_path`], [`persistent_dir`], or the skins in [`assets_path`].
    ///
    /// [`instance_path`]: Self::instance_path
    /// [`persistent_dir`]: Self::persistent_dir
    /// [`assets_path`]: Self::assets_path
    pub launcher_files_dir: Option<PathBuf>,
    /// Extra host paths that can be read by the sandboxed application.
    /// Ignored if blocked by [`MinecraftCommand::path_rules`]
    pub read_only_paths: Vec<PathBuf>,
    /// Extra host paths that can be read and written by the sandboxed application.
    /// Ignored if blocked by [`MinecraftCommand::path_rules`].
    /// If there is a read-only path rule then this will become a read only path
    pub read_write_paths: Vec<PathBuf>,
    /// Rules for which host paths can be exposed to the sandbox.
    ///
    /// [`read_only_paths`] and [`read_write_paths`] can be inside of a soft blocked
    /// path, but can't be hard blocked or contain a blocked path. A read-write path
    /// may be downgraded into a read-only path if a rule blocks writing
    ///
    /// [`read_only_paths`]: Self::read_only_paths
    /// [`read_write_paths`]: Self::read_write_paths
    pub path_rules: Vec<PathRule>,
    /// JVM arguments which do not contain sandbox-managed paths.
    ///
    /// Classpath, Java agent, native-library, and logging arguments are generated
    /// by [`setup_minecraft_command`].
    pub jvm_args: Vec<SandboxArg>,
    /// Java main class to invoke.
    pub main_class: SandboxArg,
    /// Arguments passed to [`MinecraftCommand::main_class`].
    pub main_class_args: Vec<SandboxArg>,
    /// Additional environment variables set for the sandboxed process.
    pub extra_environment: BTreeMap<SandboxArg, SandboxArg>,
    /// Default io behaviour for stdin.
    pub stdin: SandboxStdio,
    /// Default io behaviour for stdout.
    pub stdout: SandboxStdio,
    /// Default io behaviour for stderr.
    pub stderr: SandboxStdio,
}

impl Default for MinecraftCommand {
    fn default() -> Self {
        Self {
            sandbox_api_child: None,
            jre_path: PathBuf::new(),
            java_agent: None,
            classpath: Vec::new(),
            natives_path: PathBuf::new(),
            assets_path: PathBuf::new(),
            logging_config: None,
            instance_path: PathBuf::new(),
            persistent_dir: None,
            launcher_files_dir: None,
            read_only_paths: Vec::new(),
            read_write_paths: Vec::new(),
            path_rules: Vec::new(),
            jvm_args: Vec::new(),
            main_class: SandboxArg::default(),
            main_class_args: Vec::new(),
            extra_environment: BTreeMap::new(),
            stdin: SandboxStdio::Null,
            stdout: SandboxStdio::Inherit,
            stderr: SandboxStdio::Inherit,
        }
    }
}

#[derive(Debug)]
pub struct MinecraftLoggingConfig {
    /// Path to the logging configuration file on the host.
    pub path: PathBuf,
    /// JVM argument containing a `${path}` placeholder for the configuration
    /// file's path inside the sandbox.
    pub argument: String,
}

const CLASSPATH_JVM_ARGUMENT: &str = "-cp";
const JAVA_LIBRARY_PATH_JVM_ARGUMENT: &str = "-Djava.library.path";
const JNA_TMPDIR_JVM_ARGUMENT: &str = "-Djna.tmpdir";
const LWJGL_LIBRARY_EXTRACT_PATH_JVM_ARGUMENT: &str =
    "-Dorg.lwjgl.system.SharedLibraryExtractPath";
const NETTY_NATIVE_WORKDIR_JVM_ARGUMENT: &str = "-Dio.netty.native.workdir";
const USER_HOME_JVM_ARGUMENT: &str = "-Duser.home";
const LOGGING_CONFIG_JVM_ARGUMENT: &str = "-Dlog4j.configurationFile";
const JAVA_AGENT_JVM_ARGUMENT: &str = "-javaagent";

/// JVM options generated by [`setup_minecraft_command`].
///
/// Entries are option names without their `=` or `:` separated values.
pub const JVM_ARGUMENTS: &[&str] = &[
    CLASSPATH_JVM_ARGUMENT,
    JAVA_LIBRARY_PATH_JVM_ARGUMENT,
    JNA_TMPDIR_JVM_ARGUMENT,
    LWJGL_LIBRARY_EXTRACT_PATH_JVM_ARGUMENT,
    NETTY_NATIVE_WORKDIR_JVM_ARGUMENT,
    USER_HOME_JVM_ARGUMENT,
    LOGGING_CONFIG_JVM_ARGUMENT,
    JAVA_AGENT_JVM_ARGUMENT,
];

/// Used by the illuminate mod (https://github.com/creeperkatze/mc-mods/blob/main/illuminate/)
const SIGNALRGB_REGISTRY_KEY: &str =
    r"Software\WhirlwindFX\SignalRgb\effects\selected";

#[cfg(windows)]
const MINECRAFT_PASSTHROUGH_ENVIRONMENT: &[&str] = &[
    "COMPUTERNAME",
    "HOME",
    "HOMEDRIVE",
    "HOMEPATH",
    "HOSTNAME",
    "NUMBER_OF_PROCESSORS",
    "OS",
    "PROCESSOR_ARCHITECTURE",
    "PROCESSOR_IDENTIFIER",
    "PROCESSOR_LEVEL",
    "PROCESSOR_REVISION",
    "PROGRAMDATA",
    "PROGRAMFILES",
    "PROGRAMFILES(X86)",
    "PROGRAMW6432",
    "PUBLIC",
    "SYSTEMDRIVE",
    "SYSTEMROOT",
    "USERDOMAIN",
    "APPDATA",
    "LOCALAPPDATA",
    "ALLUSERSPROFILE",
    "PATH",
    "PATHEXT",
    "COMMONPROGRAMFILES",
    "COMSPEC",
    "DRIVERDATA",
    "PROCESSOR_ARCHITECTURE",
    "PROCESSOR_IDENTIFIER",
    "PROCESSOR_LEVEL",
    "SESSIONNAME",
    "USERNAME",
    "USERPROFILE",
    "WINDIR",
    "TEMP",
];

#[cfg(target_os = "macos")]
const MINECRAFT_PASSTHROUGH_ENVIRONMENT: &[&str] = &[
    "PATH",
    "LANG",
    "LANGUAGE",
    "LC_ALL",
    "LC_ADDRESS",
    "LC_COLLATE",
    "LC_CTYPE",
    "LC_IDENTIFICATION",
    "LC_MEASUREMENT",
    "LC_MESSAGES",
    "LC_MONETARY",
    "LC_NAME",
    "LC_NUMERIC",
    "LC_PAPER",
    "LC_TELEPHONE",
    "LC_TIME",
];

#[cfg(target_os = "linux")]
const MINECRAFT_PASSTHROUGH_ENVIRONMENT: &[&str] = &[
    "PATH",
    "LANG",
    "LANGUAGE",
    "LC_ALL",
    "LC_ADDRESS",
    "LC_COLLATE",
    "LC_CTYPE",
    "LC_IDENTIFICATION",
    "LC_MEASUREMENT",
    "LC_MESSAGES",
    "LC_MONETARY",
    "LC_NAME",
    "LC_NUMERIC",
    "LC_PAPER",
    "LC_TELEPHONE",
    "LC_TIME",
    "GDMSESSION",
    "DESKTOP_SESSION",
    "DISPLAY",
    "WAYLAND_DISPLAY",
    "XDG_RUNTIME_DIR",
    "XAUTHORITY",
    "XDG_SESSION_TYPE",
    "PIPEWIRE_REMOTE",
    "PULSE_SERVER",
    "TZ",
    "DRI_PRIME",
    "MESA_LOADER_DRIVER_OVERRIDE",
    "MESA_VK_DEVICE_SELECT",
    "MESA_VK_DEVICE_SELECT_FORCE_DEFAULT_DEVICE",
    "LIBGL_ALWAYS_SOFTWARE",
    "__GLX_VENDOR_LIBRARY_NAME",
    "__NV_PRIME_RENDER_OFFLOAD",
    "__NV_PRIME_RENDER_OFFLOAD_PROVIDER",
    "__VK_LAYER_NV_optimus",
    "__GL_THREADED_OPTIMIZATIONS",
    "VK_DRIVER_FILES",
    "VK_ICD_FILENAMES",
    "VK_LAYER_PATH",
    "VK_LOADER_DRIVERS_SELECT",
    "VK_LOADER_DRIVERS_DISABLE",
    "FREETYPE_PROPERTIES",
    "ALSOFT_DRIVERS",
    "SDL_AUDIODRIVER",
    "GLFW_IM_MODULE",
    "XMODIFIERS",
    "SDL_IM_MODULE",
];

/// Environment variables which are set to a folder in
/// [`MinecraftCommand::persistent_dir`].
const PERSISTENT_ENVIRONMENT: &[(&str, &str)] = &[
    #[cfg(unix)]
    ("HOME", "home"),
    #[cfg(target_os = "linux")]
    ("XDG_DATA_HOME", "data"),
    #[cfg(target_os = "linux")]
    ("XDG_CONFIG_HOME", "config"),
    #[cfg(target_os = "linux")]
    ("XDG_CACHE_HOME", "cache"),
    #[cfg(target_os = "linux")]
    ("XDG_STATE_HOME", "state"),
];

/// Games can keep running after the launcher is closed when unsandboxed or on macOS,
/// so this has to be longer than someone is likely to play for.
const MAX_LAUNCHER_JAR_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);

pub async fn setup_minecraft_command(
    minecraft: MinecraftCommand,
) -> Result<SandboxCommand> {
    let Ok(instance_path) = minecraft.instance_path.canonicalize() else {
        return Err(anyhow::anyhow!(
            "Instance path could not be canonicalized"
        ));
    };

    let java_path = {
        #[cfg(unix)]
        {
            minecraft.jre_path.join("bin").join("java")
        }
        #[cfg(windows)]
        {
            minecraft.jre_path.join("bin").join("javaw.exe")
        }
    };

    let mut persistent_environment = Vec::new();
    if let Some(persistent_dir) = &minecraft.persistent_dir {
        for (variable, folder) in PERSISTENT_ENVIRONMENT {
            persistent_environment
                .push((*variable, persistent_dir.join(folder)));
        }
    }

    let skins_path = minecraft.assets_path.join("skins");
    let mut classpath = minecraft.classpath;
    let mut read_only_paths = vec![
        minecraft.jre_path.clone(),
        minecraft.natives_path.clone(),
        minecraft.assets_path,
    ];

    // Some java installations will contain symlinks to external folders (e.g. arch symlinks conf, legal and man)
    for folder in [
        "bin", "conf", "demo", "include", "jmods", "legal", "lib", "man",
    ] {
        let real_dir = minecraft.jre_path.join(folder).canonicalize();
        if let Ok(real_dir) = real_dir
            && !real_dir.starts_with(&minecraft.jre_path)
        {
            read_only_paths.push(real_dir);
        }
    }

    read_only_paths.extend(classpath.clone());

    let mut args = minecraft.jvm_args;
    args.extend([
        jvm_path_argument(
            JAVA_LIBRARY_PATH_JVM_ARGUMENT,
            "=",
            &minecraft.natives_path,
        )
        .into(),
        #[cfg(target_os = "linux")]
        OsString::from(format!("{JNA_TMPDIR_JVM_ARGUMENT}=/tmp")).into(),
        #[cfg(target_os = "linux")]
        OsString::from(format!(
            "{LWJGL_LIBRARY_EXTRACT_PATH_JVM_ARGUMENT}=/tmp"
        ))
        .into(),
        #[cfg(target_os = "linux")]
        OsString::from(format!("{NETTY_NATIVE_WORKDIR_JVM_ARGUMENT}=/tmp"))
            .into(),
    ]);

    #[cfg(unix)]
    if let Some(persistent_dir) = &minecraft.persistent_dir {
        args.push(
            jvm_path_argument(
                USER_HOME_JVM_ARGUMENT,
                "=",
                &persistent_dir.join("home"),
            )
            .into(),
        );
    }

    if let Some(java_agent) = minecraft.java_agent.clone() {
        args.push(
            jvm_path_argument(JAVA_AGENT_JVM_ARGUMENT, ":", &java_agent).into(),
        );
        read_only_paths.push(java_agent);
    }

    if let Some(logging_config) = minecraft.logging_config {
        ensure!(
            logging_config.argument.contains("${path}"),
            "logging configuration argument must contain a `${{path}}` placeholder"
        );
        read_only_paths.push(logging_config.path.clone());
        args.push(
            replace_path_placeholder(
                &logging_config.argument,
                &logging_config.path,
            )
            .into(),
        );
    }

    // Setup sandbox java agent
    if let Some(child) = &minecraft.sandbox_api_child {
        const BYTES: &[u8] =
            include_bytes!(concat!(env!("OUT_DIR"), "/SandboxAgent.jar"));

        let launcher_files_dir = minecraft
            .launcher_files_dir
            .clone()
            .context("launcher_files_dir must be set to use the sandbox API")?;
        let sandbox_agent_path = tokio::task::spawn_blocking(move || {
            create_launcher_jar(
                &launcher_files_dir,
                "ModrinthSandboxAgent",
                BYTES,
            )
        })
        .await??;

        classpath.push(sandbox_agent_path.clone());
        read_only_paths.push(sandbox_agent_path.clone());

        let mut agent_argument = OsString::new();
        agent_argument.push(JAVA_AGENT_JVM_ARGUMENT);
        agent_argument.push(":");
        agent_argument.push(sandbox_agent_path.as_os_str());
        args.push(agent_argument.into());

        args.extend(child.get_jvm_arguments().into_iter().map(Into::into));
    }

    args.push(CLASSPATH_JVM_ARGUMENT.into());
    args.push(std::env::join_paths(&classpath)?.into());
    args.push(minecraft.main_class);
    args.extend(minecraft.main_class_args);

    let mut ensure_dirs_exist = vec![skins_path.clone()];
    ensure_dirs_exist
        .extend(persistent_environment.iter().map(|(_, dir)| dir.clone()));

    let mut extra_environment = minecraft.extra_environment;
    extra_environment.extend([
        ("INSIDE_MODRINTH_SANDBOX".into(), "1".into()),
        ("JAVA_HOME".into(), minecraft.jre_path.into()),
    ]);
    extra_environment.extend(
        persistent_environment
            .into_iter()
            .map(|(variable, dir)| (variable.into(), dir.into())),
    );

    let mut read_write_paths = vec![
        // .minecraft dir
        instance_path.clone(),
        // .minecraft assets/skins; assets/ is shared among instances,
        // and so is the skin cache, but the game writes into the skin cache
        skins_path,
    ];
    // global persistent dirs for all mods
    read_write_paths.extend(minecraft.persistent_dir);

    // Hard blocks have to be last, so that nothing else takes precedence over them
    let mut path_rules = soft_path_rules();
    path_rules.extend(minecraft.path_rules);
    path_rules.extend(hard_path_rules());
    if let Some(launcher_files_dir) = minecraft.launcher_files_dir {
        let rule = PathRule::HardBlockWrite(launcher_files_dir);
        // Check built-in read_write_paths (not minecraft.read_write_paths).
        // Downgrading these to read-only would break the game, so we
        // error instead
        let rules = PathRules::new([rule.clone()]);
        for path in &read_write_paths {
            ensure!(
                !matches!(
                    rules.calculate_bind_permission(path),
                    Ok(PathPermission::ReadOnly)
                ),
                "launcher_files_dir must not overlap with {path:?}, since the game can write to it"
            );
        }
        path_rules.push(rule);
    }
    let resolved_rules = PathRules::new(path_rules.clone());
    for path in minecraft.read_only_paths {
        match resolved_rules.calculate_bind_permission(&path) {
            Ok(_) => read_only_paths.push(path),
            Err(err) => tracing::warn!("ignoring read-only path: {err:#}"),
        }
    }
    for path in minecraft.read_write_paths {
        match resolved_rules.calculate_bind_permission(&path) {
            Ok(PathPermission::ReadOnly) => {
                tracing::warn!("only granting read access to {path:?}");
                read_only_paths.push(path);
            }
            Ok(PathPermission::ReadWrite) => read_write_paths.push(path),
            Err(err) => tracing::warn!("ignoring read-write path: {err:#}"),
        }
    }
    read_write_paths.extend(signalrgb_effects_dir());

    let mut on_start: Vec<Box<dyn FnOnce(&mut SandboxChild) + Send + Sync>> =
        Vec::new();
    let mut on_finish: Vec<Box<dyn FnOnce() + Send + Sync>> = Vec::new();
    if let Some(sandbox_api_child) = minecraft.sandbox_api_child {
        // Register handle
        on_start.push(Box::new({
            let sandbox_api_child = sandbox_api_child.clone();
            move |child| {
                sandbox_api_child
                    .set_sandbox_handle(child.get_sandbox_handle());
            }
        }));

        // Keep sandbox api child alive until process dies
        on_finish.push(Box::new(move || {
            std::mem::drop(sandbox_api_child);
        }));
    }

    read_only_paths.sort_unstable();
    read_only_paths.dedup();
    read_write_paths.sort_unstable();
    read_write_paths.dedup();

    Ok(SandboxCommand {
        executable: java_path,
        args,
        ensure_dirs_exist,
        read_only_paths,
        read_write_paths,
        path_rules,
        registry_read_keys: vec![SIGNALRGB_REGISTRY_KEY.to_owned()],
        working_directory: Some(instance_path),
        passthrough_environment: MINECRAFT_PASSTHROUGH_ENVIRONMENT
            .iter()
            .map(|s| SandboxArg::from(*s))
            .collect(),
        extra_environment,
        allow_network: true,
        is_jvm: true,
        die_with_parent: true,
        stdin: minecraft.stdin,
        stdout: minecraft.stdout,
        stderr: minecraft.stderr,
        app_container_name: "ModrinthMinecraftSandbox".into(),
        app_container_description:
            "Sandbox for Minecraft instances created by `modrinth-sandbox`"
                .into(),
        on_start,
        on_finish,
    })
}

/// Writes a JAR that's used to launch the game, unless it already exists, and returns
/// its path. The directory must not be writeable by the game.
///
/// This also attempts to remove other versions of the JAR that haven't been used
/// recently.
pub fn create_launcher_jar(
    dir: &Path,
    name: &str,
    bytes: &[u8],
) -> Result<PathBuf> {
    let hash_bytes = Sha1::digest(bytes);
    let prefix = format!("{name}-");
    let path = dir.join(format!("{prefix}{}.jar", hex::encode(hash_bytes)));

    fs::create_dir_all(dir)
        .with_context(|| format!("creating {}", dir.display()))?;

    if let Err(err) = remove_old_launcher_jars(dir, &prefix, &path) {
        tracing::warn!("failed to remove old jars from {dir:?}: {err}");
    }

    // Check that file exists and checksum matches
    if !check_sha1_hash(&path, &hash_bytes)? {
        let mut temp = path.clone();
        temp.add_extension(format!("{}", rand::rng().next_u32()));
        temp.add_extension("new");

        let mut temp_file = fs::File::create(&temp)?;

        temp_file.write_all(bytes)?;
        temp_file.flush()?;
        temp_file.sync_all()?;

        drop(temp_file);

        if let Err(err) = fs::rename(&temp, &path) {
            _ = fs::remove_file(&temp);
            // It may have been created by another launch at the same time
            if !check_sha1_hash(&path, &hash_bytes)? {
                return Err(err.into());
            }
        }
    }

    if let Err(err) = update_access_time(&path) {
        tracing::warn!("failed to update access time of {path:?}: {err}");
    }
    Ok(path)
}

fn update_access_time(path: &Path) -> std::io::Result<()> {
    fs::File::options()
        .write(true)
        .open(path)?
        .set_times(fs::FileTimes::new().set_accessed(SystemTime::now()))
}

fn remove_old_launcher_jars(
    dir: &Path,
    prefix: &str,
    current: &Path,
) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path == current
            || !entry
                .file_name()
                .as_encoded_bytes()
                .starts_with(prefix.as_bytes())
        {
            continue;
        }

        let result = fs::symlink_metadata(&path).and_then(|metadata| {
            let last_used =
                [metadata.accessed(), metadata.modified(), metadata.created()]
                    .into_iter()
                    .map(|time| time.unwrap_or(SystemTime::UNIX_EPOCH))
                    .fold(SystemTime::UNIX_EPOCH, SystemTime::max);
            let is_old = last_used
                .elapsed()
                .is_ok_and(|age| age > MAX_LAUNCHER_JAR_AGE);
            if metadata.is_file() && is_old {
                fs::remove_file(&path)?;
            }
            Ok(())
        });
        if let Err(err) = result {
            tracing::warn!("failed to remove old jar {path:?}: {err}");
        }
    }
    Ok(())
}

fn check_sha1_hash(path: &Path, expected_hash: &[u8]) -> anyhow::Result<bool> {
    let contents = match fs::read(path) {
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return Ok(false);
        }
        result => {
            result.with_context(|| format!("reading {}", path.display()))?
        }
    };

    Ok(expected_hash == &*Sha1::digest(&contents))
}

fn jvm_path_argument(argument: &str, separator: &str, path: &Path) -> OsString {
    let mut result = OsString::from(argument);
    result.push(separator);
    result.push(path);
    result
}

fn replace_path_placeholder(argument: &str, path: &Path) -> OsString {
    let (prefix, suffix) = argument
        .split_once("${path}")
        .expect("logging argument placeholder was validated");
    let mut result = OsString::from(prefix);
    result.push(path);
    result.push(suffix);
    result
}

/// Used by the illuminate mod (https://github.com/creeperkatze/mc-mods/blob/main/illuminate/)
fn signalrgb_effects_dir() -> Option<PathBuf> {
    if !cfg!(windows) {
        return None;
    }
    let home = std::env::home_dir()?;
    [
        home.join("Documents"),
        home.join("OneDrive").join("Documents"),
    ]
    .into_iter()
    .map(|documents| documents.join("WhirlwindFX").join("Effects"))
    .find(|effects| effects.is_dir())
}
