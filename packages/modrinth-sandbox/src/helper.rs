//! Support for running sandbox setup operations in a helper process.
//!
//! Call [`run_default`] near the beginning of the executable, before parsing
//! application arguments or creating a [`crate::SandboxEnv`]. On Windows, the
//! sandbox may relaunch this helper with elevated privileges to configure
//! AppContainer ACLs and network isolation.
//!
//! [`crate::SandboxEnv::new`] relaunches the current executable by default. Use
//! [`crate::SandboxEnv::with_helper`] when another executable or additional
//! command arguments are required.

use std::ffi::OsStr;

use eyre::Result;

use crate::backend;

/// Creates the command used to run privileged sandbox setup operations.
///
/// The resulting process must call [`run_default`] during startup. Arguments
/// describing the setup operation are appended to the command before launch.
pub type MakeHelper = fn() -> std::process::Command;

const HELPER_ARG: &str = "--modrinth-sandbox-helper";

/// Handles a sandbox helper invocation for the default helper command.
///
/// Returns `true` after handling a helper invocation and `false` during normal
/// application startup. When this returns `true`, the caller should exit
/// successfully without continuing application initialization.
pub fn run_default() -> Result<bool> {
    let mut args = std::env::args_os();
    if args.next().as_deref() != Some(OsStr::new(HELPER_ARG)) {
        return Ok(false);
    }
    backend::run_helper(args)
}

/// Creates the default helper command by relaunching the current executable.
///
/// The executable must call [`run_default`] during startup.
pub fn make_default() -> std::process::Command {
    let current_exe = {
        #[cfg(unix)]
        {
            std::path::PathBuf::from("/proc/self/exe")
        }
        #[cfg(windows)]
        {
            std::env::current_exe().expect("getting current exe")
        }
    };

    let mut command = std::process::Command::new(current_exe);
    command.arg(HELPER_ARG);
    command
}
