use dunce::canonicalize;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};
use std::env;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY},
    RegKey,
};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub struct JavaVersion {
    pub path: String,
    pub version: String,
}

// Entrypoint function (Windows)
// Returns a Vec of unique JavaVersions from the PATH, Windows Registry Keys and common Java locations
#[cfg(target_os = "windows")]
#[tracing::instrument]
pub fn get_all_jre() -> Result<Vec<JavaVersion>, JREError> {
    // Use HashSet to avoid duplicates
    let mut jres = HashSet::new();

    // Add JRES directly on PATH
    jres.extend(get_all_jre_path()?);

    // Hard paths for locations for commonly installed .exes
    let java_paths = [r"C:/Program Files/Java", r"C:/Program Files (x86)/Java"];
    for java_path in java_paths {
        let Ok(java_subpaths) = std::fs::read_dir(java_path) else {continue };
        for java_subpath in java_subpaths {
            let path = java_subpath?.path();
            if let Some(j) =
                check_java_at_filepath(&PathBuf::from(path).join("bin"))
            {
                jres.insert(j);
                break;
            }
        }
    }

    // Windows Registry Keys
    let key_paths = [
        r"SOFTWARE\JavaSoft\Java Runtime Environment", // Oracle
        r"SOFTWARE\JavaSoft\Java Development Kit",
        r"SOFTWARE\\JavaSoft\\JRE", // Oracle
        r"SOFTWARE\\JavaSoft\\JDK",
        r"SOFTWARE\\Eclipse Foundation\\JDK", // Eclipse
        r"SOFTWARE\\Eclipse Adoptium\\JRE",   // Eclipse
        r"SOFTWARE\\Eclipse Foundation\\JDK", // Eclipse
        r"SOFTWARE\\Microsoft\\JDK",          // Microsoft
    ];
    for key in key_paths {
        if let Ok(jre_key) = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey_with_flags(key, KEY_READ | KEY_WOW64_32KEY)
        {
            jres.extend(get_all_jre_winregkey(jre_key)?);
        }
        if let Ok(jre_key) = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey_with_flags(key, KEY_READ | KEY_WOW64_64KEY)
        {
            jres.extend(get_all_jre_winregkey(jre_key)?);
        }
    }

    Ok(jres.into_iter().collect())
}

#[cfg(target_os = "windows")]
#[tracing::instrument]
pub fn get_all_jre_winregkey(
    jre_key: RegKey,
) -> Result<HashSet<JavaVersion>, JREError> {
    let mut jres = HashSet::new();

    for subkey in jre_key.enum_keys() {
        let subkey = subkey?;
        let subkey = jre_key.open_subkey(subkey)?;

        let subkey_value_names =
            [r"JavaHome", r"InstallationPath", r"\\hotspot\\MSI"];

        for subkey_value in subkey_value_names {
            let path: Result<String, std::io::Error> =
                subkey.get_value(subkey_value);
            let Ok(path) = path else {continue};
            if let Some(j) =
                check_java_at_filepath(&PathBuf::from(path).join("bin"))
            {
                jres.insert(j);
                break;
            }
        }
    }
    Ok(jres)
}

// Entrypoint function (Mac)
// Returns a Vec of unique JavaVersions from the PATH, and common Java locations
#[cfg(target_os = "macos")]
#[tracing::instrument]
pub fn get_all_jre() -> Result<Vec<JavaVersion>, JREError> {
    // Use HashSet to avoid duplicates
    let mut jres = HashSet::new();

    // Add JREs directly on PATH
    jres.extend(get_all_jre_path()?);

    // Hard paths for locations for commonly installed .exes
    let java_paths = [
        r"/Applications/Xcode.app/Contents/Applications/Application Loader.app/Contents/MacOS/itms/java",
        r"/Library/Internet Plug-Ins/JavaAppletPlugin.plugin/Contents/Home",
        r"/System/Library/Frameworks/JavaVM.framework/Versions/Current/Commands",
    ];
    for path in java_paths {
        if let Some(j) = check_java_at_filepath(PathBuf::from(path).join("bin"))
        {
            jres.insert(j);
            break;
        }
    }
    Ok(jres.into_iter().collect())
}

// Entrypoint function (Linux)
// Returns a Vec of unique JavaVersions from the PATH, and common Java locations
#[cfg(target_os = "linux")]
#[tracing::instrument]
pub fn get_all_jre() -> Result<Vec<JavaVersion>, JREError> {
    // Use HashSet to avoid duplicates
    let mut jres = HashSet::new();

    // Add JREs directly on PATH
    jres.extend(get_all_jre_path()?);

    // Hard paths for locations for commonly installed locations
    let java_paths = [
        r"/usr",
        r"/usr/java",
        r"/usr/lib/jvm",
        r"/usr/lib64/jvm",
        r"/opt/jdk",
        r"/opt/jdks",
    ];
    for path in java_paths {
        if let Some(j) =
            check_java_at_filepath(&PathBuf::from(path).join("jre").join("bin"))
        {
            jres.insert(j);
            break;
        }
        if let Some(j) = check_java_at_filepath(&PathBuf::from(path).join("bin"))
        {
            jres.insert(j);
            break;
        }
    }
    Ok(jres.into_iter().collect())
}

// Gets all JREs from the PATH env variable
#[tracing::instrument]
fn get_all_jre_path() -> Result<HashSet<JavaVersion>, JREError> {
    // Iterate over values in PATH variable, where accessible JREs are referenced
    let paths = env::var("PATH")?;
    let paths = env::split_paths(&paths);

    let mut jres = HashSet::new();
    for path in paths {
        if let Some(j) = check_java_at_filepath(&path) {
            jres.insert(j);
        }
    }
    Ok(jres)
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
const JAVA_BIN: &str = "java.exe";

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
const JAVA_BIN: &str = "java";

// For example filepath 'path', attempt to resolve it and get a Java version at this path
// If no such path exists, or no such valid java at this path exists, returns None
#[tracing::instrument]
pub fn check_java_at_filepath(path: &Path) -> Option<JavaVersion> {
    // Attempt to canonicalize the potential java filepath
    // If it fails, this path does not exist and None is returned (no Java here)
    let Ok(path) = canonicalize(path) else { return None };

    // Checks for existence of Java at this filepath
    // Adds JAVA_BIN to the end of the path if it is not already there
    let java = if path.file_name()?.to_str()? != JAVA_BIN {
        path.join(JAVA_BIN)
    } else {
        path
    };

    if !java.exists() {
        return None;
    };

    // Run 'java -version' using found java binary
    let output = Command::new(&java).arg("-version").output().ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Match: version "1.8.0_361"
    // Extracting version numbers
    lazy_static! {
        static ref JAVA_VERSION_CAPTURE: Regex =
            Regex::new(r#"version "([\d\._]+)""#).unwrap();
    }

    // Extract version info from it
    if let Some(captures) = JAVA_VERSION_CAPTURE.captures(&stderr) {
        if let Some(version) = captures.get(1) {
            let Some(path) = java.to_str() else { return None };
            let path = path.to_string();
            return Some(JavaVersion {
                path,
                version: version.as_str().to_string(),
            });
        }
    }
    None
}

/// Extract major/minor version from a java version string
/// Gets the minor version or an error, and assumes 1 for major version if it could not find
/// "1.8.0_361" -> (1, 8)
/// "20" -> (1, 20)
pub fn extract_java_majorminor_version(
    version: &str,
) -> Result<(u32, u32), JREError> {
    let mut split = version.split('.');
    let major_opt = split.next();
    
    let mut major;
    // Try minor. If doesn't exist, in format like "20" so use major
    let mut minor = if let Some(minor) = split.next() {
        major = major_opt.unwrap_or("1").parse::<u32>()?;
        minor.parse::<u32>()?
    } else {
        // Formatted like "20", only one value means that is minor version
        major = 1;
        major_opt.ok_or_else(|| JREError::InvalidJREVersion(version.to_string()))?.parse::<u32>()?
    };
    
    // Java start should always be 1. If more than 1, it is formatted like "17.0.1.2" and starts with minor version
    if major > 1 {
        minor = major;
        major = 1;
    } 

    Ok((major, minor))
}

#[derive(thiserror::Error, Debug)]
pub enum JREError {
    #[error("Command error : {0}")]
    IOError(#[from] std::io::Error),

    #[error("Env error: {0}")]
    EnvError(#[from] env::VarError),

    #[error("No JRE found for required version: {0}")]
    NoJREFound(String),

    #[error("Invalid JRE version string: {0}")]
    InvalidJREVersion(String),

    #[error("Parsing error: {0}")]
    ParseError(#[from] std::num::ParseIntError),

    #[error("No stored tag for Minecraft Version {0}")]
    NoMinecraftVersionFound(String),
}

#[cfg(test)]
mod tests {
    use super::extract_java_majorminor_version;

    #[test]
    pub fn java_version_parsing() {
        assert_eq!(extract_java_majorminor_version("1.8").unwrap(),(1,8));
        assert_eq!(extract_java_majorminor_version("17.0.6").unwrap(),(1,17));
        assert_eq!(extract_java_majorminor_version("20").unwrap(),(1,20));
        assert_eq!(extract_java_majorminor_version("1.8.0_361").unwrap(),(1,8));
    }
}