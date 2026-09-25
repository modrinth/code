mod backend;
pub mod minecraft;
mod util;

use std::{
    io::{PipeReader, PipeWriter},
    sync::Arc,
};

use derive_more::Debug;
use eyre::Result;

pub use backend::{SandboxCommand, SandboxExitStatus, SandboxStdio};
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
        self.imp
            .spawn(command)
            .await
            .map(|imp| SandboxChild { imp })
    }
}

/// Handle to a child process created by [`SandboxEnv::spawn`].
///
/// This is analogous to [`std::process::Child`], but because operating systems
/// expose different capabilities for child processes depending on how they're
/// spawned, this type is more limited in what you can do with it.
#[derive(Debug)]
#[debug("{imp:?}")]
pub struct SandboxChild {
    imp: backend::SandboxChild,
}

impl SandboxChild {
    pub fn id(&self) -> Option<u32> {
        SandboxChildOp::id(&self.imp)
    }

    pub fn try_wait(&mut self) -> Result<Option<SandboxExitStatus>> {
        SandboxChildOp::try_wait(&mut self.imp)
    }

    pub async fn wait(&mut self) -> Result<SandboxExitStatus> {
        SandboxChildOp::wait(&mut self.imp).await
    }

    pub async fn kill(&mut self) -> Result<()> {
        SandboxChildOp::kill(&mut self.imp).await
    }

    pub fn take_stdin(&mut self) -> Option<PipeWriter> {
        SandboxChildOp::take_stdin(&mut self.imp)
    }

    pub fn take_stdout(&mut self) -> Option<PipeReader> {
        SandboxChildOp::take_stdout(&mut self.imp)
    }

    pub fn take_stderr(&mut self) -> Option<PipeReader> {
        SandboxChildOp::take_stderr(&mut self.imp)
    }
}
