use futures::{StreamExt, TryStreamExt};
use log::info;

use super::IndexingError;
use crate::models::projects::SideType;
use crate::search::UploadSearchProject;
use sqlx::postgres::PgPool;
use std::borrow::Cow;

// TODO: only loaders for recent versions? For projects that have moved from forge to fabric
pub async fn index_local(pool: PgPool) -> Result<Vec<UploadSearchProject>, IndexingError> {
    info!("Indexing local projects!");

    let mut docs_to_add: Vec<UploadSearchProject> = vec![];

    let mut projects = sqlx::query!(
        "
        SELECT m.id, m.title, m.description, m.downloads, m.follows, m.icon_url, m.body_url, m.published, m.updated, m.team_id, m.status, m.slug, m.license, m.client_side, m.server_side FROM mods m
        "
    ).fetch(&pool);

    while let Some(result) = projects.next().await {
        if let Ok(project_data) = result {
            let status = crate::models::projects::ProjectStatus::from_str(
                &sqlx::query!(
                    "
                    SELECT status FROM statuses
                    WHERE id = $1
                    ",
                    project_data.status,
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
                project_data.id
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
                project_data.id
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
                project_data.id
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
                project_data.team_id,
            )
            .fetch_one(&pool)
            .await?;

            let mut icon_url = "".to_string();

            if let Some(url) = project_data.icon_url {
                icon_url = url;
            }

            let project_id = crate::models::ids::ProjectId(project_data.id as u64);

            // TODO: is this correct? This just gets the latest version of
            // minecraft that this project has a version that supports; it doesn't
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
                    project_data.client_side,
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
                    project_data.server_side,
                )
                .fetch_one(&pool)
                .await?
                .name,
            );

            let license = crate::database::models::categories::License::get(
                crate::database::models::LicenseId(project_data.license),
                &pool,
            )
            .await?;

            docs_to_add.push(UploadSearchProject {
                project_id: format!("local-{}", project_id),
                title: project_data.title,
                description: project_data.description,
                categories,
                versions,
                follows: project_data.follows,
                downloads: project_data.downloads,
                icon_url,
                author: user.username,
                date_created: project_data.published,
                created_timestamp: project_data.published.timestamp(),
                date_modified: project_data.updated,
                modified_timestamp: project_data.updated.timestamp(),
                latest_version,
                license: license.short,
                client_side: client_side.to_string(),
                server_side: server_side.to_string(),
                host: Cow::Borrowed("modrinth"),
                slug: project_data.slug,
            });
        }
    }

    Ok(docs_to_add)
}

pub async fn query_one(
    id: crate::database::models::ProjectId,
    exec: &mut sqlx::PgConnection,
) -> Result<UploadSearchProject, IndexingError> {
    let project_data = sqlx::query!(
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
        project_data.id
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
        project_data.id
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
        project_data.id
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
        project_data.team_id,
    )
    .fetch_one(&mut *exec)
    .await?;

    let mut icon_url = "".to_string();

    if let Some(url) = project_data.icon_url {
        icon_url = url;
    }

    let project_id = crate::models::ids::ProjectId(project_data.id as u64);

    // TODO: is this correct? This just gets the latest version of
    // minecraft that this project has a version that supports; it doesn't
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
            project_data.client_side,
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
            project_data.server_side,
        )
        .fetch_one(&mut *exec)
        .await?
        .name,
    );

    let license = crate::database::models::categories::License::get(
        crate::database::models::LicenseId(project_data.license),
        &mut *exec,
    )
    .await?;

    Ok(UploadSearchProject {
        project_id: format!("local-{}", project_id),
        title: project_data.title,
        description: project_data.description,
        categories,
        versions,
        follows: project_data.follows,
        downloads: project_data.downloads,
        icon_url,
        author: user.username,
        date_created: project_data.published,
        created_timestamp: project_data.published.timestamp(),
        date_modified: project_data.updated,
        modified_timestamp: project_data.updated.timestamp(),
        latest_version,
        license: license.short,
        client_side: client_side.to_string(),
        server_side: server_side.to_string(),
        host: Cow::Borrowed("modrinth"),
        slug: project_data.slug,
    })
}
