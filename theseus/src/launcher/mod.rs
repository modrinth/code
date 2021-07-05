use std::path::Path;
use std::process::{Command, Stdio};

mod args;
pub mod auth;
pub mod download;
pub mod java;
pub mod meta;
mod rules;

pub async fn launch_minecraft(version_name: &str, root_dir: &Path) {
    let manifest = meta::fetch_version_manifest().await.unwrap();

    let version = meta::fetch_version_info(
        manifest
            .versions
            .iter()
            .find(|x| x.id == version_name)
            .unwrap(),
    )
    .await
    .unwrap();

    //download_minecraft(&version, root_dir).await;

    let auth = auth::login("username", "password", true).await;

    let arguments = version.arguments.unwrap();

    let profile = auth.selected_profile.unwrap();

    let mut child = Command::new("java")
        .args(args::get_jvm_arguments(
            arguments
                .get(&meta::ArgumentType::Jvm)
                .map(|x| x.as_slice()),
            &*root_dir.join("natives").join(&version.id),
            &*args::get_class_paths(
                &*root_dir.join("libraries"),
                version.libraries.as_slice(),
                &*root_dir
                    .join("versions")
                    .join(&version.id)
                    .join(format!("{}.jar", &version.id)),
            ),
        ))
        .arg(version.main_class)
        .args(args::get_minecraft_arguments(
            arguments
                .get(&meta::ArgumentType::Game)
                .map(|x| x.as_slice()),
            version.minecraft_arguments.as_deref(),
            &*auth.access_token,
            &*profile.name,
            &profile.id,
            &*version.id,
            &version.asset_index.id,
            root_dir,
            &*root_dir.join("assets"),
            &version.type_,
        ))
        .current_dir(root_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    child.wait().unwrap();
}

pub async fn download_minecraft(version: &meta::VersionInfo, root_dir: &Path) {
    let assets_dir = meta::fetch_assets_index(&version).await.unwrap();

    let legacy_dir = root_dir.join("resources");

    futures::future::join3(
        download::download_client(&*root_dir.join("versions"), &version),
        download::download_assets(
            &*root_dir.join("assets"),
            if version.assets == "legacy" {
                Some(legacy_dir.as_path())
            } else {
                None
            },
            &version.asset_index,
            &assets_dir,
        ),
        download::download_libraries(
            &*root_dir.join("libraries"),
            &*root_dir.join("natives").join(&version.id),
            version.libraries.as_slice(),
        ),
    )
    .await;
}
