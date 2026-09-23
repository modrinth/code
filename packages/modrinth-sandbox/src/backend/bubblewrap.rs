use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process::Stdio,
};

use async_trait::async_trait;
use derive_more::Debug;
use eyre::{Context, Result};
use tokio::process::Command;

use crate::{
    backend::{
        Backend, SandboxChild, SandboxCommand, SandboxEnv, SandboxOutput,
    },
    util::path::find_command,
};

#[derive(Debug)]
pub struct Bubblewrap;

#[async_trait]
impl Backend for Bubblewrap {
    async fn init() -> Result<Box<dyn SandboxEnv>> {
        let bwrap = find_command(OsStr::new("bwrap"))
            .await
            .wrap_err("searching for `bwrap` executable")?;
        Ok(Box::new(BubblewrapEnv { bwrap }) as Box<dyn SandboxEnv>)
    }
}

#[derive(Debug)]
#[debug("BubblewrapEnv")]
pub struct BubblewrapEnv {
    bwrap: PathBuf,
}

#[async_trait]
impl SandboxEnv for BubblewrapEnv {
    async fn spawn(&self, command: SandboxCommand) -> Result<SandboxChild> {
        spawn(self, command).await
    }
}

const SYSTEM_RUNTIME_PATHS: &[&str] = &[
    "/lib",
    "/lib64",
    "/usr",
    "/etc/fonts",
    "/etc/hosts",
    "/etc/hostname",
    "/etc/ld.so.cache",
    "/etc/ld.so.conf",
    "/etc/ld.so.conf.d",
    "/etc/localtime",
    "/etc/nsswitch.conf",
    "/etc/pki",
    "/etc/resolv.conf",
    "/etc/ssl",
    "/etc/timezone",
    "/var/cache/fontconfig",
];

const GRAPHICS_RUNTIME_PATHS: &[&str] = &["/etc/glvnd", "/etc/vulkan"];

const AUDIO_RUNTIME_PATHS: &[&str] = &["/etc/alsa"];

const GRAPHICS_DEVICES: &[&str] = &[
    "/dev/dri",
    "/dev/udmabuf",
    // Graphics (mali)
    "/dev/mali",
    "/dev/mali0",
    "/dev/umplock",
    // Graphics (adreno)
    "/dev/kgsl-3d0",
    "/dev/ion",
    "/dev/nvidia-caps",
    "/dev/nvidiactl",
    "/dev/nvidia-modeset",
    "/dev/nvidia-uvm",
    "/dev/nvidia-uvm-tools",
];

const GRAPHICS_SYSFS_PATHS: &[&str] = &[
    "/sys/bus/pci",
    "/sys/class/drm",
    "/sys/dev/char",
    "/sys/devices",
    "/sys/module",
];

const DEFAULT_ENVIRONMENT: &[(&str, &str)] = &[
    ("HOME", "/tmp/home"),
    ("TMPDIR", "/tmp"),
    ("XDG_CACHE_HOME", "/tmp/cache"),
    ("XDG_CONFIG_HOME", "/tmp/config"),
    ("XDG_DATA_HOME", "/tmp/data"),
];

async fn spawn(
    env: &BubblewrapEnv,
    command: SandboxCommand,
) -> Result<SandboxChild> {
    let sandbox_environment = resolve_environment(&command);
    let mut os_command = Command::new(&env.bwrap);
    os_command.arg("--unshare-all");

    if command.die_with_parent {
        os_command.arg("--die-with-parent");
    }

    add_read_only_paths(&mut os_command, SYSTEM_RUNTIME_PATHS);

    os_command.args(["--proc", "/proc"]);
    os_command.args(["--dev", "/dev"]);

    add_read_only_paths(&mut os_command, GRAPHICS_RUNTIME_PATHS);
    add_device_paths(&mut os_command, GRAPHICS_DEVICES);
    for index in 0..16 {
        let path = format!("/dev/nvidia{index}");
        add_device_path(&mut os_command, &path);
    }
    add_read_only_paths(&mut os_command, GRAPHICS_SYSFS_PATHS);
    add_read_only_paths(&mut os_command, AUDIO_RUNTIME_PATHS);
    add_device_path(&mut os_command, "/dev/snd");

    os_command.args(["--tmpfs", "/tmp"]);
    os_command.args(["--tmpfs", "/var/tmp"]);
    for path in ["/tmp/home", "/tmp/cache", "/tmp/config", "/tmp/data"] {
        os_command.args(["--dir", path]);
    }

    expose_graphics_sockets(&mut os_command, &sandbox_environment);
    expose_audio_sockets(&mut os_command, &sandbox_environment);
    if command.network {
        os_command.arg("--share-net");
    }

    for path in &command.read_write_paths {
        os_command.arg("--bind");
        os_command.arg(path);
        os_command.arg(path);
    }
    for path in &command.read_only_paths {
        os_command.arg("--ro-bind");
        os_command.arg(path);
        os_command.arg(path);
    }

    os_command.args(["--remount-ro", "/"]);

    if let Some(path) = &command.working_directory {
        os_command.args([Path::new("--chdir"), path]);
    }

    os_command.env_clear();
    os_command.envs(sandbox_environment);
    os_command.stdin(Stdio::piped());
    match command.output {
        SandboxOutput::Piped => {
            os_command.stdout(Stdio::piped());
            os_command.stderr(Stdio::piped());
        }
        SandboxOutput::Inherit => {
            os_command.stdout(Stdio::inherit());
            os_command.stderr(Stdio::inherit());
        }
    }

    os_command.arg("--");
    os_command.arg(command.executable);
    os_command.args(command.args);

    let child = os_command.spawn().wrap_err("spawning child")?;
    Ok(SandboxChild {
        imp: super::linux::SandboxChild::new(child),
    })
}

fn add_read_only_paths(command: &mut Command, paths: &[&str]) {
    for path in paths {
        command.args(["--ro-bind-try", path, path]);
    }
}

fn add_device_paths(command: &mut Command, paths: &[&str]) {
    for path in paths {
        add_device_path(command, path);
    }
}

fn add_device_path(command: &mut Command, path: &str) {
    command.args(["--dev-bind-try", path, path]);
}

fn expose_graphics_sockets(
    os_command: &mut Command,
    environment: &[(OsString, OsString)],
) {
    add_read_only_path(os_command, Path::new("/tmp/.X11-unix"));

    if let (Some(runtime_directory), Some(display)) = (
        environment_value(environment, "XDG_RUNTIME_DIR"),
        environment_value(environment, "WAYLAND_DISPLAY"),
    ) {
        let display = PathBuf::from(display);
        let socket = if display.is_absolute() {
            display
        } else {
            PathBuf::from(runtime_directory).join(display)
        };
        add_read_only_path(os_command, &socket);
    }

    if let Some(path) = environment_value(environment, "XAUTHORITY") {
        add_read_only_path(os_command, Path::new(&path));
    }
}

fn expose_audio_sockets(
    os_command: &mut Command,
    environment: &[(OsString, OsString)],
) {
    let Some(runtime_directory) =
        environment_value(environment, "XDG_RUNTIME_DIR").map(PathBuf::from)
    else {
        return;
    };

    let pipewire_remote = environment_value(environment, "PIPEWIRE_REMOTE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("pipewire-0"));
    let pipewire_socket = if pipewire_remote.is_absolute() {
        pipewire_remote
    } else {
        runtime_directory.join(pipewire_remote)
    };
    add_read_only_path(os_command, &pipewire_socket);

    let pulse_socket = environment_value(environment, "PULSE_SERVER")
        .and_then(OsStr::to_str)
        .and_then(|server| server.strip_prefix("unix:").map(PathBuf::from))
        .unwrap_or_else(|| runtime_directory.join("pulse/native"));
    add_read_only_path(os_command, &pulse_socket);
}

fn add_read_only_path(command: &mut Command, path: &Path) {
    if path.is_absolute() {
        command.arg("--ro-bind-try");
        command.arg(path);
        command.arg(path);
    }
}

fn resolve_environment(command: &SandboxCommand) -> Vec<(OsString, OsString)> {
    let mut environment = Vec::new();

    for name in &command.passthrough_environment {
        if let Some(value) = std::env::var_os(name) {
            set_environment(&mut environment, name.clone(), value);
        }
    }
    for (name, value) in DEFAULT_ENVIRONMENT {
        set_environment(
            &mut environment,
            OsString::from(name),
            OsString::from(value),
        );
    }
    for (name, value) in &command.extra_environment {
        set_environment(&mut environment, name.clone(), value.clone());
    }

    environment
}

fn set_environment(
    environment: &mut Vec<(OsString, OsString)>,
    name: OsString,
    value: OsString,
) {
    if let Some((_, existing)) = environment
        .iter_mut()
        .find(|(existing, _)| existing == &name)
    {
        *existing = value;
    } else {
        environment.push((name, value));
    }
}

fn environment_value<'a>(
    environment: &'a [(OsString, OsString)],
    name: &str,
) -> Option<&'a OsStr> {
    environment
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_os_str())
}
