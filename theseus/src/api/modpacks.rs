use crate::config::{MODRINTH_API_URL, REQWEST_CLIENT};
use crate::data::ModLoader;
use crate::state::{ModrinthVersion, SideType};
use crate::util::fetch::{fetch, fetch_mirrors, write};
use crate::State;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::{Component, PathBuf};
use zip::ZipArchive;

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

pub async fn install_modpack_from_version_id(
    version_id: String,
) -> crate::Result<PathBuf> {
    let version: ModrinthVersion = REQWEST_CLIENT
        .get(format!("{}version/{}", MODRINTH_API_URL, version_id))
        .send()
        .await?
        .json()
        .await?;

    let (url, hash) =
        if let Some(file) = version.files.iter().find(|x| x.primary) {
            Some((file.url.clone(), file.hashes.get("sha1")))
        } else if let Some(file) = version.files.first() {
            Some((file.url.clone(), file.hashes.get("sha1")))
        } else {
            None
        }
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "Specified version has no files".to_string(),
            )
        })?;

    let file = async {
        let state = &State::get().await?;
        let semaphore = state.io_semaphore.acquire().await?;
        fetch(&*url, hash.map(|x| &**x), &semaphore).await
    }
    .await?;

    // TODO: fetch icon and pass it here
    install_pack(file, None).await
}
//
// pub fn install_modpack_from_file(path: PathBuf) -> crate::Result<()> {}

async fn install_pack(
    file: bytes::Bytes,
    icon: Option<PathBuf>,
) -> crate::Result<PathBuf> {
    let state = &State::get().await?;

    let reader = Cursor::new(file);
    let mut zip = ZipArchive::new(reader).map_err(|_| {
        crate::Error::from(crate::ErrorKind::InputError(
            "Failed to read input modpack zip".to_string(),
        ))
    })?;

    let index_json = zip.by_name("modrinth.index.json");
    if let Ok(mut zip_file) = index_json {
        let mut file = String::new();
        zip_file.read_to_string(&mut file)?;

        let pack: PackFormat = serde_json::from_str(&file)?;

        if &*pack.game != "minecraft" {
            return Err(crate::ErrorKind::InputError(
                "Pack does not support Minecraft".to_string(),
            )
            .into());
        }

        let mut game_version = None;
        let mut mod_loader = None;
        let mut loader_version = None;
        for (key, value) in pack.dependencies {
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

        let profile = crate::api::profile_create::profile_create(
            pack.name,
            game_version.clone(),
            mod_loader.unwrap_or(ModLoader::Vanilla),
            loader_version.map(|x| format!("{game_version}-{x}")),
            icon,
        )
        .await?;

        println!("finished path profile");

        use futures::StreamExt;
        futures::stream::iter(pack.files.into_iter())
            .map(Ok::<PackFile, crate::Error>)
            .try_for_each_concurrent(None, |project| {
                let profile = profile.clone();
                async move {
                    println!("started downloading {:?}", project.path);
                    let permit = state.io_semaphore.acquire().await?;

                    let file = fetch_mirrors(
                        &project
                            .downloads
                            .iter()
                            .map(|x| &**x)
                            .collect::<Vec<&str>>(),
                        project.hashes.get(&PackFileHash::Sha1).map(|x| &**x),
                        &permit,
                    )
                    .await?;

                    let path =
                        std::path::Path::new(&project.path).components().next();
                    if let Some(path) = path {
                        match path {
                            Component::CurDir | Component::Normal(_) => {
                                let path = profile.join(project.path);
                                write(&path, &*file, &permit).await?;
                                println!("finished downloading {:?}", path);
                            }
                            _ => {}
                        };
                    }

                    Ok(())
                }
            })
            .await?;

        // for i in 0..zip.len() {
        //     let mut file = zip.by_index(i).unwrap();
        //
        //     let file_path = file.mangled_name();
        //     println!("{:?}", file_path);
        //     if file_path.starts_with("overrides") && !file_path.ends_with("/") {
        //         let mut content = Vec::new();
        //         file.read_to_end(&mut content)?;
        //     }
        // }

        Ok(profile)
    } else {
        Err(crate::Error::from(crate::ErrorKind::InputError(
            "No pack manifest found in mrpack".to_string(),
        )))
    }

    //todo: env checking for projects
}
