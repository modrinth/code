use std::sync::atomic::{self, AtomicUsize};

use anyhow::Result;
use derive_more::Display;
use futures::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    model,
    task::DownloadRunContext,
    util::{MavenCoordinate, Sha1, Sha256},
};

pub const CATALOG_URL: &str = "https://meta.quiltmc.org/v3/versions";
pub const PROFILE_BASE_URL: &str =
    "https://meta.quiltmc.org/v3/versions/loader/";
pub const MAVEN_URL: &str = "https://maven.quiltmc.org/repository/release/";

pub struct MetadataGroup {
    pub id: String,
    pub template: GameVersionName,
    pub games: Vec<GameVersionName>,
}

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
pub struct MappingVersion {
    pub maven: MavenCoordinate,
    pub version: MappingVersionName,
    #[serde(rename = "gameVersion")]
    pub game_version: GameVersionName,
    pub build: u32,
    pub separator: String,
    pub hashed: HashedVersionName,
    #[serde(rename = "file_size")]
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
    #[serde(default)]
    pub stable: bool,
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

pub async fn download(cx: &mut DownloadRunContext<'_>) -> Result<()> {
    let (catalog, sha256) = cx
        .download_json::<Catalog>(CATALOG_URL)
        .context(info_span!("fetching catalog"))
        .await?;
    info!(
        num_game_versions = catalog.game.len(),
        num_loader_versions = catalog.loader.len(),
        "downloaded Quilt catalog"
    );

    toasty::create!(model::QuiltCatalog {
        download_run_id: cx.download_run_id,
        sha256,
    })
    .exec(*cx.conn().await)
    .context(info_span!("inserting catalog"))
    .await?;

    let groups = profile_groups(&catalog.game);
    let template_game_versions = groups
        .iter()
        .map(|group| &group.template)
        .collect::<Vec<_>>();
    let num_profiles = template_game_versions.len() * catalog.loader.len();
    info!("downloading {num_profiles} loader profiles");

    let template_pairs = template_game_versions
        .iter()
        .flat_map(|game_version| {
            catalog.loader.iter().map(|loader| (*game_version, loader))
        })
        .collect::<Vec<_>>();

    let num_total = template_pairs.len();
    let num_done = AtomicUsize::new(0);
    let task =
        async |(game_version, loader): (&GameVersionName, &LoaderVersion)| {
            let loader_version = &loader.version;
            let url = profile_url(game_version, loader_version);

            cx.download_json::<GameLoaderProfile>(&url)
                .await
                .inspect_err(|err| cx.errors.push(err))
                .ok();

            let num_done = num_done.fetch_add(1, atomic::Ordering::SeqCst) + 1;
            if num_done.is_multiple_of(10) || num_done == num_total {
                info!("downloaded {num_done}/{num_total} loader profiles");
            }
        };
    stream::iter(template_pairs)
        .for_each_concurrent(cx.download_concurrency, task)
        .await;

    Ok(())
}

pub fn profile_url(
    game: &GameVersionName,
    loader: &LoaderVersionName,
) -> String {
    let mut url =
        Url::parse(PROFILE_BASE_URL).expect("valid Quilt profile base URL");
    url.path_segments_mut()
        .expect("Quilt profile URL supports path segments")
        .pop_if_empty()
        .extend([game.0.as_str(), loader.0.as_str(), "profile", "json"]);
    url.to_string()
}

pub fn profile_groups(game_versions: &[GameVersion]) -> Vec<MetadataGroup> {
    let (modern, legacy): (Vec<_>, Vec<_>) = game_versions
        .iter()
        .map(|game| game.version.clone())
        .partition(is_modern_game_version);
    let mut groups = Vec::with_capacity(2);
    if let Some(template) = legacy
        .iter()
        .find(|game| game.0 == "1.21")
        .or_else(|| legacy.first())
    {
        groups.push(MetadataGroup {
            id: "v1".to_owned(),
            template: template.clone(),
            games: legacy,
        });
    }
    if let Some(template) = modern.first() {
        groups.push(MetadataGroup {
            id: "v2".to_owned(),
            template: template.clone(),
            games: modern,
        });
    }
    groups
}

fn is_modern_game_version(game_version: &GameVersionName) -> bool {
    game_version
        .0
        .split(['.', 'w'])
        .next()
        .and_then(|major| major.parse::<u32>().ok())
        .is_some_and(|major| major >= 26)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn games(ids: &[&str]) -> Vec<GameVersion> {
        ids.iter()
            .map(|id| GameVersion {
                version: GameVersionName((*id).to_owned()),
                stable: true,
            })
            .collect()
    }

    #[test]
    fn groups_preserve_order_and_prefer_legacy_template() {
        let groups = profile_groups(&games(&[
            "26.2", "1.21.11", "26w14a", "1.21", "25w10a",
        ]));
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].id, "v1");
        assert_eq!(groups[0].template.0, "1.21");
        assert_eq!(
            groups[0].games,
            games(&["1.21.11", "1.21", "25w10a"])
                .into_iter()
                .map(|game| game.version)
                .collect::<Vec<_>>()
        );
        assert_eq!(groups[1].id, "v2");
        assert_eq!(groups[1].template.0, "26.2");
        assert_eq!(
            groups[1].games,
            games(&["26.2", "26w14a"])
                .into_iter()
                .map(|game| game.version)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn legacy_template_falls_back_to_first_legacy_game() {
        let groups = profile_groups(&games(&["26w14a", "1.20.6", "1.19"]));
        assert_eq!(groups[0].template.0, "1.20.6");
        assert_eq!(groups[1].template.0, "26w14a");
    }

    #[test]
    fn absent_groups_are_omitted() {
        assert!(profile_groups(&[]).is_empty());
        let legacy = profile_groups(&games(&["1.21"]));
        assert_eq!(legacy.len(), 1);
        assert_eq!(legacy[0].id, "v1");
        let modern = profile_groups(&games(&["26w14a", "27.1"]));
        assert_eq!(modern.len(), 1);
        assert_eq!(modern[0].id, "v2");
    }

    #[test]
    fn profile_url_matches_downloader_source() {
        assert_eq!(
            profile_url(
                &GameVersionName("1.21".to_owned()),
                &LoaderVersionName("0.30.0".to_owned())
            ),
            "https://meta.quiltmc.org/v3/versions/loader/1.21/0.30.0/profile/json"
        );
    }
}
