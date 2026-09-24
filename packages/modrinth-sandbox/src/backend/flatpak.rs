use std::path::PathBuf;

use async_trait::async_trait;
use derive_more::Debug;
use eyre::{Context, Result};

use crate::{
    SandboxChild, SandboxCommand,
    backend::{Backend, SandboxEnv},
    util::path::find_command,
};

#[derive(Debug)]
pub struct Flatpak {}

#[async_trait]
impl Backend for Flatpak {
    async fn init() -> Result<Box<dyn SandboxEnv>> {
        init().await.map(|env| Box::new(env) as Box<dyn SandboxEnv>)
    }
}

async fn init() -> Result<FlatpakEnv> {
    let flatpak_spawn = find_command("flatpak-spawn")
        .await
        .wrap_err("searching for `flatpak-spawn` executable")?;
    Ok(FlatpakEnv { flatpak_spawn })
}

#[derive(Debug)]
#[debug("FlatpakEnv")]
pub struct FlatpakEnv {
    flatpak_spawn: PathBuf,
}

impl SandboxEnv for FlatpakEnv {
    fn spawn(&self, command: SandboxCommand) -> Result<SandboxChild> {
        todo!();
    }
}
