use std::io;

const UPDATER_SEPARATOR: &str = "-updater-";
const UPDATER_RANDOM_SUFFIX_LENGTH: usize = 6;

pub async fn cleanup_updater_temp_folders(
	app_name: &str,
) -> io::Result<usize> {
	let mut entries = tokio::fs::read_dir(std::env::temp_dir()).await?;
	let mut removed = 0;

	while let Some(entry) = entries.next_entry().await? {
		let file_name = entry.file_name();
		let Some(file_name) = file_name.to_str() else {
			continue;
		};

		if !is_updater_temp_folder(app_name, file_name)
			|| !entry.file_type().await?.is_dir()
		{
			continue;
		}

		let path = entry.path();
		match tokio::fs::remove_dir_all(&path).await {
			Ok(()) => removed += 1,
			Err(error) => {
				tracing::warn!(
					"Failed to clean up updater temp folder {}: {error}",
					path.display()
				);
			}
		}
	}

	Ok(removed)
}

fn is_updater_temp_folder(app_name: &str, file_name: &str) -> bool {
	let Some(name) = file_name
		.strip_prefix(app_name)
		.and_then(|name| name.strip_prefix('-'))
	else {
		return false;
	};
	let Some((version, random_suffix)) =
		name.rsplit_once(UPDATER_SEPARATOR)
	else {
		return false;
	};

	!version.is_empty()
		&& version
			.bytes()
			.all(|byte| byte.is_ascii_alphanumeric() || b".-+".contains(&byte))
		&& random_suffix.len() == UPDATER_RANDOM_SUFFIX_LENGTH
		&& random_suffix.bytes().all(|byte| byte.is_ascii_alphanumeric())
}
