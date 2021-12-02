use std::path::{Path, PathBuf};
use daedalus::download_file_mirrors;
use futures::future;
use tokio::fs;

use crate::launcher::ModLoader;
use super::ModpackResult;

#[derive(Debug, Clone, PartialEq)]
pub struct Modpack {
    pub game: ModpackGame,
    pub version: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<ModpackFile>,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModpackGame {
    // TODO: Currently, the launcher does not support specifying mod loader versions, so I just
    // store the loader here.
    Minecraft(String, ModLoader),
}


#[derive(Debug, Clone, PartialEq)]
pub struct ModpackFile {
    pub path: PathBuf,
    pub hashes: ModpackFileHashes,
    pub env: ModpackEnv,
    pub downloads: Vec<String>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModpackEnv {
    ClientOnly,
    ServerOnly,
    Both,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModpackSide {
    Client, Server,
}


#[derive(Debug, Clone, PartialEq)]
pub struct ModpackFileHashes {
    pub sha1: String,
}

