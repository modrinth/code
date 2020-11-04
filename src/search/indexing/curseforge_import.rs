use super::IndexingError;
use crate::search::UploadSearchMod;
use log::info;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attachment<'a> {
    pub url: Cow<'a, str>,
    pub thumbnail_url: Cow<'a, str>,
    pub is_default: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Category<'a> {
    pub name: Cow<'a, str>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Author<'a> {
    pub name: Cow<'a, str>,
    pub url: Cow<'a, str>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseVersion<'a> {
    pub game_version: Cow<'a, str>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LatestFile<'a> {
    pub game_version: Vec<Cow<'a, str>>,
    pub modules: Vec<VersionModule<'a>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VersionModule<'a> {
    pub foldername: Cow<'a, str>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeMod<'a> {
    pub id: u32,
    pub name: Cow<'a, str>,
    pub authors: Vec<Option<Author<'a>>>,
    pub attachments: Vec<Attachment<'a>>,
    pub website_url: Cow<'a, str>,
    pub summary: Cow<'a, str>,
    pub download_count: f32,
    pub categories: Vec<Category<'a>>,
    pub latest_files: Vec<LatestFile<'a>>,
    pub game_version_latest_files: Vec<CurseVersion<'a>>,
    pub date_created: chrono::DateTime<chrono::Utc>,
    pub date_modified: chrono::DateTime<chrono::Utc>,
    pub category_section: CategorySection,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CategorySection {
    pub id: u32,
}

#[derive(Default)]
struct Loaders {
    forge: bool,
    fabric: bool,
    liteloader: bool,
    rift: bool,
}

lazy_static::lazy_static! {
    static ref CURSEFORGE_CATEGORIES: std::collections::HashMap<&'static str, &'static str> = {
        let mut map = std::collections::HashMap::new();
        map.insert("World Gen", "worldgen");
        map.insert("Biomes", "worldgen");
        map.insert("Ores and Resources", "worldgen");
        map.insert("Structures", "worldgen");
        map.insert("Dimensions", "worldgen");
        map.insert("Mobs", "worldgen");
        map.insert("Technology", "technology");
        map.insert("Processing", "technology");
        map.insert("Player Transport", "technology");
        map.insert("Energy, Fluid, and Item Transport", "technology");
        map.insert("Food", "food");
        map.insert("Farming", "food");
        map.insert("Energy", "technology");
        map.insert("Redstone", "technology");
        map.insert("Genetics", "technology");
        map.insert("Magic", "magic");
        map.insert("Storage", "storage");
        map.insert("API and Library", "library");
        map.insert("Adventure and RPG", "adventure");
        map.insert("Map and Information", "utility");
        map.insert("Cosmetic", "decoration");
        map.insert("Addons", "misc");
        map.insert("Thermal Expansion", "misc");
        map.insert("Tinker's Construct", "misc");
        map.insert("Industrial Craft", "misc");
        map.insert("Thaumcraft", "misc");
        map.insert("Buildcraft", "misc");
        map.insert("Forestry", "misc");
        map.insert("Blood Magic", "misc");
        map.insert("Lucky Blocks", "misc");
        map.insert("Applied Energistics 2", "misc");
        map.insert("CraftTweaker", "misc");
        map.insert("Miscellaneous", "misc");
        map.insert("Armor, Tools, and Weapons", "equipment");
        map.insert("Server Utility", "utility");
        map
    };
}

pub async fn index_curseforge(
    start_index: u32,
    end_index: u32,
    cache_path: &std::path::Path,
) -> Result<Vec<UploadSearchMod>, IndexingError> {
    info!("Indexing curseforge mods!");
    let start = std::time::Instant::now();

    let mut docs_to_add: Vec<UploadSearchMod> = vec![];

    let cache = std::fs::File::open(cache_path)
        .map(std::io::BufReader::new)
        .map(serde_json::from_reader::<_, Vec<u32>>);

    let requested_ids;

    // This caching system can't handle segmented indexing
    if let Ok(Ok(mut cache)) = cache {
        let end = cache.last().copied().unwrap_or(start_index);
        cache.extend(end..end_index);
        requested_ids = serde_json::to_string(&cache)?;
    } else {
        // This ends up being around 3 MiB
        // Serde json is better than using debug formatting since it doesn't
        // include spaces after commas, removing a lot of the extra size
        requested_ids = serde_json::to_string(&(start_index..end_index).collect::<Vec<_>>())?;
    }

    let res = reqwest::Client::new()
        .post("https://addons-ecs.forgesvc.net/api/v2/addon")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(requested_ids)
        .send()
        .await?;

    // The response ends up being about 300MiB, so we have to deal with
    // it efficiently.  Reading it as bytes and then deserializing with
    // borrowed data should avoid copying it, but it may take a bit more
    // memory.  To do this efficiently, we would have to get serde_json
    // to skip deserializing mods with category_section.id != 8
    // It's only 100MiB when using the cached ids, since that eliminates
    // all "addons" that aren't minecraft mods
    let buffer = res.bytes().await?;

    let mut curseforge_mods: Vec<CurseForgeMod> = serde_json::from_slice(&buffer)?;
    // This should remove many of the mods from the list before processing
    curseforge_mods.retain(|m| m.category_section.id == 8);

    // Only write to the cache if this doesn't skip mods at the start
    // The caching system iterates through all ids normally past the last
    // id in the cache, so the end_index shouldn't matter.
    if start_index <= 1 {
        let mut ids = curseforge_mods.iter().map(|m| m.id).collect::<Vec<_>>();
        ids.sort_unstable();
        if let Err(e) = std::fs::write(cache_path, serde_json::to_string(&ids)?) {
            log::warn!("Error writing to index id cache: {}", e);
        }
    }

    for mut curseforge_mod in curseforge_mods {
        // The gameId of minecraft is 432
        // The categorySection.id for mods is always 8
        // The categorySection.id 8 appears to be unique to minecraft mods
        // if curseforge_mod.game_slug != "minecraft"
        //     || !curseforge_mod.website_url.contains("/mc-mods/")
        // if curseforge_mod.category_section.id != 8 {
        //     continue;
        // }

        let mut mod_game_versions = vec![];

        let mut loaders = Loaders::default();

        for file in curseforge_mod.latest_files {
            for version in file.game_version {
                match &*version {
                    "Fabric" => loaders.forge = true,
                    "Forge" => loaders.fabric = true,
                    "Rift" => loaders.rift = true,
                    _ => (),
                }
            }
            for module in file.modules {
                match &*module.foldername {
                    "fabric.mod.json" => loaders.fabric = true,
                    "mcmod.info" => loaders.forge = true, // 1.13+ forge uses META-INF/mods.toml
                    "riftmod.json" => loaders.rift = true,
                    "litemod.json" => loaders.liteloader = true,
                    _ => (),
                }
            }
            // TODO: files ending with .litemod should also enable liteloader
            // if we decide to add true support for it;  That requires extra
            // deserializing work, so I'm not adding it for now
        }

        let mut latest = None;

        for version in curseforge_mod.game_version_latest_files {
            let mut split = version.game_version.split('.');
            let version_numbers = (
                split.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0),
                split.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0),
                split.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0),
            );

            if let Some((number, _)) = latest {
                if version_numbers > number {
                    latest = Some((version_numbers, version.game_version.clone()));
                }
            } else {
                latest = Some((version_numbers, version.game_version.clone()))
            }

            if ((1, 0, 0)..(1, 14, 0)).contains(&version_numbers) {
                // Is this a reasonable assumption to make?
                loaders.forge = true;
            }
            mod_game_versions.push(version.game_version);
        }

        let mut mod_categories = std::collections::HashSet::new();

        for category in curseforge_mod.categories {
            if category.name == "Fabric" {
                loaders.fabric = true;
            } else if let Some(category) = CURSEFORGE_CATEGORIES.get(&*category.name) {
                mod_categories.insert(*category);
            }
        }

        if !(loaders.fabric || loaders.rift || loaders.liteloader || loaders.forge) {
            // Assume that mods without loaders will be
            loaders.forge = true;
        }

        let mut mod_categories = mod_categories
            .into_iter()
            .take(3)
            .map(Cow::Borrowed)
            .collect::<Vec<_>>();

        if loaders.forge {
            mod_categories.push(Cow::Borrowed("forge"));
        }
        if loaders.fabric {
            mod_categories.push(Cow::Borrowed("fabric"));
        }

        let latest_version = latest
            .map(|(_, name)| name)
            .unwrap_or_else(|| Cow::Borrowed("None"));

        let icon_url = curseforge_mod
            .attachments
            .iter()
            .find(|a| a.is_default)
            .map(|a| a.thumbnail_url.replace("/256/256/", "/64/64/"))
            .unwrap_or_default();

        let author;
        let author_url;

        if let Some(user) = curseforge_mod
            .authors
            .get_mut(0)
            .map(Option::take)
            .flatten()
        {
            author = user.name.into_owned();
            author_url = user.url.into_owned();
        } else {
            author = "unknown".to_owned();
            author_url = String::from(&*curseforge_mod.website_url);
        }

        docs_to_add.push(UploadSearchMod {
            mod_id: format!("curse-{}", curseforge_mod.id),
            author,
            title: curseforge_mod.name.into_owned(),
            description: curseforge_mod.summary.chars().take(150).collect(),
            categories: mod_categories,
            versions: mod_game_versions.into_iter().map(String::from).collect(),
            downloads: curseforge_mod.download_count as i32,
            page_url: curseforge_mod.website_url.into_owned(),
            icon_url,
            author_url,
            date_created: curseforge_mod.date_created,
            created_timestamp: curseforge_mod.date_created.timestamp(),
            date_modified: curseforge_mod.date_modified,
            modified_timestamp: curseforge_mod.date_modified.timestamp(),
            latest_version,
            host: Cow::Borrowed("curseforge"),
            empty: Cow::Borrowed("{}{}{}"),
        })
    }

    let duration = start.elapsed();
    info!(
        "Finished indexing curseforge; Took {:5.2}s",
        duration.as_secs_f32()
    );

    Ok(docs_to_add)
}
