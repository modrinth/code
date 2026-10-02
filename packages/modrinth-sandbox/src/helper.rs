use std::ffi::OsStr;

use eyre::Result;

use crate::backend;

pub type MakeHelper = fn() -> std::process::Command;

const HELPER_ARG: &str = "--modrinth-sandbox-helper";

pub fn run_default() -> Result<bool> {
    let mut args = std::env::args_os();
    if args.next().as_deref() != Some(OsStr::new(HELPER_ARG)) {
        return Ok(false);
    }
    backend::run_helper(args)
}

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
