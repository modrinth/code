use crate::{format_url, upload_file_to_bucket, Error};
use semver::{VersionReq, Version};
use lazy_static::lazy_static;
use daedalus::download_file;
use std::io::Read;
use tokio::sync::{Mutex};
use std::sync::{Arc};
use daedalus::minecraft::{Library, VersionType, ArgumentType, Argument};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use daedalus::fabric::PartialVersionInfo;
use std::time::{Instant, Duration};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ForgeInstallerProfileInstallDataV1 {
    pub mirror_list: String,
    pub target: String,
    /// Path to the Forge universal library
    pub file_path: String,
    pub logo: String,
    pub welcome: String,
    pub version: String,
    /// Maven coordinates of the Forge universal library
    pub path: String,
    pub profile_name: String,
    pub minecraft: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ForgeInstallerProfileManifestV1 {
    pub id: String,
    pub libraries: Vec<Library>,
    pub main_class: Option<String>,
    pub minecraft_arguments: Option<String>,
    pub release_time: DateTime<Utc>,
    pub time: DateTime<Utc>,
    pub type_: VersionType,
    pub assets: Option<String>,
    pub inherits_from: Option<String>,
    pub jar: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ForgeInstallerProfileV1 {
    pub install: ForgeInstallerProfileInstallDataV1,
    pub version_info: ForgeInstallerProfileManifestV1,
}

lazy_static! {
    static ref FORGE_MANIFEST_V1_QUERY: VersionReq = VersionReq::parse(">=8.0.684, <23.5.2851").unwrap();
}

pub async fn retrieve_data() -> Result<(), Error> {
    let maven_metadata = daedalus::forge::fetch_maven_metadata(None).await?;

    let visited_assets_mutex = Arc::new(Mutex::new(Vec::new()));

    let mut version_futures = Vec::new();

    for (minecraft_version, loader_versions) in maven_metadata {
        if let Some(loader_version_full) = loader_versions.into_iter().last() {
            let loader_version = loader_version_full.split('-').into_iter().nth(1);

            if let Some(loader_version_raw) = loader_version {
                // This is a dirty hack to get around Forge not complying with SemVer, but whatever
                // Most of this is a hack anyways :(
                // Works for all forge versions!
                let split = loader_version_raw.split('.').collect::<Vec<&str>>();
                let loader_version =if split.len() >= 4 {
                    if split[0].parse::<i32>().unwrap() < 6 {
                        format!("{}.{}.{}", split[0], split[1], split[3])
                    } else {
                        format!("{}.{}.{}", split[1], split[2], split[3])
                    }
                } else {
                    loader_version_raw.to_string()
                };

                if FORGE_MANIFEST_V1_QUERY.matches(&Version::parse(&*loader_version).unwrap()) {
                    version_futures.push(async {
                        let visited_assets = Arc::clone(&visited_assets_mutex);
                        async move {
                            println!("installer start {}", loader_version_full.clone());
                            let bytes = download_file(&*format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{0}/forge-{0}-installer.jar", loader_version_full), None).await.unwrap();

                            let reader = std::io::Cursor::new(&*bytes);

                            if let Ok(mut archive) = zip::ZipArchive::new(reader) {
                                let install_profile = {
                                    let mut install_profile = archive.by_name("install_profile.json").unwrap();

                                    let mut contents = String::new();
                                    install_profile.read_to_string(&mut contents).unwrap();

                                    contents
                                };

                                let profile = serde_json::from_str::<ForgeInstallerProfileV1>(&*install_profile).unwrap();

                                let forge_universal_bytes = {
                                    let mut forge_universal_file = archive.by_name(&*profile.install.file_path).unwrap();
                                    let mut forge_universal =  Vec::new();
                                    forge_universal_file.read_to_end(&mut forge_universal).unwrap();

                                    bytes::Bytes::from(forge_universal)
                                };
                                let forge_universal_path = profile.install.file_path.clone();

                                let now = Instant::now();
                                let libs = futures::future::try_join_all(profile.version_info.libraries.into_iter().map(|mut lib| async {
                                    if let Some(url) = lib.url {
                                        {
                                            let mut visited_assets = visited_assets.lock().await;

                                            if visited_assets.contains(&lib.name) {
                                                lib.url = Some(format_url("maven/"));

                                                return Ok::<Library, Error>(lib);
                                            } else {
                                                visited_assets.push(lib.name.clone())
                                            }
                                        }

                                        let artifact_path =
                                            daedalus::get_path_from_artifact(&*lib.name)?;

                                        let artifact = if lib.name == forge_universal_path {
                                            forge_universal_bytes.clone()
                                        } else {
                                            let mirrors = vec![&*url, "https://maven.creeperhost.net/", "https://libraries.minecraft.net/"];

                                            daedalus::download_file_mirrors(
                                                &*artifact_path,
                                                &mirrors,
                                                None,
                                            )
                                                .await?
                                        };

                                        lib.url = Some(format_url("maven/"));

                                        upload_file_to_bucket(
                                            format!("{}/{}", "maven", artifact_path),
                                            artifact.to_vec(),
                                            Some("application/java-archive".to_string()),
                                        ).await?;
                                    }

                                    Ok::<Library, Error>(lib)
                                })).await?;

                                let elapsed = now.elapsed();
                                println!("Elapsed lib DL: {:.2?}", elapsed);

                                let new_profile = PartialVersionInfo {
                                    id: profile.version_info.id,
                                    inherits_from: profile.install.minecraft,
                                    release_time: profile.version_info.release_time,
                                    time: profile.version_info.time,
                                    main_class: profile.version_info.main_class,
                                    arguments: profile.version_info.minecraft_arguments.map(|x| [(ArgumentType::Game, x.split(' ').map(|x| Argument::Normal(x.to_string())).collect())].iter().cloned().collect()),
                                    libraries: libs,
                                    type_: profile.version_info.type_,
                                };

                                let version_path = format!(
                                    "forge/v{}/versions/{}.json",
                                    daedalus::forge::CURRENT_FORMAT_VERSION,
                                    new_profile.id
                                );

                                upload_file_to_bucket(
                                    version_path.clone(),
                                    serde_json::to_vec(&new_profile)?,
                                    Some("application/json".to_string()),
                                ).await?;
                            }


                            Ok::<(), Error>(())
                        }.await?;

                        Ok::<(), Error>(())
                    });
                }
            }
        }
    }

    let mut versions = version_futures.into_iter().peekable();
    let mut chunk_index = 0;
    while versions.peek().is_some() {
        let now = Instant::now();

        let chunk: Vec<_> = versions.by_ref().take(100).collect();
        futures::future::try_join_all(chunk).await?;

        std::thread::sleep(Duration::from_secs(1));

        chunk_index += 1;

        let elapsed = now.elapsed();
        println!("Chunk {} Elapsed: {:.2?}", chunk_index, elapsed);
    }

    Ok(())
}