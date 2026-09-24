use std::{
    borrow::Cow,
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    os::fd::{AsRawFd, OwnedFd},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use async_trait::async_trait;
use derive_more::Debug;
use eyre::{Context, ContextCompat, Result, eyre};
use uuid::Uuid;

use crate::{
    backend::{Backend, SandboxChild, SandboxCommand, SandboxEnv},
    util::{argument::SandboxArg, path::find_command},
};

#[derive(Debug)]
pub struct Bubblewrap;

#[async_trait]
impl Backend for Bubblewrap {
    async fn init() -> Result<Box<dyn SandboxEnv>> {
        let bwrap = find_command(OsStr::new("bwrap"))
            .await
            .wrap_err("searching for `bwrap` executable")?;
        let xdg_dbus_proxy =
            find_command(OsStr::new("xdg-dbus-proxy"))
                .await
                .wrap_err("searching for `xdg-dbus-proxy` executable")?;
        let dev_null =
            super::unix::open_dev_null().wrap_err("opening /dev/null")?;
        Ok(Box::new(BubblewrapEnv {
            bwrap,
            xdg_dbus_proxy,
            dbus_proxy: OnceLock::new(),
            dev_null,
        }) as Box<dyn SandboxEnv>)
    }
}

#[derive(Debug)]
#[debug("BubblewrapEnv")]
pub struct BubblewrapEnv {
    bwrap: PathBuf,
    xdg_dbus_proxy: PathBuf,
    dbus_proxy: OnceLock<eyre::Result<DbusProxy>>,
    dev_null: libc::c_int,
}

impl SandboxEnv for BubblewrapEnv {
    fn spawn(&self, command: SandboxCommand) -> Result<SandboxChild> {
        spawn(self, command)
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
) -> Result<SandboxChild> {
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
            if card_names.contains(&filename.to_os_string()) {
                if let Ok(device) = path.join("device").canonicalize() {
                    let driver = device.join("driver");
                    builder.bind_if_exists(BindType::ReadOnly, device, true);
                    if let Ok(driver) = driver.canonicalize() {
                        builder.bind_if_exists(
                            BindType::ReadOnly,
                            driver,
                            true,
                        );
                    }
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

    // todo: seccomp filtering
    // todo: dbus proxy

    // Set up /.flatpak-info
    let flatpak_info_fd1 = super::unix::WriteableMemoryFile::open(
        c"modrinth-sandbox-bwrap-flatpak-info1",
    )?;
    let flatpak_info_fd1 = flatpak_info_fd1.write(c"[Application]\nname=com.modrinth.sandbox.ModrinthSandbox\n\n[Instance]\ninstance-id=0")?;
    let flatpak_info_fd2 = super::unix::WriteableMemoryFile::open(
        c"modrinth-sandbox-bwrap-flatpak-info2",
    )?;
    let flatpak_info_fd2 = flatpak_info_fd2.write(c"[Application]\nname=com.modrinth.sandbox.ModrinthSandbox\n\n[Instance]\ninstance-id=0")?;

    builder.push("--file");
    builder.push(format!("{}", flatpak_info_fd1.as_raw_fd()));
    builder.push("/.flatpak-info");
    builder.push("--ro-bind-data");
    builder.push(format!("{}", flatpak_info_fd2.as_raw_fd()));
    builder.push("/.flatpak-info");

    // Set up $XDG_RUNTIME_DIRS/.flatpak/0/bwrapinfo.json
    if !Path::new("/tmp").is_dir() {
        return Err(eyre!("/tmp folder doesn't exist"));
    }
    let tmp_bwrapinfo =
        format!("/tmp/modrinth-sandbox-bwrapinfo-{}.json", Uuid::now_v7());
    let tmp_bwrapinfo_fd: OwnedFd =
        std::fs::File::create(tmp_bwrapinfo.clone())?.into();
    builder.push("--info-fd");
    builder.push(format!("{}", tmp_bwrapinfo_fd.as_raw_fd()));

    builder.push("--ro-bind");
    builder.push(tmp_bwrapinfo);
    builder.push(
        xdg_runtime_dir
            .join(".flatpak")
            .join("0")
            .join("bwrapinfo.json"),
    );

    // Set up xdg-dbus-proxy
    let dbus_proxy =
        start_dbus_proxy(env, xdg_runtime_dir, command.die_with_parent)
            .map_err(|err| eyre!("{err:#}").wrap_err("starting dbus proxy"))?;

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

    // todo: wait for dbus proxy to start

    builder.push("--");
    builder.push(command.executable);
    for arg in command.args {
        builder.push(arg);
    }

    // Make sure any required dirs exist, create them if not
    for path in command.ensure_dirs_exist {
        std::fs::create_dir_all(&path)
            .wrap_err_with(|| eyre!("creating directory {path:?}"))?;
    }

    Ok(SandboxChild {
        imp: super::unix::spawn(
            env.bwrap.clone().into(),
            std::mem::take(&mut builder.arguments),
            environment,
            command.working_directory,
            vec![flatpak_info_fd1, flatpak_info_fd2, tmp_bwrapinfo_fd],
            env.dev_null,
            command.die_with_parent,
        )?,
    })
}

struct DbusProxy {
    proxy_session_path: PathBuf,
}

const DBUS_ADDRESS_ENV: &str = "DBUS_SESSION_BUS_ADDRESS";

fn start_dbus_proxy<'a>(
    env: &'a BubblewrapEnv,
    runtime_dir: &Path,
    die_with_parent: bool,
) -> Result<&'a DbusProxy, &'a eyre::Report> {
    env.dbus_proxy.get_or_init(|| {
        let session_bus_address = std::env::var_os(DBUS_ADDRESS_ENV)
            .wrap_err_with(|| eyre!("reading `{DBUS_ADDRESS_ENV}`"))?;

        let mut builder = BubblewrapCommandBuilder::default();

        builder.push("--new-session");

        let proxy_session_path = runtime_dir.join(format!("modrinth-xdg-dbus-proxy-session-{}", Uuid::new_v4()));

        builder.bind_if_exists(BindType::ReadOnly, Path::new("/usr"), true);
        builder.bind_if_exists(BindType::ReadOnly, Path::new("/lib64"), true);
        builder.bind_if_exists(BindType::ReadOnly, Path::new("/nix/store"), true);
        builder.bind_if_exists(BindType::ReadWrite, runtime_dir.to_path_buf(), true);

        let flatpak_info_fd1 = super::unix::WriteableMemoryFile::open(c"modrinth-sandbox-proxy-flatpak-info1")
            .wrap_err("creating flatpak-info memory file 1")?;
        let flatpak_info_fd1 = flatpak_info_fd1.write(c"[Application]\nname=com.modrinth.sandbox.ModrinthSandbox\n\n[Instance]\ninstance-id=0")
            .wrap_err("writing to flatpak-info memory file 1")?;
        let flatpak_info_fd2 = super::unix::WriteableMemoryFile::open(c"modrinth-sandbox-proxy-flatpak-info2")
            .wrap_err("creating flatpak-info memory file 2")?;
        let flatpak_info_fd2 = flatpak_info_fd2.write(c"[Application]\nname=com.modrinth.sandbox.ModrinthSandbox\n\n[Instance]\ninstance-id=0")
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
            Some(runtime_dir.to_path_buf()),
            vec![flatpak_info_fd1, flatpak_info_fd2],
            env.dev_null,
            die_with_parent
        ).wrap_err("spawning child")?;

        eyre::Ok(DbusProxy {
            proxy_session_path
        })
    }).as_ref()
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
