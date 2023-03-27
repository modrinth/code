use std::env;
use std::process::Command;


#[derive(Debug)]
pub struct JavaVersion {
    path: String,
    version: String,
}
impl JavaVersion {
    pub fn new(path : String, version : String) -> Self {
        JavaVersion { path, version }
    }
}

#[cfg(target_os = "windows")]
pub fn get_all_jre() -> Result<Vec<JavaVersion>,JREError> {
    let jre_key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\JavaSoft\Java Runtime Environment")
        .expect("Failed to open registry key");


    for subkey in jre_key.enum_keys().expect("Failed to enumerate subkeys") {
        let subkey = subkey.expect("Failed to read subkey name");
        let subkey = jre_key.open_subkey(subkey).expect("Failed to open subkey");

        let version = subkey.get_value("JavaVersion")
            .expect("Failed to get JavaVersion value")
            .as_string()
            .expect("JavaVersion is not a string");

        let java_home = subkey.get_value("JavaHome")
            .expect("Failed to get JavaHome value")
            .as_string()
            .expect("JavaHome is not a string");

        println!("JRE version: {}", version);
        println!("Java home: {}", java_home);
    }
}


#[cfg(not(target_os = "windows"))]
pub fn get_all_jre() -> Result<Vec<JavaVersion>,JREError>  {

    // Iterate over values in PATH variable, where accessible JREs are referenced
    let paths = env::var("PATH")?;
    let paths = env::split_paths(&paths);

    let mut jres = Vec::new();
    for path in paths {
        // Check if the path is valid and has a /java bin
        let Some(path_str) = path.to_str() else { continue };
        let java = path.join("java");
        if !java.exists() { 
            continue;
        };

        // Run 'java -version' using found java binary
        let output = Command::new(&java).arg("-version").output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Extract version info from it
        if stderr.contains("java version") {
            let version_line = stderr.lines().next().ok_or_else(|| JREError::JREFormat)?;
            let version = version_line.split("\"").nth(1).ok_or_else(|| JREError::JREFormat)?.to_string();
            let path = path_str.to_string();
            jres.push(JavaVersion {
                path,
                version
            });
        }
    }

    Ok(jres)
}


#[derive(thiserror::Error, Debug)]
pub enum JREError {

    #[error("Command : {0}")]
    IOError(#[from] std::io::Error),

    #[error("Env error: {0}")]
    EnvError(#[from] env::VarError),

    #[error("'java -version' returned improper result")]
    JREFormat
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