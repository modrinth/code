use crate::launcher::LauncherError;
use std::process::Command;

pub fn check_java() -> Result<String, LauncherError> {
    let child = Command::new("java")
        .arg("-version")
        .output()
        .map_err(|inner| LauncherError::ProcessError {
            inner,
            process: "java".into(),
        })?;

    let output = String::from_utf8_lossy(&child.stderr);
    let output = output.trim_matches('\"');
    Ok(output.into())
}
