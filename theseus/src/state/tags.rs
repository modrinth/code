use serde::{Serialize, Deserialize};

use crate::config::MODRINTH_API_URL;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tags {
    pub categories : Vec<Category>,
    pub loaders : Vec<Loader>,
    pub game_versions : Vec<GameVersion>,
    pub licenses : Vec<License>,
    pub donation_platforms : Vec<DonationPlatform>,
    pub report_types : Vec<String>
}

impl Tags {
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

    pub async fn fetch(&mut self ) -> Result<(), reqwest::Error> {

        // Fetches all the tags from the modrinth api asynchronously, then joins
        // the threads and returns the results

        let categories = self.fetch_tag::<Category>("category");
        let loaders = self.fetch_tag::<Loader>("loader");
        let game_versions = self.fetch_tag::<GameVersion>("game_version");
        let licenses = self.fetch_tag::<License>("license");
        let donation_platforms = self.fetch_tag::<DonationPlatform>("donation_platform");
        let report_types = self.fetch_tag::<String>("report_type");

        let (categories, loaders, game_versions, licenses, donation_platforms, report_types) = futures::join!(categories, loaders, game_versions, licenses, donation_platforms, report_types);

        self.categories = categories?;
        self.loaders = loaders?;
        self.game_versions = game_versions?;
        self.licenses = licenses?;
        self.donation_platforms = donation_platforms?;
        self.report_types = report_types?;
    
        Ok(())
    }

    pub async fn fetch_tag<T>(&self, tag_type: &str) -> Result<Vec<T>, reqwest::Error> where T: serde::de::DeserializeOwned {
        
        let tag = reqwest::get(format!("{MODRINTH_API_URL}/tag/{}", tag_type)).await.json::<Vec<T>>()?;
        Ok(tag)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name : String,
    pub project_type : String,
    pub header: String,
    pub icon: PathBuf
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loader {
    pub name: String,
    pub icon : PathBuf,
    pub supported_project_types : Vec<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersion {
    pub version : String,
    pub version_type : String,
    pub date : String,
    pub major : bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License{
    pub short: String,
    pub name: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationPlatform {
    pub short: String,
    pub name: String,
}


