use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStringExt,
    path::Path,
};

use async_trait::async_trait;
use derive_more::Debug;
use eyre::{Context, Result, eyre};
use foldhash::{HashMap, HashMapExt};

use crate::{
    SandboxArg, SandboxChild, SandboxCommand,
    backend::{
        Backend, SandboxEnv,
        flatpak::dbus::{SandboxFlags, SpawnFlags, SpawnOptions},
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
    let system_bus = zbus::Connection::system()
        .await
        .wrap_err("connecting to system bus")?;
    let flatpak_portal = dbus::FlatpakPortalProxy::new(&system_bus)
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
        spawn(self, command).await
    }
}

async fn spawn(
    env: &FlatpakEnv,
    mut command: SandboxCommand,
) -> Result<SandboxChild> {
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
        std::fs::create_dir_all(path)
            .wrap_err_with(|| eyre!("creating directory {path:?}"))?;
    }

    env.flatpak_portal
        .spawn(&cwd_path, &argv, &fds, &envs, flags, &options)
        .await
        .wrap_err("spawning child process through Flatpak portal")?;

    Ok(SandboxChild { imp: todo!() })
}

fn option_with_value(option: &str, value: &Path) -> SandboxArg {
    let mut argument = OsString::from(option);
    argument.push(value);
    argument.into()
}

fn environment_argument(name: &SandboxArg, value: &SandboxArg) -> SandboxArg {
    let mut argument = OsString::from("--env=");
    argument.push(name.as_os_str());
    argument.push(OsStr::new("="));
    argument.push(value.as_os_str());
    argument.into()
}
