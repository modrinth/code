use daedalus::download_file_mirrors;
use futures::future;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    hash::Hash,
    path::{Path, PathBuf},
};
use tokio::fs;

use super::{
    modrinth_api::{self, ModrinthV1},
    ModpackResult, ModpackError,
};
use crate::launcher::ModLoader;

pub const MODRINTH_DEFAULT_MODPACK_DOMAINS: &'static [&'static str] = &[
    "cdn.modrinth.com",
    "edge.forgecdn.net",
    "github.com",
    "raw.githubusercontent.com",
];
pub const MODRINTH_MODPACK_DOMAIN_WHITELIST_VAR: &'static str = "WHITELISTED_MODPACK_DOMAINS";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Modpack {
    pub game: ModpackGame,
    pub version: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: HashSet<ModpackFile>,
}

impl Modpack {
    /// Download a modpack's files for a given side to a given destination
    /// Assumes the destination exists and is a directory
    pub async fn download_files(&self, dest: &Path, side: ModpackSide) -> ModpackResult<()> {
        let handles = self.files.iter().cloned().map(move |file| {
            let (dest, side) = (dest.to_owned(), side);
            tokio::spawn(async move { file.fetch(&dest, side).await })
        });
        future::try_join_all(handles)
            .await?
            .into_iter()
            .collect::<ModpackResult<_>>()?;

        // TODO Integrate instance format to save other metadata
        Ok(())
    }

    pub fn new(game: ModpackGame, version: &str, name: &str, summary: Option<&str>) -> Self {
        Self {
            game,
            version: String::from(version),
            name: String::from(name),
            summary: summary.map(String::from),
            files: HashSet::new(),
        }
    }

    pub async fn add_project(
        &mut self,
        project: &str,
        base_path: &Path,
        source: Option<&dyn modrinth_api::ModrinthAPI>,
        channel: Option<&str>,
    ) -> ModpackResult<()> {
        let default_api = ModrinthV1(String::from("https://api.modrinth.com"));
        let channel = channel.unwrap_or("release");
        let source = source.unwrap_or(&default_api);

        let files = source
            .get_latest_version(project, channel, &self.game)
            .await?
            .into_iter()
            .map(|mut it: ModpackFile| {
                it.path = base_path.join(it.path);
                it
            });

        self.files.extend(files);
        Ok(())
    }

    pub async fn add_version(
        &mut self,
        version: &str,
        base_path: &Path,
        source: Option<&dyn modrinth_api::ModrinthAPI>,
    ) -> ModpackResult<()> {
        let default_api = ModrinthV1(String::from("https://api.modrinth.com"));
        let source = source.unwrap_or(&default_api);

        let files = source
            .get_version(version)
            .await?
            .into_iter()
            .map(|mut it: ModpackFile| {
                it.path = base_path.join(it.path);
                it
            });

        self.files.extend(files);
        Ok(())
    }

    pub async fn add_file(&mut self, source: reqwest::Url, dest: &Path, hashes: Option<ModpackFileHashes>, env: Option<ModpackEnv>) -> ModpackResult<()> {
        let whitelisted_domains = std::env::var(MODRINTH_MODPACK_DOMAIN_WHITELIST_VAR)
            .map(|it| serde_json::from_str::<Vec<String>>(&it).ok().unwrap())
            .unwrap_or(
                MODRINTH_DEFAULT_MODPACK_DOMAINS
                    .iter()
                    .cloned()
                    .map(String::from)
                    .collect::<Vec<String>>(),
            );

        if (whitelisted_domains.iter().find(String::from(source.host_str().unwrap())).is_none()) {
            return Err(ModpackError::SourceWhitelistError(String::from(source.host_str().unwrap())));
        }         
        
        let file = ModpackFile {
            path: dest,
            hashes,
            env: env.unwrap_or(ModpackEnv::Both),
            downloads: HashSet::from([String::from(source)])
        };

        self.files.insert(file);
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum ModpackGame {
    // TODO: Currently, the launcher does not support specifying mod loader versions, so I just
    // store the loader here.
    Minecraft(String, ModLoader),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ModpackFile {
    pub path: PathBuf,
    pub hashes: Option<ModpackFileHashes>,
    pub env: ModpackEnv,
    pub downloads: HashSet<String>,
}

impl Hash for ModpackFile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hashes.sha1.hash(state);
        self.path.hash(state);
    }
}

impl ModpackFile {
    pub async fn fetch(&self, dest: &Path, side: ModpackSide) -> ModpackResult<()> {
        if !self.env.supports(side) {
            return Ok(());
        }

        let output = dest.join(&self.path);

        // HACK: Since Daedalus appends a file name to all mirrors and the manifest supplies full
        // URLs, I'm supplying it with an empty string to avoid reinventing the wheel.
        let bytes = download_file_mirrors(
            "",
            &self
                .downloads
                .iter()
                .map(|it| it.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
            Some(&self.hashes.sha1),
        )
        .await?;
        fs::create_dir_all(output.parent().unwrap()).await?;
        fs::write(output, bytes).await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModpackEnv {
    ClientOnly,
    ServerOnly,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModpackSide {
    Client,
    Server,
}

impl ModpackEnv {
    pub fn supports(&self, side: ModpackSide) -> bool {
        match self {
            Self::ClientOnly => side == ModpackSide::Client,
            Self::ServerOnly => side == ModpackSide::Server,
            Self::Both => true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ModpackFileHashes {
    pub sha1: String,
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        path::{Path, PathBuf},
    };

    use super::*;
    use crate::launcher::ModLoader;

    #[tokio::test]
    async fn add_version() -> ModpackResult<()> {
        const TEST_VERSION: &'static str = "TpnSObJ7";
        let mut test_pack = Modpack::new(
            ModpackGame::Minecraft(String::from("1.16.5"), ModLoader::Fabric),
            "0.1.0",
            "Example Modpack",
            None,
        );
        test_pack
            .add_version(TEST_VERSION, Path::new("mods/"), None)
            .await?;

        assert_eq!(
            test_pack,
            Modpack {
                game: ModpackGame::Minecraft(String::from("1.16.5"), ModLoader::Fabric),
                version: String::from("0.1.0"),
                name: String::from("Example Modpack"),
                summary: None,
                files: {
                    let mut files = HashSet::new();
                    files.insert(ModpackFile {
                        path: PathBuf::from("mods/gravestones-v1.9.jar"),
                        hashes: ModpackFileHashes {
                            sha1: String::from("3f0f6d523d218460310b345be03ab3f1d294e04d"),
                        },
                        env: ModpackEnv::Both,
                        downloads: {
                            let mut downloads = HashSet::new();
                            downloads.insert(String::from("https://cdn.modrinth.com/data/ssUbhMkL/versions/v1.9/gravestones-v1.9.jar"));
                            downloads
                        }
                    });
                    files
                },
            },
        );
        Ok(())
    }
}
