use std::fs::{self, File};
use std::path::Path;
use std::io::BufReader;
use std::time::SystemTime;
use zip::ZipArchive;

use crate::model::mod_type::JARLoadedMod;
use crate::util::parser::json_to_map;
use crate::util::fetch::sha1_async;

/// Extract meta-data from JAR file.
#[tracing::instrument]
pub async fn extract_info_from_jar(file_path: String) -> crate::Result<JARLoadedMod> {
    let path = Path::new(&file_path);

    let jar_buf = std::fs::read(path)
        .unwrap();
    let jar_bytes = bytes::Bytes::from(jar_buf);
    let jar_hash = sha1_async(jar_bytes).await;

    let json_entries = list_json_files_in_jar(path).expect("Failed to extract JAR");
    let metadata_file = &json_entries[0];
    let json_map = json_to_map(&metadata_file).expect("Failed to parse JSON");

    let mod_name = json_map.get(&"name".to_string()).map_or("Unknown", String::as_str); 
    let mod_description = json_map.get(&"description".to_string()).map_or("", String::as_str); 
    
    Ok(JARLoadedMod {
        file_hash: jar_hash,
        absolute_path: file_path,
        //date_added: &get_file_datetime(path),
        mod_name: mod_name.to_string(),
        description: mod_description.to_string(),
    })
}

fn get_file_datetime(file_path: &Path) -> Result<SystemTime, String> {
    let metadata = fs::metadata(file_path).map_err(|err| err.to_string())?;
    let modified_time = metadata.modified().map_err(|err| err.to_string())?;
    Ok(modified_time)
}

fn list_json_files_in_jar(jar_path: &Path) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(jar_path)?;
    let reader = BufReader::new(file);
    let mut zip = ZipArchive::new(reader)?;

    let mut json_files = Vec::new();
    for i in 0..zip.len() {
        let file = zip.by_index(i)?;
        let filename = String::from(file.name());
        if filename.ends_with(".json") {
            json_files.push(filename);
        }
    }
    Ok(json_files)
}