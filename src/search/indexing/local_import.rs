use futures::{StreamExt, TryStreamExt};
use log::info;

use super::IndexingError;
use crate::search::SearchMod;
use sqlx::postgres::PgPool;

pub async fn index_local(pool: PgPool) -> Result<Vec<SearchMod>, IndexingError> {
    info!("Indexing local mods!");

    let mut docs_to_add: Vec<SearchMod> = vec![];

    let mut results = sqlx::query!(
        "
        SELECT m.id, m.title, m.description, m.downloads, m.icon_url, m.body_url, m.published FROM mods m
        "
    )
    .fetch(&pool);

    while let Some(result) = results.next().await {
        if let Ok(result) = result {
            let versions: Vec<String> = sqlx::query!(
                "
                SELECT gv.version FROM versions
                    INNER JOIN game_versions_versions gvv ON gvv.joining_version_id=versions.id
                    INNER JOIN game_versions gv ON gvv.game_version_id=gv.id
                WHERE versions.mod_id = $1
                ",
                result.id
            )
            .fetch_many(&pool)
            .try_filter_map(|e| async { Ok(e.right().map(|c| c.version)) })
            .try_collect::<Vec<String>>()
            .await?;

            let categories = sqlx::query!(
                "
                SELECT c.category
                FROM mods_categories mc
                    INNER JOIN categories c ON mc.joining_category_id=c.id
                WHERE mc.joining_mod_id = $1
                ",
                result.id
            )
            .fetch_many(&pool)
            .try_filter_map(|e| async { Ok(e.right().map(|c| c.category)) })
            .try_collect::<Vec<String>>()
            .await?;

            let mut icon_url = "".to_string();

            if let Some(url) = result.icon_url {
                icon_url = url;
            }

            docs_to_add.push(SearchMod {
                mod_id: result.id,
                author: "".to_string(),
                title: result.title,
                description: result.description,
                keywords: categories,
                versions,
                downloads: result.downloads,
                page_url: result.body_url,
                icon_url,
                author_url: "".to_string(),
                date_created: result.published.to_string(),
                created: 0,
                date_modified: "".to_string(),
                updated: 0,
                latest_version: "".to_string(),
                empty: String::from("{}{}{}"),
            });
        }
    }

    Ok(docs_to_add)
}
