use std::path::PathBuf;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::config::MODRINTH_API_URL;
use crate::data::DirectoryInfo;
use crate::util::fetch::{
    fetch_json, read_json, write, FetchSemaphore, IoSemaphore,
};

// Serializeable struct for all tags to be fetched together by the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tags {
    pub categories: Vec<Category>,
    pub loaders: Vec<Loader>,
    pub game_versions: Vec<GameVersion>,
    pub donation_platforms: Vec<DonationPlatform>,
    pub report_types: Vec<String>,
}

impl Tags {
    #[tracing::instrument(skip(io_semaphore, fetch_semaphore))]
    #[theseus_macros::debug_pin]
    pub async fn init(
        dirs: &DirectoryInfo,
        io_semaphore: &IoSemaphore,
        fetch_semaphore: &FetchSemaphore,
    ) -> crate::Result<Self> {
        let mut tags = None;
        let tags_path = dirs.caches_meta_dir().join("tags.json");

        if let Ok(tags_json) = read_json::<Self>(&tags_path, io_semaphore).await
        {
            tags = Some(tags_json);
        } else {
            match Self::fetch(fetch_semaphore).await {
                Ok(tags_fetch) => tags = Some(tags_fetch),
                Err(err) => {
                    tracing::warn!("Unable to fetch launcher tags: {err}")
                }
            }
        }

        if let Some(tags_data) = tags {
            write(&tags_path, &serde_json::to_vec(&tags_data)?, io_semaphore)
                .await?;
            Ok(tags_data)
        } else {
            Err(crate::ErrorKind::NoValueFor(String::from("launcher tags"))
                .as_error())
        }
    }

    #[tracing::instrument]
    #[theseus_macros::debug_pin]
    pub async fn update() {
        let res = async {
            let state = crate::State::get().await?;
            let tags_fetch = Tags::fetch(&state.fetch_semaphore).await?;

            let tags_path =
                state.directories.caches_meta_dir().join("tags.json");

            write(
                &tags_path,
                &serde_json::to_vec(&tags_fetch)?,
                &state.io_semaphore,
            )
            .await
            .unwrap();

            let mut old_tags = state.tags.write().await;
            *old_tags = tags_fetch;

            Ok::<(), crate::Error>(())
        }
        .await;

        match res {
            Ok(()) => {}
            Err(err) => {
                tracing::warn!("Unable to update launcher tags: {err}")
            }
        };
    }

    // Checks the database for categories tag, returns a Vec::new() if it doesnt exist, otherwise returns the categories
    #[tracing::instrument(skip(self))]
    pub fn get_categories(&self) -> Vec<Category> {
        self.categories.clone()
    }

    // Checks the database for loaders tag, returns a Vec::new() if it doesnt exist, otherwise returns the loaders
    #[tracing::instrument(skip(self))]
    pub fn get_loaders(&self) -> Vec<Loader> {
        self.loaders.clone()
    }

    // Checks the database for game_versions tag, returns a Vec::new() if it doesnt exist, otherwise returns the game_versions
    #[tracing::instrument(skip(self))]
    pub fn get_game_versions(&self) -> Vec<GameVersion> {
        self.game_versions.clone()
    }

    // Checks the database for donation_platforms tag, returns a Vec::new() if it doesnt exist, otherwise returns the donation_platforms
    #[tracing::instrument(skip(self))]
    pub fn get_donation_platforms(&self) -> Vec<DonationPlatform> {
        self.donation_platforms.clone()
    }

    // Checks the database for report_types tag, returns a Vec::new() if it doesnt exist, otherwise returns the report_types
    #[tracing::instrument(skip(self))]
    pub fn get_report_types(&self) -> Vec<String> {
        self.report_types.clone()
    }

    // Gets all tags together as a serializable bundle
    #[tracing::instrument(skip(self))]
    pub fn get_tag_bundle(&self) -> Tags {
        self.clone()
    }

    // Fetches the tags from the Modrinth API and stores them in the database
    pub async fn fetch(semaphore: &FetchSemaphore) -> crate::Result<Self> {
        let categories = format!("{MODRINTH_API_URL}tag/category");
        let loaders = format!("{MODRINTH_API_URL}tag/loader");
        let game_versions = format!("{MODRINTH_API_URL}tag/game_version");
        let donation_platforms =
            format!("{MODRINTH_API_URL}tag/donation_platform");
        let report_types = format!("{MODRINTH_API_URL}tag/report_type");

        let categories_fut = fetch_json::<Vec<Category>>(
            Method::GET,
            &categories,
            None,
            None,
            semaphore,
        );
        let loaders_fut = fetch_json::<Vec<Loader>>(
            Method::GET,
            &loaders,
            None,
            None,
            semaphore,
        );
        let game_versions_fut = fetch_json::<Vec<GameVersion>>(
            Method::GET,
            &game_versions,
            None,
            None,
            semaphore,
        );
        let donation_platforms_fut = fetch_json::<Vec<DonationPlatform>>(
            Method::GET,
            &donation_platforms,
            None,
            None,
            semaphore,
        );
        let report_types_fut = fetch_json::<Vec<String>>(
            Method::GET,
            &report_types,
            None,
            None,
            semaphore,
        );

        let (
            categories,
            loaders,
            game_versions,
            donation_platforms,
            report_types,
        ) = tokio::try_join!(
            categories_fut,
            loaders_fut,
            game_versions_fut,
            donation_platforms_fut,
            report_types_fut
        )?;

        Ok(Self {
            categories,
            loaders,
            game_versions,
            donation_platforms,
            report_types,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub project_type: String,
    pub header: String,
    pub icon: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loader {
    pub name: String,
    pub icon: PathBuf,
    pub supported_project_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationPlatform {
    pub short: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersion {
    pub version: String,
    pub version_type: String,
    pub date: String,
    pub major: bool,
}
