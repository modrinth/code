//! Project management + inference

use crate::config::MODRINTH_API_URL;
use crate::state::Profile;
use crate::util::fetch::{
    fetch_json, write_cached_icon, FetchSemaphore, IoSemaphore,
};
use async_zip::tokio::read::fs::ZipFileReader;
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Digest;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Mod,
    DataPack,
    ResourcePack,
    ShaderPack,
}

impl ProjectType {
    pub fn get_from_loaders(loaders: Vec<String>) -> Option<Self> {
        if loaders
            .iter()
            .any(|x| ["fabric", "forge", "quilt"].contains(&&**x))
        {
            Some(ProjectType::Mod)
        } else if loaders.iter().any(|x| x == "datapack") {
            Some(ProjectType::DataPack)
        } else if loaders.iter().any(|x| ["iris", "optifine"].contains(&&**x)) {
            Some(ProjectType::ShaderPack)
        } else if loaders
            .iter()
            .any(|x| ["vanilla", "canvas", "minecraft"].contains(&&**x))
        {
            Some(ProjectType::ResourcePack)
        } else {
            None
        }
    }

    pub fn get_folder(&self) -> &'static str {
        match self {
            ProjectType::Mod => "mods",
            ProjectType::DataPack => "datapacks",
            ProjectType::ResourcePack => "resourcepacks",
            ProjectType::ShaderPack => "shaderpacks",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub sha512: String,
    pub disabled: bool,
    pub metadata: ProjectMetadata,
    pub file_name: String,
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

    pub client_side: SideType,
    pub server_side: SideType,

    pub downloads: u32,
    pub followers: u32,

    pub categories: Vec<String>,
    pub additional_categories: Vec<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,

    pub versions: Vec<String>,

    pub icon_url: Option<String>,
}

/// A specific version of a project
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModrinthVersion {
    pub id: String,
    pub project_id: String,
    pub author_id: String,

    pub featured: bool,

    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub changelog_url: Option<String>,

    pub date_published: DateTime<Utc>,
    pub downloads: u32,
    pub version_type: String,

    pub files: Vec<ModrinthVersionFile>,
    pub dependencies: Vec<Dependency>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModrinthVersionFile {
    pub hashes: HashMap<String, String>,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u32,
    pub file_type: Option<FileType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: DependencyType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModrinthTeamMember {
    pub team_id: String,
    pub user: ModrinthUser,
    pub role: String,
    pub ordering: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModrinthUser {
    pub id: String,
    pub github_id: Option<u64>,
    pub username: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created: DateTime<Utc>,
    pub role: String,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SideType {
    Required,
    Optional,
    Unsupported,
    Unknown,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum FileType {
    RequiredResourcePack,
    OptionalResourcePack,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectMetadata {
    Modrinth {
        project: Box<ModrinthProject>,
        version: Box<ModrinthVersion>,
        members: Vec<ModrinthTeamMember>,
        update_version: Option<Box<ModrinthVersion>>,
        incompatible: bool,
    },
    Inferred {
        title: Option<String>,
        description: Option<String>,
        authors: Vec<String>,
        version: Option<String>,
        icon: Option<PathBuf>,
    },
    Unknown,
}

#[tracing::instrument(skip(io_semaphore))]
#[theseus_macros::debug_pin]
async fn read_icon_from_file(
    icon_path: Option<String>,
    cache_dir: &Path,
    path: &PathBuf,
    io_semaphore: &IoSemaphore,
) -> crate::Result<Option<PathBuf>> {
    if let Some(icon_path) = icon_path {
        // we have to repoen the zip twice here :(
        let zip_file_reader = ZipFileReader::new(path).await;
        if let Ok(zip_file_reader) = zip_file_reader {
            // Get index of icon file and open it
            let zip_index_option = zip_file_reader
                .file()
                .entries()
                .iter()
                .position(|f| f.entry().filename() == icon_path);
            if let Some(index) = zip_index_option {
                let entry = zip_file_reader
                    .file()
                    .entries()
                    .get(index)
                    .unwrap()
                    .entry();
                let mut bytes = Vec::new();
                if zip_file_reader
                    .entry(zip_index_option.unwrap())
                    .await?
                    .read_to_end_checked(&mut bytes, entry)
                    .await
                    .is_ok()
                {
                    let bytes = bytes::Bytes::from(bytes);
                    let path = write_cached_icon(
                        &icon_path,
                        cache_dir,
                        bytes,
                        io_semaphore,
                    )
                    .await?;

                    return Ok(Some(path));
                }
            };
        }
    }

    Ok(None)
}

#[tracing::instrument(skip(profile, io_semaphore, fetch_semaphore))]
#[theseus_macros::debug_pin]
pub async fn infer_data_from_files(
    profile: Profile,
    paths: Vec<PathBuf>,
    cache_dir: PathBuf,
    io_semaphore: &IoSemaphore,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<HashMap<PathBuf, Project>> {
    let mut file_path_hashes = HashMap::new();

    // TODO: Make this concurrent and use progressive hashing to avoid loading each JAR in memory
    for path in paths {
        let mut file = tokio::fs::File::open(path.clone()).await?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let hash = format!("{:x}", sha2::Sha512::digest(&buffer));
        file_path_hashes.insert(hash, path.clone());
    }

    let files_url = format!("{}version_files", MODRINTH_API_URL);
    let updates_url = format!("{}version_files/update", MODRINTH_API_URL);
    let (files, update_versions) = tokio::try_join!(
        fetch_json::<HashMap<String, ModrinthVersion>>(
            Method::POST,
            &files_url,
            None,
            Some(json!({
                "hashes": file_path_hashes.keys().collect::<Vec<_>>(),
                "algorithm": "sha512",
            })),
            fetch_semaphore,
        ),
        fetch_json::<HashMap<String, ModrinthVersion>>(
            Method::POST,
            &updates_url,
            None,
            Some(json!({
                "hashes": file_path_hashes.keys().collect::<Vec<_>>(),
                "algorithm": "sha512",
                "loaders": [profile.metadata.loader],
                "game_versions": [profile.metadata.game_version]
            })),
            fetch_semaphore,
        )
    )?;

    let projects: Vec<ModrinthProject> = fetch_json(
        Method::GET,
        &format!(
            "{}projects?ids={}",
            MODRINTH_API_URL,
            serde_json::to_string(
                &files
                    .values()
                    .map(|x| x.project_id.clone())
                    .collect::<Vec<_>>()
            )?
        ),
        None,
        None,
        fetch_semaphore,
    )
    .await?;

    let teams: Vec<ModrinthTeamMember> = fetch_json::<
        Vec<Vec<ModrinthTeamMember>>,
    >(
        Method::GET,
        &format!(
            "{}teams?ids={}",
            MODRINTH_API_URL,
            serde_json::to_string(
                &projects.iter().map(|x| x.team.clone()).collect::<Vec<_>>()
            )?
        ),
        None,
        None,
        fetch_semaphore,
    )
    .await?
    .into_iter()
    .flatten()
    .collect();

    let mut return_projects = HashMap::new();
    let mut further_analyze_projects: Vec<(String, PathBuf)> = Vec::new();

    for (hash, path) in file_path_hashes {
        if let Some(version) = files.get(&hash) {
            if let Some(project) =
                projects.iter().find(|x| version.project_id == x.id)
            {
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                return_projects.insert(
                    path,
                    Project {
                        disabled: file_name.ends_with(".disabled"),
                        metadata: ProjectMetadata::Modrinth {
                            project: Box::new(project.clone()),
                            version: Box::new(version.clone()),
                            members: teams
                                .iter()
                                .filter(|x| x.team_id == project.team)
                                .cloned()
                                .collect::<Vec<_>>(),
                            update_version: if let Some(value) =
                                update_versions.get(&hash)
                            {
                                if value.id != version.id {
                                    Some(Box::new(value.clone()))
                                } else {
                                    None
                                }
                            } else {
                                None
                            },
                            incompatible: !version.loaders.contains(
                                &profile
                                    .metadata
                                    .loader
                                    .as_api_str()
                                    .to_string(),
                            ) || version
                                .game_versions
                                .contains(&profile.metadata.game_version),
                        },
                        sha512: hash,
                        file_name,
                    },
                );
                continue;
            }
        }

        further_analyze_projects.push((hash, path));
    }

    for (hash, path) in further_analyze_projects {
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let zip_file_reader = if let Ok(zip_file_reader) =
            ZipFileReader::new(path.clone()).await
        {
            zip_file_reader
        } else {
            return_projects.insert(
                path.clone(),
                Project {
                    sha512: hash,
                    disabled: file_name.ends_with(".disabled"),
                    metadata: ProjectMetadata::Unknown,
                    file_name,
                },
            );
            continue;
        };
        let zip_index_option = zip_file_reader
            .file()
            .entries()
            .iter()
            .position(|f| f.entry().filename() == "META-INF/mods.toml");
        if let Some(index) = zip_index_option {
            let file = zip_file_reader.file().entries().get(index).unwrap();
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
            if zip_file_reader
                .entry(index)
                .await?
                .read_to_string_checked(&mut file_str, file.entry())
                .await
                .is_ok()
            {
                if let Ok(pack) =
                    serde_json::from_str::<ForgeModInfo>(&file_str)
                {
                    if let Some(pack) = pack.mods.first() {
                        let icon = read_icon_from_file(
                            pack.logo_file.clone(),
                            &cache_dir,
                            &path,
                            io_semaphore,
                        )
                        .await?;

                        return_projects.insert(
                            path.clone(),
                            Project {
                                sha512: hash,
                                disabled: file_name.ends_with(".disabled"),
                                file_name,
                                metadata: ProjectMetadata::Inferred {
                                    title: Some(
                                        pack.display_name
                                            .clone()
                                            .unwrap_or_else(|| {
                                                pack.mod_id.clone()
                                            }),
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

        let zip_index_option = zip_file_reader
            .file()
            .entries()
            .iter()
            .position(|f| f.entry().filename() == "mcmod.info");
        if let Some(index) = zip_index_option {
            let file = zip_file_reader.file().entries().get(index).unwrap();
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
            if zip_file_reader
                .entry(index)
                .await?
                .read_to_string_checked(&mut file_str, file.entry())
                .await
                .is_ok()
            {
                if let Ok(pack) = serde_json::from_str::<ForgeMod>(&file_str) {
                    let icon = read_icon_from_file(
                        pack.logo_file,
                        &cache_dir,
                        &path,
                        io_semaphore,
                    )
                    .await?;

                    return_projects.insert(
                        path.clone(),
                        Project {
                            sha512: hash,
                            disabled: file_name.ends_with(".disabled"),
                            file_name,
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

        let zip_index_option = zip_file_reader
            .file()
            .entries()
            .iter()
            .position(|f| f.entry().filename() == "fabric.mod.json");
        if let Some(index) = zip_index_option {
            let file = zip_file_reader.file().entries().get(index).unwrap();
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
            if zip_file_reader
                .entry(index)
                .await?
                .read_to_string_checked(&mut file_str, file.entry())
                .await
                .is_ok()
            {
                if let Ok(pack) = serde_json::from_str::<FabricMod>(&file_str) {
                    let icon = read_icon_from_file(
                        pack.icon,
                        &cache_dir,
                        &path,
                        io_semaphore,
                    )
                    .await?;

                    return_projects.insert(
                        path.clone(),
                        Project {
                            sha512: hash,
                            disabled: file_name.ends_with(".disabled"),
                            file_name,
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

        let zip_index_option = zip_file_reader
            .file()
            .entries()
            .iter()
            .position(|f| f.entry().filename() == "quilt.mod.json");
        if let Some(index) = zip_index_option {
            let file = zip_file_reader.file().entries().get(index).unwrap();
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
            if zip_file_reader
                .entry(index)
                .await?
                .read_to_string_checked(&mut file_str, file.entry())
                .await
                .is_ok()
            {
                if let Ok(pack) = serde_json::from_str::<QuiltMod>(&file_str) {
                    let icon = read_icon_from_file(
                        pack.metadata.as_ref().and_then(|x| x.icon.clone()),
                        &cache_dir,
                        &path,
                        io_semaphore,
                    )
                    .await?;

                    return_projects.insert(
                        path.clone(),
                        Project {
                            sha512: hash,
                            disabled: file_name.ends_with(".disabled"),
                            file_name,
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

        let zip_index_option = zip_file_reader
            .file()
            .entries()
            .iter()
            .position(|f| f.entry().filename() == "pack.mcmeta");
        if let Some(index) = zip_index_option {
            let file = zip_file_reader.file().entries().get(index).unwrap();
            #[derive(Deserialize)]
            struct Pack {
                description: Option<String>,
            }

            let mut file_str = String::new();
            if zip_file_reader
                .entry(index)
                .await?
                .read_to_string_checked(&mut file_str, file.entry())
                .await
                .is_ok()
            {
                if let Ok(pack) = serde_json::from_str::<Pack>(&file_str) {
                    let icon = read_icon_from_file(
                        Some("pack.png".to_string()),
                        &cache_dir,
                        &path,
                        io_semaphore,
                    )
                    .await?;
                    return_projects.insert(
                        path.clone(),
                        Project {
                            sha512: hash,
                            disabled: file_name.ends_with(".disabled"),
                            file_name,
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
            path.clone(),
            Project {
                sha512: hash,
                disabled: file_name.ends_with(".disabled"),
                file_name,
                metadata: ProjectMetadata::Unknown,
            },
        );
    }

    Ok(return_projects)
}
