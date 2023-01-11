use std::error::Error;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::io::BufReader;
use std::time::SystemTime;
use serde_json::Value;
use zip::ZipArchive;
use zip::read::ZipFile;

use crate::model::mod_type::{JARLoadedMod, ModConfig};
use crate::util::fetch::sha1_async;

/// Extract meta-data from JAR file.
#[tracing::instrument]
pub async fn extract_info_from_jar(file_path: String) -> crate::Result<JARLoadedMod> {
    let path = Path::new(&file_path);

    //let path_str = path.as_os_str().to_str();
    let jar_buf = std::fs::read(path)?;
    let jar_bytes = bytes::Bytes::from(jar_buf);
    let jar_hash = sha1_async(jar_bytes).await;
    let timestamp_added = get_file_datetime(path);

    let mod_config = read_config_from_jar(path).expect("Failed to extract JSON from JAR");
    
    Ok(JARLoadedMod {
        file_hash: jar_hash,
        absolute_path: file_path,
        timestamp_added: timestamp_added,
        mod_config: mod_config,
    })
}

// Get timestamp of the file being modified.
fn get_file_datetime(file_path: &Path) -> Option<SystemTime> {
    fs::metadata(file_path).ok().map(|metadata| metadata.modified())?.ok()
}

fn read_config_from_jar(jar_file: &Path) -> Result<ModConfig, String> {
    let file = File::open(jar_file).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(BufReader::new(file)).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let zip_file = archive.by_index(i).map_err(|e| e.to_string())?;
        let mod_metadata = get_mod_metadata_from_file(zip_file);
        if mod_metadata.is_ok() {
            if let Some(config) = mod_metadata.unwrap() {
                return Ok(config);
            }
        } else {
            continue;
        }
    }
    return Err("No config files found!".to_string());
}


fn get_mod_metadata_from_file(file: ZipFile) -> crate::Result<Option<ModConfig>> {
    let path = Path::new(file.name());
    if let Some(ext) = Path::extension(&path).and_then(|s| s.to_str()) {
        match ext {
            "json" => {
                let content: Value = serde_json::from_reader(file)?;
                let mod_config: ModConfig = serde_json::from_value(content)?;
                return Ok(Some(mod_config));
            },
            "toml" => {
                let data = std::fs::read_to_string(path)?;
                let toml_value: toml::Value = toml::from_str(&data)?;
                let json_value = serde_json::to_value(toml_value)?;
                let mod_config: ModConfig = serde_json::from_value(json_value)?;
                return Ok(Some(mod_config));
             },
            &_ => {
                return Ok(None);
            }
        }
    }
    return Ok(None);
}

#[cfg(test)]
mod tests {
    use crate::Error;

    use super::*;
    use pretty_assertions::assert_str_eq;

    //#[tokio::test]
    async fn mod_extraction_test() -> Result<(), Error> {
        // todo: rewrite above methods to be mock-friendly by passing file as param
        let fabric_mod_path = String::from("/PATH/TO/FABRIC.jar");

        let jar_loaded_mod = extract_info_from_jar(fabric_mod_path).await?;
        assert_str_eq!(jar_loaded_mod.mod_config.name, "Fabric API");
        Ok(())
    }
}