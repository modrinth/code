use crate::util::{
    REQWEST_CLIENT, format_url, upload_file_to_bucket,
    upload_url_to_bucket_mirrors, write_file_to_local_output,
    write_url_to_local_output_mirrors,
};
use daedalus::get_path_from_artifact;
use dashmap::{DashMap, DashSet};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::sync::Semaphore;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod error;
mod fabric;
mod forge;
mod metadata_groups;
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
    let only_loader = dotenvy::var("DAEDALUS_ONLY").ok();

    if should_fetch(only_loader.as_deref(), "minecraft") {
        match minecraft::fetch(semaphore.clone()).await {
            Ok(fetched) => merge_fetch_result(&mut fetch_result, fetched),
            Err(err) => tracing::warn!(error = %err, "Minecraft fetch failed"),
        }
    }

    if should_fetch(only_loader.as_deref(), "fabric") {
        match fabric::fetch_fabric(semaphore.clone()).await {
            Ok(fetched) => merge_fetch_result(&mut fetch_result, fetched),
            Err(err) => tracing::warn!(error = %err, "Fabric fetch failed"),
        }
    }

    if should_fetch(only_loader.as_deref(), "quilt") {
        match fabric::fetch_quilt(semaphore.clone()).await {
            Ok(fetched) => merge_fetch_result(&mut fetch_result, fetched),
            Err(err) => tracing::warn!(error = %err, "Quilt fetch failed"),
        }
    }

    if should_fetch(only_loader.as_deref(), "neo") {
        match forge::fetch_neo(semaphore.clone()).await {
            Ok(fetched) => merge_fetch_result(&mut fetch_result, fetched),
            Err(err) => tracing::warn!(error = %err, "NeoForge fetch failed"),
        }
    }

    if should_fetch(only_loader.as_deref(), "forge") {
        match forge::fetch_forge(semaphore.clone()).await {
            Ok(fetched) => merge_fetch_result(&mut fetch_result, fetched),
            Err(err) => tracing::warn!(error = %err, "Forge fetch failed"),
        }
    }

    let FetchResult {
        upload_files,
        mirror_artifacts,
    } = fetch_result;
    let upload_file_total = upload_files.len();
    let mirror_file_total = mirror_artifacts.len();

    if dotenvy::var("LOCAL_OUTPUT_DIR").is_ok() {
        let written_files = Arc::new(AtomicUsize::new(0));

        tracing::info!(
            total_files = upload_file_total,
            "Writing local metadata files"
        );

        futures::future::try_join_all(upload_files.iter().map(|entry| {
            let path = entry.key().clone();
            let file = entry.value().file.clone();
            let written_files = written_files.clone();

            async move {
                write_file_to_local_output(&path, file).await?;
                let written = written_files.fetch_add(1, Ordering::Relaxed) + 1;

                if written.is_multiple_of(100) || written == upload_file_total {
                    tracing::info!(
                        written_files = written,
                        remaining_files =
                            upload_file_total.saturating_sub(written),
                        total_files = upload_file_total,
                        "Wrote local metadata files"
                    );
                }

                Ok::<_, Error>(())
            }
        }))
        .await?;

        let written_mirror_files = Arc::new(AtomicUsize::new(0));

        tracing::info!(
            total_files = mirror_file_total,
            "Writing local mirror files"
        );

        futures::future::try_join_all(mirror_artifacts.iter().map(|entry| {
            let path = format!("maven/{}", entry.key());
            let mirrors = entry
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
                .collect();
            let sha1 = entry.value().sha1.clone();
            let written_mirror_files = written_mirror_files.clone();
            let semaphore = semaphore.clone();

            async move {
                write_url_to_local_output_mirrors(
                    path, mirrors, sha1, &semaphore,
                )
                .await?;
                let written =
                    written_mirror_files.fetch_add(1, Ordering::Relaxed) + 1;

                if written.is_multiple_of(100) || written == mirror_file_total {
                    tracing::info!(
                        written_files = written,
                        remaining_files =
                            mirror_file_total.saturating_sub(written),
                        total_files = mirror_file_total,
                        "Wrote local mirror files"
                    );
                }

                Ok::<_, Error>(())
            }
        }))
        .await?;
    } else {
        let uploaded_files = Arc::new(AtomicUsize::new(0));

        tracing::info!(
            total_files = upload_file_total,
            "Uploading metadata files"
        );

        futures::future::try_join_all(upload_files.iter().map(|entry| {
            let path = entry.key().clone();
            let file = entry.value().file.clone();
            let content_type = entry.value().content_type.clone();
            let uploaded_files = uploaded_files.clone();
            let semaphore = semaphore.clone();

            async move {
                upload_file_to_bucket(path, file, content_type, &semaphore)
                    .await?;
                let uploaded =
                    uploaded_files.fetch_add(1, Ordering::Relaxed) + 1;

                if uploaded.is_multiple_of(100) || uploaded == upload_file_total {
                    tracing::info!(
                        uploaded_files = uploaded,
                        remaining_files =
                            upload_file_total.saturating_sub(uploaded),
                        total_files = upload_file_total,
                        "Uploaded metadata files"
                    );
                }

                Ok::<_, Error>(())
            }
        }))
        .await?;

        let uploaded_mirror_files = Arc::new(AtomicUsize::new(0));

        tracing::info!(
            total_files = mirror_file_total,
            "Uploading mirror files"
        );

        futures::future::try_join_all(mirror_artifacts.iter().map(|entry| {
            let path = format!("maven/{}", entry.key());
            let mirrors = entry
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
                .collect();
            let sha1 = entry.value().sha1.clone();
            let uploaded_mirror_files = uploaded_mirror_files.clone();
            let semaphore = semaphore.clone();

            async move {
                upload_url_to_bucket_mirrors(path, mirrors, sha1, &semaphore)
                    .await?;
                let uploaded =
                    uploaded_mirror_files.fetch_add(1, Ordering::Relaxed) + 1;

                if uploaded.is_multiple_of(100) || uploaded == mirror_file_total {
                    tracing::info!(
                        uploaded_files = uploaded,
                        remaining_files =
                            mirror_file_total.saturating_sub(uploaded),
                        total_files = mirror_file_total,
                        "Uploaded mirror files"
                    );
                }

                Ok::<_, Error>(())
            }
        }))
        .await?;
    }

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

fn should_fetch(only_loader: Option<&str>, loader: &str) -> bool {
    let Some(only_loader) = only_loader else {
        return true;
    };

    only_loader.split(',').any(|entry| {
        let entry = entry.trim();

        entry.eq_ignore_ascii_case("all")
            || entry.eq_ignore_ascii_case(loader)
            || (loader == "neo" && entry.eq_ignore_ascii_case("neoforge"))
    })
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

    if dotenvy::var("LOCAL_OUTPUT_DIR").is_err() {
        failed |= check_var::<String>("S3_ACCESS_TOKEN");
        failed |= check_var::<String>("S3_SECRET");
        failed |= check_var::<String>("S3_URL");
        failed |= check_var::<String>("S3_REGION");
        failed |= check_var::<String>("S3_BUCKET_NAME");
    }

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
