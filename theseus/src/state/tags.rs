use std::path::PathBuf;

use bincode::{Encode, Decode};
use serde::{Serialize, Deserialize};

use crate::{config::{MODRINTH_API_URL, REQWEST_CLIENT, BINCODE_CONFIG}};

const TAGS_DB_TREE: &[u8] = b"tags";

pub(crate) struct Tags(pub(crate) sled::Tree);

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct TagsInner {
    pub categories : Vec<Category>,
    pub loaders : Vec<Loader>,
    pub game_versions : Vec<GameVersion>,
    pub licenses : Vec<License>,
    pub donation_platforms : Vec<DonationPlatform>,
    pub report_types : Vec<String>
}

impl Tags {

    #[tracing::instrument(skip(db))]
    pub fn init(db: &sled::Db) -> crate::Result<Self> {
        Ok(Self(db.open_tree(TAGS_DB_TREE)?))
    }

    // Returns a TagsInner struct containing all the tags if it exists,
    // otherwise returns TagsInner::new()
    #[tracing::instrument(skip(self))]
    pub fn get(&self) -> crate::Result<TagsInner> {
        self.0.get("tags")?.map_or(Ok(TagsInner::new()), |prof| {
            bincode::decode_from_slice(&prof, *BINCODE_CONFIG)
                .map_err(crate::Error::from)
                .map(|it| it.0)
        })
    }

    // Gets the Licenses from the TagsInner struct if it exists, otherwise returns an empty Vec
    #[tracing::instrument(skip(self))]
    pub fn get_licenses(&self) -> crate::Result<Vec<License>> {
        Ok(self.get()?.licenses)
    }
    
    // Gets the Categories from the TagsInner struct if it exists, otherwise returns an empty Vec
    #[tracing::instrument(skip(self))]
    pub fn get_categories(&self) -> crate::Result<Vec<Category>> {
        Ok(self.get()?.categories)
    }

    // Gets the GameVersions from the TagsInner struct if it exists, otherwise returns an empty Vec
    #[tracing::instrument(skip(self))]
    pub fn get_game_versions(&self) -> crate::Result<Vec<GameVersion>> {
        Ok(self.get()?.game_versions)
    }

    // Gets the Loaders from the TagsInner struct if it exists, otherwise returns an empty Vec
    #[tracing::instrument(skip(self))]
    pub fn get_loaders(&self) -> crate::Result<Vec<Loader>> {
        Ok(self.get()?.loaders)
    }

    // Gets the DonationPlatforms from the TagsInner struct if it exists, otherwise returns an empty Vec
    #[tracing::instrument(skip(self))]
    pub fn get_donation_platforms(&self) -> crate::Result<Vec<DonationPlatform>> {
        Ok(self.get()?.donation_platforms)
    }

    // Gets the ReportTypes from the TagsInner struct if it exists, otherwise returns an empty Vec
    #[tracing::instrument(skip(self))]
    pub fn get_report_types(&self) -> crate::Result<Vec<String>> {
        Ok(self.get()?.report_types)
    }

    // Fetches the tags from the Modrinth API and stores them in the database
    #[tracing::instrument(skip(self))]
    pub async fn fetch_update(&mut self ) -> crate::Result<()> {
        let categories = self.fetch_tag::<Category>("category");
        let loaders = self.fetch_tag::<Loader>("loader");
        let game_versions = self.fetch_tag::<GameVersion>("game_version");
        let licenses = self.fetch_tag::<License>("license");
        let donation_platforms = self.fetch_tag::<DonationPlatform>("donation_platform");
        let report_types = self.fetch_tag::<String>("report_type");

        let (categories, loaders, game_versions, licenses, donation_platforms, report_types) = futures::join!(categories, loaders, game_versions, licenses, donation_platforms, report_types);

        // Create a TagsInner struct from the results
        let tags = TagsInner {
            categories : categories?,
            loaders : loaders?,
            game_versions : game_versions?,
            licenses : licenses?,
            donation_platforms : donation_platforms?,
            report_types : report_types?
        };
    
        // Insert the TagsInner struct into the database
        self.0.insert("tags", bincode::encode_to_vec(&tags, *BINCODE_CONFIG)?)?;

        dbg!("Displaying tags: {}", &tags);

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn fetch_tag<T>(&self, tag_type: &str) -> Result<Vec<T>, reqwest::Error> where T: serde::de::DeserializeOwned {
        let url = &format!("{MODRINTH_API_URL}///tag/{}", tag_type);
        dbg!(&url);
        let content = REQWEST_CLIENT.get(url).send().await?.json::<Vec<T>>().await?;
        Ok(content)
    }

}

impl TagsInner {
    pub fn new() -> Self {
        Self {
            categories : Vec::new(),
            loaders : Vec::new(),
            game_versions : Vec::new(),
            licenses : Vec::new(),
            donation_platforms : Vec::new(),
            report_types : Vec::new()
        }
    }
}

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct Category {
    pub name : String,
    pub project_type : String,
    pub header: String,
    pub icon: PathBuf
}

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct Loader {
    pub name: String,
    pub icon : PathBuf,
    pub supported_project_types : Vec<String>
}

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct GameVersion {
    pub version : String,
    pub version_type : String,
    pub date : String,
    pub major : bool
}

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct License{
    pub short: String,
    pub name: String
}

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct DonationPlatform {
    pub short: String,
    pub name: String,
}


