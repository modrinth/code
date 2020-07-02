use bson::doc;
use futures::StreamExt;
use log::info;

use crate::database::models::Item;
use crate::database::{Mod, Version};

use crate::search::{SearchError, SearchMod};

pub async fn index_local(client: mongodb::Client) -> Result<Vec<SearchMod>, SearchError> {
    info!("Indexing local mods!");

    let mut docs_to_add: Vec<SearchMod> = vec![];

    let db = client.database("modrinth");

    let mods = db.collection("mods");
    let versions = db.collection("versions");

    let mut results = mods.find(None, None).await?;

    while let Some(unparsed_result) = results.next().await {
        let result: Mod = *Mod::from_doc(unparsed_result?)?;

        let mut mod_versions = versions.find(doc! { "mod_id": result.id}, None).await?;

        let mut mod_game_versions = vec![];

        while let Some(unparsed_version) = mod_versions.next().await {
            let mut version: Version = *Version::from_doc(unparsed_version?)?;
            mod_game_versions.append(&mut version.game_versions);
        }

        docs_to_add.push(SearchMod {
            mod_id: result.id,
            author: result.author,
            title: result.title,
            description: result.description,
            keywords: result.categories,
            versions: mod_game_versions,
            downloads: result.downloads,
            page_url: "".to_string(),
            icon_url: result.icon_path,
            author_url: "".to_string(),
            date_created: "".to_string(),
            created: 0,
            date_modified: "".to_string(),
            updated: 0,
            latest_version: "".to_string(),
            empty: String::from("{}{}{}"),
        });
    }

    Ok(docs_to_add)
}
