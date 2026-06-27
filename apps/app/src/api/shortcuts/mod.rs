use crate::api::Result;
use std::path::{Path, PathBuf};
use tauri::Runtime;
use url::Url;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
use linux::{SHORTCUT_EXTENSION, create_shortcut};
#[cfg(target_os = "macos")]
use macos::{SHORTCUT_EXTENSION, create_shortcut};
#[cfg(target_os = "windows")]
use windows::{SHORTCUT_EXTENSION, create_shortcut};

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("shortcuts")
        .invoke_handler(tauri::generate_handler![create_instance_shortcut])
        .build()
}

#[tauri::command]
pub async fn create_instance_shortcut(
    instance_name: String,
    instance_id: String,
    output_path: PathBuf,
    server: Option<String>,
    singleplayer_world: Option<String>,
) -> Result<PathBuf> {
    if server.is_some() && singleplayer_world.is_some() {
        return Err(std::io::Error::other(
            "shortcut cannot launch both a server and a singleplayer world",
        )
        .into());
    }

    let launch_url =
        instance_launch_url(instance_id, server, singleplayer_world);

    let output_path = shortcut_path_with_extension(output_path);
    let output_path_existed =
        tokio::fs::try_exists(&output_path).await.unwrap_or(false);

    if let Err(error) =
        create_shortcut(&instance_name, &launch_url, &output_path).await
    {
        cleanup_shortcut_artifact(&output_path, output_path_existed).await;
        return Err(error);
    }

    Ok(output_path)
}

fn instance_launch_url(
    instance_id: String,
    server: Option<String>,
    singleplayer_world: Option<String>,
) -> Url {
    let mut launch_url = Url::parse("modrinth://launch/instance")
        .expect("static launch URL should parse");

    launch_url
        .path_segments_mut()
        .expect("launch URL should support path segments")
        .push(&instance_id);

    if let Some(server) = server {
        launch_url.query_pairs_mut().append_pair("server", &server);
    } else if let Some(singleplayer_world) = singleplayer_world {
        launch_url
            .query_pairs_mut()
            .append_pair("singleplayer_world", &singleplayer_world);
    }

    launch_url
}

fn shortcut_path_with_extension(mut path: PathBuf) -> PathBuf {
    if path
        .extension()
        .is_none_or(|current_extension| current_extension != SHORTCUT_EXTENSION)
    {
        path.set_extension(SHORTCUT_EXTENSION);
    }

    path
}

async fn cleanup_shortcut_artifact(path: &Path, existed: bool) {
    if existed {
        return;
    }

    let result = match tokio::fs::metadata(path).await {
        Ok(metadata) if metadata.is_dir() => {
            tokio::fs::remove_dir_all(path).await
        }
        _ => tokio::fs::remove_file(path).await,
    };

    if let Err(error) = result
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(
            "failed to clean up shortcut artifact {}: {}",
            path.display(),
            error
        );
    }
}
