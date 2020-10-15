use super::IndexingError;
use crate::search::UploadSearchMod;
use log::info;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub url: String,
    pub thumbnail_url: String,
    pub is_default: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Category {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Author {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseVersion {
    pub game_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeMod {
    pub id: i32,
    pub name: String,
    pub authors: Vec<Author>,
    pub attachments: Vec<Attachment>,
    pub website_url: String,
    pub summary: String,
    pub download_count: f32,
    pub categories: Vec<Category>,
    pub game_version_latest_files: Vec<CurseVersion>,
    pub date_created: String,
    pub date_modified: String,
    pub game_slug: String,
}

pub async fn index_curseforge(
    start_index: i32,
    end_index: i32,
) -> Result<Vec<UploadSearchMod>, IndexingError> {
    info!("Indexing curseforge mods!");

    let mut docs_to_add: Vec<UploadSearchMod> = vec![];

    let res = reqwest::Client::new()
        .post("https://addons-ecs.forgesvc.net/api/v2/addon")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(format!(
            "{:?}",
            (start_index..end_index).collect::<Vec<_>>()
        ))
        .send()
        .await
        .map_err(IndexingError::CurseforgeImportError)?;

    let curseforge_mods: Vec<CurseForgeMod> = res
        .json()
        .await
        .map_err(IndexingError::CurseforgeImportError)?;

    for curseforge_mod in curseforge_mods {
        if curseforge_mod.game_slug != "minecraft"
            || !curseforge_mod.website_url.contains("/mc-mods/")
        {
            continue;
        }

        let mut mod_game_versions = vec![];

        let mut using_forge = false;
        let mut using_fabric = false;

        for version in curseforge_mod.game_version_latest_files {
            if let Some(parsed) = version
                .game_version
                .get(2..)
                .and_then(|f| f.parse::<f32>().ok())
            {
                if parsed < 14.0 {
                    using_forge = true;
                }
            }

            mod_game_versions.push(version.game_version);
        }

        let mut mod_categories = vec![];

        for category in curseforge_mod.categories {
            match &category.name[..] {
                "World Gen" => mod_categories.push(String::from("worldgen")),
                "Biomes" => mod_categories.push(String::from("worldgen")),
                "Ores and Resources" => mod_categories.push(String::from("worldgen")),
                "Structures" => mod_categories.push(String::from("worldgen")),
                "Dimensions" => mod_categories.push(String::from("worldgen")),
                "Mobs" => mod_categories.push(String::from("worldgen")),
                "Technology" => mod_categories.push(String::from("technology")),
                "Processing" => mod_categories.push(String::from("technology")),
                "Player Transport" => mod_categories.push(String::from("technology")),
                "Energy, Fluid, and Item Transport" => {
                    mod_categories.push(String::from("technology"))
                }
                "Food" => mod_categories.push(String::from("food")),
                "Farming" => mod_categories.push(String::from("food")),
                "Energy" => mod_categories.push(String::from("technology")),
                "Redstone" => mod_categories.push(String::from("technology")),
                "Genetics" => mod_categories.push(String::from("technology")),
                "Magic" => mod_categories.push(String::from("magic")),
                "Storage" => mod_categories.push(String::from("storage")),
                "API and Library" => mod_categories.push(String::from("library")),
                "Adventure and RPG" => mod_categories.push(String::from("adventure")),
                "Map and Information" => mod_categories.push(String::from("utility")),
                "Cosmetic" => mod_categories.push(String::from("decoration")),
                "Addons" => mod_categories.push(String::from("misc")),
                "Thermal Expansion" => mod_categories.push(String::from("misc")),
                "Tinker's Construct" => mod_categories.push(String::from("misc")),
                "Industrial Craft" => mod_categories.push(String::from("misc")),
                "Thaumcraft" => mod_categories.push(String::from("misc")),
                "Buildcraft" => mod_categories.push(String::from("misc")),
                "Forestry" => mod_categories.push(String::from("misc")),
                "Blood Magic" => mod_categories.push(String::from("misc")),
                "Lucky Blocks" => mod_categories.push(String::from("misc")),
                "Applied Energistics 2" => mod_categories.push(String::from("misc")),
                "CraftTweaker" => mod_categories.push(String::from("misc")),
                "Miscellaneous" => mod_categories.push(String::from("misc")),
                "Armor, Tools, and Weapons" => mod_categories.push(String::from("equipment")),
                "Server Utility" => mod_categories.push(String::from("utility")),
                "Fabric" => mod_categories.push(String::from("fabric")),
                _ => {}
            }
        }

        if mod_categories.iter().any(|e| e == "fabric") {
            using_fabric = true;
        }

        mod_categories.sort_unstable();
        mod_categories.dedup();
        mod_categories.truncate(3);

        if using_forge {
            mod_categories.push(String::from("forge"));
        }
        if using_fabric {
            // The only way this could happen is if "fabric" is already a category
            // mod_categories.push(String::from("fabric"));
        }

        let mut mod_attachments = curseforge_mod.attachments;
        mod_attachments.retain(|x| x.is_default);

        if mod_attachments.is_empty() {
            mod_attachments.push(Attachment {
                url: String::new(),
                thumbnail_url: String::new(),
                is_default: true,
            })
        }

        let latest_version = if !mod_game_versions.is_empty() {
            mod_game_versions[0].to_string()
        } else {
            "None".to_string()
        };

        let icon_url = mod_attachments[0]
            .thumbnail_url
            .replace("/256/256/", "/64/64/");

        let created = curseforge_mod
            .date_created
            .parse::<chrono::DateTime<chrono::Utc>>()?;
        let modified = curseforge_mod
            .date_modified
            .parse::<chrono::DateTime<chrono::Utc>>()?;

        let mut author = String::new();
        let mut author_url = String::new();

        if curseforge_mod.authors.len() > 0 {
            author = (&curseforge_mod.authors[0].name).to_string();
            author_url = (&curseforge_mod.authors[0].url).to_string();
        }

        docs_to_add.push(UploadSearchMod {
            mod_id: format!("curse-{}", curseforge_mod.id),
            author,
            title: curseforge_mod.name,
            description: curseforge_mod.summary.chars().take(150).collect(),
            categories: mod_categories,
            versions: mod_game_versions.clone(),
            downloads: curseforge_mod.download_count as i32,
            page_url: curseforge_mod.website_url,
            icon_url,
            author_url,
            date_created: created,
            created_timestamp: created.timestamp(),
            date_modified: modified,
            modified_timestamp: modified.timestamp(),
            latest_version,
            host: Cow::Borrowed("curseforge"),
            empty: Cow::Borrowed("{}{}{}"),
        })
    }

    Ok(docs_to_add)
}
