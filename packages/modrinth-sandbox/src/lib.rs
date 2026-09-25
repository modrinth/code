#![doc = include_str!("../README.md")]

mod backend;
// pub mod ffi;
pub mod minecraft;
mod util;

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{PipeReader, PipeWriter},
    path::PathBuf,
    sync::Arc,
};

use derive_more::Debug;
use eyre::Result;

pub use backend::SandboxExitStatus;
pub use util::argument::SandboxArg;

use crate::backend::SandboxChildOp;

/// Creates the environment and initializes the required resources to perform
/// sandboxing.
///
/// On most platforms, sandboxing is done using operating system primitives, in
/// which case creating the environment will always succeed. On other platforms,
/// sandboxing may require extra dependencies to be installed in the app
/// environment - if those are not fulfilled, this returns [`Err`].
///
/// # Backends
///
/// ## Windows
///
/// TODO
///
/// ## macOS
///
/// TODO
///
/// ## Linux
///
/// Inside a Flatpak environment, this uses the [`org.freedesktop.portal.Flatpak`]
/// portal for spawning processes via the D-Bus session bus.
///
/// Outside of a Flatpak environment, this uses the [Bubblewrap] sandboxing
/// tool via the command line (`bwrap` must be installed on the user's machine).
///
/// # Errors
///
/// Errors if the implementation cannot set up sandboxing.
///
/// [`org.freedesktop.portal.Flatpak`]: https://docs.flatpak.org/en/latest/libflatpak-api-reference.html#gdbus-org.freedesktop.portal.Flatpak
/// [Bubblewrap]: https://github.com/containers/bubblewrap
pub async fn create_env() -> Result<SandboxEnv> {
    backend::create_env().await.map(|imp| SandboxEnv {
        imp: Arc::from(imp),
    })
}

/// Allows spawning processes in a sandboxed environment, configured by
/// [`SandboxCommand`].
///
/// A [`SandboxEnv`] is not aware of Minecraft, Java, the JVM, or any other
/// game-specific details - it only manages the actual process sandboxing.
/// See the [`minecraft`] module for how to set up a sandbox for a Minecraft
/// instance specifically.
#[derive(Debug, Clone)]
#[debug("{imp:?}")]
pub struct SandboxEnv {
    imp: Arc<dyn backend::SandboxEnv>,
}

impl SandboxEnv {
    /// Spawn a sandboxed process and get a [`SandboxChild`] handle to it.
    ///
    /// This function is analogous to [`std::process::Command::spawn`], but
    /// handles the complexity of sandboxing for you. See [`SandboxCommand`] on
    /// what you can configure, such as what program to run, and what files are
    /// exposed in the sandbox.
    pub async fn spawn(&self, command: SandboxCommand) -> Result<SandboxChild> {
        self.imp.spawn(command).await
    }
}

/// Configuration for launching a sandboxed process using [`SandboxEnv::spawn`].
///
/// This is a low-level configuration struct which is not aware of Minecraft,
/// Java, etc. If launching a Minecraft client, first create a command using
/// [`minecraft::create_command`], then customize it to your needs.
///
/// # Implementation notes
///
/// - Graphics and audio devices will always be passed to the child process when
///   possible.
#[derive(Debug)]
pub struct SandboxCommand {
    /// Path to the executable to run.
    pub executable: PathBuf,
    /// Arguments passed to the executable.
    pub args: Vec<SandboxArg>,
    /// When spawning a process, ensure that each of these paths exists as a
    /// directory on the host, automatically creating all parent directories.
    pub ensure_dirs_exist: Vec<PathBuf>,
    /// Host paths mounted read-only at the same path in the sandbox.
    pub read_only_paths: Vec<PathBuf>,
    /// Host paths mounted read-write at the same path in the sandbox.
    pub read_write_paths: Vec<PathBuf>,
    /// What directory the executable is ran from.
    pub working_directory: Option<PathBuf>,
    /// Names of environment variables copied from the host when present.
    pub passthrough_environment: BTreeSet<SandboxArg>,
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
    /// Default io behaviour for stdin
    pub stdin: SandboxStdio,
    /// Default io behaviour for stdout
    pub stdout: SandboxStdio,
    /// Default io behaviour for stderr
    pub stderr: SandboxStdio,
}

/// Describes what to do with a standard I/O stream for a child process when
/// passed to the [`stdin`], [`stdout`], and [`stderr`] methods of
/// [`SandboxCommand`].
///
/// This is analogous to [`std::process::Stdio`].
///
/// [`stdin`]: SandboxCommand::stdin
/// [`stdout`]: SandboxCommand::stdout
/// [`stderr`]: SandboxCommand::stderr
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SandboxStdio {
    Null,
    Inherit,
    Pipe,
}

/// Handle to a child process created by [`SandboxEnv::spawn`].
///
/// This is analogous to [`std::process::Child`], but because operating systems
/// expose different capabilities for child processes depending on how they're
/// spawned, this type is more limited in what you can do with it.
#[derive(Debug)]
#[debug("{imp:?}")]
pub struct SandboxChild {
    /// Handle for writing to the child's standard input (stdin).
    ///
    /// Analogous to [`std::process::Child::stdin`].
    pub stdin: Option<PipeWriter>,
    /// Handle for reading from the child's standard output (stdout).
    ///
    /// Analogous to [`std::process::Child::stdout`].
    pub stdout: Option<PipeReader>,
    /// Handle for reading from the child's standard error (stderr).
    ///
    /// Analogous to [`std::process::Child::stderr`].
    pub stderr: Option<PipeReader>,
    imp: backend::SandboxChild,
}

impl SandboxChild {
    pub fn id(&self) -> Option<u32> {
        SandboxChildOp::id(&self.imp)
    }

    /// Attempts to collect the exit status of the child if it has already
    /// exited.
    ///
    /// This is analogous to [`std::process::Child::try_wait`].
    pub fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>> {
        SandboxChildOp::try_wait(&mut self.imp)
    }

    /// Waits for the child to exit completely, returning the status that it
    /// exited with. This function will continue to have the same return value
    /// after it has been called at least once.
    ///
    /// This is analogous to [`std::process::Child::wait`].
    ///
    /// # Cancel safety
    ///
    /// This function is cancel safe.
    pub async fn wait(&mut self) -> Result<SandboxExitStatus> {
        SandboxChildOp::wait(&mut self.imp).await
    }

    /// Forces the child process to exit. If the child has already exited, `Ok(())`
    /// is returned.
    ///
    /// This is analogous to [`std::process::Child::kill`].
    pub async fn kill(&mut self) -> Result<()> {
        SandboxChildOp::kill(&mut self.imp).await
    }
}
