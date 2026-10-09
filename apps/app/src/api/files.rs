use crate::api::Result;
use async_zip::base::read::seek::ZipFileReader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{Manager, Runtime};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_http::reqwest;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("files")
        .setup(|app, _| {
            app.manage(ExternalSaveSessions::default());
            Ok(())
        })
        .on_event(|app, event| {
            if let tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::Destroyed,
                ..
            } = event
            {
                app.state::<ExternalSaveSessions>().0.retain(|_, session| {
                    if &session.window_label == label {
                        session.cancel.cancel();
                        false
                    } else {
                        true
                    }
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            file_extract_zip,
            file_save_as,
            files_select_external,
            files_save_external,
            files_release_external,
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

#[derive(Default)]
pub struct ExternalSaveSessions(
    dashmap::DashMap<String, Arc<ExternalSaveSession>>,
);

struct ExternalSaveSession {
    window_label: String,
    path: PathBuf,
    lock: tokio::sync::Mutex<()>,
    cancel: CancellationToken,
}

#[derive(Deserialize)]
pub struct ExternalSaveFilter {
    name: String,
    extensions: Vec<String>,
}

#[derive(Deserialize)]
pub struct ExternalSaveRequest {
    url: String,
    #[serde(default)]
    headers: HashMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSaveProgress {
    stage: &'static str,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSaveError {
    message: String,
    status_code: Option<u16>,
}

impl ExternalSaveError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status_code: None,
        }
    }
}

impl From<std::io::Error> for ExternalSaveError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<reqwest::Error> for ExternalSaveError {
    fn from(error: reqwest::Error) -> Self {
        Self {
            status_code: error.status().map(|status| status.as_u16()),
            message: error.without_url().to_string(),
        }
    }
}

fn external_save_session<R: Runtime>(
    window: &tauri::WebviewWindow<R>,
    state: &ExternalSaveSessions,
    save_id: &str,
) -> std::result::Result<Arc<ExternalSaveSession>, ExternalSaveError> {
    state
        .0
        .get(save_id)
        .filter(|session| session.window_label == window.label())
        .map(|session| Arc::clone(session.value()))
        .ok_or_else(|| ExternalSaveError::new("Unknown save session"))
}

#[tauri::command]
pub async fn files_select_external<R: Runtime>(
    window: tauri::WebviewWindow<R>,
    state: tauri::State<'_, ExternalSaveSessions>,
    filename: String,
    filter: Option<ExternalSaveFilter>,
) -> std::result::Result<Option<String>, ExternalSaveError> {
    let file_name = filename
        .rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("download");
    let mut dialog = window.dialog().file().set_file_name(file_name);
    if let Some(filter) = filter {
        let extensions: Vec<&str> =
            filter.extensions.iter().map(String::as_str).collect();
        dialog = dialog.add_filter(filter.name, &extensions);
    }
    let (tx, rx) = tokio::sync::oneshot::channel();
    dialog.save_file(|path| {
        let _ = tx.send(path);
    });
    let Some(destination) = rx.await.unwrap_or(None) else {
        return Ok(None);
    };
    let path = PathBuf::try_from(destination)
        .map_err(|error| ExternalSaveError::new(error.to_string()))?;
    let save_id = uuid::Uuid::new_v4().to_string();
    state.0.insert(
        save_id.clone(),
        Arc::new(ExternalSaveSession {
            window_label: window.label().to_string(),
            path,
            lock: tokio::sync::Mutex::new(()),
            cancel: CancellationToken::new(),
        }),
    );
    Ok(Some(save_id))
}

#[tauri::command]
pub async fn files_save_external<R: Runtime>(
    window: tauri::WebviewWindow<R>,
    state: tauri::State<'_, ExternalSaveSessions>,
    save_id: String,
    request: ExternalSaveRequest,
    on_progress: tauri::ipc::Channel<ExternalSaveProgress>,
) -> std::result::Result<String, ExternalSaveError> {
    let session = external_save_session(&window, &state, &save_id)?;
    let url = url::Url::parse(&request.url)
        .map_err(|_| ExternalSaveError::new("Invalid download URL"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(ExternalSaveError::new("Unsupported download URL"));
    }
    tokio::select! {
        _ = session.cancel.cancelled() => Err(ExternalSaveError::new("Download cancelled")),
        result = async {
            let _lock = session.lock.lock().await;
            let _ = on_progress.send(ExternalSaveProgress {
                stage: "waiting",
                downloaded_bytes: 0,
                total_bytes: None,
            });
            let client = reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(30))
                .read_timeout(Duration::from_secs(60))
                .build()?;
            let mut download = client.get(url);
            for (name, value) in request.headers {
                download = download.header(name, value);
            }
            let mut response = download.send().await?.error_for_status()?;
            let total_bytes = response.content_length();
            let mut downloaded_bytes = 0;
            let mut last_progress = Instant::now();
            let _ = on_progress.send(ExternalSaveProgress {
                stage: "downloading",
                downloaded_bytes,
                total_bytes,
            });
            let parent = session.path.parent().ok_or_else(|| {
                ExternalSaveError::new("Invalid save destination")
            })?;
            let temporary = tempfile::NamedTempFile::new_in(parent)?;
            let (file, temporary_path) = temporary.into_parts();
            let mut file = tokio::fs::File::from_std(file);
            while let Some(chunk) = response.chunk().await? {
                file.write_all(&chunk).await?;
                downloaded_bytes += chunk.len() as u64;
                if last_progress.elapsed() >= Duration::from_millis(200) {
                    let _ = on_progress.send(ExternalSaveProgress {
                        stage: "downloading",
                        downloaded_bytes,
                        total_bytes,
                    });
                    last_progress = Instant::now();
                }
            }
            let _ = on_progress.send(ExternalSaveProgress {
                stage: "saving",
                downloaded_bytes,
                total_bytes,
            });
            file.flush().await?;
            file.sync_all().await?;
            drop(file);
            temporary_path
                .persist(&session.path)
                .map_err(|error| error.error)?;
            Ok(session
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string())
        } => result,
    }
}

#[tauri::command]
pub async fn files_release_external<R: Runtime>(
    window: tauri::WebviewWindow<R>,
    state: tauri::State<'_, ExternalSaveSessions>,
    save_id: String,
) -> std::result::Result<(), ExternalSaveError> {
    let session = external_save_session(&window, &state, &save_id)?;
    session.cancel.cancel();
    state.0.remove(&save_id);
    Ok(())
}
