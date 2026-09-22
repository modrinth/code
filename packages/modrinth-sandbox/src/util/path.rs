use std::{env, ffi::OsStr, path::PathBuf};

use eyre::{ContextCompat, Result, eyre};
use tokio::fs;

pub async fn find_command(command: &OsStr) -> Result<PathBuf> {
    let path = env::var_os("PATH").wrap_err("missing `PATH`")?;
    for mut path in env::split_paths(&path) {
        if !path.is_absolute() {
            continue;
        }
        path.push(command);
        if fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(path);
        }
    }

    Err(eyre!("command not found in `PATH`"))
}
