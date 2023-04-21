use std::path::PathBuf;

use bincode::{Decode, Encode};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

use crate::config::{BINCODE_CONFIG, MODRINTH_API_URL};
use crate::loading_join;
use crate::util::fetch::fetch_json;

const CATEGORIES_DB_TREE: &[u8] = b"categories";
const LOADERS_DB_TREE: &[u8] = b"loaders";
const GAME_VERSIONS_DB_TREE: &[u8] = b"game_versions";
const LICENSES_DB_TREE: &[u8] = b"licenses";
const DONATION_PLATFORMS_DB_TREE: &[u8] = b"donation_platforms";
const REPORT_TYPES_DB_TREE: &[u8] = b"report_types";

#[derive(Clone)]
pub(crate) struct Tags(pub(crate) TagsInner);

#[derive(Debug, Clone)]
pub struct TagsInner {
    pub categories: sled::Tree,
    pub loaders: sled::Tree,
    pub game_versions: sled::Tree,
    pub licenses: sled::Tree,
    pub donation_platforms: sled::Tree,
    pub report_types: sled::Tree,
}

impl Tags {
    #[tracing::instrument(skip(db))]
    pub fn init(db: &sled::Db) -> crate::Result<Self> {
        Ok(Tags(TagsInner {
            categories: db.open_tree(CATEGORIES_DB_TREE)?,
            loaders: db.open_tree(LOADERS_DB_TREE)?,
            game_versions: db.open_tree(GAME_VERSIONS_DB_TREE)?,
            licenses: db.open_tree(LICENSES_DB_TREE)?,
            donation_platforms: db.open_tree(DONATION_PLATFORMS_DB_TREE)?,
            report_types: db.open_tree(REPORT_TYPES_DB_TREE)?,
        }))
    }

    // Checks the database for categories tag, returns a Vec::new() if it doesnt exist, otherwise returns the categories
    #[tracing::instrument(skip(self))]
    pub fn get_categories(&self) -> crate::Result<Vec<Category>> {
        self.0.categories.get("categories")?.map_or(
            Ok(Vec::new()),
            |categories| {
                bincode::decode_from_slice(&categories, *BINCODE_CONFIG)
                    .map_err(crate::Error::from)
                    .map(|it| it.0)
            },
        )
    }

    // Checks the database for loaders tag, returns a Vec::new() if it doesnt exist, otherwise returns the loaders
    #[tracing::instrument(skip(self))]
    pub fn get_loaders(&self) -> crate::Result<Vec<Loader>> {
        self.0
            .loaders
            .get("loaders")?
            .map_or(Ok(Vec::new()), |loaders| {
                bincode::decode_from_slice(&loaders, *BINCODE_CONFIG)
                    .map_err(crate::Error::from)
                    .map(|it| it.0)
            })
    }

    // Checks the database for game_versions tag, returns a Vec::new() if it doesnt exist, otherwise returns the game_versions
    #[tracing::instrument(skip(self))]
    pub fn get_game_versions(&self) -> crate::Result<Vec<GameVersion>> {
        self.0.game_versions.get("game_versions")?.map_or(
            Ok(Vec::new()),
            |game_versions| {
                bincode::decode_from_slice(&game_versions, *BINCODE_CONFIG)
                    .map_err(crate::Error::from)
                    .map(|it| it.0)
            },
        )
    }

    // Checks the database for licenses tag, returns a Vec::new() if it doesnt exist, otherwise returns the licenses
    #[tracing::instrument(skip(self))]
    pub fn get_licenses(&self) -> crate::Result<Vec<License>> {
        self.0
            .licenses
            .get("licenses")?
            .map_or(Ok(Vec::new()), |licenses| {
                bincode::decode_from_slice(&licenses, *BINCODE_CONFIG)
                    .map_err(crate::Error::from)
                    .map(|it| it.0)
            })
    }

    // Checks the database for donation_platforms tag, returns a Vec::new() if it doesnt exist, otherwise returns the donation_platforms
    #[tracing::instrument(skip(self))]
    pub fn get_donation_platforms(
        &self,
    ) -> crate::Result<Vec<DonationPlatform>> {
        self.0.donation_platforms.get("donation_platforms")?.map_or(
            Ok(Vec::new()),
            |donation_platforms| {
                bincode::decode_from_slice(&donation_platforms, *BINCODE_CONFIG)
                    .map_err(crate::Error::from)
                    .map(|it| it.0)
            },
        )
    }

    // Checks the database for report_types tag, returns a Vec::new() if it doesnt exist, otherwise returns the report_types
    #[tracing::instrument(skip(self))]
    pub fn get_report_types(&self) -> crate::Result<Vec<String>> {
        self.0.report_types.get("report_types")?.map_or(
            Ok(Vec::new()),
            |report_types| {
                bincode::decode_from_slice(&report_types, *BINCODE_CONFIG)
                    .map_err(crate::Error::from)
                    .map(|it| it.0)
            },
        )
    }

    // Gets all tags together as a serializable bundle
    #[tracing::instrument(skip(self))]
    pub fn get_tag_bundle(&self) -> crate::Result<TagBundle> {
        Ok(TagBundle {
            categories: self.get_categories()?,
            loaders: self.get_loaders()?,
            game_versions: self.get_game_versions()?,
            licenses: self.get_licenses()?,
            donation_platforms: self.get_donation_platforms()?,
            report_types: self.get_report_types()?,
        })
    }

    // Fetches the tags from the Modrinth API and stores them in the database
    #[tracing::instrument(skip(self))]
    pub async fn fetch_update(
        &mut self,
        semaphore: &RwLock<Semaphore>,
        loading_bar: Option<&Uuid>,
    ) -> crate::Result<()> {
        let categories = format!("{MODRINTH_API_URL}tag/category");
        let loaders = format!("{MODRINTH_API_URL}tag/loader");
        let game_versions = format!("{MODRINTH_API_URL}tag/game_version");
        let licenses = format!("{MODRINTH_API_URL}tag/license");
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
        let licenses_fut = fetch_json::<Vec<License>>(
            Method::GET,
            &licenses,
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
            licenses,
            donation_platforms,
            report_types,
        ) = loading_join!(loading_bar, 0.5, None;
            categories_fut,
            loaders_fut,
            game_versions_fut,
            licenses_fut,
            donation_platforms_fut,
            report_types_fut
        )?;

        // Store the tags in the database
        self.0.categories.insert(
            "categories",
            bincode::encode_to_vec(categories, *BINCODE_CONFIG)?,
        )?;
        self.0.loaders.insert(
            "loaders",
            bincode::encode_to_vec(loaders, *BINCODE_CONFIG)?,
        )?;
        self.0.game_versions.insert(
            "game_versions",
            bincode::encode_to_vec(game_versions, *BINCODE_CONFIG)?,
        )?;
        self.0.licenses.insert(
            "licenses",
            bincode::encode_to_vec(licenses, *BINCODE_CONFIG)?,
        )?;
        self.0.donation_platforms.insert(
            "donation_platforms",
            bincode::encode_to_vec(donation_platforms, *BINCODE_CONFIG)?,
        )?;
        self.0.report_types.insert(
            "report_types",
            bincode::encode_to_vec(report_types, *BINCODE_CONFIG)?,
        )?;

        Ok(())
    }
}

// Serializeable struct for all tags to be fetched together by the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagBundle {
    pub categories: Vec<Category>,
    pub loaders: Vec<Loader>,
    pub game_versions: Vec<GameVersion>,
    pub licenses: Vec<License>,
    pub donation_platforms: Vec<DonationPlatform>,
    pub report_types: Vec<String>,
}

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub project_type: String,
    pub header: String,
    pub icon: PathBuf,
}

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct Loader {
    pub name: String,
    pub icon: PathBuf,
    pub supported_project_types: Vec<String>,
}

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct License {
    pub short: String,
    pub name: String,
}

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct DonationPlatform {
    pub short: String,
    pub name: String,
}

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct GameVersion {
    pub version: String,
    pub version_type: String,
    pub date: String,
    pub major: bool,
}
