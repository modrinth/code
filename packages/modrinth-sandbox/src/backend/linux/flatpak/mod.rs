use std::{
    collections::BTreeMap,
    io::ErrorKind,
    os::{
        fd::{AsFd, OwnedFd},
        unix::{ffi::OsStringExt, fs::OpenOptionsExt},
    },
    sync::Arc,
};

use async_trait::async_trait;
use derive_more::Debug;
use anyhow::{Context, Result, bail, ensure, anyhow};
use futures::{StreamExt, TryStreamExt, stream::FuturesUnordered};
use libc::SIGKILL;
use tokio::{fs, sync::oneshot};
use tracing::{error, warn};
use zbus::Fd;

use crate::{
    SandboxCommand, SandboxExitStatus, SandboxStdio,
    backend::{
        Backend, SandboxChild, SandboxChildOp, SandboxEnv, SandboxHandle,
        SandboxHandleOp,
        linux::{
            dbus_proxy::DbusProxy,
            find_command,
            flatpak::dbus::{SandboxFlags, SpawnFlags, SpawnOptions},
        },
        pathset::ReadWritePathSet,
        unix,
    },
    path_access::{PathPermission, PathRules},
};

mod dbus;

#[derive(Debug)]
pub struct Flatpak;

#[async_trait]
impl Backend for Flatpak {
    async fn init(
        _make_helper: crate::helper::MakeHelper,
    ) -> Result<Box<dyn SandboxEnv>> {
        init().await.map(|env| Box::new(env) as Box<dyn SandboxEnv>)
    }
}

async fn init() -> Result<FlatpakEnv> {
    let session_bus = zbus::Connection::session()
        .await
        .context("connecting to session bus")?;
    let flatpak_portal = dbus::FlatpakPortalProxy::new(&session_bus)
        .await
        .context("connecting to Flatpak portal")?;
    let dev_null = unix::open_dev_null().context("opening /dev/null")?;
    Ok(FlatpakEnv {
        flatpak_portal,
        dev_null,
    })
}

#[derive(Debug)]
#[debug("FlatpakEnv")]
pub struct FlatpakEnv {
    flatpak_portal: dbus::FlatpakPortalProxy<'static>,
    dev_null: libc::c_int,
}

#[async_trait]
impl SandboxEnv for FlatpakEnv {
    async fn spawn(
        &self,
        command: SandboxCommand,
    ) -> Result<crate::SandboxChild> {
        super::ensure_valid_command(&command).await?;
        spawn(self, command).await
    }
}

async fn spawn(
    env: &FlatpakEnv,
    mut command: SandboxCommand,
) -> Result<crate::SandboxChild> {
    let mut envs = command
        .take_environment()
        .into_iter()
        .map(|(key, val)| {
            let key = key.into_os_string().into_string().map_err(|key| {
                anyhow!("environment variable {key:?} has non-UTF-8 key")
            })?;
            let val = val.into_os_string().into_string().map_err(|_| {
                anyhow!("environment variable value for key {key:?} has non-UTF-8 value")
            })?;
            anyhow::Ok((key, val))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;

    let (path_set, path_rules) = command.take_paths()?;

    let cwd_path = match command.working_directory {
        Some(path) => nul_terminate(
            path.into_os_string().into_vec(),
            "working directory",
        )?,
        None => vec![0],
    };

    let mut argv = Vec::with_capacity(command.args.len() + 1);
    argv.push(nul_terminate(
        command.executable.into_os_string().into_vec(),
        "executable path",
    )?);
    for arg in command.args {
        argv.push(nul_terminate(
            arg.into_os_string().into_vec(),
            "process argument",
        )?);
    }

    let mut fds = BTreeMap::new();

    let inherited = std::io::stdin();
    let stdin = {
        let (fd, write) = match command.stdin {
            SandboxStdio::Null => open_dev_null()
                .map(OwnedFd::from)
                .map(Fd::Owned)
                .map(|fd| (fd, None))
                .context("creating /dev/null fd")?,
            SandboxStdio::Inherit => {
                let fd = Fd::Borrowed(inherited.as_fd());
                (fd, None)
            }
            SandboxStdio::Pipe => {
                let (read, write) =
                    std::io::pipe().context("creating child stdin pipe")?;
                (OwnedFd::from(read).into(), Some(write))
            }
        };

        fds.insert(libc::STDIN_FILENO as u32, fd);
        write
    };

    let inherited = std::io::stdout();
    let stdout = {
        let (fd, read) = match command.stdout {
            SandboxStdio::Null => open_dev_null()
                .map(OwnedFd::from)
                .map(Fd::Owned)
                .map(|fd| (fd, None))
                .context("creating /dev/null fd")?,
            SandboxStdio::Inherit => {
                let fd = Fd::Borrowed(inherited.as_fd());
                (fd, None)
            }
            SandboxStdio::Pipe => {
                let (read, write) =
                    std::io::pipe().context("creating child stdout pipe")?;
                (OwnedFd::from(write).into(), Some(read))
            }
        };

        fds.insert(libc::STDOUT_FILENO as u32, fd);
        read
    };

    let inherited = std::io::stderr();
    let stderr = {
        let (fd, read) = match command.stderr {
            SandboxStdio::Null => open_dev_null()
                .map(OwnedFd::from)
                .map(Fd::Owned)
                .map(|fd| (fd, None))
                .context("creating /dev/null fd")?,
            SandboxStdio::Inherit => {
                let fd = Fd::Borrowed(inherited.as_fd());
                (fd, None)
            }
            SandboxStdio::Pipe => {
                let (read, write) =
                    std::io::pipe().context("creating child stderr pipe")?;
                (OwnedFd::from(write).into(), Some(read))
            }
        };

        fds.insert(libc::STDERR_FILENO as u32, fd);
        read
    };

    let mut flags = SpawnFlags::CLEAR_ENV
        | SpawnFlags::SANDBOX
        | SpawnFlags::WATCH_BUS
        | SpawnFlags::EMPTY_APP;
    if !command.allow_network {
        flags |= SpawnFlags::NO_NETWORK;
    }

    for path in &command.ensure_dirs_exist {
        fs::create_dir_all(path)
            .await
            .with_context(|| anyhow!("creating directory {path:?}"))?;
    }

    let sandbox_expose_fd = path_set
        .iter_read_write()
        .map(|path| async move {
            let file_result = fs::OpenOptions::new()
                .write(true)
                .custom_flags(libc::O_PATH | libc::O_NOFOLLOW)
                .open(&path)
                .await;
            if let Err(err) = &file_result
                && err.kind() == ErrorKind::NotFound
            {
                return anyhow::Ok(None);
            }
            let file = file_result
                .with_context(|| anyhow!("opening file {path:?} for writing"))?;
            let file = file.into_std().await;
            let fd = OwnedFd::from(file);
            let fd = zbus::OwnedFd::from(fd);
            anyhow::Ok(Some(fd))
        })
        .collect::<FuturesUnordered<_>>()
        .filter_map(|i| async move { i.transpose() })
        .try_collect::<Vec<_>>()
        .await?;

    let mut sandbox_expose_fd_ro = path_set
        .iter_read_only()
        .map(|path| async move {
            let file_result = fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_PATH | libc::O_NOFOLLOW)
                .open(&path)
                .await;
            if let Err(err) = &file_result
                && err.kind() == ErrorKind::NotFound
            {
                return anyhow::Ok(None);
            }
            let file = file_result
                .with_context(|| anyhow!("opening file {path:?} for reading"))?;
            let file = file.into_std().await;
            let fd = OwnedFd::from(file);
            let fd = zbus::OwnedFd::from(fd);
            anyhow::Ok(Some(fd))
        })
        .collect::<FuturesUnordered<_>>()
        .filter_map(|i| async move { i.transpose() })
        .try_collect::<Vec<_>>()
        .await?;

    let dbus_proxy = match start_dbus_proxy(env).await {
        Ok((dbus_proxy, socket_fd, address)) => {
            sandbox_expose_fd_ro.push(socket_fd);
            envs.insert("DBUS_SESSION_BUS_ADDRESS".into(), address);
            Some(dbus_proxy)
        }
        Err(err) => {
            warn!(
                "Unable to start dbus proxy, some functionality may be broken: {err:#}"
            );
            None
        }
    };

    let options = SpawnOptions {
        // SESSION_BUS must not be used, since it allows using the Flatpak portal to
        // spawn processes with all of the launcher's permissions
        sandbox_flags: Some(
            SandboxFlags::SHARE_DISPLAY
                | SandboxFlags::SHARE_SOUND
                | SandboxFlags::SHARE_GPU
                | SandboxFlags::INPUT_DEVICES,
        ),
        sandbox_expose_fd: Some(sandbox_expose_fd),
        sandbox_expose_fd_ro: Some(sandbox_expose_fd_ro),
        ..Default::default()
    };

    // make the spawn_exited stream first...
    let mut spawn_exited = env
        .flatpak_portal
        .receive_spawn_exited()
        .await
        .context("creating SpawnExited stream")?;

    let flatpak_pid = env
        .flatpak_portal
        .spawn(&cwd_path, &argv, &fds, &envs, flags, &options)
        .await
        .context("spawning child process through Flatpak portal")?;

    // ...because otherwise we can have a race condition here.
    // if we spawn the process, and it exits very quickly, then flatpak
    // sends a SpawnExited signal *now*, before we've started the task which
    // forwards exits to `tx_exited`.
    // to counteract this, we make the stream *first*, then we can receive
    // messages which were made after we spawned but before the forwarder task
    // first ran.

    let (tx_exited, rx_exited) = oneshot::channel();
    // this is the forwarder task.
    //
    // why do we make a new task on every spawn(), instead of having a single
    // long-living task in the `FlatpakEnv` which sends on a channel?
    //
    // using a broadcast channel, we are forced to have a bounded capacity, and
    // there's always a risk of a receiver channel lagging and we skip an exited
    // signal. this is bad, because we must be able to *reliably* receive
    // SpawnExited signals, and never drop any.
    // and we can't use an mpsc here, because we don't have a single consumer,
    // we have many (breaking the "sc" part of "mpsc").
    tokio::spawn(async move {
        while let Some(exited) = spawn_exited.next().await {
            let exited = match exited.args() {
                Ok(args) => args,
                Err(err) => {
                    // this is very much less than ideal since we're potentially dropping
                    // the exit signal for our process, but I'm not sure what to do here,
                    // short of panicking the whole app.
                    error!(
                        "failed to receive SpawnExited signal, skipping: {err:?}"
                    );
                    return;
                }
            };

            if exited.pid == flatpak_pid {
                let exit_status = exited.exit_status as libc::c_int;
                tx_exited
                    .send(SandboxExitStatus {
                        imp: unix::UnixSandboxExitStatus(exit_status),
                    })
                    .ok();
                return;
            }
        }
    });

    let mut child = crate::SandboxChild {
        stdin,
        stdout,
        stderr,
        on_finish: command.on_finish,
        handle: SandboxHandle::Flatpak(FlatpakSandboxHandle {
            path_set: Arc::new(path_set),
            path_rules: Arc::new(path_rules),
        }),
        imp: SandboxChild::Flatpak(FlatpakChild {
            flatpak_pid,
            flatpak_portal: env.flatpak_portal.clone(),
            rx_exited: Some(rx_exited),
            exit_status: None,
            _dbus_proxy: dbus_proxy,
        }),
    };

    for on_start in command.on_start {
        (on_start)(&mut child);
    }

    Ok(child)
}

/// Returns the proxy, a file descriptor for exposing its socket, and its address.
async fn start_dbus_proxy(
    env: &FlatpakEnv,
) -> Result<(DbusProxy, zbus::OwnedFd, String)> {
    let xdg_dbus_proxy = find_command("xdg-dbus-proxy")
        .await
        .context("searching for `xdg-dbus-proxy` executable")?;
    let dev_null = env.dev_null;

    crate::backend::run_on_spawn_thread(move || {
        let directories = directories::BaseDirs::new()
            .context("unable to determine base directories")?;

        // Flatpak can only expose the socket at the same path as on the host, so it
        // can't be in $XDG_RUNTIME_DIR or /tmp, since the launcher has its own versions
        // of those
        let cache_dir = directories.cache_dir();
        std::fs::create_dir_all(cache_dir)
            .context("creating cache directory")?;
        let cache_dir = cache_dir
            .canonicalize()
            .context("resolving cache directory")?;

        let socket_path =
            cache_dir.join(format!("bus-{:016x}", rand::random::<u64>()));
        let address = socket_path
            .to_str()
            .map(|path| format!("unix:path={path}"))
            .context("socket path isn't valid UTF-8")?;

        let dbus_proxy = DbusProxy::start(
            xdg_dbus_proxy,
            Vec::new(),
            socket_path,
            cache_dir,
            Vec::new(),
            dev_null,
        )?;

        let socket = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_PATH | libc::O_NOFOLLOW)
            .open(dbus_proxy.socket_path())
            .context("opening dbus proxy socket")?;
        let socket_fd = zbus::OwnedFd::from(OwnedFd::from(socket));

        anyhow::Ok((dbus_proxy, socket_fd, address))
    })
    .await?
}

fn nul_terminate(mut value: Vec<u8>, description: &str) -> Result<Vec<u8>> {
    ensure!(
        !value.contains(&0),
        "{description} contains an embedded NUL byte"
    );
    value.push(0);
    Ok(value)
}

fn open_dev_null() -> Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/null")
        .context("opening /dev/null")
}

/// Flatpak does not support granting additional filesystem access after spawn.
#[derive(Clone, Debug)]
pub struct FlatpakSandboxHandle {
    path_set: Arc<ReadWritePathSet>,
    path_rules: Arc<PathRules>,
}

impl SandboxHandleOp for FlatpakSandboxHandle {
    fn expose_path(
        &self,
        _path: &std::path::Path,
        _permission: PathPermission,
    ) -> Result<crate::ExposedPath> {
        bail!(
            "exposing additional paths to a running Flatpak sandbox is unsupported"
        )
    }

    fn is_path_readable(&self, path: &std::path::Path) -> bool {
        self.path_set.can_read(path.to_path_buf())
    }

    fn is_path_writeable(&self, path: &std::path::Path) -> bool {
        self.path_set.can_write(path.to_path_buf())
    }

    fn is_path_blocked(&self, path: &std::path::Path) -> bool {
        self.path_rules.is_blocked(path)
    }
}

#[derive(Debug)]
pub struct FlatpakChild {
    /// Flatpak's opaque process ID for this child - *not* a Unix process ID.
    flatpak_pid: u32,
    flatpak_portal: dbus::FlatpakPortalProxy<'static>,
    rx_exited: Option<oneshot::Receiver<SandboxExitStatus>>,
    exit_status: Option<SandboxExitStatus>,
    _dbus_proxy: Option<DbusProxy>,
}

#[async_trait]
impl SandboxChildOp for FlatpakChild {
    fn id(&self) -> Option<u32> {
        // TODO: can we get some kind of persistent id here? not sure
        None
    }

    fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>> {
        if let Some(exit_status) = self.exit_status {
            return Ok(Some(exit_status));
        }

        let rx_exited = self
            .rx_exited
            .as_mut()
            .context("missing exit status receiver")?;
        match rx_exited.try_recv() {
            Ok(exit_status) => {
                self.rx_exited = None;
                self.exit_status = Some(exit_status);
                Ok(Some(exit_status))
            }
            Err(oneshot::error::TryRecvError::Empty) => Ok(None),
            Err(oneshot::error::TryRecvError::Closed) => {
                bail!("channel closed")
            }
        }
    }

    async fn wait(&mut self) -> Result<SandboxExitStatus> {
        if let Some(exit_status) = self.exit_status {
            return Ok(exit_status);
        }

        let exit_status = self
            .rx_exited
            .as_mut()
            .context("missing exit status receiver")?
            .await
            .context("channel closed")?;
        self.rx_exited = None;
        self.exit_status = Some(exit_status);
        Ok(exit_status)
    }

    async fn kill(&mut self) -> Result<()> {
        self.flatpak_portal
            .spawn_signal(self.flatpak_pid, SIGKILL as u32, true)
            .await
            .context("sending SIGKILL to child process")
    }
}
