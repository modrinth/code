use crate::api::Result;
use std::path::{Path, PathBuf};
use tauri::Runtime;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
use linux::{create_shortcut, SHORTCUT_EXTENSION};
#[cfg(target_os = "macos")]
use macos::{create_shortcut, SHORTCUT_EXTENSION};
#[cfg(target_os = "windows")]
use windows::{create_shortcut, SHORTCUT_EXTENSION};

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
	tauri::plugin::Builder::new("shortcuts")
		.invoke_handler(tauri::generate_handler![create_profile_shortcut])
		.build()
}

#[tauri::command]
pub async fn create_profile_shortcut(
	profile_name: String,
	profile_path: String,
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

	let mut launch_url = format!(
		"modrinth://launch/profile/{}",
		urlencoding::encode(&profile_path)
	);
	if let Some(server) = server {
		launch_url.push_str("?server=");
		launch_url.push_str(&urlencoding::encode(&server));
	} else if let Some(singleplayer_world) = singleplayer_world {
		launch_url.push_str("?singleplayer_world=");
		launch_url.push_str(&urlencoding::encode(&singleplayer_world));
	}

	let output_path = shortcut_path_with_extension(output_path);
	let output_path_existed =
		tokio::fs::try_exists(&output_path).await.unwrap_or(false);

	if let Err(error) =
		create_shortcut(&profile_name, &launch_url, &output_path).await
	{
		cleanup_shortcut_artifact(&output_path, output_path_existed).await;
		return Err(error);
	}

	Ok(output_path)
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
