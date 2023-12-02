use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashSet;
use futures::TryStreamExt;
use log::info;

use super::IndexingError;
use crate::database::models::loader_fields::VersionField;
use crate::database::models::ProjectId;
use crate::search::UploadSearchProject;
use sqlx::postgres::PgPool;

pub async fn index_local(
    pool: PgPool,
) -> Result<(Vec<UploadSearchProject>, Vec<String>), IndexingError> {
    info!("Indexing local projects!");
    let loader_field_keys: Arc<DashSet<String>> = Arc::new(DashSet::new());
    let uploads =
        sqlx::query!(
            "
            WITH version_fields_cte AS (
                SELECT version_id, field_id, int_value, enum_value, string_value
                FROM version_fields
            ),
            version_fields_json AS (
                SELECT DISTINCT version_id,
                JSONB_AGG( 
                    DISTINCT jsonb_build_object('field_id', field_id, 'int_value', int_value, 'enum_value', enum_value, 'string_value', string_value)
                ) version_fields_json
                FROM version_fields_cte
                GROUP BY version_id
            ),
            loader_fields_cte AS (
                SELECT DISTINCT vf.version_id, lf.*, l.loader
                FROM loader_fields lf
                INNER JOIN version_fields_cte vf ON lf.id = vf.field_id
                LEFT JOIN loaders_versions lv ON vf.version_id = lv.version_id
                LEFT JOIN loaders l ON lv.loader_id = l.id
                GROUP BY vf.version_id, lf.enum_type, lf.id, l.loader
            ),
            loader_fields_json AS (
                SELECT DISTINCT version_id,
                    JSONB_AGG(
                    DISTINCT jsonb_build_object(
                        'version_id', lf.version_id,
                        'lf_id', id, 'loader_name', loader, 'field', field, 'field_type', field_type, 'enum_type', enum_type, 'min_val', min_val, 'max_val', max_val, 'optional', optional
                    )
                ) filter (where lf.id is not null) loader_fields_json
                FROM loader_fields_cte lf
                GROUP BY version_id
            ),
            loader_field_enum_values_json AS (
                SELECT DISTINCT version_id,
                    JSONB_AGG(
                    DISTINCT jsonb_build_object(
                        'id', lfev.id, 'enum_id', lfev.enum_id, 'value', lfev.value, 'ordering', lfev.ordering, 'created', lfev.created, 'metadata', lfev.metadata
                    ) 
                ) filter (where lfev.id is not null) loader_field_enum_values_json
                FROM loader_field_enum_values lfev
                INNER JOIN loader_fields_cte lf on lf.enum_type = lfev.enum_id
                GROUP BY version_id
            )

            SELECT m.id id, v.id version_id, m.name name, m.description description, m.downloads downloads, m.follows follows,
            m.icon_url icon_url, m.published published, m.approved approved, m.updated updated,
            m.team_id team_id, m.license license, m.slug slug, m.status status_name, m.color color,
            u.username username,
            ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null and mc.is_additional is false) categories,
            ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null and mc.is_additional is true) additional_categories,
            ARRAY_AGG(DISTINCT lo.loader) filter (where lo.loader is not null) loaders,
            ARRAY_AGG(DISTINCT pt.name) filter (where pt.name is not null) project_types,
            ARRAY_AGG(DISTINCT g.slug) filter (where g.slug is not null) games,
            ARRAY_AGG(DISTINCT mg.image_url) filter (where mg.image_url is not null and mg.featured is false) gallery,
            ARRAY_AGG(DISTINCT mg.image_url) filter (where mg.image_url is not null and mg.featured is true) featured_gallery,
            vf.version_fields_json version_fields,
            lf.loader_fields_json loader_fields,
            lfev.loader_field_enum_values_json loader_field_enum_values
            FROM versions v
            INNER JOIN mods m ON v.mod_id = m.id AND m.status = ANY($2)
            LEFT OUTER JOIN mods_categories mc ON joining_mod_id = m.id
            LEFT OUTER JOIN categories c ON mc.joining_category_id = c.id
            LEFT OUTER JOIN loaders_versions lv ON lv.version_id = v.id
            LEFT OUTER JOIN loaders lo ON lo.id = lv.loader_id
            LEFT JOIN loaders_project_types lpt ON lpt.joining_loader_id = lo.id
            LEFT JOIN project_types pt ON pt.id = lpt.joining_project_type_id
            LEFT JOIN loaders_project_types_games lptg ON lptg.loader_id = lo.id AND lptg.project_type_id = pt.id
            LEFT JOIN games g ON lptg.game_id = g.id
            LEFT OUTER JOIN mods_gallery mg ON mg.mod_id = m.id
            INNER JOIN team_members tm ON tm.team_id = m.team_id AND tm.is_owner = TRUE AND tm.accepted = TRUE
            INNER JOIN users u ON tm.user_id = u.id
            LEFT OUTER JOIN version_fields_json vf ON v.id = vf.version_id
            LEFT OUTER JOIN loader_fields_json lf ON v.id = lf.version_id
            LEFT OUTER JOIN loader_field_enum_values_json lfev ON v.id = lfev.version_id
            WHERE v.status != ANY($1)
            GROUP BY v.id, vf.version_fields_json, lf.loader_fields_json, lfev.loader_field_enum_values_json, m.id, u.id;
            ",
            &*crate::models::projects::VersionStatus::iterator().filter(|x| x.is_hidden()).map(|x| x.to_string()).collect::<Vec<String>>(),
            &*crate::models::projects::ProjectStatus::iterator().filter(|x| x.is_searchable()).map(|x| x.to_string()).collect::<Vec<String>>(),
        )
            .fetch_many(&pool)
            .try_filter_map(|e| {
                let loader_field_keys = loader_field_keys.clone();
                async move {
                Ok(e.right().map(|m| {
                    let mut additional_categories = m.additional_categories.unwrap_or_default();
                    let mut categories = m.categories.unwrap_or_default();

                    categories.append(&mut m.loaders.unwrap_or_default());

                    let display_categories = categories.clone();
                    categories.append(&mut additional_categories);

                    let version_fields = VersionField::from_query_json(m.loader_fields, m.version_fields, m.loader_field_enum_values, false);

                    let loader_fields : HashMap<String, Vec<String>> = version_fields.into_iter().map(|vf| {
                        (vf.field_name, vf.value.as_strings())
                    }).collect();

                    for v in loader_fields.keys().cloned() {
                        loader_field_keys.insert(v);
                    }

                    let project_id: crate::models::projects::ProjectId = ProjectId(m.id).into();
                    let version_id: crate::models::projects::ProjectId = ProjectId(m.version_id).into();

                    let license = match m.license.split(' ').next() {
                        Some(license) => license.to_string(),
                        None => m.license,
                    };

                    let open_source = match spdx::license_id(&license) {
                        Some(id) => id.is_osi_approved(),
                        _ => false,
                    };

                    // SPECIAL BEHAVIOUR
                    // Todo: revisit.
                    // For consistency with v2 searching, we consider the loader field 'mrpack_loaders' to be a category.
                    // These were previously considered the loader, and in v2, the loader is a category for searching.
                    // So to avoid breakage or awkward conversions, we just consider those loader_fields to be categories.
                    // The loaders are kept in loader_fields as well, so that no information is lost on retrieval.
                    let mrpack_loaders = loader_fields.get("mrpack_loaders").cloned().unwrap_or_default();
                    categories.extend(mrpack_loaders);

                    UploadSearchProject {
                        version_id: version_id.to_string(),
                        project_id: project_id.to_string(),
                        name: m.name,
                        description: m.description,
                        categories,
                        follows: m.follows,
                        downloads: m.downloads,
                        icon_url: m.icon_url.unwrap_or_default(),
                        author: m.username,
                        date_created: m.approved.unwrap_or(m.published),
                        created_timestamp: m.approved.unwrap_or(m.published).timestamp(),
                        date_modified: m.updated,
                        modified_timestamp: m.updated.timestamp(),
                        license,
                        slug: m.slug,
                        project_types: m.project_types.unwrap_or_default(),
                        gallery: m.gallery.unwrap_or_default(),
                        display_categories,
                        open_source,
                        color: m.color.map(|x| x as u32),
                        featured_gallery: m.featured_gallery.unwrap_or_default().first().cloned(),
                        loader_fields
                    }
                }))
}})
            .try_collect::<Vec<_>>()
            .await?;
    Ok((
        uploads,
        Arc::try_unwrap(loader_field_keys)
            .unwrap_or_default()
            .into_iter()
            .collect(),
    ))
}
