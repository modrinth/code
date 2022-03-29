use std::path::Path;

use crate::{data::DataError, LAUNCHER_WORK_DIR};
use once_cell::sync;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard};

const META_FILE: &str = "meta.json";
const META_URL: &str = "https://meta.modrinth.com/gamedata";

static METADATA: sync::OnceCell<RwLock<Metadata>> = sync::OnceCell::new();

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub minecraft: daedalus::minecraft::VersionManifest,
    pub forge: daedalus::modded::Manifest,
    pub fabric: daedalus::modded::Manifest,
}

impl Metadata {
    pub async fn init() -> Result<(), DataError> {
        let meta_path = Path::new(LAUNCHER_WORK_DIR).join(META_FILE);

        if meta_path.exists() {
            let meta_data = std::fs::read_to_string(meta_path)
                .ok()
                .and_then(|x| serde_json::from_str::<Metadata>(&x).ok());

            if let Some(metadata) = meta_data {
                METADATA.get_or_init(|| RwLock::new(metadata));
            }
        }

        let future = async {
            for attempt in 0..=3 {
                let res = async {
                    let new = Self::fetch().await?;

                    std::fs::write(
                        Path::new(LAUNCHER_WORK_DIR).join(META_FILE),
                        &serde_json::to_string(&new)?,
                    )?;

                    if let Some(metadata) = METADATA.get() {
                        *metadata.write().await = new;
                    } else {
                        METADATA.get_or_init(|| RwLock::new(new));
                    }

                    Ok::<(), DataError>(())
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
                };
            }
        };

        if METADATA.get().is_some() {
            tokio::task::spawn(future);
        } else {
            future.await;
        }

        Ok(())
    }

    pub async fn fetch() -> Result<Self, DataError> {
        let (game, forge, fabric) = futures::future::join3(
            daedalus::minecraft::fetch_version_manifest(Some(&format!(
                "{}/minecraft/v0/manifest.json",
                META_URL
            ))),
            daedalus::modded::fetch_manifest(&format!(
                "{}/forge/v0/manifest.json",
                META_URL
            )),
            daedalus::modded::fetch_manifest(&format!(
                "{}/fabric/v0/manifest.json",
                META_URL
            )),
        )
        .await;

        Ok(Self {
            minecraft: game?,
            forge: forge?,
            fabric: fabric?,
        })
    }

    pub async fn get<'a>() -> Result<RwLockReadGuard<'a, Self>, DataError> {
        let res = METADATA
            .get()
            .ok_or_else(|| DataError::InitializedError("metadata".to_string()))?
            .read()
            .await;

        Ok(res)
    }
}
