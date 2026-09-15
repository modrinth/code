use crate::api::Result;
use async_zip::base::read::seek::ZipFileReader;
use serde::Serialize;
use std::io::Cursor;
use tauri::Runtime;
use tauri_plugin_dialog::DialogExt;

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("files")
        .invoke_handler(tauri::generate_handler![
            file_extract_zip,
            file_save_as,
            file_read_dragged_file,
            file_list,
            file_read,
            file_write,
            file_create_directory,
            file_rename,
            file_delete,
        ])
        .build()
}

#[derive(Serialize)]
pub struct ExtractDryRunResult {
    modpack_name: Option<String>,
    conflicting_files: Vec<String>,
}

#[tauri::command]
pub async fn file_read_dragged_file(path: String) -> Result<Vec<u8>> {
    let metadata = tokio::fs::metadata(&path).await?;
    if !metadata.is_file() {
        return Err(theseus::Error::from(theseus::ErrorKind::OtherError(
            "Dropped path is not a file".to_string(),
        ))
        .into());
    }

    Ok(tokio::fs::read(path).await?)
}

#[tauri::command]
pub async fn file_extract_zip(
    instance_id: &str,
    file_path: &str,
    override_conflicts: bool,
    dry_run: bool,
) -> Result<Option<ExtractDryRunResult>> {
    theseus::instance::validate_instance_file_write(instance_id, file_path)
        .await?;
    let parent = file_path
        .trim_start_matches('/')
        .rsplit_once('/')
        .map_or("", |(parent, _)| parent);
    let file_bytes =
        theseus::instance::read_instance_file(instance_id, file_path).await?;
    let zip_reader = ZipFileReader::with_tokio(Cursor::new(file_bytes))
        .await
        .map_err(|error| {
        theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Failed to read zip file: {error}"
        )))
    })?;
    let mut entries = Vec::new();
    for (index, entry) in zip_reader.file().entries().iter().enumerate() {
        let name = entry.filename().as_str().map_err(|error| {
            theseus::Error::from(theseus::ErrorKind::InputError(
                error.to_string(),
            ))
        })?;
        if name.ends_with('/') {
            continue;
        }
        if name.starts_with('/') || name.contains('\\') {
            return Err(theseus::Error::from(theseus::ErrorKind::InputError(
                "Invalid archive path".to_string(),
            ))
            .into());
        }
        let target = if parent.is_empty() {
            name.to_string()
        } else {
            format!("{parent}/{name}")
        };
        let resolved = theseus::instance::validate_instance_file_write(
            instance_id,
            &target,
        )
        .await?;
        entries.push((index, target, resolved));
    }
    if dry_run {
        let conflicting_files = entries
            .iter()
            .filter(|(_, _, path)| path.exists())
            .map(|(_, name, _)| name.clone())
            .collect();
        return Ok(Some(ExtractDryRunResult {
            modpack_name: None,
            conflicting_files,
        }));
    }
    let mut zip_reader = zip_reader;
    for (index, path, resolved) in entries {
        if !override_conflicts && resolved.exists() {
            continue;
        }
        let mut bytes = Vec::new();
        let mut reader =
            zip_reader.reader_with_entry(index).await.map_err(|error| {
                theseus::Error::from(theseus::ErrorKind::OtherError(
                    error.to_string(),
                ))
            })?;
        reader
            .read_to_end_checked(&mut bytes)
            .await
            .map_err(|error| {
                theseus::Error::from(theseus::ErrorKind::OtherError(
                    error.to_string(),
                ))
            })?;
        theseus::instance::write_instance_file(
            instance_id,
            &path,
            &bytes,
            !override_conflicts,
        )
        .await?;
    }
    Ok(None)
}

#[tauri::command]
pub async fn file_list(
    instance_id: &str,
    path: &str,
) -> Result<Vec<theseus::instance::InstanceFileItem>> {
    Ok(theseus::instance::list_instance_files(instance_id, path).await?)
}

#[tauri::command]
pub async fn file_read(instance_id: &str, path: &str) -> Result<Vec<u8>> {
    Ok(theseus::instance::read_instance_file(instance_id, path).await?)
}

#[tauri::command]
pub async fn file_write(
    instance_id: &str,
    path: &str,
    bytes: Vec<u8>,
    create_only: bool,
) -> Result<()> {
    Ok(theseus::instance::write_instance_file(
        instance_id,
        path,
        &bytes,
        create_only,
    )
    .await?)
}

#[tauri::command]
pub async fn file_create_directory(
    instance_id: &str,
    path: &str,
) -> Result<()> {
    Ok(theseus::instance::create_instance_directory(instance_id, path).await?)
}

#[tauri::command]
pub async fn file_rename(
    instance_id: &str,
    source: &str,
    destination: &str,
) -> Result<()> {
    Ok(theseus::instance::rename_instance_file(
        instance_id,
        source,
        destination,
    )
    .await?)
}

#[tauri::command]
pub async fn file_delete(
    instance_id: &str,
    path: &str,
    recursive: bool,
) -> Result<()> {
    Ok(
        theseus::instance::delete_instance_file(instance_id, path, recursive)
            .await?,
    )
}

#[tauri::command]
pub async fn file_save_as<R: Runtime>(
    app: tauri::AppHandle<R>,
    instance_id: &str,
    file_path: &str,
) -> Result<()> {
    let source = std::path::Path::new(file_path);
    let file_name = source
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_file_name(&file_name)
        .save_file(|path| {
            let _ = tx.send(path);
        });

    if let Some(dest) = rx.await.unwrap_or(None) {
        let dest_path = std::path::PathBuf::try_from(dest).map_err(|e| {
            theseus::Error::from(theseus::ErrorKind::OtherError(format!(
                "Invalid save path: {e}"
            )))
        })?;
        theseus::instance::save_instance_file_as(
            instance_id,
            file_path,
            &dest_path,
        )
        .await?;
    }

    Ok(())
}
