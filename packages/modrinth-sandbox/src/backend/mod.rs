//! Backend sandbox implementations, using OS-specific primitives.

use std::{collections::BTreeMap, fmt::Debug};

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use eyre::{Context, Result, eyre};
use tokio::fs;

use crate::{SandboxCommand, util::argument::SandboxArg};

// TODO cfgs
mod bubblewrap;
mod flatpak;
mod unix;

/// See [`crate::create_env`].
#[async_trait]
pub trait Backend {
    /// See [`crate::create_env`].
    async fn init() -> Result<Box<dyn SandboxEnv>>;
}

/// See [`crate::SandboxEnv`].
#[async_trait]
pub trait SandboxEnv: Debug + Send + Sync {
    /// See [`crate::SandboxEnv::spawn`].
    async fn spawn(
        &self,
        command: SandboxCommand,
    ) -> Result<crate::SandboxChild>;
}

/// See [`crate::create_env`].
pub async fn create_env() -> Result<Box<dyn SandboxEnv>> {
    const FLATPAK_INFO_PATH: &str = "/.flatpak-info";

    if fs::try_exists(FLATPAK_INFO_PATH)
        .await
        .wrap_err_with(|| eyre!("checking if `{FLATPAK_INFO_PATH}` exists"))?
    {
        flatpak::Flatpak::init().await
    } else {
        bubblewrap::Bubblewrap::init().await
    }
}

impl SandboxCommand {
    pub(crate) fn take_environment(
        &mut self,
    ) -> BTreeMap<SandboxArg, SandboxArg> {
        if !self.passthrough_environment.is_empty() {
            for (k, v) in std::env::vars_os() {
                let k: SandboxArg = k.into();
                if self.extra_environment.contains_key(&k) {
                    continue;
                }
                if !self.passthrough_environment.contains(&k) {
                    continue;
                }
                self.extra_environment.insert(k, v.into());
            }
        }
        std::mem::take(&mut self.extra_environment)
    }
}

#[async_trait]
#[enum_dispatch]
pub trait SandboxChildOp {
    fn id(&self) -> Option<u32>;
    fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>>;
    async fn wait(&mut self) -> Result<SandboxExitStatus>;
    async fn kill(&mut self) -> Result<()>;
}

#[derive(Debug)]
#[enum_dispatch(SandboxChildOp)]
pub enum SandboxChild {
    Flatpak(flatpak::FlatpakChild),
    Bubblewrap(bubblewrap::BubblewrapChild),
}

/// Describes the result of a process after it has terminated.
///
/// This is analogous to [`std::process::ExitStatus`].
#[derive(Default, Debug, Clone, Copy)]
pub struct SandboxExitStatus {
    // TODO cfg
    imp: unix::UnixSandboxExitStatus,
}

impl SandboxExitStatus {
    /// Was termination successful? Signal termination is not considered a
    /// success, and success is defined as a zero exit status.
    ///
    /// This is analogous to [`std::process::ExitStatus::success`].
    pub fn success(&self) -> bool {
        self.imp.success()
    }

    /// Returns the exit code of the process, if any.
    ///
    /// This is analogous to [`std::process::ExitStatus::code`].
    pub fn code(&self) -> Option<i32> {
        self.imp.code()
    }
}

impl std::fmt::Display for SandboxExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.imp, f)
    }
}
