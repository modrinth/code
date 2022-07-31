use futures::TryStreamExt;
use log::info;

use super::IndexingError;
use crate::database::models::ProjectId;
use crate::search::UploadSearchProject;
use sqlx::postgres::PgPool;

pub async fn index_local(
    pool: PgPool,
) -> Result<Vec<UploadSearchProject>, IndexingError> {
    info!("Indexing local projects!");
    Ok(
        sqlx::query!(
            "
            SELECT m.id id, m.project_type project_type, m.title title, m.description description, m.downloads downloads, m.follows follows,
            m.icon_url icon_url, m.published published, m.approved approved, m.updated updated,
            m.team_id team_id, m.license license, m.slug slug,
            s.status status_name, cs.name client_side_type, ss.name server_side_type, l.short short, pt.name project_type_name, u.username username,
            ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null) categories, ARRAY_AGG(DISTINCT ca.category) filter (where ca.category is not null) additional_categories, ARRAY_AGG(DISTINCT lo.loader) filter (where lo.loader is not null) loaders, ARRAY_AGG(DISTINCT gv.version) filter (where gv.version is not null) versions,
            ARRAY_AGG(DISTINCT mg.image_url) filter (where mg.image_url is not null) gallery
            FROM mods m
            LEFT OUTER JOIN mods_categories mc ON joining_mod_id = m.id
            LEFT OUTER JOIN categories c ON mc.joining_category_id = c.id AND mc.is_additional = FALSE
            LEFT OUTER JOIN categories ca ON mc.joining_category_id = ca.id AND mc.is_additional = TRUE
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
            INNER JOIN team_members tm ON tm.team_id = m.team_id AND tm.role = $3 AND tm.accepted = TRUE
            INNER JOIN users u ON tm.user_id = u.id
            WHERE s.status = $1 OR s.status = $2
            GROUP BY m.id, s.id, cs.id, ss.id, l.id, pt.id, u.id;
            ",
            crate::models::projects::ProjectStatus::Approved.as_str(),
            crate::models::projects::ProjectStatus::Archived.as_str(),
            crate::models::teams::OWNER_ROLE,
        )
            .fetch_many(&pool)
            .try_filter_map(|e| async {
                Ok(e.right().map(|m| {
                    let mut categories = m.categories.unwrap_or_default();

                    categories.append(&mut m.loaders.unwrap_or_default());

                    let primary_categories = categories.clone();

                    categories.append(&mut m.additional_categories.unwrap_or_default());

                    let versions = m.versions.unwrap_or_default();

                    let project_id: crate::models::projects::ProjectId = ProjectId(m.id).into();

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
                        created_timestamp: m.approved.unwrap_or(m.published).timestamp(),
                        date_modified: m.updated,
                        modified_timestamp: m.updated.timestamp(),
                        latest_version: versions.last().cloned().unwrap_or_else(|| "None".to_string()),
                        versions,
                        license: m.short,
                        client_side: m.client_side_type,
                        server_side: m.server_side_type,
                        slug: m.slug,
                        project_type: m.project_type_name,
                        gallery: m.gallery.unwrap_or_default(),
                        display_categories: primary_categories
                    }
                }))
            })
            .try_collect::<Vec<_>>()
            .await?
    )
}
