use std::{
    collections::HashSet,
    convert::TryFrom,
    path::{Path, PathBuf},
};

use crate::launcher::ModLoader;

use super::{
    manifest::{ManifestEnv, ManifestEnvs, ManifestHashes},
    pack::{ModpackEnv, ModpackFile, ModpackFileHashes, ModpackGame},
    ModpackError, ModpackResult,
};
use async_trait::async_trait;
use bytes::Bytes;
use futures::future::try_join_all;
use serde::Deserialize;
use tokio::try_join;

#[async_trait]
pub trait ModrinthAPI {
    async fn get_latest_version(
        &self,
        project: &str,
        channel: &str,
        game: &ModpackGame,
    ) -> ModpackResult<HashSet<ModpackFile>>;
    async fn get_version(&self, version: &str) -> ModpackResult<HashSet<ModpackFile>>;
}

#[derive(Debug)]
pub struct ModrinthV1(pub String);

#[derive(Debug, Deserialize)]
struct ModrinthV1Project<'a> {
    title: &'a str,
    client_side: &'a str,
    server_side: &'a str,
}

#[derive(Debug, Deserialize)]
struct ModrinthV1ProjectVersion<'a> {
    #[serde(borrow)]
    dependencies: HashSet<&'a str>,
    #[serde(borrow)]
    game_versions: HashSet<&'a str>,
    version_type: &'a str,
    files: Vec<ModrinthV1ProjectVersionFile<'a>>,
    #[serde(borrow)]
    loaders: HashSet<&'a str>,
}

#[derive(Clone, Debug, Deserialize)]
struct ModrinthV1ProjectVersionFile<'a> {
    hashes: ManifestHashes<'a>,
    url: &'a str,
    filename: &'a str,
}

impl From<ModrinthV1ProjectVersionFile<'_>> for ModpackFile {
    fn from(file: ModrinthV1ProjectVersionFile<'_>) -> Self {
        Self {
            hashes: Some(ModpackFileHashes::from(file.hashes)),
            downloads: {
                let mut downloads: HashSet<String> = HashSet::new();
                downloads.insert(String::from(file.url));
                downloads
            },
            path: PathBuf::from(file.filename),
            // WARNING: Since the sidedness of version 1 API requests is unknown, the environemnt is
            // set here as both.
            env: ModpackEnv::Both,
        }
    }
}

#[async_trait]
impl ModrinthAPI for ModrinthV1 {
    async fn get_latest_version(
        &self,
        project: &str,
        channel: &str,
        game: &ModpackGame,
    ) -> ModpackResult<HashSet<ModpackFile>> {
        // Fetch metadata
        let (project_json, versions_json): (Bytes, Bytes) = try_join!(
            try_get_json(format!("{}/api/v1/mod/{}", self.0, project)),
            try_get_json(format!("{}/api/v1/mod/{}/version", self.0, project)),
        )?;

        let (mut project_deserializer, mut versions_deserializer) = (
            serde_json::Deserializer::from_slice(&project_json),
            serde_json::Deserializer::from_slice(&versions_json),
        );

        let (project, versions) = (
            ModrinthV1Project::deserialize(&mut project_deserializer)?,
            Vec::deserialize(&mut versions_deserializer)?,
        );

        let (game_version, loader) = match game {
            ModpackGame::Minecraft(_, ModLoader::Vanilla) => Err(ModpackError::VersionError(
                String::from("Modrinth V1 does not support vanilla projects"),
            )),
            ModpackGame::Minecraft(ref version, ref loader) => Ok((version, loader)),
            _ => Err(ModpackError::VersionError(String::from(
                "Attempted to use Modrinth API V1 to install a non-Minecraft project!",
            ))),
        }?;

        let version: ModrinthV1ProjectVersion = versions
            .into_iter()
            .find(|it: &ModrinthV1ProjectVersion| {
                let loader_str = match loader {
                    ModLoader::Fabric => "fabric",
                    ModLoader::Forge => "forge",
                    ModLoader::Vanilla => unreachable!(),
                };
                it.version_type == channel
                    && it.game_versions.contains(&game_version.as_str())
                    && it.loaders.contains(&loader_str)
            })
            .ok_or(ModpackError::VersionError(format!(
                "Unable to find compatible version of mod {}",
                project.title
            )))?;

        // Project fields
        let envs = ModpackEnv::try_from(ManifestEnvs {
            client: serde_json::from_str(project.client_side)?,
            server: serde_json::from_str(project.server_side)?,
        })?;

        // Conversions
        let files = version
            .files
            .iter()
            .cloned()
            .map(ModpackFile::from)
            .collect::<HashSet<ModpackFile>>();

        let dep_futures = version.dependencies.iter().map(|it| self.get_version(&it));
        let deps = try_join_all(dep_futures)
            .await?
            .into_iter()
            .flatten()
            .collect::<HashSet<ModpackFile>>();

        Ok(files
            .into_iter()
            .chain(deps.into_iter())
            .map(|mut it| {
                it.env = envs;
                it
            })
            .collect())
    }

    async fn get_version(&self, version: &str) -> ModpackResult<HashSet<ModpackFile>> {
        let version_json = try_get_json(format!("{}/api/v1/version/{}", self.0, version)).await?;
        let mut version_deserializer = serde_json::Deserializer::from_slice(&version_json);
        let version = ModrinthV1ProjectVersion::deserialize(&mut version_deserializer)?;
        let base_path = PathBuf::from("mods/");

        Ok(version
            .files
            .into_iter()
            .map(ModpackFile::from)
            .collect::<HashSet<ModpackFile>>())
    }
}

// Helpers
async fn try_get_json(url: String) -> ModpackResult<Bytes> {
    Ok(reqwest::get(url).await?.error_for_status()?.bytes().await?)
}
