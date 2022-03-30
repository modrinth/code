use futures::TryStreamExt;
use log::info;

use super::IndexingError;
use crate::database::models::ProjectId;
use crate::search::UploadSearchProject;
use sqlx::postgres::PgPool;

// TODO: Move this away from STRING_AGG to multiple queries - however this may be more efficient?
pub async fn index_local(
    pool: PgPool,
) -> Result<Vec<UploadSearchProject>, IndexingError> {
    info!("Indexing local projects!");
    Ok(
        sqlx::query!(
            //FIXME: there must be a way to reduce the duplicate lines between this query and the one in `query_one` here...
            //region query
            "
            SELECT m.id id, m.project_type project_type, m.title title, m.description description, m.downloads downloads, m.follows follows,
            m.icon_url icon_url, m.published published,
            m.updated updated,
            m.team_id team_id, m.license license, m.slug slug,
            s.status status_name, cs.name client_side_type, ss.name server_side_type, l.short short, pt.name project_type_name, u.username username,
            STRING_AGG(DISTINCT c.category, ',') categories, STRING_AGG(DISTINCT lo.loader, ',') loaders, STRING_AGG(DISTINCT gv.version, ',') versions,
            STRING_AGG(DISTINCT mg.image_url, ',') gallery
            FROM mods m
            LEFT OUTER JOIN mods_categories mc ON joining_mod_id = m.id
            LEFT OUTER JOIN categories c ON mc.joining_category_id = c.id
            LEFT OUTER JOIN versions v ON v.mod_id = m.id
            LEFT OUTER JOIN game_versions_versions gvv ON gvv.joining_version_id = v.id
            LEFT OUTER JOIN game_versions gv ON gvv.game_version_id = gv.id
            LEFT OUTER JOIN loaders_versions lv ON lv.version_id = v.id
            LEFT OUTER JOIN loaders lo ON lo.id = lv.loader_id
            LEFT OUTER JOIN mods_gallery mg ON mg.mod_id = m.id
            INNER JOIN statuses s ON s.id = m.status
            INNER JOIN project_types pt ON pt.id = m.project_type
            INNER JOIN side_types cs ON m.client_side = cs.id
            INNER JOIN side_types ss ON m.server_side = ss.id
            INNER JOIN licenses l ON m.license = l.id
            INNER JOIN team_members tm ON tm.team_id = m.team_id AND tm.role = $2 AND tm.accepted = TRUE
            INNER JOIN users u ON tm.user_id = u.id
            WHERE s.status = $1
            GROUP BY m.id, s.id, cs.id, ss.id, l.id, pt.id, u.id;
            ",
            //endregion query
            crate::models::projects::ProjectStatus::Approved.as_str(),
            crate::models::teams::OWNER_ROLE,
        )
            .fetch_many(&pool)
            .try_filter_map(|e| async {
                Ok(e.right().map(|m| {
                    let mut categories = split_to_strings(m.categories);
                    categories.append(&mut split_to_strings(m.loaders));
                    let versions = split_to_strings(m.versions);

                    let project_id: crate::models::projects::ProjectId = ProjectId(m.id).into();

                    // TODO: Cleanup - This method has a lot of code in common with the method below.
                    // But, since the macro returns an (de facto) unnamed struct,
                    // We cannot reuse the code easily. Ugh.
                    UploadSearchProject {
                        project_id: format!("{}", project_id),
                        title: m.title,
                        description: m.description,
                        categories,
                        follows: m.follows,
                        downloads: m.downloads,
                        icon_url: m.icon_url.unwrap_or_default(),
                        author: m.username,
                        date_created: m.published,
                        created_timestamp: m.published.unix_timestamp(),
                        date_modified: m.updated,
                        modified_timestamp: m.updated.unix_timestamp(),
                        latest_version: versions.last().cloned().unwrap_or_else(|| "None".to_string()),
                        versions,
                        license: m.short,
                        client_side: m.client_side_type,
                        server_side: m.server_side_type,
                        slug: m.slug,
                        project_type: m.project_type_name,
                        gallery: m.gallery.map(|x| x.split(',').map(|x| x.to_string()).collect()).unwrap_or_default()
                    }
                }))
            })
            .try_collect::<Vec<_>>()
            .await?
    )
}
pub async fn query_one(
    id: ProjectId,
    exec: &mut sqlx::PgConnection,
) -> Result<UploadSearchProject, IndexingError> {
    let m = sqlx::query!(
        //region query
        "
        SELECT m.id id, m.project_type project_type, m.title title, m.description description, m.downloads downloads, m.follows follows,
        m.icon_url icon_url, m.published published,
        m.updated updated,
        m.team_id team_id, m.license license, m.slug slug,
        s.status status_name, cs.name client_side_type, ss.name server_side_type, l.short short, pt.name project_type_name, u.username username,
        STRING_AGG(DISTINCT c.category, ',') categories, STRING_AGG(DISTINCT lo.loader, ',') loaders, STRING_AGG(DISTINCT gv.version, ',') versions,
        STRING_AGG(DISTINCT mg.image_url, ',') gallery
        FROM mods m
        LEFT OUTER JOIN mods_categories mc ON joining_mod_id = m.id
        LEFT OUTER JOIN categories c ON mc.joining_category_id = c.id
        LEFT OUTER JOIN versions v ON v.mod_id = m.id
        LEFT OUTER JOIN game_versions_versions gvv ON gvv.joining_version_id = v.id
        LEFT OUTER JOIN game_versions gv ON gvv.game_version_id = gv.id
        LEFT OUTER JOIN loaders_versions lv ON lv.version_id = v.id
        LEFT OUTER JOIN loaders lo ON lo.id = lv.loader_id
        LEFT OUTER JOIN mods_gallery mg ON mg.mod_id = m.id
        INNER JOIN statuses s ON s.id = m.status
        INNER JOIN project_types pt ON pt.id = m.project_type
        INNER JOIN side_types cs ON m.client_side = cs.id
        INNER JOIN side_types ss ON m.server_side = ss.id
        INNER JOIN licenses l ON m.license = l.id
        INNER JOIN team_members tm ON tm.team_id = m.team_id AND tm.role = $2 AND tm.accepted = TRUE
        INNER JOIN users u ON tm.user_id = u.id
        WHERE m.id = $1
        GROUP BY m.id, s.id, cs.id, ss.id, l.id, pt.id, u.id;
        ",
        //endregion query
        id as ProjectId,
        crate::models::teams::OWNER_ROLE
    )
        .fetch_one(exec)
        .await?;

    let mut categories = split_to_strings(m.categories);
    categories.append(&mut split_to_strings(m.loaders));
    let versions = split_to_strings(m.versions);

    let project_id: crate::models::projects::ProjectId = ProjectId(m.id).into();

    Ok(UploadSearchProject {
        project_id: format!("{}", project_id),
        title: m.title,
        description: m.description,
        categories,
        follows: m.follows,
        downloads: m.downloads,
        icon_url: m.icon_url.unwrap_or_default(),
        author: m.username,
        date_created: m.published,
        created_timestamp: m.published.unix_timestamp(),
        date_modified: m.updated,
        modified_timestamp: m.updated.unix_timestamp(),
        latest_version: versions
            .last()
            .cloned()
            .unwrap_or_else(|| "None".to_string()),
        versions,
        license: m.short,
        client_side: m.client_side_type,
        server_side: m.server_side_type,
        slug: m.slug,
        project_type: m.project_type_name,
        gallery: split_to_strings(m.gallery),
    })
}

fn split_to_strings(s: Option<String>) -> Vec<String> {
    s.map(|x| x.split(',').map(ToString::to_string).collect())
        .unwrap_or_default()
}
