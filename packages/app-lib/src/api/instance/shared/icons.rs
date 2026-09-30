use super::client::{get_remote_server_icon, shared_instances_request_error};
use super::*;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct CachedServerIcon {
    filename: String,
    etag: Option<String>,
}

#[tracing::instrument]
pub async fn cache_shared_instance_server_icon(
    instance_id: &str,
) -> crate::Result<Option<String>> {
    let state = State::get().await?;
    shared_instance_server_icon(instance_id, &state).await
}

#[tracing::instrument(skip(state))]
pub(super) async fn shared_instance_server_icon(
    instance_id: &str,
    state: &State,
) -> crate::Result<Option<String>> {
    let credentials =
        ModrinthCredentials::get_and_refresh(&state.pool, &state.api_semaphore)
            .await?
            .ok_or(crate::ErrorKind::NoCredentialsError)?;
    let key = sha1_smol::Sha1::from(
        format!("{}:{instance_id}", credentials.user_id).as_bytes(),
    )
    .hexdigest();
    let icons = state.directories.caches_dir().join("icons");
    let mapping = icons.join(format!("shared-server-{key}.json"));
    let cached = if tokio::fs::metadata(&mapping)
        .await
        .is_ok_and(|metadata| metadata.len() <= 1024)
    {
        tokio::fs::read(&mapping).await.ok().and_then(|bytes| {
            serde_json::from_slice::<CachedServerIcon>(&bytes).ok()
        })
    } else {
        None
    };
    let cached = match cached {
        Some(cached)
            if cached.filename.len() == 44
                && cached.filename.ends_with(".png")
                && cached.filename.as_bytes()[..40]
                    .iter()
                    .all(u8::is_ascii_hexdigit)
                && tokio::fs::try_exists(icons.join(&cached.filename))
                    .await? =>
        {
            Some(cached)
        }
        _ => None,
    };
    let mut response = get_remote_server_icon(
        instance_id,
        cached.as_ref().and_then(|cached| cached.etag.as_deref()),
        state,
    )
    .await?;
    if response.status() == StatusCode::NOT_FOUND {
        if tokio::fs::try_exists(&mapping).await? {
            tokio::fs::remove_file(&mapping).await?;
        }
        return Ok(None);
    }
    if response.status() == StatusCode::NOT_MODIFIED
        && let Some(cached) = cached
    {
        return Ok(Some(
            crate::util::io::canonicalize(icons.join(cached.filename))?
                .to_string_lossy()
                .to_string(),
        ));
    }
    if !response.status().is_success() {
        return shared_instances_request_error(
            "get_instance_server_icon",
            Method::GET,
            &format!("/instances/{instance_id}/server-icon"),
            response,
        )
        .await;
    }
    let etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > 4 * 1024 * 1024 {
            return Err(crate::ErrorKind::InputError(
                "Server icons must be at most 4 MiB".to_string(),
            )
            .into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let path: PathBuf =
        crate::api::instance::cache_icon(bytes::Bytes::from(bytes), state)
            .await?;
    let filename = path
        .file_name()
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "Cached server icon has no filename".to_string(),
            )
        })?
        .to_string_lossy()
        .to_string();
    crate::util::fetch::write(
        &mapping,
        &serde_json::to_vec(&CachedServerIcon { filename, etag })?,
        &state.io_semaphore,
    )
    .await?;
    Ok(Some(path.to_string_lossy().to_string()))
}
