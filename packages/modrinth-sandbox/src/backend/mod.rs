//! Backend sandbox implementations, using OS-specific primitives.

use std::{ffi::OsString, fmt::Debug, path::PathBuf, process::ExitStatus};

use async_trait::async_trait;
use eyre::Result;

mod bubblewrap;

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
#[derive(Debug)]
pub struct SandboxCommand {
    /// Path to the executable to run.
    pub executable: String,
    /// Arguments passed to the executable.
    pub args: Vec<String>,
    /// Host paths mounted read-only at the configured path in the sandbox.
    pub read_only_paths: Vec<(PathBuf, PathBuf)>,
    /// Host paths mounted read-write at the configured path in the sandbox.
    pub read_write_paths: Vec<(PathBuf, PathBuf)>,
    /// What directory the executable is ran from.
    pub working_directory: Option<PathBuf>,
    /// Names of environment variables copied from the host when present.
    pub passthrough_environment: Vec<OsString>,
    /// Environment variables explicitly set in the sandbox.
    ///
    /// These take precedence over passthrough variables with the same name.
    pub extra_environment: Vec<(OsString, OsString)>,
    pub output: SandboxOutput,
    /// Expose read-only operating-system libraries and configuration needed by
    /// dynamically linked applications.
    pub system_runtime: bool,
    /// Allow access to the host network namespace.
    pub network: bool,
    /// Allow access to GPU devices and the active Wayland or X11 display.
    pub graphics: bool,
    /// Allow access to audio devices and the active PipeWire or PulseAudio socket.
    pub audio: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SandboxOutput {
    #[default]
    Piped,
    Inherit,
}

#[derive(Debug)]
pub struct SandboxChild {
    child: tokio::process::Child,
}

impl SandboxChild {
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }

    pub fn take_stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        self.child.stdout.take()
    }

    pub fn take_stderr(&mut self) -> Option<tokio::process::ChildStderr> {
        self.child.stderr.take()
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        Ok(self.child.try_wait()?)
    }

    pub async fn wait(&mut self) -> Result<ExitStatus> {
        Ok(self.child.wait().await?)
    }

    pub async fn kill(&mut self) -> Result<()> {
        Ok(self.child.kill().await?)
    }
}
