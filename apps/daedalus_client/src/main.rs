use crate::util::{
    REQWEST_CLIENT, format_url, upload_file_to_bucket,
    upload_url_to_bucket_mirrors,
};
use daedalus::get_path_from_artifact;
use dashmap::{DashMap, DashSet};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod error;
mod fabric;
mod forge;
mod minecraft;
pub mod util;

pub use error::{Error, ErrorKind, Result};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let subscriber = tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .with(ErrorLayer::default());

    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("Initialized tracing. Starting Daedalus!");

    if check_env_vars() {
        tracing::error!("Some environment variables are missing!");

        return Ok(());
    }

    let semaphore = Arc::new(Semaphore::new(
        dotenvy::var("CONCURRENCY_LIMIT")
            .ok()
            .and_then(|x| x.parse().ok())
            .unwrap_or(10),
    ));

    let mut fetch_result = FetchResult::default();

    match minecraft::fetch(semaphore.clone()).await {
        Ok(fetched) => merge_fetch_result(&mut fetch_result, fetched),
        Err(err) => tracing::warn!(error = %err, "Minecraft fetch failed"),
    }

    match fabric::fetch_fabric(semaphore.clone()).await {
        Ok(fetched) => merge_fetch_result(&mut fetch_result, fetched),
        Err(err) => tracing::warn!(error = %err, "Fabric fetch failed"),
    }

    match fabric::fetch_quilt(semaphore.clone()).await {
        Ok(fetched) => merge_fetch_result(&mut fetch_result, fetched),
        Err(err) => tracing::warn!(error = %err, "Quilt fetch failed"),
    }

    match forge::fetch_neo(semaphore.clone()).await {
        Ok(fetched) => merge_fetch_result(&mut fetch_result, fetched),
        Err(err) => tracing::warn!(error = %err, "NeoForge fetch failed"),
    }

    match forge::fetch_forge(semaphore.clone()).await {
        Ok(fetched) => merge_fetch_result(&mut fetch_result, fetched),
        Err(err) => tracing::warn!(error = %err, "Forge fetch failed"),
    }

    let FetchResult {
        upload_files,
        mirror_artifacts,
    } = fetch_result;

    futures::future::try_join_all(upload_files.iter().map(|entry| {
        upload_file_to_bucket(
            entry.key().clone(),
            entry.value().file.clone(),
            entry.value().content_type.clone(),
            &semaphore,
        )
    }))
    .await?;

    futures::future::try_join_all(mirror_artifacts.iter().map(|entry| {
        upload_url_to_bucket_mirrors(
            format!("maven/{}", entry.key()),
            entry
                .value()
                .mirrors
                .iter()
                .map(|mirror| {
                    if mirror.entire_url {
                        mirror.path.clone()
                    } else {
                        format!("{}{}", mirror.path, entry.key())
                    }
                })
                .collect(),
            entry.value().sha1.clone(),
            &semaphore,
        )
    }))
    .await?;

    if dotenvy::var("CLOUDFLARE_INTEGRATION")
        .ok()
        .and_then(|x| x.parse::<bool>().ok())
        .unwrap_or(false)
        && let Ok(token) = dotenvy::var("CLOUDFLARE_TOKEN")
        && let Ok(zone_id) = dotenvy::var("CLOUDFLARE_ZONE_ID")
    {
        let cache_clears = upload_files
            .into_iter()
            .map(|x| format_url(&x.0))
            .chain(
                mirror_artifacts
                    .into_iter()
                    .map(|x| format_url(&format!("maven/{}", x.0))),
            )
            .collect::<Vec<_>>();

        // Cloudflare ratelimits cache clears to 500 files per request
        for chunk in cache_clears.chunks(500) {
            REQWEST_CLIENT.post(format!("https://api.cloudflare.com/client/v4/zones/{zone_id}/purge_cache"))
                        .bearer_auth(&token)
                        .json(&serde_json::json!({
                        "files": chunk
                    }))
                        .send()
                        .await
                        .map_err(|err| {
                            ErrorKind::Fetch {
                                inner: err,
                                item: "cloudflare clear cache".to_string(),
                            }
                        })?
                        .error_for_status()
                        .map_err(|err| {
                            ErrorKind::Fetch {
                                inner: err,
                                item: "cloudflare clear cache".to_string(),
                            }
                        })?;
        }
    }

    Ok(())
}

pub struct UploadFile {
    file: bytes::Bytes,
    content_type: Option<String>,
}

#[derive(Default)]
pub struct FetchResult {
    pub upload_files: DashMap<String, UploadFile>,
    pub mirror_artifacts: DashMap<String, MirrorArtifact>,
}

pub struct MirrorArtifact {
    pub sha1: Option<String>,
    pub mirrors: DashSet<Mirror>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Mirror {
    path: String,
    entire_url: bool,
}

fn merge_fetch_result(fetch_result: &mut FetchResult, fetched: FetchResult) {
    for (path, upload_file) in fetched.upload_files {
        fetch_result.upload_files.insert(path, upload_file);
    }

    for (artifact_path, fetched_mirror_artifact) in fetched.mirror_artifacts {
        let mut val = fetch_result
            .mirror_artifacts
            .entry(artifact_path)
            .or_insert(MirrorArtifact {
                sha1: fetched_mirror_artifact.sha1.clone(),
                mirrors: DashSet::new(),
            });

        if val.sha1.is_none() {
            val.sha1 = fetched_mirror_artifact.sha1;
        }

        for mirror in fetched_mirror_artifact.mirrors {
            val.mirrors.insert(mirror);
        }
    }
}

#[tracing::instrument(skip(mirror_artifacts))]
pub fn insert_mirrored_artifact(
    artifact: &str,
    sha1: Option<String>,
    mirrors: Vec<String>,
    entire_url: bool,
    mirror_artifacts: &DashMap<String, MirrorArtifact>,
) -> Result<()> {
    let mut val = mirror_artifacts
        .entry(get_path_from_artifact(artifact)?)
        .or_insert(MirrorArtifact {
            sha1: sha1.clone(),
            mirrors: DashSet::new(),
        });

    if val.sha1.is_none() {
        val.sha1 = sha1;
    }

    for mirror in mirrors {
        val.mirrors.insert(Mirror {
            path: mirror,
            entire_url,
        });
    }

    Ok(())
}

fn check_env_vars() -> bool {
    let mut failed = false;

    fn check_var<T: std::str::FromStr>(var: &str) -> bool {
        if dotenvy::var(var)
            .ok()
            .and_then(|s| s.parse::<T>().ok())
            .is_none()
        {
            tracing::warn!(
                "Variable `{}` missing in dotenvy or not of type `{}`",
                var,
                std::any::type_name::<T>()
            );
            true
        } else {
            false
        }
    }

    failed |= check_var::<String>("BASE_URL");

    failed |= check_var::<String>("S3_ACCESS_TOKEN");
    failed |= check_var::<String>("S3_SECRET");
    failed |= check_var::<String>("S3_URL");
    failed |= check_var::<String>("S3_REGION");
    failed |= check_var::<String>("S3_BUCKET_NAME");

    if dotenvy::var("CLOUDFLARE_INTEGRATION")
        .ok()
        .and_then(|x| x.parse::<bool>().ok())
        .unwrap_or(false)
    {
        failed |= check_var::<String>("CLOUDFLARE_TOKEN");
        failed |= check_var::<String>("CLOUDFLARE_ZONE_ID");
    }

    failed
}
