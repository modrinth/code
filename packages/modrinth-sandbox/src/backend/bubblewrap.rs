use std::{ffi::OsStr, path::PathBuf};

use async_trait::async_trait;
use derive_more::Debug;
use eyre::{Context, Result};
use tokio::process::Command;

use crate::{
    SandboxCommand,
    backend::{Backend, SandboxChild, SandboxEnv},
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
    async fn spawn(
        &self,
        command: SandboxCommand,
    ) -> Result<Box<dyn SandboxChild>> {
        spawn(self, command)
            .await
            .map(|b| Box::new(b) as Box<dyn SandboxChild>)
    }
}

struct BubblewrapChild {
    child: tokio::process::Child,
}

#[async_trait]
impl SandboxChild for BubblewrapChild {
    async fn wait(&mut self) -> Result<std::process::ExitStatus> {
        self.child.wait().await.wrap_err("waiting for child")
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
];

async fn spawn(
    env: &BubblewrapEnv,
    command: SandboxCommand,
) -> Result<BubblewrapChild> {
    let mut os_command = Command::new(&env.bwrap);
    os_command.arg("--unshare-all");
    os_command.arg("--die-with-parent");

    // create fresh `/proc`
    os_command.args(["--proc", "/proc"]);

    // create fresh `/dev`
    os_command.args(["--dev", "/dev"]);
    for bind in DEV_BINDS {
        os_command.args(["--symlink", bind, bind]);
    }

    // this instance owns its own tmpfs at `/tmp`
    os_command.args(["--tmpfs", "/tmp"]);

    if command.allow_network {
        os_command.arg("--share-net");
    }
    for path in command.read_only_paths {
        os_command.arg("--ro-bind");
        os_command.arg(&path);
        os_command.arg(path);
    }
    for path in command.writable_paths {
        os_command.arg("--bind");
        os_command.arg(&path);
        os_command.arg(path);
    }
    if let Some(path) = command.working_directory {
        os_command.arg("--chdir");
        os_command.arg(path);
    }

    os_command.arg("--");
    os_command.arg(command.executable);
    os_command.args(command.args);

    let child = os_command.spawn().wrap_err("spawning child")?;
    Ok(BubblewrapChild { child })
}
