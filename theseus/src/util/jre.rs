use std::env;
use std::path::PathBuf;
use regex::Regex;
use lazy_static::lazy_static;
use winreg::RegKey;
use std::process::Command;
use std::collections::HashSet;

#[derive(Debug, PartialEq ,Eq, Hash)]
pub struct JavaVersion {
    path: String,
    version: String,
}

// Entrypoint function (Windows)
// Returns a Vec of unique JavaVersions from the PATH, Windows Registry Keys and common Java locations
#[cfg(target_os = "windows")]
#[tracing::instrument]
pub fn get_all_jre  () -> Result<Vec<JavaVersion>,JREError> {
    use winreg::{RegKey, enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY}};

    // Use HashSet to avoid duplicates
    let mut jres = HashSet::new();

    // Add JRES directly on PATH
    jres.extend(get_all_jre_path()?);

    // Hard paths for locations for commonly installed .exes
    let java_paths = [
        r"C:/Program Files/Java/jre7",
        r"C:/Program Files/Java/jre8",
        r"C:/Program Files (x86)/Java/jre7",
        r"C:/Program Files (x86)/Java/jre8"
    ];
    for path in java_paths {
        if let Some(j) = java_filepath_to_javaversion(PathBuf::from(path).join("bin")) {
            jres.insert(j);
            break;
        }
    }

    // Windows Registry Keys
    let key_paths = [
        r"SOFTWARE\JavaSoft\Java Runtime Environment", // Oracle
        r"SOFTWARE\JavaSoft\Java Development Kit",
        r"SOFTWARE\\JavaSoft\\JRE", // Oracle
        r"SOFTWARE\\JavaSoft\\JDK",
        r"SOFTWARE\\Eclipse Foundation\\JDK", // Eclipse
        r"SOFTWARE\\Eclipse Adoptium\\JRE", // Eclipse
        r"SOFTWARE\\Eclipse Foundation\\JDK", // Eclipse
        r"SOFTWARE\\Microsoft\\JDK", // Microsoft
    ];
    for key in key_paths {
        if let Ok(jre_key) =  RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey_with_flags(key, KEY_READ | KEY_WOW64_32KEY) {
            jres.extend(get_all_jre_winregkey(jre_key)?);
        }
        if let Ok(jre_key) =  RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey_with_flags(key, KEY_READ | KEY_WOW64_64KEY) {
            jres.extend(get_all_jre_winregkey(jre_key)?);
        }
    }

    Ok(jres.into_iter().collect())
}

#[cfg(target_os = "windows")]
#[tracing::instrument]
pub fn get_all_jre_winregkey(jre_key : RegKey) -> Result<HashSet<JavaVersion>,JREError> {

    let mut jres = HashSet::new();

    for subkey in jre_key.enum_keys() {
        let subkey = subkey?;
        let subkey = jre_key.open_subkey(subkey)?;

        let subkey_value_names = [
            r"JavaHome",
            r"InstallationPath",
            r"\\hotspot\\MSI",
        ];
    
        for subkey_value in subkey_value_names {
            let path : Result<String, std::io::Error> = subkey.get_value(subkey_value);
            let Ok(path) = path else {continue};
            if let Some(j) = java_filepath_to_javaversion(PathBuf::from(path).join("bin")) {
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
pub fn get_all_jre() -> Result<Vec<JavaVersion>,JREError> {
    // Use HashSet to avoid duplicates
    let mut jres = HashSet::new();

    // Add JREs directly on PATH
    jres.extend(get_all_jre_path()?);

    // Hard paths for locations for commonly installed .exes
    let java_paths = [
        r"/Applications/Xcode.app/Contents/Applications/Application Loader.app/Contents/MacOS/itms/java",
        r"/Library/Internet Plug-Ins/JavaAppletPlugin.plugin/Contents/Home",
        r"/System/Library/Frameworks/JavaVM.framework/Versions/Current/Commands",
        r"C:/Program Files (x86)/Java/jre8"
    ];
    for path in java_paths {
        if let Some(j) = java_filepath_to_javaversion(PathBuf::from(path).join("bin")) {
            jres.push(j);
            break;
        }
    }

    jres.extend(get_all_jre_search()?);
    Ok(jres.into_iter().collect())
}

// Entrypoint function (Linux)
// Returns a Vec of unique JavaVersions from the PATH, and common Java locations
#[cfg(target_os = "linux")]
#[tracing::instrument]
pub fn get_all_jre() -> Result<Vec<JavaVersion>,JREError> {
    // Use HashSet to avoid duplicates
    let mut jres = HashSet::new();

    // Add JREs directly on PATH
    jres.extend(get_all_jre_path()?);

    // Hard paths for locations for commonly installed locations
    let java_paths = [
        r"/usr/java",
        r"/usr/lib/jvm",
        r"/usr/lib64/jvm",
        r"/opt/jdk",
        r"/opt/jdks"
    ];
    for path in java_paths {
        if let Some(j) = java_filepath_to_javaversion(PathBuf::from(path).join("jre/bin")) {
            jres.push(j);
            break;
        }
        if let Some(j) = java_filepath_to_javaversion(PathBuf::from(path).join("bin")) {
            jres.push(j);
            break;
        }
    }
    Ok(jres.into_iter().collect())
}


#[tracing::instrument]
pub fn get_all_jre_path() -> Result<HashSet<JavaVersion>,JREError> {
    // Iterate over values in PATH variable, where accessible JREs are referenced
    let paths = env::var("PATH")?;
    let paths = env::split_paths(&paths);

    let mut jres = HashSet::new();
    for path in paths {
        if let Some(j) = java_filepath_to_javaversion(path) {
            jres.insert(j);
        }
    }
    Ok(jres)
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
const JAVA_BIN : &'static str = "java.exe";

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
const JAVA_BIN : &'static str = "java";

#[tracing::instrument]
pub fn java_filepath_to_javaversion(path : PathBuf) -> Option<JavaVersion> {
    let Some(path_str) = path.to_str() else { return None };
    let java = path.join(JAVA_BIN);
    if !java.exists() { 
        return None;
    };

    // Run 'java -version' using found java binary
    let output = Command::new(&java).arg("-version").output().ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Match: java version "1.8.0_361"
    // Extracting version numbers
    lazy_static! {
        static ref JAVA_VERSION_CAPTURE : Regex = Regex::new(r#"^java version "([\d\._]+)""#).unwrap();
    }

    // Extract version info from it
    if let Some(captures) = JAVA_VERSION_CAPTURE.captures(&stderr) {
        if let Some(version) = captures.get(1) {
            let path = path_str.to_string();
            return Some(JavaVersion {
                path,
                version: version.as_str().to_string()
            });        
        }
    }
    None
}

#[derive(thiserror::Error, Debug)]
pub enum JREError {
    #[error("Command error : {0}")]
    IOError(#[from] std::io::Error),

    #[error("Env error: {0}")]
    EnvError(#[from] env::VarError),
}


#[cfg(test)]
mod tests {
    use super::get_all_jre;

    #[test]
    fn find_jre() {
        let jres = get_all_jre().unwrap();
        dbg!(&jres);
        panic!("fail");
    }
}