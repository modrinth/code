use std::{
    collections::BTreeMap,
    os::{
        fd::{AsFd, OwnedFd},
        unix::ffi::OsStringExt,
    },
};

use async_trait::async_trait;
use derive_more::Debug;
use eyre::{Context, OptionExt, Result, bail, ensure, eyre};
use futures::{StreamExt, TryStreamExt, stream::FuturesUnordered};
use libc::SIGKILL;
use tokio::{fs, sync::oneshot};
use tracing::error;
use zbus::zvariant::Fd;

use crate::{
    SandboxCommand, SandboxExitStatus, SandboxStdio,
    backend::{
        Backend, SandboxChild, SandboxChildOp, SandboxEnv,
        flatpak::dbus::{SandboxFlags, SpawnFlags, SpawnOptions},
        unix,
    },
};

mod dbus;

#[derive(Debug)]
pub struct Flatpak;

#[async_trait]
impl Backend for Flatpak {
    async fn init() -> Result<Box<dyn SandboxEnv>> {
        init().await.map(|env| Box::new(env) as Box<dyn SandboxEnv>)
    }
}

async fn init() -> Result<FlatpakEnv> {
    let session_bus = zbus::Connection::session()
        .await
        .wrap_err("connecting to session bus")?;
    let flatpak_portal = dbus::FlatpakPortalProxy::new(&session_bus)
        .await
        .context("connecting to Flatpak portal")?;
    Ok(FlatpakEnv { flatpak_portal })
}

#[derive(Debug)]
#[debug("FlatpakEnv")]
pub struct FlatpakEnv {
    flatpak_portal: dbus::FlatpakPortalProxy<'static>,
}

#[async_trait]
impl SandboxEnv for FlatpakEnv {
    async fn spawn(
        &self,
        command: SandboxCommand,
    ) -> Result<crate::SandboxChild> {
        spawn(self, command).await
    }
}

async fn spawn(
    env: &FlatpakEnv,
    mut command: SandboxCommand,
) -> Result<crate::SandboxChild> {
    let envs = command
        .take_environment()
        .into_iter()
        .map(|(key, val)| {
            let key = key.into_os_string().into_string().map_err(|key| {
                eyre!("environment variable {key:?} has non-UTF-8 key")
            })?;
            let val = val.into_os_string().into_string().map_err(|_| {
                eyre!("environment variable value for key {key:?} has non-UTF-8 value")
            })?;
            eyre::Ok((key, val))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;

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
                .wrap_err("creating /dev/null fd")?,
            SandboxStdio::Inherit => {
                let fd = Fd::Borrowed(inherited.as_fd());
                (fd, None)
            }
            SandboxStdio::Pipe => {
                let (read, write) =
                    std::io::pipe().wrap_err("creating child stdin pipe")?;
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
                .wrap_err("creating /dev/null fd")?,
            SandboxStdio::Inherit => {
                let fd = Fd::Borrowed(inherited.as_fd());
                (fd, None)
            }
            SandboxStdio::Pipe => {
                let (read, write) =
                    std::io::pipe().wrap_err("creating child stdout pipe")?;
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
                .wrap_err("creating /dev/null fd")?,
            SandboxStdio::Inherit => {
                let fd = Fd::Borrowed(inherited.as_fd());
                (fd, None)
            }
            SandboxStdio::Pipe => {
                let (read, write) =
                    std::io::pipe().wrap_err("creating child stderr pipe")?;
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
    if !command.network {
        flags |= SpawnFlags::NO_NETWORK;
    }

    let sandbox_expose_fd = command
        .read_write_paths
        .into_iter()
        .map(|path| async move {
            let file = fs::OpenOptions::new()
                .write(true)
                .custom_flags(libc::O_PATH | libc::O_NOFOLLOW)
                .open(&path)
                .await
                .wrap_err_with(|| eyre!("opening file {path:?} for writing"))?;
            let file = file.into_std().await;
            let fd = OwnedFd::from(file);
            let fd = zbus::zvariant::OwnedFd::from(fd);
            eyre::Ok(fd)
        })
        .collect::<FuturesUnordered<_>>()
        .try_collect::<Vec<_>>()
        .await?;

    let sandbox_expose_fd_ro = command
        .read_only_paths
        .into_iter()
        .map(|path| async move {
            let file = fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_PATH | libc::O_NOFOLLOW)
                .open(&path)
                .await
                .wrap_err_with(|| eyre!("opening file {path:?} for reading"))?;
            let file = file.into_std().await;
            let fd = OwnedFd::from(file);
            let fd = zbus::zvariant::OwnedFd::from(fd);
            eyre::Ok(fd)
        })
        .collect::<FuturesUnordered<_>>()
        .try_collect::<Vec<_>>()
        .await?;

    let options = SpawnOptions {
        sandbox_flags: Some(
            SandboxFlags::SHARE_DISPLAY
                | SandboxFlags::SHARE_SOUND
                | SandboxFlags::SHARE_GPU
                | SandboxFlags::SESSION_BUS
                | SandboxFlags::INPUT_DEVICES,
        ),
        sandbox_expose_fd: Some(sandbox_expose_fd),
        sandbox_expose_fd_ro: Some(sandbox_expose_fd_ro),
        ..Default::default()
    };

    // Make sure any required dirs exist, create them if not
    // Do this right before we do the actual spawn, so that we don't create these
    // if we have an error earlier on
    for path in &command.ensure_dirs_exist {
        fs::create_dir_all(path)
            .await
            .wrap_err_with(|| eyre!("creating directory {path:?}"))?;
    }

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
        .wrap_err("spawning child process through Flatpak portal")?;

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

    Ok(crate::SandboxChild {
        stdin,
        stdout,
        stderr,
        imp: SandboxChild::Flatpak(FlatpakChild {
            flatpak_pid,
            flatpak_portal: env.flatpak_portal.clone(),
            rx_exited: Some(rx_exited),
            exit_status: None,
        }),
    })
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
        .wrap_err("opening /dev/null")
}

#[derive(Debug)]
pub struct FlatpakChild {
    /// Flatpak's opaque process ID for this child - *not* a Unix process ID.
    flatpak_pid: u32,
    flatpak_portal: dbus::FlatpakPortalProxy<'static>,
    rx_exited: Option<oneshot::Receiver<SandboxExitStatus>>,
    exit_status: Option<SandboxExitStatus>,
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
            .ok_or_eyre("missing exit status receiver")?;
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
            .ok_or_eyre("missing exit status receiver")?
            .await
            .wrap_err("channel closed")?;
        self.rx_exited = None;
        self.exit_status = Some(exit_status);
        Ok(exit_status)
    }

    async fn kill(&mut self) -> Result<()> {
        self.flatpak_portal
            .spawn_signal(self.flatpak_pid, SIGKILL as u32, true)
            .await
            .wrap_err("sending SIGKILL to child process")
    }
}
