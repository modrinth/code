use super::io;
use crate::state::JavaVersion;
use futures::prelude::*;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::{collections::HashSet, path::Path};
use tokio::task::JoinError;

use crate::{State, get_resource_file};
#[cfg(target_os = "windows")]
use winreg::{
    RegKey,
    enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY},
};

// Entrypoint function (Windows)
// Returns a Vec of unique JavaVersions from the PATH, Windows Registry Keys and common Java locations
#[cfg(target_os = "windows")]
#[tracing::instrument]
pub async fn get_all_jre() -> Result<Vec<JavaVersion>, JREError> {
    let mut jre_paths = HashSet::new();

    // Add JRES directly on PATH
    jre_paths.extend(get_all_jre_path().await);
    jre_paths.extend(get_all_autoinstalled_jre_path().await?);
    if let Ok(java_home) = env::var("JAVA_HOME") {
        jre_paths.insert(PathBuf::from(java_home));
    }

    // Hard paths for locations for commonly installed .exes
    let java_paths = [
        r"C:/Program Files/Java",
        r"C:/Program Files (x86)/Java",
        r"C:\Program Files\Eclipse Adoptium",
        r"C:\Program Files (x86)\Eclipse Adoptium",
    ];
    for java_path in java_paths {
        let Ok(java_subpaths) = std::fs::read_dir(java_path) else {
            continue;
        };
        for java_subpath in java_subpaths.flatten() {
            let path = java_subpath.path();
            jre_paths.insert(path.join("bin"));
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
            jre_paths.extend(get_paths_from_jre_winregkey(jre_key));
        }
        if let Ok(jre_key) = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey_with_flags(key, KEY_READ | KEY_WOW64_64KEY)
        {
            jre_paths.extend(get_paths_from_jre_winregkey(jre_key));
        }
    }

    // Get JRE versions from potential paths concurrently
    let j = check_java_at_filepaths(jre_paths)
        .await
        .into_iter()
        .collect();
    Ok(j)
}

// Gets paths rather than search directly as RegKeys should not be passed asynchronously (do not impl Send)
#[cfg(target_os = "windows")]
#[tracing::instrument]
pub fn get_paths_from_jre_winregkey(jre_key: RegKey) -> HashSet<PathBuf> {
    let mut jre_paths = HashSet::new();

    for subkey in jre_key.enum_keys().flatten() {
        if let Ok(subkey) = jre_key.open_subkey(subkey) {
            let subkey_value_names =
                [r"JavaHome", r"InstallationPath", r"\\hotspot\\MSI"];

            for subkey_value in subkey_value_names {
                let path: Result<String, std::io::Error> =
                    subkey.get_value(subkey_value);
                let Ok(path) = path else { continue };

                jre_paths.insert(PathBuf::from(path).join("bin"));
            }
        }
    }
    jre_paths
}

// Entrypoint function (Mac)
// Returns a Vec of unique JavaVersions from the PATH, and common Java locations
#[cfg(target_os = "macos")]
#[tracing::instrument]
pub async fn get_all_jre() -> Result<Vec<JavaVersion>, JREError> {
    // Use HashSet to avoid duplicates
    let mut jre_paths = HashSet::new();

    // Add JREs directly on PATH
    jre_paths.extend(get_all_jre_path().await);
    jre_paths.extend(get_all_autoinstalled_jre_path().await?);

    // Hard paths for locations for commonly installed .exes
    let java_paths = [
        r"/Applications/Xcode.app/Contents/Applications/Application Loader.app/Contents/MacOS/itms/java",
        r"/Library/Internet Plug-Ins/JavaAppletPlugin.plugin/Contents/Home",
        r"/System/Library/Frameworks/JavaVM.framework/Versions/Current/Commands",
    ];
    for path in java_paths {
        jre_paths.insert(PathBuf::from(path));
    }
    // Iterate over JavaVirtualMachines/(something)/Contents/Home/bin
    let base_path = PathBuf::from("/Library/Java/JavaVirtualMachines/");
    if let Ok(dir) = std::fs::read_dir(base_path) {
        for entry in dir.flatten() {
            let entry = entry.path().join("Contents/Home/bin");
            jre_paths.insert(entry);
        }
    }

    // Get JRE versions from potential paths concurrently
    let j = check_java_at_filepaths(jre_paths)
        .await
        .into_iter()
        .collect();
    Ok(j)
}

// Entrypoint function (Linux)
// Returns a Vec of unique JavaVersions from the PATH, and common Java locations
#[cfg(target_os = "linux")]
#[tracing::instrument]
pub async fn get_all_jre() -> Result<Vec<JavaVersion>, JREError> {
    // Use HashSet to avoid duplicates
    let mut jre_paths = HashSet::new();

    // Add JREs directly on PATH
    jre_paths.extend(get_all_jre_path().await);
    jre_paths.extend(get_all_autoinstalled_jre_path().await?);

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
        let path = PathBuf::from(path);
        jre_paths.insert(PathBuf::from(&path).join("jre").join("bin"));
        jre_paths.insert(PathBuf::from(&path).join("bin"));
        if let Ok(dir) = std::fs::read_dir(path) {
            for entry in dir.flatten() {
                let entry_path = entry.path();
                jre_paths.insert(entry_path.join("jre").join("bin"));
                jre_paths.insert(entry_path.join("bin"));
            }
        }
    }

    // Get JRE versions from potential paths concurrently
    let j = check_java_at_filepaths(jre_paths)
        .await
        .into_iter()
        .collect();
    Ok(j)
}

// Gets all JREs from the PATH env variable
#[tracing::instrument]
async fn get_all_autoinstalled_jre_path() -> Result<HashSet<PathBuf>, JREError>
{
    Box::pin(async move {
        let state = State::get().await.map_err(|_| JREError::StateError)?;

        let mut jre_paths = HashSet::new();
        let base_path = state.directories.java_versions_dir();

        if base_path.is_dir() {
            if let Ok(dir) = std::fs::read_dir(base_path) {
                for entry in dir.flatten() {
                    let file_path = entry.path().join("bin");

                    if let Ok(contents) =
                        std::fs::read_to_string(file_path.clone())
                    {
                        let entry = entry.path().join(contents);
                        jre_paths.insert(entry);
                    } else {
                        #[cfg(not(target_os = "macos"))]
                        {
                            let file_path = file_path.join(JAVA_BIN);
                            jre_paths.insert(file_path);
                        }
                    }
                }
            }
        }

        Ok(jre_paths)
    })
    .await
}

// Gets all JREs from the PATH env variable
#[tracing::instrument]
async fn get_all_jre_path() -> HashSet<PathBuf> {
    // Iterate over values in PATH variable, where accessible JREs are referenced
    let paths =
        env::var("PATH").map(|x| env::split_paths(&x).collect::<HashSet<_>>());
    paths.unwrap_or_else(|_| HashSet::new())
}

pub const JAVA_BIN: &str = if cfg!(target_os = "windows") {
    "javaw.exe"
} else {
    "java"
};

// For each example filepath in 'paths', perform check_java_at_filepath, checking each one concurrently
// and returning a JavaVersion for every valid path that points to a java bin
#[tracing::instrument]
pub async fn check_java_at_filepaths(
    paths: HashSet<PathBuf>,
) -> HashSet<JavaVersion> {
    stream::iter(paths.into_iter())
        .map(|p: PathBuf| {
            tokio::task::spawn(async move { check_java_at_filepath(&p).await })
        })
        .buffer_unordered(64)
        .filter_map(async |x| x.ok().and_then(Result::ok))
        .collect()
        .await
}

// For example filepath 'path', attempt to resolve it and get a Java version at this path
// If no such path exists, or no such valid java at this path exists, returns None
#[tracing::instrument]
pub async fn check_java_at_filepath(path: &Path) -> crate::Result<JavaVersion> {
    // Attempt to canonicalize the potential java filepath
    // If it fails, this path does not exist and None is returned (no Java here)
    let path = io::canonicalize(path)?;

    // Checks for existence of Java at this filepath
    // Adds JAVA_BIN to the end of the path if it is not already there
    let java = if path
        .file_name()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x != JAVA_BIN)
    {
        path.join(JAVA_BIN)
    } else {
        path
    };

    if !java.exists() {
        return Err(JREError::NoExecutable(java).into());
    };

    let (_temp, file_path) =
        get_resource_file!(env "JAVA_JARS_DIR" / "theseus.jar")?;

    let output = Command::new(&java)
        .arg("-cp")
        .arg(file_path)
        .arg("com.modrinth.theseus.JavaInfo")
        .env_remove("_JAVA_OPTIONS")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut java_version = None;
    let mut java_arch = None;

    for line in stdout.lines() {
        let mut parts = line.split('=');
        let key = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default();

        if key == "os.arch" {
            java_arch = Some(value);
        } else if key == "java.version" {
            java_version = Some(value);
        }
    }

    // Extract version info from it
    if let Some(arch) = java_arch {
        if let Some(version) = java_version {
            if let Ok(version) = extract_java_version(version) {
                let path = java.to_string_lossy().to_string();
                return Ok(JavaVersion {
                    parsed_version: version,
                    path,
                    version: version.to_string(),
                    architecture: arch.to_string(),
                });
            }

            return Err(JREError::InvalidJREVersion(version.to_owned()).into());
        }
    }

    Err(JREError::FailedJavaCheck(java).into())
}

pub fn extract_java_version(version: &str) -> Result<u32, JREError> {
    let mut split = version.split('.');

    let version = split.next().unwrap();
    let version = version.split_once('-').map_or(version, |(x, _)| x);
    let mut version = version.parse::<u32>()?;
    if version == 1 {
        version = split.next().map_or(Ok(1), |x| x.parse::<u32>())?;
    }

    Ok(version)
}

#[derive(thiserror::Error, Debug)]
pub enum JREError {
    #[error("Command error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Env error: {0}")]
    EnvError(#[from] env::VarError),

    #[error("No executable found at {0}")]
    NoExecutable(PathBuf),

    #[error("Could not check Java version at path {0}")]
    FailedJavaCheck(PathBuf),

    #[error("Invalid JRE version string: {0}")]
    InvalidJREVersion(String),

    #[error("Parsing error: {0}")]
    ParseError(#[from] std::num::ParseIntError),

    #[error("Join error: {0}")]
    JoinError(#[from] JoinError),

    #[error("No stored tag for Minecraft version {0}")]
    NoMinecraftVersionFound(String),

    #[error("Error getting launcher state")]
    StateError,
}
