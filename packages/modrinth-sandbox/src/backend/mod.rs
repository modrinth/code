use std::{fmt::Debug, process::ExitStatus};

use async_trait::async_trait;
use eyre::Result;

use crate::SandboxCommand;

mod bubblewrap;

#[async_trait]
pub trait Backend {
    async fn init() -> Result<Box<dyn SandboxEnv>>;
}

#[async_trait]
pub trait SandboxEnv: Debug + Send + Sync {
    async fn spawn(
        &self,
        command: SandboxCommand,
    ) -> Result<Box<dyn SandboxChild>>;
}

#[async_trait]
pub trait SandboxChild: Send {
    async fn wait(&mut self) -> Result<ExitStatus>;
}

pub async fn init_env() -> Result<Box<dyn SandboxEnv>> {
    bubblewrap::Bubblewrap::init().await
}
