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

    // path, upload file
    let upload_files: DashMap<String, UploadFile> = DashMap::new();
    // path, mirror artifact
    let mirror_artifacts: DashMap<String, MirrorArtifact> = DashMap::new();

    minecraft::fetch(semaphore.clone(), &upload_files, &mirror_artifacts)
        .await?;
    fabric::fetch_fabric(semaphore.clone(), &upload_files, &mirror_artifacts)
        .await?;
    fabric::fetch_quilt(semaphore.clone(), &upload_files, &mirror_artifacts)
        .await?;
    forge::fetch_neo(semaphore.clone(), &upload_files, &mirror_artifacts)
        .await?;
    forge::fetch_forge(semaphore.clone(), &upload_files, &mirror_artifacts)
        .await?;

    futures::future::try_join_all(upload_files.iter().map(|x| {
        upload_file_to_bucket(
            x.key().clone(),
            x.value().file.clone(),
            x.value().content_type.clone(),
            &semaphore,
        )
    }))
    .await?;

    futures::future::try_join_all(mirror_artifacts.iter().map(|x| {
        upload_url_to_bucket_mirrors(
            format!("maven/{}", x.key()),
            x.value()
                .mirrors
                .iter()
                .map(|mirror| {
                    if mirror.entire_url {
                        mirror.path.clone()
                    } else {
                        format!("{}{}", mirror.path, x.key())
                    }
                })
                .collect(),
            x.sha1.clone(),
            &semaphore,
        )
    }))
    .await?;

    if dotenvy::var("CLOUDFLARE_INTEGRATION")
        .ok()
        .and_then(|x| x.parse::<bool>().ok())
        .unwrap_or(false)
    {
        if let Ok(token) = dotenvy::var("CLOUDFLARE_TOKEN") {
            if let Ok(zone_id) = dotenvy::var("CLOUDFLARE_ZONE_ID") {
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
        }
    }

    Ok(())
}

pub struct UploadFile {
    file: bytes::Bytes,
    content_type: Option<String>,
}

pub struct MirrorArtifact {
    pub sha1: Option<String>,
    pub mirrors: DashSet<Mirror>,
}

#[derive(Eq, PartialEq, Hash)]
pub struct Mirror {
    path: String,
    entire_url: bool,
}

#[tracing::instrument(skip(mirror_artifacts))]
pub fn insert_mirrored_artifact(
    artifact: &str,
    sha1: Option<String>,
    mirrors: Vec<String>,
    entire_url: bool,
    mirror_artifacts: &DashMap<String, MirrorArtifact>,
) -> Result<()> {
    let val = mirror_artifacts
        .entry(get_path_from_artifact(artifact)?)
        .or_insert(MirrorArtifact {
            sha1,
            mirrors: DashSet::new(),
        });

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
