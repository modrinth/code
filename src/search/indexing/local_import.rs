use futures::{StreamExt, TryStreamExt};
use log::info;

use super::IndexingError;
use crate::models::mods::SideType;
use crate::search::UploadSearchMod;
use sqlx::postgres::PgPool;
use std::borrow::Cow;

// TODO: only loaders for recent versions? For mods that have moved from forge to fabric
pub async fn index_local(pool: PgPool) -> Result<Vec<UploadSearchMod>, IndexingError> {
    info!("Indexing local mods!");

    let mut docs_to_add: Vec<UploadSearchMod> = vec![];

    let mut mods = sqlx::query!(
        "
        SELECT m.id, m.title, m.description, m.downloads, m.follows, m.icon_url, m.body_url, m.published, m.updated, m.team_id, m.status, m.slug, m.license, m.client_side, m.server_side FROM mods m
        "
    ).fetch(&pool);

    while let Some(result) = mods.next().await {
        if let Ok(mod_data) = result {
            let status = crate::models::mods::ModStatus::from_str(
                &sqlx::query!(
                    "
                SELECT status FROM statuses
                WHERE id = $1
                ",
                    mod_data.status,
                )
                .fetch_one(&pool)
                .await?
                .status,
            );

            if !status.is_searchable() {
                continue;
            }

            let versions = sqlx::query!(
                "
                SELECT DISTINCT gv.version, gv.created FROM versions
                    INNER JOIN game_versions_versions gvv ON gvv.joining_version_id=versions.id
                    INNER JOIN game_versions gv ON gvv.game_version_id=gv.id
                WHERE versions.mod_id = $1
                ORDER BY gv.created ASC
                ",
                mod_data.id
            )
            .fetch_many(&pool)
            .try_filter_map(|e| async { Ok(e.right().map(|c| c.version)) })
            .try_collect::<Vec<String>>()
            .await?;

            let loaders = sqlx::query!(
                "
                SELECT DISTINCT loaders.loader FROM versions
                INNER JOIN loaders_versions lv ON lv.version_id = versions.id
                INNER JOIN loaders ON loaders.id = lv.loader_id
                WHERE versions.mod_id = $1
                ",
                mod_data.id
            )
            .fetch_many(&pool)
            .try_filter_map(|e| async { Ok(e.right().map(|c| Cow::Owned(c.loader))) })
            .try_collect::<Vec<Cow<str>>>()
            .await?;

            let mut categories = sqlx::query!(
                "
                SELECT c.category
                FROM mods_categories mc
                    INNER JOIN categories c ON mc.joining_category_id=c.id
                WHERE mc.joining_mod_id = $1
                ",
                mod_data.id
            )
            .fetch_many(&pool)
            .try_filter_map(|e| async { Ok(e.right().map(|c| Cow::Owned(c.category))) })
            .try_collect::<Vec<Cow<str>>>()
            .await?;

            categories.extend(loaders);

            let user = sqlx::query!(
                "
                SELECT u.id, u.username FROM users u
                INNER JOIN team_members tm ON tm.user_id = u.id
                WHERE tm.team_id = $2 AND tm.role = $1
                ",
                crate::models::teams::OWNER_ROLE,
                mod_data.team_id,
            )
            .fetch_one(&pool)
            .await?;

            let mut icon_url = "".to_string();

            if let Some(url) = mod_data.icon_url {
                icon_url = url;
            }

            let mod_id = crate::models::ids::ModId(mod_data.id as u64);
            let author_id = crate::models::ids::UserId(user.id as u64);

            // TODO: is this correct? This just gets the latest version of
            // minecraft that this mod has a version that supports; it doesn't
            // take betas or other info into account.
            let latest_version = versions
                .last()
                .cloned()
                .map(Cow::Owned)
                .unwrap_or_else(|| Cow::Borrowed(""));

            let client_side = SideType::from_str(
                &sqlx::query!(
                    "
                SELECT name FROM side_types
                WHERE id = $1
                ",
                    mod_data.client_side,
                )
                .fetch_one(&pool)
                .await?
                .name,
            );

            let server_side = SideType::from_str(
                &sqlx::query!(
                    "
                SELECT name FROM side_types
                WHERE id = $1
                ",
                    mod_data.server_side,
                )
                .fetch_one(&pool)
                .await?
                .name,
            );

            let license = crate::database::models::categories::License::get(
                crate::database::models::LicenseId(mod_data.license),
                &pool,
            )
            .await?;

            docs_to_add.push(UploadSearchMod {
                mod_id: format!("local-{}", mod_id),
                title: mod_data.title,
                description: mod_data.description,
                categories,
                versions,
                follows: mod_data.follows,
                downloads: mod_data.downloads,
                page_url: format!("https://modrinth.com/mod/{}", mod_id),
                icon_url,
                author: user.username,
                author_url: format!("https://modrinth.com/user/{}", author_id),
                date_created: mod_data.published,
                created_timestamp: mod_data.published.timestamp(),
                date_modified: mod_data.updated,
                modified_timestamp: mod_data.updated.timestamp(),
                latest_version,
                license: license.short,
                client_side: client_side.to_string(),
                server_side: server_side.to_string(),
                host: Cow::Borrowed("modrinth"),
                slug: mod_data.slug,
            });
        }
    }

    Ok(docs_to_add)
}

pub async fn query_one(
    id: crate::database::models::ModId,
    exec: &mut sqlx::PgConnection,
) -> Result<UploadSearchMod, IndexingError> {
    let mod_data = sqlx::query!(
        "
        SELECT m.id, m.title, m.description, m.downloads, m.follows, m.icon_url, m.body_url, m.published, m.updated, m.team_id, m.slug, m.license, m.client_side, m.server_side
        FROM mods m
        WHERE id = $1
        ",
        id.0,
    ).fetch_one(&mut *exec).await?;

    let versions = sqlx::query!(
        "
        SELECT DISTINCT gv.version, gv.created FROM versions
            INNER JOIN game_versions_versions gvv ON gvv.joining_version_id=versions.id
            INNER JOIN game_versions gv ON gvv.game_version_id=gv.id
        WHERE versions.mod_id = $1
        ORDER BY gv.created ASC
        ",
        mod_data.id
    )
    .fetch_many(&mut *exec)
    .try_filter_map(|e| async { Ok(e.right().map(|c| c.version)) })
    .try_collect::<Vec<String>>()
    .await?;

    let loaders = sqlx::query!(
        "
        SELECT DISTINCT loaders.loader FROM versions
        INNER JOIN loaders_versions lv ON lv.version_id = versions.id
        INNER JOIN loaders ON loaders.id = lv.loader_id
        WHERE versions.mod_id = $1
        ",
        mod_data.id
    )
    .fetch_many(&mut *exec)
    .try_filter_map(|e| async { Ok(e.right().map(|c| Cow::Owned(c.loader))) })
    .try_collect::<Vec<Cow<str>>>()
    .await?;

    let mut categories = sqlx::query!(
        "
        SELECT c.category
        FROM mods_categories mc
            INNER JOIN categories c ON mc.joining_category_id=c.id
        WHERE mc.joining_mod_id = $1
        ",
        mod_data.id
    )
    .fetch_many(&mut *exec)
    .try_filter_map(|e| async { Ok(e.right().map(|c| Cow::Owned(c.category))) })
    .try_collect::<Vec<Cow<str>>>()
    .await?;

    categories.extend(loaders);

    let user = sqlx::query!(
        "
        SELECT u.id, u.username FROM users u
        INNER JOIN team_members tm ON tm.user_id = u.id
        WHERE tm.team_id = $2 AND tm.role = $1
        ",
        crate::models::teams::OWNER_ROLE,
        mod_data.team_id,
    )
    .fetch_one(&mut *exec)
    .await?;

    let mut icon_url = "".to_string();

    if let Some(url) = mod_data.icon_url {
        icon_url = url;
    }

    let mod_id = crate::models::ids::ModId(mod_data.id as u64);
    let author_id = crate::models::ids::UserId(user.id as u64);

    // TODO: is this correct? This just gets the latest version of
    // minecraft that this mod has a version that supports; it doesn't
    // take betas or other info into account.
    let latest_version = versions
        .last()
        .cloned()
        .map(Cow::Owned)
        .unwrap_or_else(|| Cow::Borrowed(""));

    let client_side = SideType::from_str(
        &sqlx::query!(
            "
                SELECT name FROM side_types
                WHERE id = $1
                ",
            mod_data.client_side,
        )
        .fetch_one(&mut *exec)
        .await?
        .name,
    );

    let server_side = SideType::from_str(
        &sqlx::query!(
            "
                SELECT name FROM side_types
                WHERE id = $1
                ",
            mod_data.server_side,
        )
        .fetch_one(&mut *exec)
        .await?
        .name,
    );

    let license = crate::database::models::categories::License::get(
        crate::database::models::LicenseId(mod_data.license),
        &mut *exec,
    )
    .await?;

    Ok(UploadSearchMod {
        mod_id: format!("local-{}", mod_id),
        title: mod_data.title,
        description: mod_data.description,
        categories,
        versions,
        follows: mod_data.follows,
        downloads: mod_data.downloads,
        page_url: format!("https://modrinth.com/mod/{}", mod_id),
        icon_url,
        author: user.username,
        author_url: format!("https://modrinth.com/user/{}", author_id),
        date_created: mod_data.published,
        created_timestamp: mod_data.published.timestamp(),
        date_modified: mod_data.updated,
        modified_timestamp: mod_data.updated.timestamp(),
        latest_version,
        license: license.short,
        client_side: client_side.to_string(),
        server_side: server_side.to_string(),
        host: Cow::Borrowed("modrinth"),
        slug: mod_data.slug,
    })
}
