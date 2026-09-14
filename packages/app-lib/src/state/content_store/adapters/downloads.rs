use crate::util::fetch::{
    self, DownloadMeta, DownloadedFile, FetchProgressFn, FetchSemaphore,
};
use sqlx::SqlitePool;
use std::path::Path;

pub(in crate::state::content_store) async fn download(
    staging: &Path,
    pool: &SqlitePool,
    mirrors: &[&str],
    metadata: Option<&DownloadMeta>,
    semaphore: &FetchSemaphore,
    progress: Option<&mut FetchProgressFn<'_>>,
) -> crate::Result<DownloadedFile> {
    fetch::fetch_file_mirrors_in(
        mirrors,
        None,
        metadata,
        None,
        semaphore,
        pool,
        progress,
        Some(staging),
    )
    .await
}

pub(in crate::state::content_store) async fn repair_sources(
    pool: &SqlitePool,
    sha512: &str,
    state: &crate::State,
) -> Vec<String> {
    let version = fetch::fetch_json::<crate::state::Version>(
        reqwest::Method::GET,
        &format!(
            "{}version_file/{sha512}?algorithm=sha512",
            env!("MODRINTH_API_URL")
        ),
        None,
        None,
        Some("/v2/version_file/:hash"),
        &state.api_semaphore,
        pool,
    )
    .await;
    match version {
        Ok(version) => version
            .files
            .into_iter()
            .filter(|file| {
                file.hashes.get("sha512").map(String::as_str) == Some(sha512)
            })
            .map(|file| file.url)
            .collect(),
        Err(_) => Vec::new(),
    }
}
