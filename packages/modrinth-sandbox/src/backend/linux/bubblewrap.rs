use std::{
    borrow::Cow,
    collections::BTreeMap,
    ffi::{CStr, CString, OsStr, OsString},
    os::fd::{AsRawFd, OwnedFd},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, anyhow, bail};
use async_trait::async_trait;
use derive_more::Debug;
use libseccomp::{
    ScmpAction, ScmpArgCompare, ScmpCompareOp, ScmpFilterContext, ScmpSyscall,
};
use parking_lot::Mutex;
use uuid::Uuid;

use crate::{
    ExposedPath,
    backend::{
        Backend, SandboxChild, SandboxChildOp, SandboxCommand, SandboxEnv,
        SandboxExitStatus, SandboxHandle, SandboxHandleOp,
        linux::{
            WritableMemoryFile, dbus_proxy::DbusProxy,
            file_sharing::FileSharing, find_command,
        },
        pathset::{PathSet, ReadWritePathSet},
        unix::{self, UnixChild},
    },
    path_access::{PathPermission, PathRules},
    util::SandboxArg,
};

#[derive(Debug)]
pub struct Bubblewrap;

#[async_trait]
impl Backend for Bubblewrap {
    async fn init(
        _make_helper: crate::helper::MakeHelper,
    ) -> Result<Box<dyn SandboxEnv>> {
        init().await.map(|env| Box::new(env) as Box<dyn SandboxEnv>)
    }
}

async fn init() -> Result<BubblewrapEnv> {
    let bwrap = find_command("bwrap")
        .await
        .context("searching for `bwrap` executable")?;
    let xdg_dbus_proxy = find_command("xdg-dbus-proxy")
        .await
        .context("searching for `xdg-dbus-proxy` executable")?;
    let dev_null = unix::open_dev_null().context("opening /dev/null")?;
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
    async fn spawn(
        &self,
        command: SandboxCommand,
    ) -> Result<crate::SandboxChild> {
        super::ensure_valid_command(&command).await?;
        let env = self.clone();
        crate::backend::run_on_spawn_thread(move || spawn(&env, command))
            .await?
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
    "/sys/block",
    "/sys/bus",
    "/sys/class",
    "/sys/dev",
    "/sys/devices",
    "/sys/firmware/devicetree/base/model",
    "/proc/apm",
    "/proc/acpi",
    "/etc/glvnd",
    "/etc/vulkan",
    "/etc/alsa",
    "/etc/egl",
    "/etc/static/egl",
    "/nix/store",
    // NVIDIA kernel module state (needed for driver init in user ns)
    "/sys/module/nvidia",
    "/sys/module/nvidia_drm",
    "/sys/module/nvidia_modeset",
    "/sys/module/nvidia_uvm",
];

const FAKE_FLATPAK_APP_NAME: &'static str =
    "com.modrinth.sandbox.ModrinthSandbox";

#[derive(Default)]
struct BubblewrapCommandBuilder {
    arguments: Vec<SandboxArg>,
    read: PathSet,
    write: PathSet,
    symlinks: BTreeMap<PathBuf, PathBuf>,
}

enum BindBehaviour {
    Device,
    FollowRead,
    FollowWrite,
    NoFollowRead,
    NoFollowWrite,
}

impl BubblewrapCommandBuilder {
    pub fn push(&mut self, arg: impl Into<SandboxArg>) {
        self.arguments.push(arg.into());
    }

    pub fn create_dir(
        &mut self,
        path: impl Into<SandboxArg>,
        permissions: u32,
    ) {
        self.push("--perms");
        self.push(format!("{permissions:o}"));
        self.push("--dir");
        self.push(path);
    }

    /// Bind every intermediate symlink returning the final path
    fn bind_links(&mut self, path: &Path) -> std::io::Result<PathBuf> {
        const MAX_HOPS: u8 = 40;

        fn push_components(
            path: &Path,
            components: &mut Vec<OsString>,
        ) -> std::io::Result<()> {
            for component in path.components().rev() {
                match component {
                    std::path::Component::Prefix(_) => {
                        return Err(std::io::ErrorKind::InvalidFilename.into());
                    }
                    std::path::Component::RootDir
                    | std::path::Component::CurDir => {}
                    std::path::Component::ParentDir => {
                        components.push(OsStr::new("..").to_os_string());
                    }
                    std::path::Component::Normal(os_str) => {
                        components.push(os_str.to_os_string());
                    }
                }
            }
            Ok(())
        }

        let mut components = Vec::new();
        push_components(path, &mut components)?;

        let mut resolved = PathBuf::from("/");
        let mut hops = 0;

        while let Some(component) = components.pop() {
            if component == ".." {
                resolved.pop();
                continue;
            }
            resolved.push(component);

            if let Ok(link) = std::fs::read_link(&resolved) {
                hops += 1;
                if hops > MAX_HOPS {
                    return Err(std::io::ErrorKind::TooManyLinks.into());
                }
                if !self.read.contains(resolved.clone())
                    && !self.write.contains(resolved.clone())
                {
                    self.symlinks.insert(resolved.clone(), link.clone());
                }
                if link.is_absolute() {
                    resolved = PathBuf::from("/");
                } else {
                    resolved.pop();
                }
                push_components(&link, &mut components)?;
            }
        }

        Ok(resolved)
    }

    pub fn bind_if_exists(
        &mut self,
        bind_behaviour: BindBehaviour,
        path: &Path,
    ) {
        let Ok(path) = std::path::absolute(path) else {
            return;
        };
        match bind_behaviour {
            BindBehaviour::FollowRead => {
                if let Ok(target) = self.bind_links(&path) {
                    if target.parent().is_none() {
                        return;
                    }
                    self.read.insert(target);
                }
            }
            BindBehaviour::FollowWrite => {
                if let Ok(target) = self.bind_links(&path) {
                    if target.parent().is_none() {
                        return;
                    }
                    self.write.insert(target);
                }
            }
            BindBehaviour::Device => {
                self.push("--dev-bind-try");
                self.push(path.clone());
                self.push(path);
            }
            BindBehaviour::NoFollowRead => {
                let Some(parent) = path.parent() else {
                    return;
                };
                let Some(filename) = path.file_name() else {
                    return;
                };
                let Ok(real_parent) = self.bind_links(parent) else {
                    return;
                };
                let path = real_parent.join(filename);
                if let Ok(src) = std::fs::read_link(&path) {
                    self.symlinks.insert(path, src);
                } else {
                    self.read.insert(path);
                }
            }
            BindBehaviour::NoFollowWrite => {
                let Some(parent) = path.parent() else {
                    return;
                };
                let Some(filename) = path.file_name() else {
                    return;
                };
                let Ok(real_parent) = self.bind_links(parent) else {
                    return;
                };
                let path = real_parent.join(filename);
                if let Ok(src) = std::fs::read_link(&path) {
                    self.symlinks.insert(path, src);
                } else {
                    self.write.insert(path);
                }
            }
        }
    }

    pub fn finish(mut self) -> (Vec<SandboxArg>, ReadWritePathSet) {
        let mut read_write_path_set = ReadWritePathSet::default();

        for (symlink_dest, symlink_src) in std::mem::take(&mut self.symlinks) {
            if self.read.contains(symlink_dest.clone())
                || self.write.contains(symlink_dest.clone())
            {
                continue;
            }

            self.push("--symlink");
            self.push(symlink_src);
            self.push(symlink_dest);
        }

        for read_path in std::mem::take(&mut self.read).iter() {
            read_write_path_set.insert_readable(read_path.clone());

            self.push("--ro-bind-try");
            self.push(read_path.clone());
            self.push(read_path.clone());
        }

        for write_path in std::mem::take(&mut self.write).iter() {
            read_write_path_set.insert_writeable(write_path.clone());

            self.push("--bind-try");
            self.push(write_path.clone());
            self.push(write_path.clone());
        }

        (self.arguments, read_write_path_set)
    }
}

fn spawn(
    env: &BubblewrapEnv,
    mut command: SandboxCommand,
) -> Result<crate::SandboxChild> {
    let mut builder = BubblewrapCommandBuilder::default();

    let Some(directories) = directories::BaseDirs::new() else {
        return Err(anyhow!("unable to determine base directories"));
    };

    let mut environment = command.take_environment();

    let xdg_runtime_dir = directories
        .runtime_dir()
        .context("XDG_RUNTIME_DIR is not set")?;

    let file_sharing_mountpoint = xdg_runtime_dir
        .join(format!("modrinth-file-sharing-{}", Uuid::now_v7()));
    let file_sharing = match super::file_sharing::mount(
        file_sharing_mountpoint.clone(),
    ) {
        Ok(file_sharing) => Some(file_sharing),
        Err(err) => {
            tracing::warn!(
                "Unable to start file sharing, some features may be unavailable: {}",
                err
            );
            None
        }
    };

    // Namespaces
    builder.push("--unshare-all");
    if command.allow_network {
        builder.push("--share-net");
    }

    // Detach from the controlling terminal so TIOCSTI can't inject input into it (CVE-2017-5226)
    builder.push("--new-session");

    // Special filesystems
    builder.push("--proc");
    builder.push("/proc");

    builder.push("--dev");
    builder.push("/dev");

    builder.push("--tmpfs");
    builder.push("/tmp");
    // set sticky bit on the tmp dir
    // required for some mods which do weird stuff like Waylandcraft;
    // it spawns xwayland-satellite, which spawns X11, which expects
    // the temp dir to have the sticky bit set for safety.
    builder.push("--chmod");
    builder.push("1777");
    builder.push("/tmp");

    builder.create_dir("/tmp/.X11-unix", 0o1777);

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
        builder.bind_if_exists(BindBehaviour::Device, Path::new(*dev_bind));
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
                builder.bind_if_exists(BindBehaviour::Device, &path);
            }
        }
    }

    for file_ro in SYSTEM_FILES_RO {
        builder.bind_if_exists(BindBehaviour::FollowRead, Path::new(*file_ro));
    }

    // Bind everything in $PATH
    if let Some(path) = std::env::var_os("PATH") {
        let home = directories.home_dir();
        let home = home.canonicalize().unwrap_or_else(|_| home.to_path_buf());
        for path in std::env::split_paths(&path) {
            if !path.is_absolute() {
                continue;
            }
            if path.canonicalize().is_ok_and(|path| home.starts_with(path)) {
                tracing::warn!(
                    "not exposing {path:?} from PATH, since it contains the home directory"
                );
                continue;
            }
            builder.bind_if_exists(BindBehaviour::FollowRead, &path);
        }
    }

    // Bind files in xdg runtime dir
    builder.create_dir(xdg_runtime_dir.to_path_buf(), 0o700);

    let wayland_display_path = xdg_runtime_dir.join(
        std::env::var_os("WAYLAND_DISPLAY")
            .as_deref()
            .unwrap_or(OsStr::new("wayland-0")),
    );
    builder.bind_if_exists(BindBehaviour::FollowRead, &wayland_display_path);

    let pipewire_path = xdg_runtime_dir.join("pipewire-0");
    builder.bind_if_exists(BindBehaviour::FollowRead, &pipewire_path);

    // Set up flatpak document portal
    let document_portal_path = xdg_runtime_dir.join("doc");
    let mut document_portal_path_by_app = document_portal_path.join("by-app");
    document_portal_path_by_app.push(FAKE_FLATPAK_APP_NAME);
    _ = std::fs::create_dir_all(&document_portal_path_by_app);

    builder.push("--bind");
    builder.push(document_portal_path_by_app);
    builder.push("/run/flatpak/doc");

    builder.push("--symlink");
    builder.push("/run/flatpak/doc");
    builder.push(document_portal_path);

    let pulse_path = xdg_runtime_dir.join("pulse");
    builder.create_dir(pulse_path.clone(), 0o700);

    // Bind our file-sharing
    if file_sharing.is_some() {
        // We could bind it to $XDG_RUNTIME_DIR/modrinth-file-sharing, but it's
        // simpler just to bind it to the real location on disk
        builder.bind_if_exists(
            BindBehaviour::NoFollowWrite,
            &file_sharing_mountpoint,
        );
    }

    if let Some(pulse_server) = std::env::var_os("PULSE_SERVER") {
        if let Some(pulse_server_path) =
            pulse_server.as_encoded_bytes().strip_prefix(b"unix:")
        {
            let pulse_server_path = unsafe {
                OsStr::from_encoded_bytes_unchecked(pulse_server_path)
            };
            builder.bind_if_exists(
                BindBehaviour::FollowRead,
                Path::new(pulse_server_path),
            );
        }
    } else {
        let pulse_server_path = pulse_path.join("native");
        builder.bind_if_exists(BindBehaviour::FollowRead, &pulse_server_path);

        let mut pulse_server = OsString::from("unix:");
        pulse_server.push(pulse_server_path);
        environment.insert("PULSE_SERVER".into(), pulse_server.into());
    }

    // Bind a bunch of pulse audio stuff
    let pulse_config_dir = directories.config_dir().join("pulse");
    builder.bind_if_exists(BindBehaviour::FollowRead, &pulse_config_dir);
    let pulse_home_config_dir = directories.home_dir().join(".pulse");
    builder.bind_if_exists(BindBehaviour::FollowRead, &pulse_home_config_dir);
    let asound_home_config_dir = directories.home_dir().join(".asoundrc");
    builder.bind_if_exists(BindBehaviour::FollowRead, &asound_home_config_dir);

    builder.bind_if_exists(BindBehaviour::FollowWrite, Path::new("/run/pulse"));

    if let Some(pulse_clientconfig) = std::env::var_os("PULSE_CLIENTCONFIG") {
        builder.bind_if_exists(
            BindBehaviour::FollowRead,
            Path::new(&pulse_clientconfig),
        );
    }

    // Bind X11 sockets/xauthority
    let display_index = std::env::var_os("DISPLAY")
        .and_then(parse_x11_display_index)
        .unwrap_or(0);

    builder.bind_if_exists(
        BindBehaviour::FollowRead,
        Path::new(&format!("/tmp/.X11-unix/X{display_index}")),
    );
    if let Some(xauthority) = std::env::var_os("XAUTHORITY") {
        builder
            .bind_if_exists(BindBehaviour::FollowRead, Path::new(&xauthority));
    } else {
        builder.bind_if_exists(
            BindBehaviour::FollowRead,
            &directories.home_dir().join(".Xauthority"),
        );
    }

    // Bind executable
    builder.bind_if_exists(BindBehaviour::FollowRead, &command.executable);

    for path in &command.ensure_dirs_exist {
        std::fs::create_dir_all(path)
            .with_context(|| anyhow!("creating directory {path:?}"))?;
    }

    // Bind custom paths
    // Read-only directories can contain a read-write, but not vice-versa. So we bind read then write
    let (path_set, path_rules) = command.take_paths()?;
    for path in path_set.iter_read_only() {
        builder.bind_if_exists(BindBehaviour::NoFollowRead, path);
    }
    for path in path_set.iter_read_write() {
        builder.bind_if_exists(BindBehaviour::NoFollowWrite, path);
    }

    let (mut arguments, path_set) = builder.finish();

    let flatpak_instances_dir = xdg_runtime_dir.join(".flatpak");
    let mut instance_id = rand::random::<u32>() & 0x7FFFFFFF;
    let mut instance_dir = flatpak_instances_dir.join(instance_id.to_string());
    _ = std::fs::create_dir_all(&flatpak_instances_dir);
    for attempt in 0..=100 {
        if std::fs::create_dir(&instance_dir).is_ok() {
            break;
        }

        if attempt == 100 {
            return Err(anyhow!(
                "Unable to find unique instance-id for .flatpak-info"
            ));
        } else {
            instance_id = rand::random::<u32>() & 0x7FFFFFFF;
            instance_dir = flatpak_instances_dir.join(instance_id.to_string());
        }
    }

    // Set up /.flatpak-info
    let flatpak_info = format!(
        "
[Application]
name={FAKE_FLATPAK_APP_NAME}

[Instance]
instance-id={instance_id}
\0"
    );
    let flatpak_info: CString =
        CString::from_vec_with_nul(flatpak_info.into_bytes())?;
    let flatpak_info_fd1 =
        WritableMemoryFile::open(c"modrinth-sandbox-bwrap-flatpak-info1")?;
    let flatpak_info_fd1 = flatpak_info_fd1.write(flatpak_info.as_c_str())?;
    let flatpak_info_fd2 =
        WritableMemoryFile::open(c"modrinth-sandbox-bwrap-flatpak-info2")?;
    let flatpak_info_fd2 = flatpak_info_fd2.write(flatpak_info.as_c_str())?;

    arguments.push("--file".into());
    arguments.push(format!("{}", flatpak_info_fd1.as_raw_fd()).into());
    arguments.push("/.flatpak-info".into());
    arguments.push("--ro-bind-data".into());
    arguments.push(format!("{}", flatpak_info_fd2.as_raw_fd()).into());
    arguments.push("/.flatpak-info".into());

    // Set up $XDG_RUNTIME_DIR/.flatpak/{instance_id}/bwrapinfo.json
    let bwrapinfo = instance_dir.join("bwrapinfo.json");
    let bwrapinfo_fd: OwnedFd = std::fs::File::create(bwrapinfo)?.into();
    arguments.push("--info-fd".into());
    arguments.push(format!("{}", bwrapinfo_fd.as_raw_fd()).into());

    // Set up xdg-dbus-proxy
    let dbus_proxy_result =
        start_dbus_proxy(env, xdg_runtime_dir, &flatpak_info);
    let dbus_proxy = match dbus_proxy_result {
        Ok(dbus_proxy) => {
            arguments.push("--bind".into());
            arguments.push(dbus_proxy.socket_path().to_path_buf().into());
            let mapped_bus_dir = xdg_runtime_dir.join("bus");
            arguments.push(mapped_bus_dir.clone().into());

            let mut dbus_session_bus_address = OsString::new();
            dbus_session_bus_address.push("unix:path=");
            dbus_session_bus_address.push(mapped_bus_dir);

            environment.insert(
                "DBUS_SESSION_BUS_ADDRESS".into(),
                dbus_session_bus_address.into(),
            );

            Some(dbus_proxy)
        }
        Err(err) => {
            tracing::warn!(
                "Unable to start dbus proxy, some functionality may be broken: {err}"
            );
            None
        }
    };

    // Set up seccomp filtering
    let seccomp_fd = create_seccomp_filter()?;
    arguments.push("--seccomp".into());
    arguments.push(format!("{}", seccomp_fd.as_raw_fd()).into());

    arguments.push("--".into());
    arguments.push(command.executable.into());
    for arg in command.args {
        arguments.push(arg.into());
    }

    let (pipes, child) = unix::spawn(
        env.bwrap.clone().into(),
        arguments,
        environment,
        command.stdin,
        command.stdout,
        command.stderr,
        command.working_directory,
        vec![flatpak_info_fd1, flatpak_info_fd2, bwrapinfo_fd, seccomp_fd],
        env.dev_null,
        false,
        command.die_with_parent,
    )
    .context("spawning child")?;

    let mut child = crate::SandboxChild {
        stdin: pipes.stdin,
        stdout: pipes.stdout,
        stderr: pipes.stderr,
        imp: SandboxChild::Bubblewrap(BubblewrapChild {
            child,
            instance_dir,
            _dbus_proxy: dbus_proxy,
        }),
        handle: SandboxHandle::Bubblewrap(BubblewrapSandboxHandle {
            file_sharing,
            path_set: Arc::new(path_set),
            path_rules: Arc::new(path_rules),
        }),
        on_finish: command.on_finish,
    };

    for on_start in command.on_start {
        (on_start)(&mut child);
    }

    Ok(child)
}

/// Parse `:{display}.{screen}` into the display index
fn parse_x11_display_index(display: OsString) -> Option<u32> {
    let display_bytes = display.as_encoded_bytes();
    if display_bytes.len() < 2 || display_bytes[0] != b':' {
        return None;
    }

    let mut index = 0;
    for display_byte in &display_bytes[1..] {
        if *display_byte == b'.' {
            return Some(index);
        }
        index = index.checked_mul(10)?;
        if *display_byte >= b'0' && *display_byte <= b'9' {
            index = index.checked_add((*display_byte - b'0') as u32)?;
        } else {
            return None;
        }
    }
    Some(index)
}

#[derive(Debug)]
pub(crate) struct BubblewrapChild {
    child: UnixChild,
    instance_dir: PathBuf,
    _dbus_proxy: Option<DbusProxy>,
}

impl Drop for BubblewrapChild {
    fn drop(&mut self) {
        _ = std::fs::remove_dir_all(&self.instance_dir);
    }
}

#[async_trait]
impl SandboxChildOp for BubblewrapChild {
    fn id(&self) -> Option<u32> {
        self.child.id()
    }

    fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>> {
        self.child.try_wait()
    }

    async fn wait(&mut self) -> Result<SandboxExitStatus> {
        self.child.wait().await
    }

    async fn kill(&mut self) -> Result<()> {
        self.child.kill().await
    }
}

#[derive(Clone, Debug)]
pub struct BubblewrapSandboxHandle {
    file_sharing: Option<FileSharing>,
    path_set: Arc<ReadWritePathSet>,
    path_rules: Arc<PathRules>,
}

impl SandboxHandleOp for BubblewrapSandboxHandle {
    fn expose_path(
        &self,
        path: &Path,
        permission: PathPermission,
    ) -> anyhow::Result<ExposedPath> {
        let is_accessible = |path: &Path, permission| match permission {
            PathPermission::ReadOnly => self.is_path_readable(path),
            PathPermission::ReadWrite => self.is_path_writeable(path),
        };
        if is_accessible(path, permission) {
            return Ok(ExposedPath::Direct);
        }
        let permission =
            permission.min(self.path_rules.calculate_expose_permission(path)?);
        if is_accessible(path, permission) {
            return Ok(ExposedPath::Direct);
        }
        let Some(file_sharing) = &self.file_sharing else {
            bail!(
                "file cannot be shared because file sharing filesystem failed to start"
            );
        };

        let proxied_path = file_sharing.add(path.to_path_buf(), permission)?;
        Ok(ExposedPath::Proxied(proxied_path))
    }

    fn is_path_readable(&self, path: &Path) -> bool {
        self.path_set.can_read(path.to_path_buf())
    }

    fn is_path_writeable(&self, path: &Path) -> bool {
        self.path_set.can_write(path.to_path_buf())
    }

    fn is_path_blocked(&self, path: &Path) -> bool {
        self.path_rules.is_blocked(path)
    }
}

fn start_dbus_proxy(
    env: &BubblewrapEnv,
    runtime_dir: &Path,
    flatpak_info: &CStr,
) -> Result<DbusProxy> {
    let mut builder = BubblewrapCommandBuilder::default();

    builder.push("--new-session");

    let socket_path = runtime_dir.join(format!(
        "modrinth-xdg-dbus-proxy-session-{}",
        Uuid::now_v7()
    ));

    builder.bind_if_exists(BindBehaviour::FollowRead, Path::new("/usr"));
    builder.bind_if_exists(BindBehaviour::FollowRead, Path::new("/lib64"));
    builder.bind_if_exists(BindBehaviour::FollowRead, Path::new("/nix/store"));
    builder.bind_if_exists(BindBehaviour::FollowWrite, &runtime_dir);

    let (mut arguments, _) = builder.finish();

    let flatpak_info_fd1 =
        WritableMemoryFile::open(c"modrinth-sandbox-proxy-flatpak-info1")
            .context("creating flatpak-info memory file 1")?;
    let flatpak_info_fd1 = flatpak_info_fd1
        .write(flatpak_info)
        .context("writing to flatpak-info memory file 1")?;
    let flatpak_info_fd2 =
        WritableMemoryFile::open(c"modrinth-sandbox-proxy-flatpak-info2")
            .context("creating flatpak-info memory file 2")?;
    let flatpak_info_fd2 = flatpak_info_fd2
        .write(flatpak_info)
        .context("writing to flatpak-info memory file 2")?;

    arguments.push("--file".into());
    arguments.push(format!("{}", flatpak_info_fd1.as_raw_fd()).into());
    arguments.push("/.flatpak-info".into());
    arguments.push("--ro-bind-data".into());
    arguments.push(format!("{}", flatpak_info_fd2.as_raw_fd()).into());
    arguments.push("/.flatpak-info".into());

    arguments.push("--".into());
    arguments.push(env.xdg_dbus_proxy.clone().into());

    DbusProxy::start(
        env.bwrap.clone(),
        arguments,
        socket_path,
        runtime_dir.to_path_buf(),
        vec![flatpak_info_fd1, flatpak_info_fd2],
        env.dev_null,
    )
}

// Syscall allowlist copied from https://github.com/moby/profiles/blob/fa50b7287199d1c781284d1a34d1395a62e57f1e/seccomp/default.json
// Licensed as Apache 2.0 (https://github.com/moby/profiles/blob/fa50b7287199d1c781284d1a34d1395a62e57f1e/LICENSE)
const ALLOWED_SYSCALLS: &[&str] = &[
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

    let Ok(mut filter) = ScmpFilterContext::new(ScmpAction::Errno(libc::EPERM))
    else {
        return Err(anyhow!("unable to init seccomp filter context"));
    };

    for syscall in ALLOWED_SYSCALLS {
        let Ok(syscall) = ScmpSyscall::from_name(syscall) else {
            continue;
        };
        _ = filter.add_rule(ScmpAction::Allow, syscall);
    }
    if let Ok(syscall) = ScmpSyscall::from_name("clone") {
        let disallowed_clones = libc::CLONE_NEWNS
            | libc::CLONE_NEWUTS
            | libc::CLONE_NEWIPC
            | libc::CLONE_NEWUSER
            | libc::CLONE_NEWPID
            | libc::CLONE_NEWNET
            | libc::CLONE_NEWCGROUP;
        _ = filter.add_rule_conditional(
            ScmpAction::Allow,
            syscall,
            &[ScmpArgCompare::new(
                0,
                ScmpCompareOp::MaskedEqual(disallowed_clones as u64),
                0,
            )],
        );
    }
    if let Ok(syscall) = ScmpSyscall::from_name("clone3") {
        _ = filter.add_rule(ScmpAction::Errno(libc::ENOSYS), syscall);
    }
    if let Ok(syscall) = ScmpSyscall::from_name("socket") {
        _ = filter.add_rule_conditional(
            ScmpAction::Allow,
            syscall,
            &[ScmpArgCompare::new(
                0,
                ScmpCompareOp::NotEqual,
                libc::AF_VSOCK as u64,
            )],
        );
    }
    if let Ok(syscall) = ScmpSyscall::from_name("personality") {
        _ = filter.add_rule_conditional(
            ScmpAction::Allow,
            syscall,
            &[ScmpArgCompare::new(0, ScmpCompareOp::Equal, 0x0)],
        );
        _ = filter.add_rule_conditional(
            ScmpAction::Allow,
            syscall,
            &[ScmpArgCompare::new(0, ScmpCompareOp::Equal, 0x8)],
        );
        _ = filter.add_rule_conditional(
            ScmpAction::Allow,
            syscall,
            &[ScmpArgCompare::new(0, ScmpCompareOp::Equal, 0x20000)],
        );
        _ = filter.add_rule_conditional(
            ScmpAction::Allow,
            syscall,
            &[ScmpArgCompare::new(0, ScmpCompareOp::Equal, 0x20008)],
        );
        _ = filter.add_rule_conditional(
            ScmpAction::Allow,
            syscall,
            &[ScmpArgCompare::new(0, ScmpCompareOp::Equal, 0xffffffff)],
        );
    }
    let fd = WritableMemoryFile::open(c"modrinth-sandbox-seccomp-bpf")?;
    let fd = fd.write_filter(filter)?;
    Ok(fd)
}
