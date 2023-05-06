use crate::config::MODRINTH_API_URL;
use crate::data::ModLoader;
use crate::event::emit::{
    emit_loading, init_loading, loading_try_for_each_concurrent,
};
use crate::event::LoadingBarType;
use crate::state::{LinkedData, ModrinthProject, ModrinthVersion, SideType};
use crate::util::fetch::{
    fetch, fetch_json, fetch_mirrors, write, write_cached_icon,
};
use crate::State;
use async_zip::tokio::read::seek::ZipFileReader;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Component, PathBuf};
use tokio::fs;

#[derive(Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
struct PackFormat {
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
struct PackFile {
    pub path: String,
    pub hashes: HashMap<PackFileHash, String>,
    pub env: Option<HashMap<EnvType, SideType>>,
    pub downloads: Vec<String>,
    pub file_size: u32,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase", from = "String")]
enum PackFileHash {
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
enum EnvType {
    Client,
    Server,
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum PackDependency {
    Forge,
    FabricLoader,
    QuiltLoader,
    Minecraft,
}

pub async fn install_pack_from_version_id(
    version_id: String,
) -> crate::Result<PathBuf> {
    let state = State::get().await?;

    let version: ModrinthVersion = fetch_json(
        Method::GET,
        &format!("{}version/{}", MODRINTH_API_URL, version_id),
        None,
        None,
        &state.io_semaphore,
    )
    .await?;

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

    let file = fetch(&url, hash.map(|x| &**x), &state.io_semaphore).await?;

    let project: ModrinthProject = fetch_json(
        Method::GET,
        &format!("{}project/{}", MODRINTH_API_URL, version.project_id),
        None,
        None,
        &state.io_semaphore,
    )
    .await?;

    let icon = if let Some(icon_url) = project.icon_url {
        let state = State::get().await?;
        let icon_bytes = fetch(&icon_url, None, &state.io_semaphore).await?;

        let filename = icon_url.rsplit('/').next();

        if let Some(filename) = filename {
            Some(
                write_cached_icon(
                    filename,
                    &state.directories.caches_dir(),
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

    install_pack(
        file,
        icon,
        Some(project.title),
        Some(version.project_id),
        Some(version.id),
    )
    .await
}

pub async fn install_pack_from_file(path: PathBuf) -> crate::Result<PathBuf> {
    let file = fs::read(path).await?;

    install_pack(bytes::Bytes::from(file), None, None, None, None).await
}

async fn install_pack(
    file: bytes::Bytes,
    icon: Option<PathBuf>,
    override_title: Option<String>,
    project_id: Option<String>,
    version_id: Option<String>,
) -> crate::Result<PathBuf> {
    let state = &State::get().await?;

    let reader = Cursor::new(&file);

    // Create zip reader around file
    let mut zip_reader = ZipFileReader::new(reader).await.map_err(|_| {
        crate::Error::from(crate::ErrorKind::InputError(
            "Failed to read input modpack zip".to_string(),
        ))
    })?;

    // Extract index of modrinth.index.json
    let zip_index_option = zip_reader
        .file()
        .entries()
        .iter()
        .position(|f| f.entry().filename() == "modrinth.index.json");
    if let Some(zip_index) = zip_index_option {
        let mut manifest = String::new();
        let entry = zip_reader
            .file()
            .entries()
            .get(zip_index)
            .unwrap()
            .entry()
            .clone();
        let mut reader = zip_reader.entry(zip_index).await?;
        reader.read_to_string_checked(&mut manifest, &entry).await?;

        let pack: PackFormat = serde_json::from_str(&manifest)?;

        if &*pack.game != "minecraft" {
            return Err(crate::ErrorKind::InputError(
                "Pack does not support Minecraft".to_string(),
            )
            .into());
        }

        let mut game_version = None;
        let mut mod_loader = None;
        let mut loader_version = None;
        for (key, value) in &pack.dependencies {
            match key {
                PackDependency::Forge => {
                    mod_loader = Some(ModLoader::Forge);
                    loader_version = Some(value);
                }
                PackDependency::FabricLoader => {
                    mod_loader = Some(ModLoader::Fabric);
                    loader_version = Some(value);
                }
                PackDependency::QuiltLoader => {
                    mod_loader = Some(ModLoader::Quilt);
                    loader_version = Some(value);
                }
                PackDependency::Minecraft => game_version = Some(value),
            }
        }

        let game_version = if let Some(game_version) = game_version {
            game_version
        } else {
            return Err(crate::ErrorKind::InputError(
                "Pack did not specify Minecraft version".to_string(),
            )
            .into());
        };

        let profile_raw = crate::api::profile_create::profile_create(
            override_title.unwrap_or_else(|| pack.name.clone()),
            game_version.clone(),
            mod_loader.unwrap_or(ModLoader::Vanilla),
            loader_version.cloned(),
            icon,
            Some(LinkedData {
                project_id: project_id.clone(),
                version_id: version_id.clone(),
            }),
            Some(true),
        )
        .await?;
        let profile = profile_raw.clone();
        let result = async {
            let loading_bar = init_loading(
                LoadingBarType::PackDownload {
                    pack_name: pack.name.clone(),
                    pack_id: project_id,
                    pack_version: version_id,
                },
                100.0,
                "Downloading modpack...",
            )
            .await?;

            let num_files = pack.files.len();
            use futures::StreamExt;
            loading_try_for_each_concurrent(
                futures::stream::iter(pack.files.into_iter())
                    .map(Ok::<PackFile, crate::Error>),
                None,
                Some(&loading_bar),
                90.0,
                num_files,
                None,
                |project| {
                    let profile = profile.clone();
                    async move {
                        //TODO: Future update: prompt user for optional files in a modpack
                        if let Some(env) = project.env {
                            if env
                                .get(&EnvType::Client)
                                .map(|x| x == &SideType::Unsupported)
                                .unwrap_or(false)
                            {
                                return Ok(());
                            }
                        }

                        let file = fetch_mirrors(
                            &project
                                .downloads
                                .iter()
                                .map(|x| &**x)
                                .collect::<Vec<&str>>(),
                            project
                                .hashes
                                .get(&PackFileHash::Sha1)
                                .map(|x| &**x),
                            &state.io_semaphore,
                        )
                        .await?;

                        let path = std::path::Path::new(&project.path)
                            .components()
                            .next();
                        if let Some(path) = path {
                            match path {
                                Component::CurDir | Component::Normal(_) => {
                                    let path = profile.join(project.path);
                                    write(&path, &file, &state.io_semaphore)
                                        .await?;
                                }
                                _ => {}
                            };
                        }
                        Ok(())
                    }
                },
            )
            .await?;

            let extract_overrides = |overrides: String| async {
                let reader = Cursor::new(&file);

                let mut overrides_zip =
                    ZipFileReader::new(reader).await.map_err(|_| {
                        crate::Error::from(crate::ErrorKind::InputError(
                            "Failed extract overrides Zip".to_string(),
                        ))
                    })?;

                let profile = profile.clone();
                async move {
                    for index in 0..overrides_zip.file().entries().len() {
                        let file = overrides_zip
                            .file()
                            .entries()
                            .get(index)
                            .unwrap()
                            .entry()
                            .clone();

                        let file_path = PathBuf::from(file.filename());
                        if file.filename().starts_with(&overrides)
                            && !file.filename().ends_with('/')
                        {
                            // Reads the file into the 'content' variable
                            let mut content = Vec::new();
                            let mut reader = overrides_zip.entry(index).await?;
                            reader
                                .read_to_end_checked(&mut content, &file)
                                .await?;

                            let mut new_path = PathBuf::new();
                            let components = file_path.components().skip(1);

                            for component in components {
                                new_path.push(component);
                            }

                            if new_path.file_name().is_some() {
                                write(
                                    &profile.join(new_path),
                                    &content,
                                    &state.io_semaphore,
                                )
                                .await?;
                            }
                        }
                    }

                    Ok::<(), crate::Error>(())
                }
                .await
            };

            emit_loading(&loading_bar, 5.0, Some("Extracting overrides"))
                .await?;
            extract_overrides("overrides".to_string()).await?;
            extract_overrides("client_overrides".to_string()).await?;
            emit_loading(&loading_bar, 5.0, Some("Done extacting overrides"))
                .await?;

            if let Some(profile) = crate::api::profile::get(&profile).await? {
                tokio::try_join!(
                    super::profile::sync(&profile.path),
                    crate::launcher::install_minecraft(
                        &profile,
                        Some(loading_bar)
                    ),
                )?;
            } else {
                emit_loading(
                    &loading_bar,
                    10.0,
                    Some("Done extacting overrides"),
                )
                .await?;
            }

            Ok::<PathBuf, crate::Error>(profile)
        }
        .await;

        match result {
            Ok(profile) => Ok(profile),
            Err(err) => {
                let _ = crate::api::profile::remove(&profile_raw).await;

                Err(err)
            }
        }
    } else {
        Err(crate::Error::from(crate::ErrorKind::InputError(
            "No pack manifest found in mrpack".to_string(),
        )))
    }
}
