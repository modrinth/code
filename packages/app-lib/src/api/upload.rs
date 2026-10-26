//! Uploads of local files to HTTP endpoints, streamed from disk.

use crate::util::fetch::INSECURE_NO_TIMEOUT_REQWEST_CLIENT;
use crate::util::io::IOError;
use crate::{ErrorKind, Result};
use futures::TryStreamExt;
use reqwest::header::CONTENT_LENGTH;
use reqwest::multipart::{Form, Part};
use std::path::PathBuf;
use tokio_util::io::ReaderStream;

const CHUNK_SIZE: usize = 64 * 1024;

/// A file on disk and the request to upload it with.
pub struct LocalFileUpload {
    pub url: url::Url,
    pub headers: Vec<(String, String)>,
    pub path: PathBuf,
    /// The file name sent with a multipart upload, the name of `path` by default.
    pub file_name: Option<String>,
    /// Sends the file as this multipart form field rather than as the raw request body.
    pub multipart_field: Option<String>,
}

/// The response to an upload, whatever its status.
pub struct UploadResponse {
    pub status: u16,
    pub body: String,
}

/// Uploads a local file without reading it into memory, calling `on_progress` with the bytes
/// sent so far and the size of the file.
pub async fn upload_local_file(
    upload: LocalFileUpload,
    mut on_progress: impl FnMut(u64, u64) + Send + 'static,
) -> Result<UploadResponse> {
    if !matches!(upload.url.scheme(), "http" | "https") {
        return Err(ErrorKind::InputError(format!(
            "Unsupported upload URL scheme: {}",
            upload.url.scheme()
        ))
        .into());
    }

    let file = tokio::fs::File::open(&upload.path)
        .await
        .map_err(|e| IOError::with_path(e, &upload.path))?;
    let metadata = file
        .metadata()
        .await
        .map_err(|e| IOError::with_path(e, &upload.path))?;
    if !metadata.is_file() {
        return Err(ErrorKind::InputError(format!(
            "{} is not a file",
            upload.path.display()
        ))
        .into());
    }

    let total = metadata.len();
    let mut loaded = 0;
    on_progress(loaded, total);
    let stream = ReaderStream::with_capacity(file, CHUNK_SIZE).inspect_ok(
        move |chunk| {
            loaded += chunk.len() as u64;
            on_progress(loaded, total);
        },
    );
    let body = reqwest::Body::wrap_stream(stream);

    let mut request = INSECURE_NO_TIMEOUT_REQWEST_CLIENT.post(upload.url);
    for (name, value) in upload.headers {
        request = request.header(name, value);
    }
    request = match upload.multipart_field {
        Some(field) => {
            let file_name = upload.file_name.unwrap_or_else(|| {
                upload
                    .path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "file".to_string())
            });
            let part =
                Part::stream_with_length(body, total).file_name(file_name);
            request.multipart(Form::new().part(field, part))
        }
        None => request.header(CONTENT_LENGTH, total).body(body),
    };

    let response = request.send().await?;
    let status = response.status().as_u16();
    let body = response.text().await?;
    Ok(UploadResponse { status, body })
}
