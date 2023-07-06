use crate::config::MODRINTH_API_URL;
use crate::data::ModLoader;
use crate::event::emit::{emit_loading, init_loading};
use crate::event::{LoadingBarId, LoadingBarType};
use crate::state::{LinkedData, ModrinthProject, ModrinthVersion, SideType};
use crate::util::fetch::{
    fetch, fetch_advanced, fetch_json, write_cached_icon,
};
use crate::State;

use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::path::PathBuf;
use tokio::fs;

#[derive(Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PackFormat {
    pub game: String,
    pub format_version: i32,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<PackFile>,
    pub dependencies: HashMap<PackDependency, String>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PackFile {
    pub path: String,
    pub hashes: HashMap<PackFileHash, String>,
    pub env: Option<HashMap<EnvType, SideType>>,
    pub downloads: Vec<String>,
    pub file_size: u32,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase", from = "String")]
pub enum PackFileHash {
    Sha1,
    Sha512,
    Unknown(String),
}

impl From<String> for PackFileHash {
    fn from(s: String) -> Self {
        return match s.as_str() {
            "sha1" => PackFileHash::Sha1,
            "sha512" => PackFileHash::Sha512,
            _ => PackFileHash::Unknown(s),
        };
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum EnvType {
    Client,
    Server,
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PackDependency {
    Forge,
    FabricLoader,
    QuiltLoader,
    Minecraft,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CreatePackLocation {
    FromVersionId {
        project_id: String,
        version_id: String,
        title: String,
        icon_url: Option<String>,
    },
    FromFile {
        path: PathBuf,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePackProfile {
    pub name: String, // the name of the profile, and relative path
    pub game_version: String, // the game version of the profile
    pub modloader: ModLoader, // the modloader to use
    pub loader_version: Option<String>, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader. defaults to latest
    pub icon: Option<PathBuf>,          // the icon for the profile
    pub icon_url: Option<String>, // the URL icon for a profile (ONLY USED FOR TEMPORARY PROFILES)
    pub linked_data: Option<LinkedData>, // the linked project ID (mainly for modpacks)- used for updating
    pub skip_install_profile: Option<bool>,
}

pub struct CreatePackDescription {
    pub file: bytes::Bytes,
    pub icon: Option<PathBuf>,
    pub override_title: Option<String>,
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub existing_loading_bar: Option<LoadingBarId>,
    pub profile: PathBuf,
}

pub fn get_profile_from_pack(
    location: CreatePackLocation,
) -> CreatePackProfile {
    match location {
        CreatePackLocation::FromVersionId {
            project_id,
            version_id,
            title,
            icon_url,
        } => CreatePackProfile {
            name: title,
            game_version: "1.19.4".to_string(),
            modloader: ModLoader::Vanilla,
            loader_version: None,
            icon: None,
            icon_url,
            linked_data: Some(LinkedData {
                project_id: Some(project_id),
                version_id: Some(version_id),
            }),
            skip_install_profile: Some(true),
        },
        CreatePackLocation::FromFile { path } => {
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            CreatePackProfile {
                name: file_name,
                game_version: "1.19.4".to_string(),
                modloader: ModLoader::Vanilla,
                loader_version: None,
                icon: None,
                icon_url: None,
                linked_data: None,
                skip_install_profile: Some(true),
            }
        }
    }
}

#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn generate_pack_from_version_id(
    project_id: String,
    version_id: String,
    title: String,
    icon_url: Option<String>,
    profile: PathBuf,
) -> crate::Result<CreatePackDescription> {
    let state = State::get().await?;

    let loading_bar = init_loading(
        LoadingBarType::PackFileDownload {
            profile_path: profile.clone(),
            pack_name: title,
            icon: icon_url,
            pack_version: version_id.clone(),
        },
        100.0,
        "Downloading pack file",
    )
    .await?;

    emit_loading(&loading_bar, 0.0, Some("Fetching version")).await?;
    let version: ModrinthVersion = fetch_json(
        Method::GET,
        &format!("{}version/{}", MODRINTH_API_URL, version_id),
        None,
        None,
        &state.fetch_semaphore,
    )
    .await?;
    emit_loading(&loading_bar, 10.0, None).await?;

    let (url, hash) =
        if let Some(file) = version.files.iter().find(|x| x.primary) {
            Some((file.url.clone(), file.hashes.get("sha1")))
        } else {
            version
                .files
                .first()
                .map(|file| (file.url.clone(), file.hashes.get("sha1")))
        }
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "Specified version has no files".to_string(),
            )
        })?;

    let file = fetch_advanced(
        Method::GET,
        &url,
        hash.map(|x| &**x),
        None,
        None,
        Some((&loading_bar, 70.0)),
        &state.fetch_semaphore,
    )
    .await?;
    emit_loading(&loading_bar, 0.0, Some("Fetching project metadata")).await?;

    let project: ModrinthProject = fetch_json(
        Method::GET,
        &format!("{}project/{}", MODRINTH_API_URL, version.project_id),
        None,
        None,
        &state.fetch_semaphore,
    )
    .await?;

    emit_loading(&loading_bar, 10.0, Some("Retrieving icon")).await?;
    let icon = if let Some(icon_url) = project.icon_url {
        let state = State::get().await?;
        let icon_bytes = fetch(&icon_url, None, &state.fetch_semaphore).await?;

        let filename = icon_url.rsplit('/').next();

        if let Some(filename) = filename {
            Some(
                write_cached_icon(
                    filename,
                    &state.directories.caches_dir().await,
                    icon_bytes,
                    &state.io_semaphore,
                )
                .await?,
            )
        } else {
            None
        }
    } else {
        None
    };
    emit_loading(&loading_bar, 10.0, None).await?;

    Ok(CreatePackDescription {
        file,
        icon,
        override_title: None,
        project_id: Some(project_id),
        version_id: Some(version_id),
        existing_loading_bar: Some(loading_bar),
        profile,
    })
}

#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn generate_pack_from_file(
    path: PathBuf,
    profile: PathBuf,
) -> crate::Result<CreatePackDescription> {
    let file = fs::read(&path).await?;
    Ok(CreatePackDescription {
        file: bytes::Bytes::from(file),
        icon: None,
        override_title: None,
        project_id: None,
        version_id: None,
        existing_loading_bar: None,
        profile,
    })
}
