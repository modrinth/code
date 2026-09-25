//! Backend sandbox implementations, using OS-specific primitives.

use std::{
    collections::{BTreeMap, HashSet},
    fmt::Debug,
    path::PathBuf,
};

use async_trait::async_trait;
use eyre::{Context, Result, eyre};
use tokio::fs;

use crate::util::argument::SandboxArg;

// TODO cfgs
mod bubblewrap;
mod flatpak;
mod unix;

/// Entry point into the sandboxing mechanism.
///
/// Each backend can create a [`SandboxEnv`], from which you can spawn sandboxed
/// processes.
///
/// # Platform
///
/// Each platform has its own sandboxing backend(s):
/// - Linux
///   - In Flatpak: [`flatpak`]
///   - Outside of Flatpak: [`bubblewrap`]
#[async_trait]
pub trait Backend {
    /// Sets up the sandboxing mechanism.
    ///
    /// # Errors
    ///
    /// Errors if the implementation could not create a sandboxing environment.
    async fn init() -> Result<Box<dyn SandboxEnv>>;
}

/// Allows spawning a process in a sandbox.
///
/// [`SandboxEnv`]s are not aware of Minecraft, Java, or any other high-level
/// details. They are purely low-level sandboxing mechanisms.
#[async_trait]
pub trait SandboxEnv: Debug + Send + Sync {
    /// Spawn a sandboxed process and get a [`SandboxChild`] handle to it.
    async fn spawn(&self, command: SandboxCommand) -> Result<SandboxChild>;
}

/// Creates a [`SandboxEnv`] by automatically determining the best environment
/// to create.
pub async fn init_env() -> Result<Box<dyn SandboxEnv>> {
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

/// Configuration for launching a sandboxed process.
///
/// This is a low-level configuration struct which is not aware of Minecraft,
/// Java, etc.
///
/// # Implementation notes
///
/// - Standard output/error cannot be read from the spawned child process, since
///   on some platforms we cannot reliably inherit these handles.
/// - Graphics and audio devices will always be passed to the child process when
///   possible.
#[derive(Debug)]
pub struct SandboxCommand {
    /// Path to the executable to run.
    pub executable: PathBuf,
    /// Arguments passed to the executable.
    pub args: Vec<SandboxArg>,
    /// When spawning a process, ensure that each of these paths exists as a
    /// directory on the host, using `create_dir_all`.
    pub ensure_dirs_exist: Vec<PathBuf>,
    /// Host paths mounted read-only at the same path in the sandbox.
    pub read_only_paths: Vec<PathBuf>,
    /// Host paths mounted read-write at the same path in the sandbox.
    pub read_write_paths: Vec<PathBuf>,
    /// What directory the executable is ran from.
    pub working_directory: Option<PathBuf>,
    /// Names of environment variables copied from the host when present.
    pub passthrough_environment: HashSet<SandboxArg>,
    /// Environment variables explicitly set in the sandbox.
    ///
    /// These take precedence over passthrough variables with the same name.
    pub extra_environment: BTreeMap<SandboxArg, SandboxArg>,
    /// Allow access to the host network namespace.
    pub network: bool,
    /// Whether the program to run is a Java virtual machine.
    ///
    /// If set, performs some extra platform-specific setup to get the JVM to
    /// work.
    pub is_jvm: bool,
    /// Whether the spawned child should terminate when the parent process
    /// terminates.
    pub die_with_parent: bool,
}

impl SandboxCommand {
    pub fn take_environment(&mut self) -> BTreeMap<SandboxArg, SandboxArg> {
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
pub trait SandboxChildTrait {
     fn id(&self) -> Option<u32>;
     fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>>;
     async fn wait(&mut self) -> Result<SandboxExitStatus>;
     async fn kill(&mut self) -> Result<()>;
}

#[derive(Debug)]
pub enum SandboxChildImpl {
    Bubblewrap(bubblewrap::BubblewrapSandboxChild),
}

#[derive(Debug)]
pub struct SandboxChild(pub(crate) SandboxChildImpl);

#[async_trait]
impl SandboxChildTrait for SandboxChild {
    fn id(&self) -> Option<u32> {
        match &self.0 {
            SandboxChildImpl::Bubblewrap(child) => SandboxChildTrait::id(child),
        }
    }

    fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>> {
        match &mut self.0 {
            SandboxChildImpl::Bubblewrap(child) => SandboxChildTrait::try_wait(child),
        }
    }

    async fn wait(&mut self) -> Result<SandboxExitStatus> {
        match &mut self.0 {
            SandboxChildImpl::Bubblewrap(child) => SandboxChildTrait::wait(child).await,
        }
    }

    async fn kill(&mut self) -> Result<()> {
        match &mut self.0 {
            SandboxChildImpl::Bubblewrap(child) => SandboxChildTrait::kill(child).await,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SandboxExitStatus {
    // TODO cfg
    imp: unix::UnixSandboxExitStatus,
}

impl SandboxExitStatus {
    pub fn success(&self) -> bool {
        self.imp.success()
    }

    pub fn code(&self) -> Option<i32> {
        self.imp.code()
    }
}

impl std::fmt::Display for SandboxExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.imp, f)
    }
}
