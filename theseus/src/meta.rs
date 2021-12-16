use serde::{Deserialize, Serialize};
use std::io;
use tokio::sync::RwLockReadGuard;

const META_FILE: &str = "meta.json";
const META_URL: &str = "https://staging-cdn.modrinth.com/gamedata";

#[derive(thiserror::Error, Debug)]
pub enum MetaError {
    #[error("I/O error while reading metadata: {0}")]
    IOError(#[from] io::Error),

    #[error("Daedalus error: {0}")]
    DaedalusError(#[from] daedalus::Error),

    #[error("Attempted to access metadata without initializing it!")]
    InitializedError,

    #[error("Error while serializing/deserializing JSON")]
    SerdeError(#[from] serde_json::Error),
}

use once_cell::sync;
use tokio::sync::RwLock;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub minecraft: daedalus::minecraft::VersionManifest,
    pub forge: daedalus::modded::Manifest,
    pub fabric: daedalus::modded::Manifest,
}

static METADATA: sync::OnceCell<RwLock<Metadata>> = sync::OnceCell::new();

impl Metadata {
    pub async fn init() -> Result<(), MetaError> {
        let meta_path = crate::LAUNCHER_WORK_DIR.join(META_FILE);

        if meta_path.exists() {
            let meta_data = std::fs::read_to_string(meta_path)
                .map(|x| serde_json::from_str::<Metadata>(&*x).ok())
                .ok()
                .flatten();

            if let Some(metadata) = meta_data {
                METADATA.get_or_init(|| RwLock::new(metadata));
            }
        }

        let future = async {
            for attempt in 0..=3 {
                let res = async {
                    let new = Self::fetch().await?;

                    std::fs::write(
                        crate::LAUNCHER_WORK_DIR.join(META_FILE),
                        &*serde_json::to_string(&new)?,
                    )?;

                    if let Some(metadata) = METADATA.get() {
                        *metadata.write().await = new;
                    } else {
                        METADATA.get_or_init(|| RwLock::new(new));
                    }

                    Ok::<(), MetaError>(())
                }
                .await;

                match res {
                    Ok(_) => {
                        break;
                    }
                    Err(_) if attempt <= 3 => continue,
                    Err(err) => {
                        log::warn!("Unable to fetch launcher metadata: {}", err)
                    }
                }
            }
        };

        if METADATA.get().is_some() {
            tokio::task::spawn(future);
        } else {
            future.await;
        }

        Ok(())
    }

    pub async fn fetch() -> Result<Self, MetaError> {
        let (game, forge, fabric) = futures::future::join3(
            daedalus::minecraft::fetch_version_manifest(Some(&*format!(
                "{}/minecraft/v0/manifest.json",
                META_URL
            ))),
            daedalus::modded::fetch_manifest(&*format!("{}/forge/v0/manifest.json", META_URL)),
            daedalus::modded::fetch_manifest(&*format!("{}/fabric/v0/manifest.json", META_URL)),
        )
        .await;

        Ok(Self {
            minecraft: game?,
            forge: forge?,
            fabric: fabric?,
        })
    }

    pub async fn get<'a>() -> Result<RwLockReadGuard<'a, Self>, MetaError> {
        Ok(METADATA
            .get()
            .ok_or_else(|| MetaError::InitializedError)?
            .read()
            .await)
    }
}
