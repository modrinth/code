//! Backend sandbox implementations, using OS-specific primitives.

use std::{
    collections::{BTreeMap, HashSet},
    ffi::OsString,
    fmt::Debug,
    path::Path,
    sync::OnceLock,
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;

use crate::{
    ExposedPath, SandboxCommand,
    backend::pathset::{PathSet, ReadWritePathSet},
    helper::MakeHelper,
    path_access::{PathPermission, PathRules},
    util::{SandboxArg, resolve_path},
};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
pub mod noop;
pub(super) mod pathset;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

/// See [`crate::SandboxEnv::new`].
#[async_trait]
pub trait Backend {
    /// See [`crate::SandboxEnv::new`].
    async fn init(make_helper: MakeHelper) -> Result<Box<dyn SandboxEnv>>;
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

/// See [`crate::SandboxEnv::new`].
pub async fn create_env(
    make_helper: MakeHelper,
) -> Result<Box<dyn SandboxEnv>> {
    #[cfg(target_os = "linux")]
    {
        const FLATPAK_INFO_PATH: &str = "/.flatpak-info";

        if tokio::fs::try_exists(FLATPAK_INFO_PATH)
            .await
            .map_err(|err| {
                anyhow::anyhow!(err)
                    .context("checking if `{FLATPAK_INFO_PATH}` exists")
            })?
        {
            linux::flatpak::Flatpak::init(make_helper).await
        } else {
            linux::bubblewrap::Bubblewrap::init(make_helper).await
        }
    }

    #[cfg(target_os = "macos")]
    {
        return macos::Macos::init(make_helper).await;
    }

    #[cfg(windows)]
    {
        return windows::appcontainer::AppContainer::init(make_helper).await;
    }
}

impl SandboxCommand {
    /// Fails if any of the paths are hard blocked, or contain a blocked path.
    pub(crate) fn take_paths(
        &mut self,
    ) -> anyhow::Result<(ReadWritePathSet, PathRules)> {
        let mut read_only_paths = std::mem::take(&mut self.read_only_paths);
        let mut requested_write_paths =
            std::mem::take(&mut self.read_write_paths);

        read_only_paths.push(resolve_path(&self.executable)?);
        if let Some(working) = self.working_directory.clone() {
            read_only_paths.push(working);
        }

        // Rules for parents have to be before rules for their children
        read_only_paths.sort();
        read_only_paths.dedup();
        requested_write_paths.sort();
        requested_write_paths.dedup();

        // Paths inside of other read-only paths don't need their own rule
        let mut read_only_set = PathSet::default();
        for path in read_only_paths {
            read_only_set.insert(path);
        }
        let mut read_only_paths =
            read_only_set.iter().cloned().collect::<Vec<_>>();

        let mut rules = PathRules::new(std::mem::take(&mut self.path_rules));
        for path in &read_only_paths {
            rules.add_bind(path, PathPermission::ReadOnly)?;
        }
        let mut read_write_paths = Vec::new();
        for path in requested_write_paths {
            match rules.add_bind(&path, PathPermission::ReadWrite)? {
                PathPermission::ReadOnly => {
                    tracing::warn!(
                        "only giving the sandbox read access to {path:?}, since it is, or contains, a path that must not be written to"
                    );
                    read_only_paths.push(path);
                }
                PathPermission::ReadWrite => read_write_paths.push(path),
            }
        }

        let path_set =
            ReadWritePathSet::from_vecs(read_only_paths, read_write_paths);
        Ok((path_set, rules))
    }

    pub(crate) fn take_environment(
        &mut self,
    ) -> BTreeMap<SandboxArg, SandboxArg> {
        let mut passthrough_upper =
            std::mem::take(&mut self.passthrough_environment)
                .into_iter()
                .map(|k| {
                    let mut k = k.into_os_string();
                    k.make_ascii_uppercase();
                    k
                })
                .collect::<HashSet<_>>();

        if cfg!(windows) {
            passthrough_upper.insert("APPDATA".into());
            passthrough_upper.insert("LOCALAPPDATA".into());
        }

        if !passthrough_upper.is_empty() {
            let ignore_overrides_upper = self
                .extra_environment
                .keys()
                .map(|k| k.as_os_str().to_ascii_uppercase())
                .collect::<HashSet<_>>();

            for (k, v) in std::env::vars_os() {
                let upper_k = k.to_ascii_uppercase();
                if ignore_overrides_upper.contains(&upper_k) {
                    continue;
                }
                if !passthrough_upper.contains(&upper_k) {
                    continue;
                }

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
    Noop(noop::StdChild),
    #[cfg(target_os = "linux")]
    Flatpak(linux::flatpak::FlatpakChild),
    #[cfg(target_os = "linux")]
    Bubblewrap(linux::bubblewrap::BubblewrapChild),
    #[cfg(target_os = "macos")]
    Macos(macos::MacosChild),
    #[cfg(target_os = "windows")]
    AppContainer(windows::AppContainerChild),
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

#[enum_dispatch]
pub trait SandboxHandleOp {
    fn expose_path(
        &self,
        path: &Path,
        permission: PathPermission,
    ) -> anyhow::Result<ExposedPath>;
    fn is_path_readable(&self, path: &Path) -> bool;
    fn is_path_writeable(&self, path: &Path) -> bool;
    fn is_path_blocked(&self, path: &Path) -> bool;
}

#[derive(Clone, Debug)]
#[enum_dispatch(SandboxHandleOp)]
pub enum SandboxHandle {
    Noop(noop::NoopSandboxHandle),
    #[cfg(target_os = "linux")]
    Flatpak(linux::flatpak::FlatpakSandboxHandle),
    #[cfg(target_os = "linux")]
    Bubblewrap(linux::bubblewrap::BubblewrapSandboxHandle),
    #[cfg(target_os = "macos")]
    Macos(macos::MacosSandboxHandle),
    #[cfg(target_os = "windows")]
    AppContainer(windows::WindowsSandboxHandle),
}

pub fn run_helper(args: impl IntoIterator<Item = OsString>) -> Result<bool> {
    #[cfg(windows)]
    {
        windows::appcontainer::run_helper(args)
    }
    #[cfg(not(windows))]
    {
        drop(args);
        Ok(false)
    }
}

/// We use our own thread for spawning commands instead of tokio::spawn_blocking since
/// tokio::spawn_blocking threads can die after inactivity which can trigger PR_SET_PDEATHSIG.
/// On Windows the thread has COM initialized, so it is also used for shell calls like ShellExecuteExW.
pub async fn run_on_spawn_thread<T: Send + 'static>(
    run: impl FnOnce() -> T + Send + 'static,
) -> Result<T> {
    let (sender, receiver) = tokio::sync::oneshot::channel();

    type SpawnTask = Box<dyn FnOnce() + Send>;

    static SPAWNING_CHANNEL: OnceLock<std::sync::mpsc::Sender<SpawnTask>> =
        OnceLock::new();
    let channel = SPAWNING_CHANNEL.get_or_init(|| {
        let (send, recv) = std::sync::mpsc::channel::<SpawnTask>();

        std::thread::Builder::new()
            .name("Modrinth Sandbox Command Spawner".to_string())
            .spawn(|| {
                #[cfg(windows)]
                unsafe {
                    use ::windows::Win32::System::Com::{
                        COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
                        CoInitializeEx,
                    };
                    _ = CoInitializeEx(
                        None,
                        COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
                    );
                }

                for task in recv {
                    task();
                }
            })
            .unwrap();

        send
    });
    channel
        .send(Box::new(move || {
            _ = sender.send(run());
        }))
        .unwrap();

    receiver.await.context("spawn task dropped")
}
