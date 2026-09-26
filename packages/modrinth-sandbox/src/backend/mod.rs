//! Backend sandbox implementations, using OS-specific primitives.

use std::{collections::{BTreeMap, HashSet}, fmt::Debug};

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use eyre::Result;

use crate::{SandboxCommand, util::SandboxArg};

#[cfg(target_os = "linux")]
mod bubblewrap;
#[cfg(target_os = "linux")]
mod flatpak;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

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
    #[cfg(target_os = "linux")]
    {
        const FLATPAK_INFO_PATH: &str = "/.flatpak-info";

        if tokio::fs::try_exists(FLATPAK_INFO_PATH)
            .await
            .map_err(|err| {
                err.wrap_err(eyre::eyre!(
                    "checking if `{FLATPAK_INFO_PATH}` exists"
                ))
            })?
        {
            flatpak::Flatpak::init().await
        } else {
            bubblewrap::Bubblewrap::init().await
        }
    }

    #[cfg(target_os = "macos")]
    {
        return macos::Macos::init().await;
    }

    #[cfg(windows)]
    {
        return windows::appcontainer::AppContainer::init().await;
    }
}

impl SandboxCommand {
    pub(crate) fn take_environment(
        &mut self,
    ) -> BTreeMap<SandboxArg, SandboxArg> {
        let mut passthrough = std::mem::take(&mut self.passthrough_environment).into_iter().map(|k| {
            let mut k = k.into_os_string();
            k.make_ascii_uppercase();
            k
        }).collect::<HashSet<_>>();

        if cfg!(windows) {
            passthrough.insert("APPDATA".into());
            passthrough.insert("LOCALAPPDATA".into());
        }

        if !passthrough.is_empty() {
            let ignore_overrides = self.extra_environment.keys().map(|k| k.as_os_str().to_ascii_uppercase()).collect::<HashSet<_>>();

            for (k, v) in std::env::vars_os() {
                if k.as_encoded_bytes().contains(&b'=') || v.as_encoded_bytes().contains(&b'=') {
                    continue;
                }

                let upper_k = k.to_ascii_uppercase();
                if ignore_overrides.contains(&upper_k) {
                    continue;
                }
                // if !passthrough.contains(&upper_k) {
                //     continue;
                // }

                self.extra_environment.insert(k.into(), v.into());
            }
        }

        std::mem::take(&mut self.extra_environment)
    }
}

#[derive(Debug)]
pub struct Pipes {
    pub stdin: Option<std::io::PipeWriter>,
    pub stdout: Option<std::io::PipeReader>,
    pub stderr: Option<std::io::PipeReader>,
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
    #[cfg(target_os = "linux")]
    Flatpak(flatpak::FlatpakChild),
    #[cfg(target_os = "linux")]
    Bubblewrap(bubblewrap::BubblewrapChild),
    #[cfg(target_os = "macos")]
    Macos(macos::MacosChild),
    #[cfg(target_os = "windows")]
    AppContainer(windows::WindowsChild),
}

/// Describes the result of a process after it has terminated.
///
/// This is analogous to [`std::process::ExitStatus`].
#[derive(Default, Debug, Clone, Copy)]
pub struct SandboxExitStatus {
    #[cfg(unix)]
    imp: unix::UnixSandboxExitStatus,
    #[cfg(windows)]
    imp: windows::WindowsSandboxExitStatus,
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
