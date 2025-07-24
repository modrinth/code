use std::io::Cursor;

use async_zip::base::read::seek::ZipFileReader;
use serde::{Deserialize, Serialize};

use crate::{
    State,
    event::{LoadingBarType, ProfilePayloadType},
    prelude::ModLoader,
    state::{LinkedData, ProfileInstallStage},
    util::fetch::fetch,
};

use super::copy_dotminecraft;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeManifest {
    pub minecraft: CurseForgeMinecraft,
    pub manifest_type: String,
    pub manifest_version: i32,
    pub name: String,
    pub version: String,
    pub author: String,
    pub files: Vec<CurseForgeFile>,
    pub overrides: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeMinecraft {
    pub version: String,
    pub mod_loaders: Vec<CurseForgeModLoader>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeModLoader {
    pub id: String,
    pub primary: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurseForgeFile {
    #[serde(rename = "projectID")]
    pub project_id: u32,
    #[serde(rename = "fileID")]
    pub file_id: u32,
    pub required: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurseForgeProfileMetadata {
    pub name: String,
    pub download_url: String,
}

pub async fn fetch_curseforge_profile_metadata(
    profile_code: &str,
) -> crate::Result<CurseForgeProfileMetadata> {
    let state = State::get().await?;

    let url = format!(
        "https://api.curseforge.com/v1/shared-profile/{}",
        profile_code
    );

    let response = fetch(&url, None, &state.fetch_semaphore, &state.pool).await;

    let download_url = match response {
        Ok(_bytes) => {
            url
        }
        Err(e) => {
            let error_msg = format!("{:?}", e);
            if let Some(redirect_start) =
                error_msg.find("https://shared-profile-media.forgecdn.net/")
            {
                let redirect_end = error_msg[redirect_start..]
                    .find(' ')
                    .unwrap_or(error_msg.len() - redirect_start);
                error_msg[redirect_start..redirect_start + redirect_end]
                    .to_string()
            } else {
                return Err(crate::ErrorKind::InputError(format!(
                    "Failed to fetch CurseForge profile metadata: {}",
                    e
                ))
                .into());
            }
        }
    };

    let zip_bytes =
        fetch(&download_url, None, &state.fetch_semaphore, &state.pool).await?;

    let cursor = std::io::Cursor::new(zip_bytes);
    let mut zip_reader =
        ZipFileReader::with_tokio(cursor).await.map_err(|e| {
            crate::ErrorKind::InputError(format!(
                "Failed to read profile ZIP: {}",
                e
            ))
        })?;

    let manifest_index = zip_reader
        .file()
        .entries()
        .iter()
        .position(|f| {
            f.filename().as_str().unwrap_or_default() == "manifest.json"
        })
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "No manifest.json found in profile".to_string(),
            )
        })?;

    let mut manifest_content = String::new();
    let mut reader = zip_reader
        .reader_with_entry(manifest_index)
        .await
        .map_err(|e| {
            crate::ErrorKind::InputError(format!(
                "Failed to read manifest.json: {}",
                e
            ))
        })?;

    reader.read_to_string_checked(&mut manifest_content).await?;

    let manifest: CurseForgeManifest = serde_json::from_str(&manifest_content)?;

    let profile_name = if manifest.name.is_empty() {
        format!("CurseForge Profile {}", profile_code)
    } else {
        manifest.name.clone()
    };

    Ok(CurseForgeProfileMetadata {
        name: profile_name,
        download_url,
    })
}

pub async fn import_curseforge_profile(
    profile_code: &str,
    profile_path: &str,
) -> crate::Result<()> {
    let state = State::get().await?;

    // Initialize loading bar
    let loading_bar = crate::event::emit::init_loading(
        LoadingBarType::CurseForgeProfileDownload {
            profile_name: profile_path.to_string(),
        },
        100.0,
        "Importing CurseForge profile...",
    )
    .await?;

    crate::event::emit::emit_loading(
        &loading_bar,
        10.0,
        Some("Fetching profile metadata..."),
    )?;
    let metadata = fetch_curseforge_profile_metadata(profile_code).await?;

    crate::event::emit::emit_loading(
        &loading_bar,
        5.0,
        Some("Downloading profile ZIP..."),
    )?;
    let zip_bytes = fetch(
        &metadata.download_url,
        None,
        &state.fetch_semaphore,
        &state.pool,
    )
    .await?;

    crate::event::emit::emit_loading(
        &loading_bar,
        5.0,
        Some("Extracting ZIP contents..."),
    )?;
    let cursor = Cursor::new(zip_bytes);
    let mut zip_reader =
        ZipFileReader::with_tokio(cursor).await.map_err(|e| {
            crate::ErrorKind::InputError(format!(
                "Failed to read profile ZIP: {}",
                e
            ))
        })?;

    let manifest_index = zip_reader
        .file()
        .entries()
        .iter()
        .position(|f| {
            f.filename().as_str().unwrap_or_default() == "manifest.json"
        })
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "No manifest.json found in profile".to_string(),
            )
        })?;

    let mut manifest_content = String::new();
    let mut reader = zip_reader
        .reader_with_entry(manifest_index)
        .await
        .map_err(|e| {
            crate::ErrorKind::InputError(format!(
                "Failed to read manifest.json: {}",
                e
            ))
        })?;

    reader.read_to_string_checked(&mut manifest_content).await?;

    crate::event::emit::emit_loading(
        &loading_bar,
        5.0,
        Some("Parsing profile manifest..."),
    )?;
    let manifest: CurseForgeManifest = serde_json::from_str(&manifest_content)?;

    crate::event::emit::emit_loading(
        &loading_bar,
        5.0,
        Some("Configuring profile..."),
    )?;
    let (mod_loader, loader_version) = if let Some(primary_loader) =
        manifest.minecraft.mod_loaders.iter().find(|l| l.primary)
    {
        parse_modloader(&primary_loader.id)
    } else if let Some(first_loader) = manifest.minecraft.mod_loaders.first() {
        parse_modloader(&first_loader.id)
    } else {
        (ModLoader::Vanilla, None)
    };

    let game_version = manifest.minecraft.version.clone();

    let final_loader_version = if mod_loader != ModLoader::Vanilla {
        crate::launcher::get_loader_version_from_profile(
            &game_version,
            mod_loader,
            loader_version.as_deref(),
        )
        .await?
    } else {
        None
    };

    crate::api::profile::edit(profile_path, |prof| {
        prof.name = if manifest.name.is_empty() {
            format!("CurseForge Profile {}", profile_code)
        } else {
            manifest.name.clone()
        };
        prof.install_stage = ProfileInstallStage::PackInstalling;
        prof.game_version = game_version.clone();
        prof.loader_version = final_loader_version.clone().map(|x| x.id);
        prof.loader = mod_loader;

        prof.linked_data = Some(LinkedData {
            project_id: String::new(),
            version_id: String::new(),
            locked: false,
        });

        async { Ok(()) }
    })
    .await?;

    let temp_dir = state
        .directories
        .caches_dir()
        .join(format!("curseforge_profile_{}", profile_code));
    tokio::fs::create_dir_all(&temp_dir).await?;

    crate::event::emit::emit_loading(
        &loading_bar,
        10.0,
        Some("Extracting profile files..."),
    )?;
    let overrides_dir = temp_dir.join(&manifest.overrides);
    tokio::fs::create_dir_all(&overrides_dir).await?;

    // Extract all files that are in the overrides directory
    // First collect the entries we need to extract to avoid borrowing conflicts
    let entries_to_extract: Vec<(usize, String)> = {
        let zip_file = zip_reader.file();
        zip_file
            .entries()
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let file_path = entry.filename().as_str().unwrap_or_default();
                if file_path.starts_with(&format!("{}/", manifest.overrides)) {
                    Some((index, file_path.to_string()))
                } else {
                    None
                }
            })
            .collect()
    };

    for (index, file_path) in entries_to_extract {
        let relative_path = file_path
            .strip_prefix(&format!("{}/", manifest.overrides))
            .unwrap();
        let output_path = overrides_dir.join(relative_path);

        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut reader =
            zip_reader.reader_with_entry(index).await.map_err(|e| {
                crate::ErrorKind::InputError(format!(
                    "Failed to read file {}: {}",
                    file_path, e
                ))
            })?;

        let mut file_content = Vec::new();
        reader.read_to_end_checked(&mut file_content).await?;

        tokio::fs::write(&output_path, file_content).await?;
    }

    crate::event::emit::emit_loading(
        &loading_bar,
        5.0,
        Some("Copying profile files..."),
    )?;
    let _loading_bar = copy_dotminecraft(
        profile_path,
        overrides_dir,
        &state.io_semaphore,
        None,
    )
    .await?;

    crate::event::emit::emit_loading(
        &loading_bar,
        10.0,
        Some("Downloading mods..."),
    )?;
    install_curseforge_mods(
        &manifest.files,
        profile_path,
        &state,
        &loading_bar,
    )
    .await?;

    // Clean up temporary directory
    tokio::fs::remove_dir_all(&temp_dir).await.ok();

    crate::event::emit::emit_loading(
        &loading_bar,
        20.0,
        Some("Installing Minecraft..."),
    )?;
    if let Some(profile_val) = crate::api::profile::get(profile_path).await? {
        crate::launcher::install_minecraft(
            &profile_val,
            Some(_loading_bar),
            false,
        )
        .await?;
    }

    crate::event::emit::emit_loading(
        &loading_bar,
        20.0,
        Some("Finalizing profile..."),
    )?;
    crate::api::profile::edit(profile_path, |prof| {
        prof.install_stage = ProfileInstallStage::Installed;
        async { Ok(()) }
    })
    .await?;

    crate::event::emit::emit_profile(profile_path, ProfilePayloadType::Synced)
        .await?;

    // Complete the loading bar
    crate::event::emit::emit_loading(
        &loading_bar,
        5.0,
        Some("Import completed!"),
    )?;

    Ok(())
}

fn parse_modloader(id: &str) -> (ModLoader, Option<String>) {
    if id.starts_with("forge-") {
        let version = id.strip_prefix("forge-").unwrap_or("").to_string();
        (ModLoader::Forge, Some(version))
    } else if id.starts_with("fabric-") {
        let version = id.strip_prefix("fabric-").unwrap_or("").to_string();
        (ModLoader::Fabric, Some(version))
    } else if id.starts_with("quilt-") {
        let version = id.strip_prefix("quilt-").unwrap_or("").to_string();
        (ModLoader::Quilt, Some(version))
    } else if id.starts_with("neoforge-") {
        let version = id.strip_prefix("neoforge-").unwrap_or("").to_string();
        (ModLoader::NeoForge, Some(version))
    } else {
        (ModLoader::Vanilla, None)
    }
}

async fn install_curseforge_mods(
    files: &[CurseForgeFile],
    profile_path: &str,
    state: &State,
    loading_bar: &crate::event::LoadingBarId,
) -> crate::Result<()> {
    if files.is_empty() {
        return Ok(());
    }

    let num_files = files.len();
    tracing::info!("Installing {} CurseForge mods", num_files);

    // Download mods sequentially to track progress properly
    for (index, file) in files.iter().enumerate() {
        let progress_message =
            format!("Downloading mod {} of {}", index + 1, num_files);
        crate::event::emit::emit_loading(
            loading_bar,
            0.0, // Don't increment here, just update message
            Some(&progress_message),
        )?;

        download_curseforge_mod(file, profile_path, state).await?;

        // Emit progress for each downloaded mod (20% total for mods, divided by number of mods)
        let mod_progress = 20.0 / num_files as f64;
        crate::event::emit::emit_loading(
            loading_bar,
            mod_progress,
            Some(&format!("Downloaded mod {} of {}", index + 1, num_files)),
        )?;
    }

    Ok(())
}

async fn download_curseforge_mod(
    file: &CurseForgeFile,
    profile_path: &str,
    _state: &State,
) -> crate::Result<()> {
    tracing::info!(
        "Downloading CurseForge mod: project_id={}, file_id={}",
        file.project_id,
        file.file_id
    );

    let profile_full_path =
        crate::api::profile::get_full_path(profile_path).await?;
    let mods_dir = profile_full_path.join("mods");
    tokio::fs::create_dir_all(&mods_dir).await?;

    let metadata_url = format!(
        "https://www.curseforge.com/api/v1/mods/{}/files/{}",
        file.project_id, file.file_id
    );

    tracing::info!("Fetching metadata from: {}", metadata_url);

    let client = reqwest::Client::new();
    let metadata_response =
        client.get(&metadata_url).send().await.map_err(|e| {
            crate::ErrorKind::InputError(format!(
                "Failed to fetch metadata for mod {}/{}: {}",
                file.project_id, file.file_id, e
            ))
        })?;

    if !metadata_response.status().is_success() {
        return Err(crate::ErrorKind::InputError(format!(
            "HTTP error fetching metadata for mod {}/{}: {}",
            file.project_id,
            file.file_id,
            metadata_response.status()
        ))
        .into());
    }

    let metadata_json: serde_json::Value =
        metadata_response.json().await.map_err(|e| {
            crate::ErrorKind::InputError(format!(
                "Failed to parse metadata JSON for mod {}/{}: {}",
                file.project_id, file.file_id, e
            ))
        })?;

    let original_filename = metadata_json
        .get("data")
        .and_then(|data| data.get("fileName"))
        .and_then(|name| name.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Fallback to the old format if API response is unexpected
            format!("mod_{}_{}.jar", file.project_id, file.file_id)
        });

    tracing::info!("Original filename: {}", original_filename);

    let download_url = format!(
        "https://www.curseforge.com/api/v1/mods/{}/files/{}/download",
        file.project_id, file.file_id
    );

    tracing::info!("Downloading from: {}", download_url);

    let response = client.get(&download_url).send().await.map_err(|e| {
        crate::ErrorKind::InputError(format!(
            "Failed to download mod {}/{}: {}",
            file.project_id, file.file_id, e
        ))
    })?;

    if !response.status().is_success() {
        return Err(crate::ErrorKind::InputError(format!(
            "HTTP error downloading mod {}/{}: {}",
            file.project_id,
            file.file_id,
            response.status()
        ))
        .into());
    }

    let final_path = mods_dir.join(&original_filename);
    let bytes = response.bytes().await.map_err(|e| {
        crate::ErrorKind::InputError(format!(
            "Failed to read response bytes for mod {}/{}: {}",
            file.project_id, file.file_id, e
        ))
    })?;

    tokio::fs::write(&final_path, &bytes).await.map_err(|e| {
        crate::ErrorKind::InputError(format!(
            "Failed to write mod file {:?}: {}",
            final_path, e
        ))
    })?;

    tracing::info!(
        "Successfully downloaded mod: {} ({} bytes)",
        original_filename,
        bytes.len()
    );

    Ok(())
}
