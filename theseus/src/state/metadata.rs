//! Theseus metadata
use daedalus::{
    minecraft::{fetch_version_manifest, VersionManifest as MinecraftManifest},
    modded::{
        fetch_manifest as fetch_loader_manifest, Manifest as LoaderManifest,
    },
};
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;

const METADATA_URL: &str = "https://meta.modrinth.com/gamedata";
const METADATA_DB_FIELD: &[u8] = b"metadata";

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub minecraft: MinecraftManifest,
    pub forge: LoaderManifest,
    pub fabric: LoaderManifest,
}

impl Metadata {
    fn get_manifest(name: &str) -> String {
        format!("{METADATA_URL}/{name}/v0/manifest.json")
    }

    async fn fetch() -> crate::Result<Self> {
        let (minecraft, forge, fabric) = tokio::try_join! {
            async {
                let url = Self::get_manifest("minecraft");
                fetch_version_manifest(Some(&url)).await
            },
            async {
                let url = Self::get_manifest("forge");
                fetch_loader_manifest(&url).await
            },
            async {
                let url = Self::get_manifest("fabric");
                fetch_loader_manifest(&url).await
            }
        }?;

        Ok(Self {
            minecraft,
            forge,
            fabric,
        })
    }

    pub async fn init(db: &sled::Db) -> crate::Result<Self> {
        let mut metadata = None;

        if let Some(ref meta_bin) = db.get(METADATA_DB_FIELD)? {
            metadata = Some(bincode::deserialize::<Metadata>(meta_bin)?);
        }

        let mut fetch_futures = LinkedList::new();
        for _ in 0..3 {
            fetch_futures.push_back(Self::fetch().boxed());
        }

        match future::select_ok(fetch_futures).await {
            Ok(meta) => metadata = Some(meta.0),
            Err(err) => log::warn!("Unable to fetch launcher metadata: {err}"),
        }

        if let Some(meta) = metadata {
            db.insert(METADATA_DB_FIELD, bincode::serialize(&meta)?)?;
            Ok(meta)
        } else {
            Err(crate::Error::NoValueFor(String::from("launcher metadata")))
        }
    }
}
