use crate::launcher::meta::{
    Asset, AssetIndex, AssetsIndex, DownloadType, Library, Os, OsRule, RuleAction, VersionInfo,
};
use futures::future;
use regex::Regex;
use reqwest::{Error, Response};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

pub async fn download_client(client_path: &Path, version_info: &VersionInfo) {
    let client = download_file(
        &version_info
            .downloads
            .get(&DownloadType::Client)
            .unwrap()
            .url,
    )
    .await;

    save_file(
        &*client_path
            .join(&version_info.id)
            .join(format!("{}.jar", &version_info.id)),
        &client,
    );
    save_file(
        &*client_path
            .join(&version_info.id)
            .join(format!("{}.json", &version_info.id)),
        &bytes::Bytes::from(serde_json::to_string(version_info).unwrap()),
    );
}

pub async fn download_assets(
    assets_path: &Path,
    legacy_path: Option<&Path>,
    meta: &AssetIndex,
    index: &AssetsIndex,
) {
    save_file(
        &*assets_path
            .join("indexes")
            .join(format!("{}.json", meta.id)),
        &bytes::Bytes::from(serde_json::to_string(index).unwrap()),
    );

    future::join_all(
        index
            .objects
            .iter()
            .map(|x| download_asset(assets_path, legacy_path, x.0, x.1)),
    )
    .await;
}

async fn download_asset(
    assets_path: &Path,
    legacy_path: Option<&Path>,
    name: &String,
    asset: &Asset,
) {
    let sub_hash = &&asset.hash[..2];

    let resource = download_file(&format!(
        "https://resources.download.minecraft.net/{}/{}",
        sub_hash, asset.hash
    ))
    .await;

    let resource_path = assets_path.join("objects").join(sub_hash).join(&asset.hash);
    save_file(resource_path.as_path(), &resource);

    if let Some(legacy_path) = legacy_path {
        let resource_path =
            legacy_path.join(name.replace('/', &*std::path::MAIN_SEPARATOR.to_string()));
        save_file(resource_path.as_path(), &resource);
    }
}

pub async fn download_libraries(libraries_path: &Path, natives_path: &Path, libraries: &[Library]) {
    future::join_all(
        libraries
            .iter()
            .map(|x| download_library(libraries_path, natives_path, x)),
    )
    .await;
}

async fn download_library(libraries_path: &Path, natives_path: &Path, library: &Library) {
    if let Some(rules) = &library.rules {
        if !super::rules::parse_rules(rules.as_slice()) {
            return;
        }
    }

    let name_items = library.name.split(':').collect::<Vec<&str>>();

    let package = name_items.get(0).unwrap();
    let name = name_items.get(1).unwrap();
    let version = name_items.get(2).unwrap();

    future::join(
        download_library_jar(libraries_path, library, package, name, version),
        download_native(
            libraries_path,
            natives_path,
            library,
            package,
            name,
            version,
        ),
    )
    .await;
}

async fn download_library_jar(
    libraries_path: &Path,
    library: &Library,
    package: &str,
    name: &str,
    version: &str,
) {
    if let Some(library) = &library.downloads.artifact {
        let bytes = download_file(&library.url).await;

        let mut path = libraries_path.to_path_buf();

        for directory in package.split(".") {
            path.push(directory);
        }

        path.push(name);
        path.push(version);
        path.push(format!("{}-{}.jar", name, version));

        save_file(&path, &bytes);
    }
}

async fn download_native(
    libraries_path: &Path,
    natives_path: &Path,
    library: &Library,
    package: &str,
    name: &str,
    version: &str,
) {
    if let Some(natives) = &library.natives {
        if let Some(os_key) = natives.get(&get_os()) {
            if let Some(classifiers) = &library.downloads.classifiers {
                #[cfg(target_pointer_width = "64")]
                let parsed_key = os_key.replace("${arch}", "64");
                #[cfg(target_pointer_width = "32")]
                let parsed_key = os_key.replace("${arch}", "32");

                if let Some(native) = classifiers.get(&*parsed_key) {
                    let mut path = libraries_path.to_path_buf();

                    for directory in package.split(".") {
                        path.push(directory);
                    }

                    path.push(name);
                    path.push(version);
                    path.push(format!("{}-{}-{}.jar", name, version, parsed_key));

                    let bytes = download_file(&native.url).await;

                    save_file(&path, &bytes);

                    let file = File::open(&path).unwrap();
                    let reader = BufReader::new(file);

                    let mut archive = zip::ZipArchive::new(reader).unwrap();
                    archive.extract(natives_path).unwrap();
                }
            }
        }
    }
}

fn save_file(path: &Path, bytes: &bytes::Bytes) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut file = File::create(path).unwrap();
    file.write_all(bytes).unwrap();
}

async fn download_file(url: &str) -> bytes::Bytes {
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .tcp_keepalive(Some(std::time::Duration::from_secs(10)))
        .build()
        .unwrap();

    for attempt in 1..4 {
        let result = client.get(url).send().await;

        match result {
            Ok(x) => return x.bytes().await.unwrap(),
            Err(e) if attempt <= 3 => continue,
            Err(e) => panic!(e),
        }
    }

    unreachable!()
}

pub fn get_os() -> Os {
    match std::env::consts::OS {
        "windows" => Os::Windows,
        "macos" => Os::Osx,
        "linux" => Os::Linux,
        _ => Os::Unknown,
    }
}
