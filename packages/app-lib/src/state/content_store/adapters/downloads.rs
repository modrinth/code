use crate::util::fetch::{
    self, DownloadMeta, DownloadedFile, FetchProgressFn, FetchSemaphore,
};
use sqlx::SqlitePool;
use std::path::Path;

pub(in crate::state::content_store) async fn download(
    staging: &Path,
    pool: &SqlitePool,
    mirrors: &[&str],
    sha1: Option<&str>,
    metadata: Option<&DownloadMeta>,
    semaphore: &FetchSemaphore,
    progress: Option<&mut FetchProgressFn<'_>>,
) -> crate::Result<DownloadedFile> {
    fetch::fetch_file_mirrors_in(
        mirrors,
        sha1,
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
    sha1: &str,
    sha512: &str,
    state: &crate::State,
) -> Vec<String> {
    let mut sources = Vec::new();
    if let Ok(files) = crate::state::CachedEntry::get_file_many(
        &[sha1],
        None,
        pool,
        &state.api_semaphore,
    )
    .await
    {
        for file in files {
            if let Ok(Some(version)) = crate::state::CachedEntry::get_version(
                &file.version_id,
                None,
                pool,
                &state.api_semaphore,
            )
            .await
            {
                for candidate in version.files {
                    if candidate.hashes.get("sha512").map(String::as_str)
                        == Some(sha512)
                    {
                        sources.push(candidate.url);
                    }
                }
            }
        }
    }
    sources
}
