use crate::api::Result;
use async_zip::base::read::seek::ZipFileReader;
use serde::Serialize;
use std::io::Cursor;
use tauri::Runtime;
use tauri_plugin_dialog::DialogExt;
use theseus::profile::get_full_path;

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("files")
        .invoke_handler(tauri::generate_handler![
            file_extract_zip,
            file_save_as,
        ])
        .build()
}

#[derive(Serialize)]
pub struct ExtractDryRunResult {
    modpack_name: Option<String>,
    conflicting_files: Vec<String>,
}

#[tauri::command]
pub async fn file_extract_zip(
    instance_path: &str,
    file_path: &str,
    override_conflicts: bool,
    dry_run: bool,
) -> Result<Option<ExtractDryRunResult>> {
    let base = get_full_path(instance_path).await?;
    let zip_path = base.join(file_path);
    let extract_dir = zip_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| base.clone());

    let file_bytes = tokio::fs::read(&zip_path).await?;
    let reader = Cursor::new(file_bytes);

    let zip_reader = ZipFileReader::with_tokio(reader).await.map_err(|e| {
        theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Failed to read zip file: {e}"
        )))
    })?;

    let entries: Vec<(usize, String)> = zip_reader
        .file()
        .entries()
        .iter()
        .enumerate()
        .filter_map(|(i, entry)| {
            let name = entry.filename().as_str().ok()?.to_string();
            if name.ends_with('/') {
                None
            } else {
                Some((i, name))
            }
        })
        .collect();

    if dry_run {
        let mut conflicting_files = Vec::new();
        for (_, name) in &entries {
            let target = extract_dir.join(name);
            if target.exists() {
                conflicting_files.push(name.clone());
            }
        }
        return Ok(Some(ExtractDryRunResult {
            modpack_name: None,
            conflicting_files,
        }));
    }

    let mut zip_reader = zip_reader;
    for (index, name) in &entries {
        let target = extract_dir.join(name);

        if !override_conflicts && target.exists() {
            continue;
        }

        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file_bytes = Vec::new();
        let mut entry_reader =
            zip_reader.reader_with_entry(*index).await.map_err(|e| {
                theseus::Error::from(theseus::ErrorKind::OtherError(format!(
                    "Failed to read zip entry: {e}"
                )))
            })?;
        entry_reader
            .read_to_end_checked(&mut file_bytes)
            .await
            .map_err(|e| {
                theseus::Error::from(theseus::ErrorKind::OtherError(format!(
                    "Failed to extract zip entry: {e}"
                )))
            })?;

        tokio::fs::write(&target, &file_bytes).await?;
    }

    Ok(None)
}

#[tauri::command]
pub async fn file_save_as<R: Runtime>(
    app: tauri::AppHandle<R>,
    instance_path: &str,
    file_path: &str,
) -> Result<()> {
    let base = get_full_path(instance_path).await?;
    let source = base.join(file_path);
    let file_name = source
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let dest = app
        .dialog()
        .file()
        .set_file_name(&file_name)
        .blocking_save_file();

    if let Some(dest) = dest {
        let dest_path = std::path::PathBuf::try_from(dest).map_err(|e| {
            theseus::Error::from(theseus::ErrorKind::OtherError(format!(
                "Invalid save path: {e}"
            )))
        })?;
        tokio::fs::copy(&source, &dest_path).await?;
    }

    Ok(())
}
