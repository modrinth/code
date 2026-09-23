//! Backend sandbox implementations, using OS-specific primitives.

use std::{ffi::OsString, fmt::Debug, path::PathBuf, process::ExitStatus};

use async_trait::async_trait;
use eyre::Result;

// TODO cfgs
mod bubblewrap;
mod linux;

/// Entry point into the sandboxing mechanism.
///
/// Each backend can create a [`SandboxEnv`], from which you can spawn sandboxed
/// processes.
///
/// # Platform
///
/// Each platform has its own sandboxing backend(s):
/// - Linux
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
    bubblewrap::Bubblewrap::init().await
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
    pub executable: OsString,
    /// Arguments passed to the executable.
    pub args: Vec<OsString>,
    /// Host paths mounted read-only at the same path in the sandbox.
    pub read_only_paths: Vec<OsString>,
    /// Host paths mounted read-write at the same path in the sandbox.
    pub read_write_paths: Vec<OsString>,
    /// What directory the executable is ran from.
    pub working_directory: Option<PathBuf>,
    /// Names of environment variables copied from the host when present.
    pub passthrough_environment: Vec<OsString>,
    /// Environment variables explicitly set in the sandbox.
    ///
    /// These take precedence over passthrough variables with the same name.
    pub extra_environment: Vec<(OsString, OsString)>,
    /// How the child process's output is routed to the parent process.
    pub output: SandboxOutput,
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

#[derive(Debug, Clone, Copy, Default)]
pub enum SandboxOutput {
    #[default]
    Piped,
    Inherit,
}

#[derive(Debug)]
pub struct SandboxChild {
    // TODO cfg
    imp: linux::SandboxChild,
}

impl SandboxChild {
    pub fn id(&self) -> Option<u32> {
        self.imp.id()
    }

    pub fn take_stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        self.imp.take_stdout()
    }

    pub fn take_stderr(&mut self) -> Option<tokio::process::ChildStderr> {
        self.imp.take_stderr()
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        self.imp.try_wait()
    }

    pub async fn wait(&mut self) -> Result<ExitStatus> {
        self.imp.wait().await
    }

    pub async fn kill(&mut self) -> Result<()> {
        self.imp.kill().await
    }
}
