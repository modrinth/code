//! Project management + inference

use crate::config::{MODRINTH_API_URL, REQWEST_CLIENT};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Digest;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;
use zip::ZipArchive;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub sha512: String,
    pub disabled: bool,
    pub metadata: ProjectMetadata,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModrinthProject {
    pub id: String,
    pub slug: Option<String>,
    pub project_type: String,
    pub team: String,
    pub title: String,
    pub description: String,
    pub body: String,

    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,

    pub client_side: String,
    pub server_side: String,

    pub downloads: u32,
    pub followers: u32,

    pub categories: Vec<String>,
    pub additional_categories: Vec<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,

    pub versions: Vec<String>,

    pub icon_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectMetadata {
    Modrinth(Box<ModrinthProject>),
    Inferred {
        title: Option<String>,
        description: Option<String>,
        authors: Vec<String>,
        version: Option<String>,
        icon: Option<PathBuf>,
    },
    Unknown,
}

pub async fn infer_data_from_files(
    paths: Vec<PathBuf>,
    cache_dir: PathBuf,
) -> crate::Result<HashMap<PathBuf, Project>> {
    let mut file_path_hashes = HashMap::new();

    // TODO: Make this concurrent and use progressive hashing to avoid loading each JAR in memory
    for path in paths.clone() {
        let mut file = tokio::fs::File::open(path.clone()).await?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let hash = format!("{:x}", sha2::Sha512::digest(&buffer));
        file_path_hashes.insert(hash, path.clone());
    }

    // TODO: add disabled mods
    // TODO: add retrying
    #[derive(Deserialize)]
    pub struct ModrinthVersion {
        pub project_id: String,
    }
    let files: HashMap<String, ModrinthVersion> = REQWEST_CLIENT
        .post(format!("{}version_files", MODRINTH_API_URL))
        .json(&json!({
            "hashes": file_path_hashes.keys().collect::<Vec<_>>(),
            "algorithm": "sha512",
        }))
        .send()
        .await?
        .json()
        .await?;

    let projects: Vec<ModrinthProject> = REQWEST_CLIENT
        .get(format!(
            "{}projects?ids={}",
            MODRINTH_API_URL,
            serde_json::to_string(
                &files
                    .values()
                    .map(|x| x.project_id.clone())
                    .collect::<Vec<_>>()
            )?
        ))
        .send()
        .await?
        .json()
        .await?;

    let mut return_projects = HashMap::new();
    let mut further_analyze_projects: Vec<(String, PathBuf)> = Vec::new();

    for (hash, path) in file_path_hashes {
        if let Some(file) = files.get(&hash) {
            if let Some(project) =
                projects.iter().find(|x| file.project_id == x.id)
            {
                return_projects.insert(
                    path,
                    Project {
                        sha512: hash,
                        disabled: false,
                        metadata: ProjectMetadata::Modrinth(Box::new(
                            project.clone(),
                        )),
                    },
                );
                continue;
            }
        }

        further_analyze_projects.push((hash, path));
    }

    for (hash, path) in further_analyze_projects {
        let file = File::open(path.clone())?;

        // TODO: get rid of below unwrap
        let mut zip = ZipArchive::new(file).unwrap();

        let read_icon_from_file =
            |icon_path: Option<String>| -> crate::Result<Option<PathBuf>> {
                if let Some(icon_path) = icon_path {
                    // we have to repoen the zip twice here :(
                    let zip_file = File::open(path.clone())?;
                    if let Ok(mut zip) = ZipArchive::new(zip_file) {
                        if let Ok(mut file) = zip.by_name(&icon_path) {
                            let mut bytes = Vec::new();
                            if file.read_to_end(&mut bytes).is_ok() {
                                let extension = Path::new(&icon_path)
                                    .extension()
                                    .and_then(OsStr::to_str);
                                let hash = sha1::Sha1::from(&bytes).hexdigest();
                                let path = cache_dir.join("icons").join(
                                    if let Some(ext) = extension {
                                        format!("{hash}.{ext}")
                                    } else {
                                        hash
                                    },
                                );

                                if !path.exists() {
                                    if let Some(parent) = path.parent() {
                                        std::fs::create_dir_all(parent)?;
                                    }

                                    let mut file = File::create(path.clone())?;
                                    file.write_all(&bytes)?;
                                }

                                return Ok(Some(path));
                            }
                        };
                    }
                }

                Ok(None)
            };

        if let Ok(mut file) = zip.by_name("META-INF/mods.toml") {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct ForgeModInfo {
                pub mods: Vec<ForgeMod>,
            }
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct ForgeMod {
                mod_id: String,
                version: Option<String>,
                display_name: Option<String>,
                description: Option<String>,
                logo_file: Option<String>,
                authors: Option<String>,
            }

            let mut file_str = String::new();
            if file.read_to_string(&mut file_str).is_ok() {
                if let Ok(pack) =
                    serde_json::from_str::<ForgeModInfo>(&file_str)
                {
                    if let Some(pack) = pack.mods.first() {
                        let icon = read_icon_from_file(pack.logo_file.clone())?;

                        return_projects.insert(
                            path.clone(),
                            Project {
                                sha512: hash,
                                disabled: false,
                                metadata: ProjectMetadata::Inferred {
                                    title: Some(
                                        pack.display_name
                                            .clone()
                                            .unwrap_or_else(|| pack.mod_id.clone()),
                                    ),
                                    description: pack.description.clone(),
                                    authors: pack
                                        .authors
                                        .clone()
                                        .map(|x| vec![x])
                                        .unwrap_or_default(),
                                    version: pack.version.clone(),
                                    icon,
                                },
                            },
                        );
                        continue;
                    }
                }
            }
        }

        if let Ok(mut file) = zip.by_name("mcmod.info") {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct ForgeMod {
                modid: String,
                name: String,
                description: Option<String>,
                version: Option<String>,
                author_list: Option<Vec<String>>,
                logo_file: Option<String>,
            }

            let mut file_str = String::new();
            if file.read_to_string(&mut file_str).is_ok() {
                if let Ok(pack) = serde_json::from_str::<ForgeMod>(&file_str) {
                    let icon = read_icon_from_file(pack.logo_file)?;

                    return_projects.insert(
                        path.clone(),
                        Project {
                            sha512: hash,
                            disabled: false,
                            metadata: ProjectMetadata::Inferred {
                                title: Some(if pack.name.is_empty() {
                                    pack.modid
                                } else {
                                    pack.name
                                }),
                                description: pack.description,
                                authors: pack.author_list.unwrap_or_default(),
                                version: pack.version,
                                icon,
                            },
                        },
                    );
                    continue;
                }
            }
        }

        if let Ok(mut file) = zip.by_name("fabric.mod.json") {
            #[derive(Deserialize)]
            #[serde(untagged)]
            enum FabricAuthor {
                String(String),
                Object { name: String },
            }
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct FabricMod {
                id: String,
                version: String,
                name: Option<String>,
                description: Option<String>,
                authors: Vec<FabricAuthor>,
                icon: Option<String>,
            }

            let mut file_str = String::new();
            if file.read_to_string(&mut file_str).is_ok() {
                if let Ok(pack) = serde_json::from_str::<FabricMod>(&file_str) {
                    let icon = read_icon_from_file(pack.icon)?;

                    return_projects.insert(
                        path.clone(),
                        Project {
                            sha512: hash,
                            disabled: false,
                            metadata: ProjectMetadata::Inferred {
                                title: Some(pack.name.unwrap_or(pack.id)),
                                description: pack.description,
                                authors: pack
                                    .authors
                                    .into_iter()
                                    .map(|x| match x {
                                        FabricAuthor::String(name) => name,
                                        FabricAuthor::Object { name } => name,
                                    })
                                    .collect(),
                                version: Some(pack.version),
                                icon,
                            },
                        },
                    );
                    continue;
                }
            }
        }

        if let Ok(mut file) = zip.by_name("quilt.mod.json") {
            #[derive(Deserialize)]
            struct QuiltMetadata {
                pub name: Option<String>,
                pub description: Option<String>,
                pub contributors: Option<HashMap<String, String>>,
                pub icon: Option<String>,
            }
            #[derive(Deserialize)]
            struct QuiltMod {
                id: String,
                version: String,
                metadata: Option<QuiltMetadata>,
            }

            let mut file_str = String::new();
            if file.read_to_string(&mut file_str).is_ok() {
                if let Ok(pack) = serde_json::from_str::<QuiltMod>(&file_str) {
                    let icon = read_icon_from_file(
                        pack.metadata.as_ref().and_then(|x| x.icon.clone()),
                    )?;

                    return_projects.insert(
                        path.clone(),
                        Project {
                            sha512: hash,
                            disabled: false,
                            metadata: ProjectMetadata::Inferred {
                                title: Some(
                                    pack.metadata
                                        .as_ref()
                                        .and_then(|x| x.name.clone())
                                        .unwrap_or(pack.id),
                                ),
                                description: pack
                                    .metadata
                                    .as_ref()
                                    .and_then(|x| x.description.clone()),
                                authors: pack
                                    .metadata
                                    .map(|x| {
                                        x.contributors
                                            .unwrap_or_default()
                                            .keys()
                                            .cloned()
                                            .collect()
                                    })
                                    .unwrap_or_default(),
                                version: Some(pack.version),
                                icon,
                            },
                        },
                    );
                    continue;
                }
            }
        }

        if let Ok(mut file) = zip.by_name("pack.mcmeta") {
            #[derive(Deserialize)]
            struct Pack {
                description: Option<String>,
            }

            let mut file_str = String::new();
            if file.read_to_string(&mut file_str).is_ok() {
                if let Ok(pack) = serde_json::from_str::<Pack>(&file_str) {
                    let icon =
                        read_icon_from_file(Some("pack.png".to_string()))?;

                    return_projects.insert(
                        path.clone(),
                        Project {
                            sha512: hash,
                            disabled: false,
                            metadata: ProjectMetadata::Inferred {
                                title: None,
                                description: pack.description,
                                authors: Vec::new(),
                                version: None,
                                icon,
                            },
                        },
                    );
                    continue;
                }
            }
        }

        return_projects.insert(
            path,
            Project {
                sha512: hash,
                disabled: false,
                metadata: ProjectMetadata::Unknown,
            },
        );
    }

    Ok(return_projects)
}
