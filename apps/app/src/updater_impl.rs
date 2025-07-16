use crate::api::Result;
use std::sync::{Arc, Mutex};
use tauri::http::HeaderValue;
use tauri::http::header::ACCEPT;
use tauri::{Manager, ResourceId, Runtime, Webview};
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::ClientBuilder;
use tauri_plugin_updater::Error;
use tauri_plugin_updater::Update;
use theseus::{
    LAUNCHER_USER_AGENT, LoadingBarType, emit_loading, init_loading,
};
use tokio::time::Instant;

#[derive(Default)]
pub struct PendingUpdateData(pub Mutex<Option<(Arc<Update>, Vec<u8>)>>);

// Reimplementation of Update::download mostly, minus the actual download part
#[tauri::command]
pub async fn get_update_size<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
) -> Result<Option<u64>> {
    let update = webview.resources_table().get::<Update>(rid)?;

    let mut headers = update.headers.clone();
    if !headers.contains_key(ACCEPT) {
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/octet-stream"),
        );
    }

    let mut request = ClientBuilder::new().user_agent(LAUNCHER_USER_AGENT);
    if let Some(timeout) = update.timeout {
        request = request.timeout(timeout);
    }
    if let Some(ref proxy) = update.proxy {
        let proxy = reqwest::Proxy::all(proxy.as_str())?;
        request = request.proxy(proxy);
    }
    let response = request
        .build()?
        .head(update.download_url.clone())
        .headers(headers)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Error::Network(format!(
            "Download request failed with status: {}",
            response.status()
        ))
        .into());
    }

    let content_length = response
        .headers()
        .get("Content-Length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok());

    Ok(content_length)
}

#[tauri::command]
pub async fn enqueue_update_for_installation<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
) -> Result<()> {
    let pending_data = webview.state::<PendingUpdateData>().inner();

    let update = webview.resources_table().get::<Update>(rid)?;

    let progress = init_loading(
        LoadingBarType::LauncherUpdate {
            version: update.version.clone(),
            current_version: update.current_version.clone(),
        },
        1.0,
        "Downloading update...",
    )
    .await?;

    let download_start = Instant::now();
    let update_data = update
        .download(
            |chunk_size, total_size| {
                let Some(total_size) = total_size else {
                    return;
                };
                if let Err(e) = emit_loading(
                    &progress,
                    chunk_size as f64 / total_size as f64,
                    None,
                ) {
                    tracing::error!(
                        "Failed to update download progress bar: {e}"
                    );
                }
            },
            || {},
        )
        .await?;
    let download_duration = download_start.elapsed();
    tracing::info!("Downloaded update in {download_duration:?}");

    pending_data
        .0
        .lock()
        .unwrap()
        .replace((update, update_data));

    Ok(())
}

#[tauri::command]
pub fn remove_enqueued_update<R: Runtime>(webview: Webview<R>) {
    let pending_data = webview.state::<PendingUpdateData>().inner();
    pending_data.0.lock().unwrap().take();
}
