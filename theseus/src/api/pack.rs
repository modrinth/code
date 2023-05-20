use crate::config::MODRINTH_API_URL;
use crate::data::ModLoader;
use crate::event::emit::{
    emit_loading, init_loading, init_or_edit_loading,
    loading_try_for_each_concurrent,
};
use crate::event::{LoadingBarId, LoadingBarType};
use crate::state::{
    LinkedData, ModrinthProject, ModrinthVersion, ProfileInstallStage, SideType,
};
use crate::util::fetch::{
    fetch, fetch_advanced, fetch_json, fetch_mirrors, write, write_cached_icon,
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

#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn install_pack_from_version_id(
    version_id: String,
    title: String,
    icon_url: Option<String>,
) -> crate::Result<PathBuf> {
    let state = State::get().await?;
    let profile = crate::api::profile_create::profile_create(
        title.clone(),
        "1.19.4".to_string(),
        ModLoader::Vanilla,
        None,
        None,
        icon_url.clone(),
        None,
        Some(true),
    )
    .await?;

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
    emit_loading(&loading_bar, 10.0, None).await?;

    install_pack(
        file,
        icon,
        Some(project.title),
        Some(version.project_id),
        Some(version.id),
        Some(loading_bar),
        profile,
    )
    .await
}

#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn install_pack_from_file(path: PathBuf) -> crate::Result<PathBuf> {
    let file = fs::read(&path).await?;

    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let profile = crate::api::profile_create::profile_create(
        file_name,
        "1.19.4".to_string(),
        ModLoader::Vanilla,
        None,
        None,
        None,
        None,
        Some(true),
    )
    .await?;

    install_pack(
        bytes::Bytes::from(file),
        None,
        None,
        None,
        None,
        None,
        profile,
    )
    .await
}

#[tracing::instrument(skip(file))]
#[theseus_macros::debug_pin]
async fn install_pack(
    file: bytes::Bytes,
    icon: Option<PathBuf>,
    override_title: Option<String>,
    project_id: Option<String>,
    version_id: Option<String>,
    existing_loading_bar: Option<LoadingBarId>,
    profile: PathBuf,
) -> crate::Result<PathBuf> {
    let state = &State::get().await?;

    let reader: Cursor<&bytes::Bytes> = Cursor::new(&file);

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

        let loader_version =
            crate::profile_create::get_loader_version_from_loader(
                game_version.clone(),
                mod_loader.unwrap_or(ModLoader::Vanilla),
                loader_version.cloned(),
            )
            .await?;
        crate::api::profile::edit(&profile, |prof| {
            prof.metadata.name =
                override_title.clone().unwrap_or_else(|| pack.name.clone());
            prof.install_stage = ProfileInstallStage::PackInstalling;
            prof.metadata.linked_data = Some(LinkedData {
                project_id: project_id.clone(),
                version_id: version_id.clone(),
            });
            prof.metadata.icon = icon.clone();
            prof.metadata.game_version = game_version.clone();
            prof.metadata.loader_version = loader_version.clone();
            prof.metadata.loader = mod_loader.unwrap_or(ModLoader::Vanilla);

            async { Ok(()) }
        })
        .await?;
        State::sync().await?;

        let profile = profile.clone();
        let result = async {
            let loading_bar = init_or_edit_loading(
                existing_loading_bar,
                LoadingBarType::PackDownload {
                    profile_path: profile.clone(),
                    pack_name: pack.name.clone(),
                    icon,
                    pack_id: project_id,
                    pack_version: version_id,
                },
                100.0,
                "Downloading modpack",
            )
            .await?;

            let num_files = pack.files.len();
            use futures::StreamExt;
            loading_try_for_each_concurrent(
                futures::stream::iter(pack.files.into_iter())
                    .map(Ok::<PackFile, crate::Error>),
                None,
                Some(&loading_bar),
                70.0,
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
                            &state.fetch_semaphore,
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

            emit_loading(&loading_bar, 0.0, Some("Extracting overrides"))
                .await?;

            let mut total_len = 0;

            for index in 0..zip_reader.file().entries().len() {
                let file =
                    zip_reader.file().entries().get(index).unwrap().entry();

                if (file.filename().starts_with("overrides")
                    || file.filename().starts_with("client_overrides"))
                    && !file.filename().ends_with('/')
                {
                    total_len += 1;
                }
            }

            for index in 0..zip_reader.file().entries().len() {
                let file = zip_reader
                    .file()
                    .entries()
                    .get(index)
                    .unwrap()
                    .entry()
                    .clone();

                let file_path = PathBuf::from(file.filename());
                if (file.filename().starts_with("overrides")
                    || file.filename().starts_with("client_overrides"))
                    && !file.filename().ends_with('/')
                {
                    // Reads the file into the 'content' variable
                    let mut content = Vec::new();
                    let mut reader = zip_reader.entry(index).await?;
                    reader.read_to_end_checked(&mut content, &file).await?;

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

                    emit_loading(
                        &loading_bar,
                        30.0 / total_len as f64,
                        Some(&format!(
                            "Extracting override {}/{}",
                            index, total_len
                        )),
                    )
                    .await?;
                }
            }

            if let Some(profile_val) =
                crate::api::profile::get(&profile, None).await?
            {
                crate::launcher::install_minecraft(
                    &profile_val,
                    Some(loading_bar),
                )
                .await?;
            }

            Ok::<PathBuf, crate::Error>(profile.clone())
        }
        .await;

        match result {
            Ok(profile) => Ok(profile),
            Err(err) => {
                let _ = crate::api::profile::remove(&profile).await;

                Err(err)
            }
        }
    } else {
        let _ = crate::api::profile::remove(&profile).await;

        Err(crate::Error::from(crate::ErrorKind::InputError(
            "No pack manifest found in mrpack".to_string(),
        )))
    }
}
