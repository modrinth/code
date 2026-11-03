#![doc = include_str!("../README.md")]

pub mod api;
mod backend;
pub mod ffi;
#[cfg(unix)]
pub mod file_sharing;
pub mod helper;
pub mod minecraft;
mod util;

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{PipeReader, PipeWriter},
    path::{Path, PathBuf},
    sync::Arc,
};

use derive_more::Debug;
use eyre::Result;

pub use backend::SandboxExitStatus;
pub use util::SandboxArg;

use crate::{
    backend::{SandboxChildOp, SandboxHandleOp},
    helper::MakeHelper,
};

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
    /// Uses the Windows [AppContainer] API for isolating the process from the host.
    ///
    /// ## macOS
    ///
    /// Uses macOS [Seatbelt] API.
    ///
    /// ## Linux
    ///
    /// Inside a Flatpak environment, this uses the [`org.freedesktop.portal.Flatpak`]
    /// portal for spawning processes via the D-Bus session bus.
    ///
    /// Outside of a Flatpak environment, this uses the [Bubblewrap] sandboxing
    /// tool via the command line (`bwrap` must be installed on the user's machine).
    ///
    /// # Helper command
    ///
    /// This uses [`helper::make_default`] to create the helper command. If you
    /// need to customize how the helper process is launched, use
    /// [`SandboxEnv::with_helper`].
    ///
    /// Call [`helper::run_default`] near the start of the executable, before
    /// creating a sandbox environment. See [`helper`] for details.
    ///
    /// # Errors
    ///
    /// Errors if the implementation cannot set up sandboxing.
    ///
    /// [AppContainer]: https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation
    /// [Seatbelt]: https://theapplewiki.com/wiki/Dev:Seatbelt
    /// [`org.freedesktop.portal.Flatpak`]: https://docs.flatpak.org/en/latest/libflatpak-api-reference.html#gdbus-org.freedesktop.portal.Flatpak
    /// [Bubblewrap]: https://github.com/containers/bubblewrap
    pub async fn new() -> Result<Self> {
        Self::with_helper(helper::make_default).await
    }

    /// Creates the environment and initializes the required resources to perform
    /// sandboxing, passing a custom function to make the helper command.
    ///
    /// See [`SandboxEnv::new`] for more details.
    ///
    /// `make_helper` must create a command whose process calls
    /// [`helper::run_default`] during startup. See [`helper`] for details.
    pub async fn with_helper(make_helper: MakeHelper) -> Result<Self> {
        backend::create_env(make_helper).await.map(|imp| Self {
            imp: Arc::from(imp),
        })
    }

    /// Creates a [`SandboxEnv`] which performs no real sandboxing.
    ///
    /// This may be used to retain the safe API surface whether you're creating
    /// a sandboxed or unsandboxed instance.
    #[must_use]
    pub fn noop() -> Self {
        Self {
            imp: Arc::new(backend::noop::NoSandbox),
        }
    }

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
/// [`minecraft::setup_minecraft_command`], then customize it to your needs.
///
/// # Implementation notes
///
/// - Graphics and audio devices will always be passed to the child process when
///   possible.
/// - On Linux, you aren't allowed to mount any paths from the host under `/tmp`
///   since Flatpak doesn't support mounting those files. This applies to both
///   Bubblewrap and Flatpak, so your code stays portable across backends.
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
    pub allow_network: bool,
    /// Whether the program to run is a Java virtual machine.
    ///
    /// If set, performs some extra platform-specific setup to get the JVM to
    /// work.
    pub is_jvm: bool,
    /// Whether the spawned child should terminate when the parent process
    /// terminates.
    pub die_with_parent: bool,
    /// Default io behaviour for stdin.
    pub stdin: SandboxStdio,
    /// Default io behaviour for stdout.
    pub stdout: SandboxStdio,
    /// Default io behaviour for stderr.
    pub stderr: SandboxStdio,
    /// (Windows only) Stable, machine-facing identifier for the [AppContainer]
    /// which this process runs in.
    ///
    /// Must remain stable and unique, and may be displayed to users, but don't
    /// treat it as a localized string.
    ///
    /// [AppContainer]: https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation
    pub app_container_name: SandboxArg,
    /// (Windows only) Human-friendly description of the [AppContainer] which
    /// this process runs in.
    ///
    /// [AppContainer]: https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation
    pub app_container_description: SandboxArg,
    /// Vec of callbacks to be run when the process starts
    pub on_start: Vec<Box<dyn FnOnce(&mut SandboxChild) + Send + Sync>>,
    /// Vec of callbacks to be run when the process terminates or is dropped
    pub on_finish: Vec<Box<dyn FnOnce() + Send + Sync>>,
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
    on_finish: Vec<Box<dyn FnOnce() + Send + Sync>>,
    handle: backend::SandboxHandle,
    imp: backend::SandboxChild,
}

impl SandboxChild {
    pub fn id(&self) -> Option<u32> {
        SandboxChildOp::id(&self.imp)
    }

    /// Add a callback to be run when the process finishes or the struct is dropped
    /// Requires one of the kill/wait/try_wait/etc. methods to be called
    pub fn run_on_finish(
        &mut self,
        on_finish: Box<dyn FnOnce() + Send + Sync>,
    ) {
        self.on_finish.push(on_finish);
    }

    /// Attempts to collect the exit status of the child if it has already
    /// exited.
    ///
    /// This is analogous to [`std::process::Child::try_wait`].
    pub fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>> {
        let res = SandboxChildOp::try_wait(&mut self.imp);
        if matches!(res, Ok(Some(_))) {
            self.finish();
        }
        res
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
        let res = SandboxChildOp::wait(&mut self.imp).await;
        if res.is_ok() {
            self.finish();
        }
        res
    }

    /// Forces the child process to exit. If the child has already exited, `Ok(())`
    /// is returned.
    ///
    /// This is analogous to [`std::process::Child::kill`].
    pub async fn kill(&mut self) -> Result<()> {
        let res = SandboxChildOp::kill(&mut self.imp).await;
        self.finish();
        res
    }

    pub fn get_sandbox_handle(&self) -> SandboxHandle {
        SandboxHandle {
            imp: self.handle.clone(),
        }
    }

    fn finish(&mut self) {
        for on_finish in self.on_finish.drain(..) {
            (on_finish)();
        }
    }
}

#[derive(Clone, Debug)]
pub enum ExposedPath {
    Direct,
    Proxied(PathBuf),
}

#[derive(Clone, Debug)]
pub struct SandboxHandle {
    imp: backend::SandboxHandle,
}

impl SandboxHandle {
    pub fn expose_path(
        &self,
        path: &Path,
        writeable: bool,
    ) -> eyre::Result<ExposedPath> {
        self.imp.expose_path(path, writeable)
    }

    pub fn is_path_readable(&self, path: &Path) -> bool {
        self.imp.is_path_readable(path)
    }

    pub fn is_path_writeable(&self, path: &Path) -> bool {
        self.imp.is_path_writeable(path)
    }
}

pub fn start_file_sharing_test(mountpoint: PathBuf, binds: Vec<PathBuf>) -> eyre::Result<()> {
    let file_sharing = file_sharing::mount(mountpoint, true)?;
    for bind in binds {
        let bound_to = file_sharing.add(bind.clone(), true)?;
        println!("{bind:?} => {bound_to:?}");
    }
    file_sharing.join()?;
    Ok(())
}
