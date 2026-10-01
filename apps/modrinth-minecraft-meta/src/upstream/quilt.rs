use anyhow::Result;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    task::DownloadRunContext,
    util::{ErrorVec, MavenCoordinate, Sha1, Sha256},
};

pub const CATALOG_URL: &str = "https://meta.quiltmc.org/v3/versions";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub game: Vec<GameVersion>,
    pub mappings: Vec<MappingVersion>,
    pub hashed: Vec<HashedVersion>,
    pub loader: Vec<LoaderVersion>,
    pub installer: Vec<InstallerVersion>,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct GameVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersion {
    pub version: GameVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct MappingVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MappingVersion {
    pub maven: MavenCoordinate,
    pub version: MappingVersionName,
    pub game_version: GameVersionName,
    pub build: u32,
    pub separator: String,
    pub hashed: HashedVersionName,
    pub file_size: u64,
    pub hashes: Hashes,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct HashedVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HashedVersion {
    pub maven: MavenCoordinate,
    pub version: HashedVersionName,
    pub file_size: u64,
    pub hashes: Hashes,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct LoaderVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LoaderVersion {
    pub maven: MavenCoordinate,
    pub version: LoaderVersionName,
    pub build: u32,
    pub separator: String,
    pub file_size: u64,
    pub hashes: Hashes,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct InstallerVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InstallerVersion {
    pub maven: MavenCoordinate,
    pub version: InstallerVersionName,
    pub url: Url,
    pub file_size: u64,
    pub hashes: Hashes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hashes {
    pub sha1: Sha1,
    pub sha256: Sha256,
    pub sha512: String,
}

/// Example URL: <https://meta.quiltmc.org/v3/versions/loader/1.21/0.19.5/profile/json>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLoaderProfile {
    pub id: String,
}

pub async fn download(
    cx: &mut DownloadRunContext<'_>,
    errors: &mut ErrorVec,
) -> Result<()> {
    let catalog = cx
        .download_json::<Catalog>(CATALOG_URL)
        .context(info_span!("fetching catalog"))
        .await?;
    info!(
        num_game_versions = catalog.game.len(),
        num_loader_versions = catalog.loader.len(),
        "downloaded Quilt catalog"
    );

    let template_game_versions = template_game_versions(&catalog.game);
    let num_profiles = template_game_versions.len() * catalog.loader.len();
    let mut num_downloaded_profiles = 0;

    for game_version in template_game_versions {
        for loader in &catalog.loader {
            let loader_version = &loader.version;
            let url = format!(
                "{CATALOG_URL}/loader/{game_version}/{loader_version}/profile/json"
            );

            cx.download_json::<GameLoaderProfile>(&url)
                .await
                .inspect_err(|err| errors.push(err))
                .ok();

            num_downloaded_profiles += 1;
            if num_downloaded_profiles % 10 == 0
                || num_downloaded_profiles == num_profiles
            {
                info!(
                    num_downloaded_profiles,
                    num_profiles, "downloaded Quilt loader profiles"
                );
            }
        }
    }

    Ok(())
}

fn template_game_versions(
    game_versions: &[GameVersion],
) -> Vec<&GameVersionName> {
    let mut templates = Vec::with_capacity(2);

    if let Some(game_version) = game_versions
        .iter()
        .find(|game_version| game_version.version.0 == "1.21")
        .or_else(|| {
            game_versions.iter().find(|game_version| {
                !is_modern_game_version(&game_version.version)
            })
        })
    {
        templates.push(&game_version.version);
    }

    if let Some(game_version) = game_versions
        .iter()
        .find(|game_version| is_modern_game_version(&game_version.version))
    {
        templates.push(&game_version.version);
    }

    templates
}

fn is_modern_game_version(game_version: &GameVersionName) -> bool {
    game_version
        .0
        .split(['.', 'w'])
        .next()
        .and_then(|major| major.parse::<u32>().ok())
        .is_some_and(|major| major >= 26)
}
