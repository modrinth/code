use lazy_static::lazy_static;
use regex::Regex;
use std::process::Command;

#[derive(thiserror::Error, Debug)]
pub enum JavaError {
    #[error("System Error")]
    SystemError(#[from] std::io::Error),
}

lazy_static! {
    static ref JAVA_VERSION_REGEX: Regex = Regex::new(r#""(.*?)""#).unwrap();
}

pub fn check_java() -> Result<Option<String>, JavaError> {
    let child = Command::new("/usr/lib/jvm/java-8-openjdk/jre/bin/java")
        .arg("-version")
        .output()?;

    let output = &*String::from_utf8_lossy(&*child.stderr);

    if let Some(version_raw) = JAVA_VERSION_REGEX.find(output) {
        let mut raw = version_raw.as_str().chars();
        raw.next();
        raw.next_back();

        return Ok(Some(raw.as_str().to_string()));
    }

    Ok(None)
}
