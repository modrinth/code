use crate::launcher::LauncherError;
use lazy_static::lazy_static;
use regex::Regex;
use std::process::Command;

lazy_static! {
    static ref JAVA_VERSION_REGEX: Regex = Regex::new(r#""(.*?)""#).unwrap();
}

pub fn check_java() -> Result<Option<String>, LauncherError> {
    let child = Command::new("java")
        .arg("-version")
        .output()
        .map_err(|err| LauncherError::ProcessError {
            inner: err,
            process: "java".to_string(),
        })?;

    let output = &*String::from_utf8_lossy(&*child.stderr);

    if let Some(version_raw) = JAVA_VERSION_REGEX.find(output) {
        let mut raw = version_raw.as_str().chars();
        raw.next();
        raw.next_back();

        return Ok(Some(raw.as_str().to_string()));
    }

    Ok(None)
}
