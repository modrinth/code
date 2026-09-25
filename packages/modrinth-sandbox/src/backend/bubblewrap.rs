use std::{
    borrow::Cow, collections::BTreeMap, ffi::{CStr, CString, OsStr, OsString}, io::{PipeReader, PipeWriter}, os::fd::{AsRawFd, OwnedFd}, path::{Path, PathBuf},
};

use async_trait::async_trait;
use derive_more::Debug;
use eyre::{Context, ContextCompat, Result, eyre};
use libseccomp::{ScmpAction, ScmpArgCompare, ScmpCompareOp, ScmpFilterContext, ScmpSyscall};
use uuid::Uuid;

use crate::{
    backend::{Backend, SandboxChild, SandboxChildImpl, SandboxChildTrait, SandboxCommand, SandboxEnv, unix::UnixSandboxChild}, util::{argument::SandboxArg, path::find_command},
};

#[derive(Debug)]
pub struct Bubblewrap;

#[async_trait]
impl Backend for Bubblewrap {
    async fn init() -> Result<Box<dyn SandboxEnv>> {
        init().await.map(|env| Box::new(env) as Box<dyn SandboxEnv>)
    }
}

async fn init() -> Result<BubblewrapEnv> {
    let bwrap = find_command("bwrap")
        .await
        .wrap_err("searching for `bwrap` executable")?;
    let xdg_dbus_proxy = find_command("xdg-dbus-proxy")
        .await
        .wrap_err("searching for `xdg-dbus-proxy` executable")?;
    let dev_null =
        super::unix::open_dev_null().wrap_err("opening /dev/null")?;
    Ok(BubblewrapEnv {
        bwrap,
        xdg_dbus_proxy,
        dev_null,
    })
}

#[derive(Debug, Clone)]
#[debug("BubblewrapEnv")]
pub struct BubblewrapEnv {
    bwrap: PathBuf,
    xdg_dbus_proxy: PathBuf,
    dev_null: libc::c_int,
}

#[async_trait]
impl SandboxEnv for BubblewrapEnv {
    async fn spawn(&self, command: SandboxCommand) -> Result<SandboxChild> {
        let this = self.clone();
        let bubblewrap_child = tokio::task::spawn_blocking(move || spawn(&this, command))
            .await
            .context("spawn task dropped")??;
        Ok(SandboxChild(SandboxChildImpl::Bubblewrap(bubblewrap_child)))
    }
}

const DEV_BINDS: &[&str] = &[
    // Graphics
    "/dev/dri",
    "/dev/udmabuf",
    // Graphics (mali)
    "/dev/mali",
    "/dev/mali0",
    "/dev/umplock",
    // Graphics (adreno)
    "/dev/kgsl-3d0",
    "/dev/ion",
    // System info
    "/dev/disk/by-uuid",
    "/dev/dm",
    "/dev/loop",
    "/dev/mapper",
    "/dev/ram",
    // NT Sync Primitives (Wine)
    "/dev/ntsync",
    // Raw ALSA
    "/dev/snd",
    // AMD Compute
    "/dev/kfd",
];

const SYSTEM_FILES_RO: &[&str] = &[
    "/bin",
    "/sbin",
    "/usr",
    "/lib",
    "/lib32",
    "/lib64",
    "/etc/alternatives",
    "/etc/resolv.conf",
    "/run/systemd/resolve",
    "/usr/share/ca-certificates",
    "/etc/ca-certificates",
    "/etc/ssl",
    "/etc/pki",
    "/etc/pkcs11",
    "/etc/hosts",
    "/etc/ld.so.cache",
    "/etc/ld.so.conf.d",
    "/etc/localtime",
    "/etc/os-release",
    "/etc/machine-id",
    "/etc/timezone",
    "/etc/fonts",
    "/sys/dev/char",
    "/sys/bus/pci/devices",
    "/sys/devices/system/cpu",
    "/sys/devices/virtual/dmi/id",
    "/sys/class/net",
    "/sys/firmware/devicetree/base/model",
    "/sys/class/power_supply",
    "/sys/class/hwmon",
    "/sys/class/thermal",
    "/sys/class/drm",
    "/etc/glvnd",
    "/etc/vulkan",
    "/etc/alsa",
    "/nix/store",
    // NVIDIA kernel module state (needed for driver init in user ns)
    "/sys/module/nvidia",
    "/sys/module/nvidia_drm",
    "/sys/module/nvidia_modeset",
    "/sys/module/nvidia_uvm",
];

#[derive(Default)]
struct BubblewrapCommandBuilder {
    arguments: Vec<SandboxArg>,
    bound_paths: Vec<Cow<'static, Path>>,
}

#[derive(Debug)]
enum BindType {
    Device,
    ReadOnly,
    ReadWrite,
}

impl BubblewrapCommandBuilder {
    pub fn push(&mut self, arg: impl Into<SandboxArg>) {
        self.arguments.push(arg.into());
    }

    pub fn bind_if_exists(
        &mut self,
        bind_type: BindType,
        path: impl Into<Cow<'static, Path>>,
        follow_symlinks: bool,
    ) {
        self.bind_if_exists_inner(bind_type, path.into(), follow_symlinks);
    }

    fn bind_if_exists_inner(
        &mut self,
        bind_type: BindType,
        mut path: Cow<'static, Path>,
        follow_symlinks: bool,
    ) {
        if follow_symlinks {
            loop {
                let Ok(resolved) = path.canonicalize() else {
                    return;
                };

                if resolved == path {
                    break;
                } else {
                    for already_bound in &self.bound_paths {
                        if path.starts_with(already_bound) {
                            return;
                        }
                    }
                    self.bound_paths.push(path.clone());

                    if !path.starts_with(&resolved) {
                        self.push("--symlink");
                        self.push(resolved.clone());
                        self.push(path);
                    }

                    path = Cow::Owned(resolved);
                }
            }
        } else if !path.exists() {
            return;
        }

        for already_bound in &self.bound_paths {
            if path.starts_with(already_bound) {
                return;
            }
        }
        self.bound_paths.push(path.clone());

        match bind_type {
            BindType::Device => self.push("--dev-bind-try"),
            BindType::ReadOnly => self.push("--ro-bind-try"),
            BindType::ReadWrite => self.push("--bind-try"),
        }

        self.push(path.clone());
        self.push(path);
    }
}

fn spawn(
    env: &BubblewrapEnv,
    mut command: SandboxCommand,
) -> Result<BubblewrapSandboxChild> {
    let mut builder = BubblewrapCommandBuilder::default();

    let Some(directories) = directories::BaseDirs::new() else {
        return Err(eyre!("unable to determine base directories"));
    };

    let mut environment = command.take_environment();

    // Namespaces
    builder.push("--unshare-all");
    if command.network {
        builder.push("--share-net");
    }

    // Special filesystems
    builder.push("--proc");
    builder.push("/proc");
    builder.push("--dev");
    builder.push("/dev");
    builder.push("--tmpfs");
    builder.push("/tmp");

    // Other arguments
    if command.die_with_parent {
        builder.push("--die-with-parent");
    }
    if let Some(path) = command.working_directory.clone() {
        builder.push("--chdir");
        builder.push(path);
    }

    // Binds
    for dev_bind in DEV_BINDS {
        builder.bind_if_exists(BindType::Device, Path::new(*dev_bind), true);
    }
    if let Ok(read_dir) = std::fs::read_dir("/dev") {
        for entry in read_dir {
            let Ok(entry) = entry else {
                break;
            };
            let path = entry.path();
            let Some(file_name) = path.file_name() else {
                continue;
            };
            if file_name.as_encoded_bytes().starts_with(b"nvidia") {
                builder.bind_if_exists(BindType::Device, path, true);
            }
        }
    }

    for file_ro in SYSTEM_FILES_RO {
        builder.bind_if_exists(BindType::ReadOnly, Path::new(*file_ro), true);
    }

    // Bind graphics card devices
    let card_names = get_card_names();
    if let Ok(devices) = std::fs::read_dir("/sys/dev/char") {
        for entry in devices {
            let Ok(entry) = entry else {
                continue;
            };
            let Ok(path) = entry.path().canonicalize() else {
                continue;
            };
            let Some(filename) = path.file_name() else {
                continue;
            };
            if card_names.contains(&filename.to_os_string())
                && let Ok(device) = path.join("device").canonicalize()
            {
                let driver = device.join("driver");
                builder.bind_if_exists(BindType::ReadOnly, device, true);
                if let Ok(driver) = driver.canonicalize() {
                    builder.bind_if_exists(BindType::ReadOnly, driver, true);
                }
            }
        }
    }

    // Bind everything in $PATH
    if let Some(path) = std::env::var_os("PATH") {
        for path in std::env::split_paths(&path) {
            builder.bind_if_exists(BindType::ReadOnly, path, true);
        }
    }

    // Bind files in xdg runtime dir
    let xdg_runtime_dir = directories
        .runtime_dir()
        .unwrap_or(Path::new("/run/user/1000"));
    builder.push("--dir");
    builder.push(xdg_runtime_dir.to_path_buf());

    let wayland_display_path = xdg_runtime_dir.join(
        std::env::var_os("WAYLAND_DISPLAY")
            .as_deref()
            .unwrap_or(OsStr::new("wayland-0")),
    );
    builder.bind_if_exists(BindType::ReadOnly, wayland_display_path, true);

    let pipewire_path = xdg_runtime_dir.join("pipewire-0");
    builder.bind_if_exists(BindType::ReadOnly, pipewire_path, true);

    let document_portal_path = xdg_runtime_dir.join("doc");
    builder.bind_if_exists(BindType::ReadWrite, document_portal_path, true);

    if let Some(pulse_server) = std::env::var_os("PULSE_SERVER") {
        if let Some(pulse_server_path) =
            pulse_server.as_encoded_bytes().strip_prefix(b"unix:")
        {
            let pulse_server_path = unsafe {
                OsStr::from_encoded_bytes_unchecked(pulse_server_path)
            };
            builder.bind_if_exists(
                BindType::ReadOnly,
                PathBuf::from(pulse_server_path),
                true,
            );
        }
    } else {
        let pulse_path = xdg_runtime_dir.join("pulse");
        builder.bind_if_exists(BindType::ReadOnly, pulse_path, true);
    }

    // Bind a bunch of pulse audio stuff
    let pulse_config_dir = directories.config_dir().join("pulse");
    builder.bind_if_exists(BindType::ReadOnly, pulse_config_dir, true);
    let pulse_home_config_dir = directories.home_dir().join(".pulse");
    builder.bind_if_exists(BindType::ReadOnly, pulse_home_config_dir, true);
    let asound_home_config_dir = directories.home_dir().join(".asoundrc");
    builder.bind_if_exists(BindType::ReadOnly, asound_home_config_dir, true);

    builder.bind_if_exists(BindType::ReadWrite, Path::new("/run/pulse"), true);

    if let Some(pulse_clientconfig) = std::env::var_os("PULSE_CLIENTCONFIG") {
        builder.bind_if_exists(
            BindType::ReadOnly,
            PathBuf::from(pulse_clientconfig),
            true,
        );
    }

    // Bind X11 sockets/xauthority
    let display_index = std::env::var_os("DISPLAY")
        .and_then(|display| {
            let display_bytes = display.as_encoded_bytes();
            if display_bytes.len() == 2
                && display_bytes[0] == b':'
                && display_bytes[1] >= b'0'
                && display_bytes[1] <= b'9'
            {
                Some(display_bytes[1] - b'0')
            } else {
                None
            }
        })
        .unwrap_or(0);

    builder.bind_if_exists(
        BindType::ReadOnly,
        PathBuf::from(format!("/tmp/.X11-unix/X{display_index}")),
        true,
    );
    if let Some(xauthority) = std::env::var_os("XAUTHORITY") {
        builder.bind_if_exists(
            BindType::ReadOnly,
            PathBuf::from(xauthority),
            true,
        );
    } else {
        builder.bind_if_exists(
            BindType::ReadOnly,
            directories.home_dir().join(".Xauthority"),
            true,
        );
    }

    // Bind executable
    builder.bind_if_exists(
        BindType::ReadOnly,
        command.executable.clone(),
        true,
    );

    // Bind custom paths
    for path in command.read_only_paths {
        builder.bind_if_exists(BindType::ReadOnly, path.clone(), false);
    }
    for path in command.read_write_paths {
        builder.bind_if_exists(BindType::ReadWrite, path.clone(), false);
    }

    let flatpak_instances_dir = xdg_runtime_dir.join(".flatpak");
    let mut instance_id = rand::random::<u32>() & 0x7FFFFFFF;
    let mut instance_dir = flatpak_instances_dir.join(instance_id.to_string());
    _ = std::fs::create_dir_all(&flatpak_instances_dir);
    for attempt in 0..=100 {
        if std::fs::create_dir(&instance_dir).is_ok() {
            break;
        }

        if attempt == 100 {
            return Err(eyre!("Unable to find unique instance-id for .flatpak-info"));
        } else {
            instance_id = rand::random::<u32>() & 0x7FFFFFFF;
            instance_dir = flatpak_instances_dir.join(instance_id.to_string());
        }
    }

    // Set up /.flatpak-info
    let flatpak_info = format!(
        "[Application]\nname=com.modrinth.sandbox.ModrinthSandbox\n\n[Instance]\ninstance-id={}\n\0",
        instance_id
    );
    let flatpak_info: CString = CString::from_vec_with_nul(flatpak_info.into_bytes())?;
    let flatpak_info_fd1 = super::unix::WriteableMemoryFile::open(
        c"modrinth-sandbox-bwrap-flatpak-info1",
    )?;
    let flatpak_info_fd1 = flatpak_info_fd1.write(flatpak_info.as_c_str())?;
    let flatpak_info_fd2 = super::unix::WriteableMemoryFile::open(
        c"modrinth-sandbox-bwrap-flatpak-info2",
    )?;
    let flatpak_info_fd2 = flatpak_info_fd2.write(flatpak_info.as_c_str())?;

    builder.push("--file");
    builder.push(format!("{}", flatpak_info_fd1.as_raw_fd()));
    builder.push("/.flatpak-info");
    builder.push("--ro-bind-data");
    builder.push(format!("{}", flatpak_info_fd2.as_raw_fd()));
    builder.push("/.flatpak-info");

    // Set up $XDG_RUNTIME_DIRS/.flatpak/{instance_id}/bwrapinfo.json
    if !Path::new("/tmp").is_dir() {
        return Err(eyre!("/tmp folder doesn't exist"));
    }
    let bwrapinfo = instance_dir.join("bwrapinfo.json");
    let bwrapinfo_fd: OwnedFd = std::fs::File::create(bwrapinfo.clone())?.into();
    builder.push("--info-fd");
    builder.push(format!("{}", bwrapinfo_fd.as_raw_fd()));

    // Set up xdg-dbus-proxy
    let dbus_proxy =
        start_dbus_proxy(env, xdg_runtime_dir, &flatpak_info, command.die_with_parent)
            .wrap_err("starting dbus proxy")?;

    builder.push("--bind");
    builder.push(dbus_proxy.proxy_session_path.clone());
    let mapped_bus_dir = xdg_runtime_dir.join("bus");
    builder.push(mapped_bus_dir.clone());

    let mut dbus_session_bus_address = OsString::new();
    dbus_session_bus_address.push("unix:path=");
    dbus_session_bus_address.push(mapped_bus_dir);

    environment.insert(
        "DBUS_SESSION_BUS_ADDRESS".into(),
        dbus_session_bus_address.into(),
    );
    if directories.runtime_dir().is_none() {
        environment.insert("XDG_RUNTIME_DIR".into(), "/run/user/1000".into());
    }

    // Set up seccomp filtering
    let seccomp_fd = create_seccomp_filter()?;
    builder.push("--seccomp");
    builder.push(format!("{}", seccomp_fd.as_raw_fd()));

    builder.push("--");
    builder.push(command.executable);
    for arg in command.args {
        builder.push(arg);
    }

    // Make sure any required dirs exist, create them if not
    // Do this right before we do the actual spawn, so that we don't create these
    // if we have an error earlier on
    for path in command.ensure_dirs_exist {
        std::fs::create_dir_all(&path)
            .wrap_err_with(|| eyre!("creating directory {path:?}"))?;
    }

    Ok(BubblewrapSandboxChild {
        child: super::unix::spawn(
            env.bwrap.clone().into(),
            std::mem::take(&mut builder.arguments),
            environment,
            command.stdin,
            command.stdout,
            command.stderr,
            command.working_directory,
            vec![flatpak_info_fd1, flatpak_info_fd2, bwrapinfo_fd, seccomp_fd],
            env.dev_null,
            command.die_with_parent,
        )?,
        instance_dir,
        _dbus_proxy: dbus_proxy
    })
}

#[derive(Debug)]
pub(crate) struct BubblewrapSandboxChild {
    child: UnixSandboxChild,
    instance_dir: PathBuf,
    _dbus_proxy: DbusProxy,
}

impl Drop for BubblewrapSandboxChild {
    fn drop(&mut self) {
        _ = std::fs::remove_dir_all(&self.instance_dir);
    }
}

#[async_trait]
impl SandboxChildTrait for BubblewrapSandboxChild {
    fn id(&self) -> Option<u32> {
        self.child.id()
    }

    fn try_wait(&mut self) -> Result<Option<super::SandboxExitStatus>> {
        self.child.try_wait()
    }

    async fn wait(&mut self) -> Result<super::SandboxExitStatus> {
        self.child.wait().await
    }

    async fn kill(&mut self) -> Result<()> {
        self.child.kill().await
    }

    fn take_stdin(&mut self) -> Option<PipeWriter>  {
        self.child.take_stdin()
    }

    fn take_stdout(&mut self) -> Option<PipeReader>  {
        self.child.take_stdout()
    }

    fn take_stderr(&mut self) -> Option<PipeReader>  {
        self.child.take_stderr()
    }
}

#[derive(Debug)]
struct DbusProxy {
    proxy_session_path: PathBuf,
    _keep_alive_read_fd: OwnedFd,
}

impl Drop for DbusProxy {
    fn drop(&mut self) {
        _ = std::fs::remove_file(&self.proxy_session_path);
    }
}

fn start_dbus_proxy(
    env: &BubblewrapEnv,
    runtime_dir: &Path,
    flatpak_info: &CStr,
    die_with_parent: bool,
) -> Result<DbusProxy> {
    const DBUS_ADDRESS_ENV: &str = "DBUS_SESSION_BUS_ADDRESS";

    let (keep_alive_read_fd, keep_alive_write_fd) = super::unix::open_pipe()?;

    let session_bus_address = std::env::var_os(DBUS_ADDRESS_ENV)
        .wrap_err_with(|| eyre!("reading `{DBUS_ADDRESS_ENV}`"))?;

    let mut builder = BubblewrapCommandBuilder::default();

    builder.push("--new-session");

    let proxy_session_path = runtime_dir.join(format!("modrinth-xdg-dbus-proxy-session-{}", Uuid::now_v7()));

    builder.bind_if_exists(BindType::ReadOnly, Path::new("/usr"), true);
    builder.bind_if_exists(BindType::ReadOnly, Path::new("/lib64"), true);
    builder.bind_if_exists(BindType::ReadOnly, Path::new("/nix/store"), true);
    builder.bind_if_exists(BindType::ReadWrite, runtime_dir.to_path_buf(), true);

    let flatpak_info_fd1 = super::unix::WriteableMemoryFile::open(c"modrinth-sandbox-proxy-flatpak-info1")
        .wrap_err("creating flatpak-info memory file 1")?;
    let flatpak_info_fd1 = flatpak_info_fd1.write(flatpak_info)
        .wrap_err("writing to flatpak-info memory file 1")?;
    let flatpak_info_fd2 = super::unix::WriteableMemoryFile::open(c"modrinth-sandbox-proxy-flatpak-info2")
        .wrap_err("creating flatpak-info memory file 2")?;
    let flatpak_info_fd2 = flatpak_info_fd2.write(flatpak_info)
        .wrap_err("writing to flatpak-info memory file 2")?;

    builder.push("--file");
    builder.push(format!("{}", flatpak_info_fd1.as_raw_fd()));
    builder.push("/.flatpak-info");
    builder.push("--ro-bind-data");
    builder.push(format!("{}", flatpak_info_fd2.as_raw_fd()));
    builder.push("/.flatpak-info");

    if die_with_parent {
        builder.push("--die-with-parent");
    }

    builder.push("--");
    builder.push(env.xdg_dbus_proxy.clone());
    builder.push(session_bus_address);
    builder.push(proxy_session_path.clone());
    builder.push(format!("--fd={}", keep_alive_write_fd.as_raw_fd()));
    builder.push("--filter");
    builder.push("--talk=com.feralinteractive.GameMode");
    builder.push("--call=com.feralinteractive.GameMode=/com/feralinteractive/GameMode");
    builder.push("--talk=org.kde.StatusNotifierWatcher");
    builder.push("--call=org.kde.StatusNotifierWatcher=/StatusNotifierWatcher");
    builder.push("--talk=org.freedesktop.Notifications");
    builder.push("--call=org.freedesktop.Notifications=/org/freedesktop/Notifications");
    builder.push("--talk=org.freedesktop.portal.*");
    builder.push("--call=org.freedesktop.portal.Desktop=org.freedesktop.portal.Settings.Read@/org/freedesktop/portal/desktop");
    builder.push("--broadcast=org.freedesktop.portal.Desktop=org.freedesktop.portal.Settings.SettingChanged@/org/freedesktop/portal/desktop");
    builder.push("--talk=org.mpris.MediaPlayer2.*");

    let mut environment = BTreeMap::default();
    for (arg, value) in std::env::vars_os() {
        environment.insert(arg.into(), value.into());
    }

    super::unix::spawn(
        env.bwrap.clone().into(),
        std::mem::take(&mut builder.arguments),
        environment,
        crate::SandboxStdio::Null,
        crate::SandboxStdio::Null,
        crate::SandboxStdio::Null,
        Some(runtime_dir.to_path_buf()),
        vec![flatpak_info_fd1, flatpak_info_fd2, keep_alive_write_fd],
        env.dev_null,
        die_with_parent
    ).wrap_err("spawning child")?;

    // Wait for proxy to start by reading from fd
    let mut buf = 0 as libc::c_char;
    unsafe {
        let start = std::time::Instant::now();
        if libc::read(keep_alive_read_fd.as_raw_fd(), &mut buf as *mut libc::c_char as *mut _, 1) != 1 {
            return Err(eyre!("Failed to sync with xdg-dbus-proxy"));
        }
        tracing::info!("xdg-dbus-proxy took {:?} to start", std::time::Instant::now() - start);
    }

    eyre::Ok(DbusProxy {
        proxy_session_path,
        _keep_alive_read_fd: keep_alive_read_fd
    })
}

fn get_card_names() -> Vec<OsString> {
    let mut card_names = Vec::new();
    let Ok(read_dir) = std::fs::read_dir("/dev/dri") else {
        return card_names;
    };

    for entry in read_dir.flatten() {
        card_names.push(entry.file_name());
    }

    card_names
}

// Syscall allowlist copied from https://github.com/moby/profiles/blob/fa50b7287199d1c781284d1a34d1395a62e57f1e/seccomp/default.json
// Licensed as Apache 2.0 (https://github.com/moby/profiles/blob/fa50b7287199d1c781284d1a34d1395a62e57f1e/LICENSE)
const ALLOWED_SYSCALLS: &[&'static str] = &[
    "accept",
    "accept4",
    "access",
    "adjtimex",
    "alarm",
    "bind",
    "brk",
    "cachestat",
    "capget",
    "capset",
    "chdir",
    "chmod",
    "chown",
    "chown32",
    "clock_adjtime",
    "clock_adjtime64",
    "clock_getres",
    "clock_getres_time64",
    "clock_gettime",
    "clock_gettime64",
    "clock_nanosleep",
    "clock_nanosleep_time64",
    "close",
    "close_range",
    "connect",
    "copy_file_range",
    "creat",
    "dup",
    "dup2",
    "dup3",
    "epoll_create",
    "epoll_create1",
    "epoll_ctl",
    "epoll_ctl_old",
    "epoll_pwait",
    "epoll_pwait2",
    "epoll_wait",
    "epoll_wait_old",
    "eventfd",
    "eventfd2",
    "execve",
    "execveat",
    "exit",
    "exit_group",
    "faccessat",
    "faccessat2",
    "fadvise64",
    "fadvise64_64",
    "fallocate",
    "fanotify_mark",
    "fchdir",
    "fchmod",
    "fchmodat",
    "fchmodat2",
    "fchown",
    "fchown32",
    "fchownat",
    "fcntl",
    "fcntl64",
    "fdatasync",
    "fgetxattr",
    "flistxattr",
    "flock",
    "fork",
    "fremovexattr",
    "fsetxattr",
    "fstat",
    "fstat64",
    "fstatat64",
    "fstatfs",
    "fstatfs64",
    "fsync",
    "ftruncate",
    "ftruncate64",
    "futex",
    "futex_requeue",
    "futex_time64",
    "futex_wait",
    "futex_waitv",
    "futex_wake",
    "futimesat",
    "getcpu",
    "getcwd",
    "getdents",
    "getdents64",
    "getegid",
    "getegid32",
    "geteuid",
    "geteuid32",
    "getgid",
    "getgid32",
    "getgroups",
    "getgroups32",
    "getitimer",
    "getpeername",
    "getpgid",
    "getpgrp",
    "getpid",
    "getppid",
    "getpriority",
    "getrandom",
    "getresgid",
    "getresgid32",
    "getresuid",
    "getresuid32",
    "getrlimit",
    "get_robust_list",
    "getrusage",
    "getsid",
    "getsockname",
    "getsockopt",
    "get_thread_area",
    "gettid",
    "gettimeofday",
    "getuid",
    "getuid32",
    "getxattr",
    "getxattrat",
    "inotify_add_watch",
    "inotify_init",
    "inotify_init1",
    "inotify_rm_watch",
    "io_cancel",
    "ioctl",
    "io_destroy",
    "io_getevents",
    "io_pgetevents",
    "io_pgetevents_time64",
    "ioprio_get",
    "ioprio_set",
    "io_setup",
    "io_submit",
    "ipc",
    "kill",
    "landlock_add_rule",
    "landlock_create_ruleset",
    "landlock_restrict_self",
    "lchown",
    "lchown32",
    "lgetxattr",
    "link",
    "linkat",
    "listen",
    "listmount",
    "listxattr",
    "listxattrat",
    "llistxattr",
    "_llseek",
    "lremovexattr",
    "lseek",
    "lsetxattr",
    "lstat",
    "lstat64",
    "madvise",
    "map_shadow_stack",
    "membarrier",
    "memfd_create",
    "memfd_secret",
    "mincore",
    "mkdir",
    "mkdirat",
    "mknod",
    "mknodat",
    "mlock",
    "mlock2",
    "mlockall",
    "mmap",
    "mmap2",
    "mprotect",
    "mq_getsetattr",
    "mq_notify",
    "mq_open",
    "mq_timedreceive",
    "mq_timedreceive_time64",
    "mq_timedsend",
    "mq_timedsend_time64",
    "mq_unlink",
    "mremap",
    "mseal",
    "msgctl",
    "msgget",
    "msgrcv",
    "msgsnd",
    "msync",
    "munlock",
    "munlockall",
    "munmap",
    "name_to_handle_at",
    "nanosleep",
    "newfstatat",
    "_newselect",
    "open",
    "openat",
    "openat2",
    "pause",
    "pidfd_open",
    "pidfd_send_signal",
    "pipe",
    "pipe2",
    "pkey_alloc",
    "pkey_free",
    "pkey_mprotect",
    "poll",
    "ppoll",
    "ppoll_time64",
    "prctl",
    "pread64",
    "preadv",
    "preadv2",
    "prlimit64",
    "process_mrelease",
    "pselect6",
    "pselect6_time64",
    "pwrite64",
    "pwritev",
    "pwritev2",
    "read",
    "readahead",
    "readlink",
    "readlinkat",
    "readv",
    "recv",
    "recvfrom",
    "recvmmsg",
    "recvmmsg_time64",
    "recvmsg",
    "remap_file_pages",
    "removexattr",
    "removexattrat",
    "rename",
    "renameat",
    "renameat2",
    "restart_syscall",
    "riscv_hwprobe",
    "rmdir",
    "rseq",
    "rt_sigaction",
    "rt_sigpending",
    "rt_sigprocmask",
    "rt_sigqueueinfo",
    "rt_sigreturn",
    "rt_sigsuspend",
    "rt_sigtimedwait",
    "rt_sigtimedwait_time64",
    "rt_tgsigqueueinfo",
    "sched_getaffinity",
    "sched_getattr",
    "sched_getparam",
    "sched_get_priority_max",
    "sched_get_priority_min",
    "sched_getscheduler",
    "sched_rr_get_interval",
    "sched_rr_get_interval_time64",
    "sched_setaffinity",
    "sched_setattr",
    "sched_setparam",
    "sched_setscheduler",
    "sched_yield",
    "seccomp",
    "select",
    "semctl",
    "semget",
    "semop",
    "semtimedop",
    "semtimedop_time64",
    "send",
    "sendfile",
    "sendfile64",
    "sendmmsg",
    "sendmsg",
    "sendto",
    "setfsgid",
    "setfsgid32",
    "setfsuid",
    "setfsuid32",
    "setgid",
    "setgid32",
    "setgroups",
    "setgroups32",
    "setitimer",
    "setpgid",
    "setpriority",
    "setregid",
    "setregid32",
    "setresgid",
    "setresgid32",
    "setresuid",
    "setresuid32",
    "setreuid",
    "setreuid32",
    "setrlimit",
    "set_robust_list",
    "setsid",
    "setsockopt",
    "set_thread_area",
    "set_tid_address",
    "setuid",
    "setuid32",
    "setxattr",
    "setxattrat",
    "shmat",
    "shmctl",
    "shmdt",
    "shmget",
    "shutdown",
    "sigaltstack",
    "signalfd",
    "signalfd4",
    "sigprocmask",
    "sigreturn",
    "socketcall",
    "socketpair",
    "splice",
    "stat",
    "stat64",
    "statfs",
    "statfs64",
    "statmount",
    "statx",
    "symlink",
    "symlinkat",
    "sync",
    "sync_file_range",
    "syncfs",
    "sysinfo",
    "tee",
    "tgkill",
    "time",
    "timer_create",
    "timer_delete",
    "timer_getoverrun",
    "timer_gettime",
    "timer_gettime64",
    "timer_settime",
    "timer_settime64",
    "timerfd_create",
    "timerfd_gettime",
    "timerfd_gettime64",
    "timerfd_settime",
    "timerfd_settime64",
    "times",
    "tkill",
    "truncate",
    "truncate64",
    "ugetrlimit",
    "umask",
    "uname",
    "unlink",
    "unlinkat",
    "uretprobe",
    "utime",
    "utimensat",
    "utimensat_time64",
    "utimes",
    "vfork",
    "vmsplice",
    "wait4",
    "waitid",
    "waitpid",
    "write",
    "writev",
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    "arch_prctl",
];

fn create_seccomp_filter() -> Result<std::os::fd::OwnedFd> {
    // We could cache the fd and dup, but I'm worried that this might allow the child
    // to modify the memfd even when it is sealed, so we just recreate the bpf memfd from scratch

    let Ok(mut filter) = ScmpFilterContext::new(ScmpAction::Errno(libc::EPERM)) else {
        return Err(eyre!("unable to init seccomp filter context"));
    };

    for syscall in ALLOWED_SYSCALLS {
        let Ok(syscall) = ScmpSyscall::from_name(*syscall) else {
            continue;
        };
        _ = filter.add_rule(ScmpAction::Allow, syscall);
    }
    if let Ok(syscall) = ScmpSyscall::from_name("clone") {
        let disallowed_clones = libc::CLONE_NEWNS | libc::CLONE_NEWUTS | libc::CLONE_NEWIPC | libc::CLONE_NEWUSER
            | libc::CLONE_NEWPID | libc::CLONE_NEWNET | libc::CLONE_NEWCGROUP;
        _ = filter.add_rule_conditional(ScmpAction::Allow, syscall, &[
           ScmpArgCompare::new(0, ScmpCompareOp::MaskedEqual(disallowed_clones as u64), 0)
        ]);
    }
    if let Ok(syscall) = ScmpSyscall::from_name("clone3") {
        _ = filter.add_rule(ScmpAction::Errno(libc::ENOSYS), syscall);
    }
    if let Ok(syscall) = ScmpSyscall::from_name("socket") {
        _ = filter.add_rule_conditional(ScmpAction::Allow, syscall, &[
           ScmpArgCompare::new(0, ScmpCompareOp::NotEqual, libc::AF_VSOCK as u64)
        ]);
    }
    if let Ok(syscall) = ScmpSyscall::from_name("personality") {
        _ = filter.add_rule_conditional(ScmpAction::Allow, syscall, &[
           ScmpArgCompare::new(0, ScmpCompareOp::Equal, 0x0)
        ]);
        _ = filter.add_rule_conditional(ScmpAction::Allow, syscall, &[
           ScmpArgCompare::new(0, ScmpCompareOp::Equal, 0x8)
        ]);
        _ = filter.add_rule_conditional(ScmpAction::Allow, syscall, &[
           ScmpArgCompare::new(0, ScmpCompareOp::Equal, 0x20000)
        ]);
        _ = filter.add_rule_conditional(ScmpAction::Allow, syscall, &[
           ScmpArgCompare::new(0, ScmpCompareOp::Equal, 0x20008)
        ]);
        _ = filter.add_rule_conditional(ScmpAction::Allow, syscall, &[
           ScmpArgCompare::new(0, ScmpCompareOp::Equal, 0xffffffff)
        ]);
    }
    if let Ok(syscall) = ScmpSyscall::from_name("ioctl") {
        _ = filter.add_rule_conditional(ScmpAction::Errno(libc::EPERM), syscall, &[
           ScmpArgCompare::new(1, ScmpCompareOp::MaskedEqual(0xFFFFFFFF), libc::TIOCSTI)
        ]);
    }
    if let Ok(syscall) = ScmpSyscall::from_name("ioctl") {
        _ = filter.add_rule_conditional(ScmpAction::Errno(libc::EPERM), syscall, &[
           ScmpArgCompare::new(1, ScmpCompareOp::MaskedEqual(0xFFFFFFFF), libc::TIOCLINUX)
        ]);
    }

    let fd = super::unix::WriteableMemoryFile::open(c"modrinth-sandbox-seccomp-bpf")?;
    let fd = fd.write_filter(filter)?;
    Ok(fd)
}
