use std::collections::HashMap;
use std::fs;

pub fn json_to_map(json_file: &str) -> Result<HashMap<String, String>, String> {
    let json_string = fs::read_to_string(json_file).map_err(|err| err.to_string())?;
    let json_value: serde_json::Value = serde_json::from_str(&json_string)
        .map_err(|err| err.to_string())?;
    let mut map: HashMap<String, String> = HashMap::new();
    for (key, value) in json_value.as_object().unwrap().iter() {
        map.insert(key.to_owned(), value.to_string());
    }
    Ok(map)
}