use crate::{download_file, Error};

use std::collections::HashMap;

/// The latest version of the format the model structs deserialize to
pub const CURRENT_FORMAT_VERSION: usize = 0;

const DEFAULT_MAVEN_METADATA_URL: &str =
    "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";

/// Fetches the forge maven metadata from the specified URL. If no URL is specified, the default is used.
/// Returns a hashmap specifying the versions of the forge mod loader
/// The hashmap key is a Minecraft version, and the value is the loader versions that work on
/// the specified Minecraft version
pub async fn fetch_maven_metadata(
    url: Option<&str>,
) -> Result<HashMap<String, Vec<String>>, Error> {
    Ok(serde_json::from_slice(
        &download_file(url.unwrap_or(DEFAULT_MAVEN_METADATA_URL), None).await?,
    )?)
}
