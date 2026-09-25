use std::{
    io::{PipeReader, PipeWriter},
    os::unix::ffi::OsStringExt,
};

use async_trait::async_trait;
use derive_more::Debug;
use eyre::{Context, OptionExt, Result, bail, eyre};
use foldhash::{HashMap, HashMapExt};
use futures::StreamExt;
use tokio::{fs, sync::oneshot};
use tracing::error;

use crate::{
    SandboxCommand, SandboxExitStatus,
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
    async fn spawn(&self, command: SandboxCommand) -> Result<SandboxChild> {
        spawn(self, command).await.map(SandboxChild::Flatpak)
    }
}

async fn spawn(
    env: &FlatpakEnv,
    mut command: SandboxCommand,
) -> Result<FlatpakChild> {
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
        .collect::<Result<HashMap<_, _>>>()?;

    let cwd_path = command
        .working_directory
        .map(|buf| buf.into_os_string().into_vec())
        .unwrap_or_default();

    let mut argv = vec![command.executable.into_os_string().into_vec()];
    for arg in command.args {
        argv.push(arg.into_os_string().into_vec());
    }

    let fds = HashMap::new();

    let flags = SpawnFlags::CLEAR_ENV
        | SpawnFlags::SANDBOX
        | SpawnFlags::WATCH_BUS
        | SpawnFlags::EMPTY_APP;

    let options = SpawnOptions {
        sandbox_flags: Some(
            SandboxFlags::SHARE_DISPLAY
                | SandboxFlags::SHARE_SOUND
                | SandboxFlags::SHARE_GPU
                | SandboxFlags::SESSION_BUS
                | SandboxFlags::INPUT_DEVICES,
        ),
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

    let pid = env
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

            if exited.pid == pid {
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

    Ok(FlatpakChild {
        pid,
        rx_exited: Some(rx_exited),
    })
}

#[derive(Debug)]
pub struct FlatpakChild {
    pid: u32,
    rx_exited: Option<oneshot::Receiver<SandboxExitStatus>>,
}

#[async_trait]
impl SandboxChildOp for FlatpakChild {
    fn id(&self) -> Option<u32> {
        todo!();
    }

    fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>> {
        let rx_exited = self
            .rx_exited
            .as_mut()
            .ok_or_eyre("already received exit status")?;
        match rx_exited.try_recv() {
            Ok(t) => Ok(Some(t)),
            Err(oneshot::error::TryRecvError::Empty) => Ok(None),
            Err(oneshot::error::TryRecvError::Closed) => {
                bail!("channel closed")
            }
        }
    }

    async fn wait(&mut self) -> Result<SandboxExitStatus> {
        self.rx_exited
            .take()
            .ok_or_eyre("already received exit status")?
            .await
            .wrap_err("channel closed")
    }

    async fn kill(&mut self) -> Result<()> {
        todo!();
    }
    fn take_stdin(&mut self) -> Option<PipeWriter> {
        todo!();
    }
    fn take_stdout(&mut self) -> Option<PipeReader> {
        todo!();
    }
    fn take_stderr(&mut self) -> Option<PipeReader> {
        todo!();
    }
}
