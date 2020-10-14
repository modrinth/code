use futures::{StreamExt, TryStreamExt};
use log::info;

use super::IndexingError;
use crate::search::UploadSearchMod;
use sqlx::postgres::PgPool;
use std::borrow::Cow;

// TODO: only loaders for recent versions? For mods that have moved from forge to fabric
pub async fn index_local(pool: PgPool) -> Result<Vec<UploadSearchMod>, IndexingError> {
    info!("Indexing local mods!");

    let mut docs_to_add: Vec<UploadSearchMod> = vec![];

    let mut results = sqlx::query!(
        "
        SELECT m.id, m.title, m.description, m.downloads, m.icon_url, m.body_url, m.published, m.updated, m.team_id FROM mods m
        "
    ).fetch(&pool);

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

            let loaders: Vec<String> = sqlx::query!(
                "
                SELECT loaders.loader FROM versions
                INNER JOIN loaders_versions lv ON lv.version_id = versions.id
                INNER JOIN loaders ON loaders.id = lv.loader_id
                WHERE versions.mod_id = $1
                ",
                result.id
            )
            .fetch_many(&pool)
            .try_filter_map(|e| async { Ok(e.right().map(|c| c.loader)) })
            .try_collect::<Vec<String>>()
            .await?;

            let mut categories = sqlx::query!(
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

            categories.extend(loaders);

            let user = sqlx::query!(
                "
                SELECT u.id, u.username FROM users u
                INNER JOIN team_members tm ON tm.role = $1
                WHERE tm.team_id = $2
                ",
                crate::models::teams::OWNER_ROLE,
                result.team_id,
            )
            .fetch_one(&pool)
            .await?;

            let mut icon_url = "".to_string();

            if let Some(url) = result.icon_url {
                icon_url = url;
            }

            docs_to_add.push(UploadSearchMod {
                mod_id: format!("local-{}", crate::models::ids::ModId(result.id as u64)),
                title: result.title,
                description: result.description,
                categories,
                versions,
                downloads: result.downloads,
                page_url: format!("https://modrinth.com/mod/{}", result.id),
                icon_url,
                author: user.username,
                author_url: format!("https://modrinth.com/user/{}", user.id),
                date_created: result.published,
                created_timestamp: result.published.timestamp(),
                date_modified: result.updated,
                modified_timestamp: result.updated.timestamp(),
                latest_version: "".to_string(), // TODO: Info about latest version
                host: Cow::Borrowed("modrinth"),
                empty: Cow::Borrowed("{}{}{}"),
            });
        }
    }

    Ok(docs_to_add)
}
