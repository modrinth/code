use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures::TryStreamExt;
use itertools::Itertools;
use std::collections::HashMap;
use tracing::info;

use super::IndexingError;
use crate::database::models::loader_fields::{
    QueryLoaderField, QueryLoaderFieldEnumValue, QueryVersionField,
    VersionField,
};
use crate::database::models::{
    DBProjectId, DBVersionId, LoaderFieldEnumId, LoaderFieldEnumValueId,
    LoaderFieldId,
};
use crate::models::projects::from_duplicate_version_fields;
use crate::models::v2::projects::LegacyProject;
use crate::routes::v2_reroute;
use crate::search::UploadSearchProject;
use sqlx::postgres::PgPool;

pub async fn index_local(
    pool: &PgPool,
) -> Result<Vec<UploadSearchProject>, IndexingError> {
    info!("Indexing local projects!");

    // todo: loaders, project type, game versions
    struct PartialProject {
        id: DBProjectId,
        name: String,
        summary: String,
        downloads: i32,
        follows: i32,
        icon_url: Option<String>,
        updated: DateTime<Utc>,
        approved: DateTime<Utc>,
        slug: Option<String>,
        color: Option<i32>,
        license: String,
    }

    let db_projects = sqlx::query!(
        "
        SELECT m.id id, m.name name, m.summary summary, m.downloads downloads, m.follows follows,
        m.icon_url icon_url, m.updated updated, m.approved approved, m.published, m.license license, m.slug slug, m.color
        FROM mods m
        WHERE m.status = ANY($1)
        GROUP BY m.id;
        ",
        &*crate::models::projects::ProjectStatus::iterator()
        .filter(|x| x.is_searchable())
        .map(|x| x.to_string())
        .collect::<Vec<String>>(),
    )
        .fetch(pool)
        .map_ok(|m| {
            PartialProject {
                id: DBProjectId(m.id),
                name: m.name,
                summary: m.summary,
                downloads: m.downloads,
                follows: m.follows,
                icon_url: m.icon_url,
                updated: m.updated,
                approved: m.approved.unwrap_or(m.published),
                slug: m.slug,
                color: m.color,
                license: m.license,
            }
        })
        .try_collect::<Vec<PartialProject>>()
        .await?;

    let project_ids = db_projects.iter().map(|x| x.id.0).collect::<Vec<i64>>();

    struct PartialGallery {
        url: String,
        featured: bool,
        ordering: i64,
    }

    info!("Indexing local gallery!");

    let mods_gallery: DashMap<DBProjectId, Vec<PartialGallery>> = sqlx::query!(
        "
        SELECT mod_id, image_url, featured, ordering
        FROM mods_gallery
        WHERE mod_id = ANY($1)
        ",
        &*project_ids,
    )
    .fetch(pool)
    .try_fold(
        DashMap::new(),
        |acc: DashMap<DBProjectId, Vec<PartialGallery>>, m| {
            acc.entry(DBProjectId(m.mod_id)).or_default().push(
                PartialGallery {
                    url: m.image_url,
                    featured: m.featured.unwrap_or(false),
                    ordering: m.ordering,
                },
            );
            async move { Ok(acc) }
        },
    )
    .await?;

    info!("Indexing local categories!");

    let categories: DashMap<DBProjectId, Vec<(String, bool)>> = sqlx::query!(
        "
        SELECT mc.joining_mod_id mod_id, c.category name, mc.is_additional is_additional
        FROM mods_categories mc
        INNER JOIN categories c ON mc.joining_category_id = c.id
        WHERE joining_mod_id = ANY($1)
        ",
        &*project_ids,
    )
    .fetch(pool)
    .try_fold(
        DashMap::new(),
        |acc: DashMap<DBProjectId, Vec<(String, bool)>>, m| {
            acc.entry(DBProjectId(m.mod_id))
                .or_default()
                .push((m.name, m.is_additional));
            async move { Ok(acc) }
        },
    )
    .await?;

    info!("Indexing local versions!");
    let mut versions = index_versions(pool, project_ids.clone()).await?;

    info!("Indexing local org owners!");

    let mods_org_owners: DashMap<DBProjectId, String> = sqlx::query!(
        "
        SELECT m.id mod_id, u.username
        FROM mods m
        INNER JOIN organizations o ON o.id = m.organization_id
        INNER JOIN team_members tm ON tm.is_owner = TRUE and tm.team_id = o.team_id
        INNER JOIN users u ON u.id = tm.user_id
        WHERE m.id = ANY($1)
        ",
        &*project_ids,
    )
    .fetch(pool)
    .try_fold(DashMap::new(), |acc: DashMap<DBProjectId, String>, m| {
        acc.insert(DBProjectId(m.mod_id), m.username);
        async move { Ok(acc) }
    })
    .await?;

    info!("Indexing local team owners!");

    let mods_team_owners: DashMap<DBProjectId, String> = sqlx::query!(
        "
        SELECT m.id mod_id, u.username
        FROM mods m
        INNER JOIN team_members tm ON tm.is_owner = TRUE and tm.team_id = m.team_id
        INNER JOIN users u ON u.id = tm.user_id
        WHERE m.id = ANY($1)
        ",
        &project_ids,
    )
    .fetch(pool)
    .try_fold(DashMap::new(), |acc: DashMap<DBProjectId, String>, m| {
        acc.insert(DBProjectId(m.mod_id), m.username);
        async move { Ok(acc) }
    })
    .await?;

    info!("Getting all loader fields!");
    let loader_fields: Vec<QueryLoaderField> = sqlx::query!(
        "
        SELECT DISTINCT id, field, field_type, enum_type, min_val, max_val, optional
        FROM loader_fields lf
        ",
    )
    .fetch(pool)
    .map_ok(|m| QueryLoaderField {
        id: LoaderFieldId(m.id),
        field: m.field,
        field_type: m.field_type,
        enum_type: m.enum_type.map(LoaderFieldEnumId),
        min_val: m.min_val,
        max_val: m.max_val,
        optional: m.optional,
    })
    .try_collect()
    .await?;
    let loader_fields: Vec<&QueryLoaderField> = loader_fields.iter().collect();

    info!("Getting all loader field enum values!");

    let loader_field_enum_values: Vec<QueryLoaderFieldEnumValue> =
        sqlx::query!(
            "
        SELECT DISTINCT id, enum_id, value, ordering, created, metadata
        FROM loader_field_enum_values lfev
        ORDER BY enum_id, ordering, created DESC
        "
        )
        .fetch(pool)
        .map_ok(|m| QueryLoaderFieldEnumValue {
            id: LoaderFieldEnumValueId(m.id),
            enum_id: LoaderFieldEnumId(m.enum_id),
            value: m.value,
            ordering: m.ordering,
            created: m.created,
            metadata: m.metadata,
        })
        .try_collect()
        .await?;

    info!("Indexing loaders, project types!");
    let mut uploads = Vec::new();

    let total_len = db_projects.len();
    let mut count = 0;
    for project in db_projects {
        count += 1;
        info!("projects index prog: {count}/{total_len}");

        let owner =
            if let Some((_, org_owner)) = mods_org_owners.remove(&project.id) {
                org_owner
            } else if let Some((_, team_owner)) =
                mods_team_owners.remove(&project.id)
            {
                team_owner
            } else {
                println!(
                    "org owner not found for project {} id: {}!",
                    project.name, project.id.0
                );
                continue;
            };

        let license = match project.license.split(' ').next() {
            Some(license) => license.to_string(),
            None => project.license.clone(),
        };

        let open_source = match spdx::license_id(&license) {
            Some(id) => id.is_osi_approved(),
            _ => false,
        };

        let (featured_gallery, gallery) =
            if let Some((_, gallery)) = mods_gallery.remove(&project.id) {
                let mut vals = Vec::new();
                let mut featured = None;

                for x in gallery
                    .into_iter()
                    .sorted_by(|a, b| a.ordering.cmp(&b.ordering))
                {
                    if x.featured && featured.is_none() {
                        featured = Some(x.url);
                    } else {
                        vals.push(x.url);
                    }
                }

                (featured, vals)
            } else {
                (None, vec![])
            };

        let (categories, display_categories) =
            if let Some((_, categories)) = categories.remove(&project.id) {
                let mut vals = Vec::new();
                let mut featured_vals = Vec::new();

                for (val, is_additional) in categories {
                    if !is_additional {
                        featured_vals.push(val.clone());
                    }

                    vals.push(val);
                }

                (vals, featured_vals)
            } else {
                (vec![], vec![])
            };

        if let Some(versions) = versions.remove(&project.id) {
            // Aggregated project loader fields
            let project_version_fields = versions
                .iter()
                .flat_map(|x| x.version_fields.clone())
                .collect::<Vec<_>>();
            let aggregated_version_fields = VersionField::from_query_json(
                project_version_fields,
                &loader_fields,
                &loader_field_enum_values,
                true,
            );
            let project_loader_fields =
                from_duplicate_version_fields(aggregated_version_fields);

            // aggregated project loaders
            let project_loaders = versions
                .iter()
                .flat_map(|x| x.loaders.clone())
                .collect::<Vec<_>>();

            for version in versions {
                let version_fields = VersionField::from_query_json(
                    version.version_fields,
                    &loader_fields,
                    &loader_field_enum_values,
                    false,
                );
                let unvectorized_loader_fields = version_fields
                    .iter()
                    .map(|vf| {
                        (vf.field_name.clone(), vf.value.serialize_internal())
                    })
                    .collect();
                let mut loader_fields =
                    from_duplicate_version_fields(version_fields);
                let project_types = version.project_types;

                let mut version_loaders = version.loaders;

                // Uses version loaders, not project loaders.
                let mut categories = categories.clone();
                categories.append(&mut version_loaders.clone());

                let display_categories = display_categories.clone();
                categories.append(&mut version_loaders);

                // SPECIAL BEHAVIOUR
                // Todo: revisit.
                // For consistency with v2 searching, we consider the loader field 'mrpack_loaders' to be a category.
                // These were previously considered the loader, and in v2, the loader is a category for searching.
                // So to avoid breakage or awkward conversions, we just consider those loader_fields to be categories.
                // The loaders are kept in loader_fields as well, so that no information is lost on retrieval.
                let mrpack_loaders = loader_fields
                    .get("mrpack_loaders")
                    .cloned()
                    .map(|x| {
                        x.into_iter()
                            .filter_map(|x| x.as_str().map(String::from))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                categories.extend(mrpack_loaders);
                if loader_fields.contains_key("mrpack_loaders") {
                    categories.retain(|x| *x != "mrpack");
                }

                // SPECIAL BEHAVIOUR:
                // For consitency with v2 searching, we manually input the
                // client_side and server_side fields from the loader fields into
                // separate loader fields.
                // 'client_side' and 'server_side' remain supported by meilisearch even though they are no longer v3 fields.
                let (_, v2_og_project_type) =
                    LegacyProject::get_project_type(&project_types);
                let (client_side, server_side) =
                    v2_reroute::convert_side_types_v2(
                        &unvectorized_loader_fields,
                        Some(&v2_og_project_type),
                    );

                if let Ok(client_side) = serde_json::to_value(client_side) {
                    loader_fields
                        .insert("client_side".to_string(), vec![client_side]);
                }
                if let Ok(server_side) = serde_json::to_value(server_side) {
                    loader_fields
                        .insert("server_side".to_string(), vec![server_side]);
                }

                let usp = UploadSearchProject {
                    version_id: crate::models::ids::VersionId::from(version.id)
                        .to_string(),
                    project_id: crate::models::ids::ProjectId::from(project.id)
                        .to_string(),
                    name: project.name.clone(),
                    summary: project.summary.clone(),
                    categories: categories.clone(),
                    display_categories: display_categories.clone(),
                    follows: project.follows,
                    downloads: project.downloads,
                    icon_url: project.icon_url.clone(),
                    author: owner.clone(),
                    date_created: project.approved,
                    created_timestamp: project.approved.timestamp(),
                    date_modified: project.updated,
                    modified_timestamp: project.updated.timestamp(),
                    license: license.clone(),
                    slug: project.slug.clone(),
                    // TODO
                    project_types,
                    gallery: gallery.clone(),
                    featured_gallery: featured_gallery.clone(),
                    open_source,
                    color: project.color.map(|x| x as u32),
                    loader_fields,
                    project_loader_fields: project_loader_fields.clone(),
                    // 'loaders' is aggregate of all versions' loaders
                    loaders: project_loaders.clone(),
                };

                uploads.push(usp);
            }
        }
    }

    Ok(uploads)
}

struct PartialVersion {
    id: DBVersionId,
    loaders: Vec<String>,
    project_types: Vec<String>,
    version_fields: Vec<QueryVersionField>,
}

async fn index_versions(
    pool: &PgPool,
    project_ids: Vec<i64>,
) -> Result<HashMap<DBProjectId, Vec<PartialVersion>>, IndexingError> {
    let versions: HashMap<DBProjectId, Vec<DBVersionId>> = sqlx::query!(
        "
        SELECT v.id, v.mod_id
        FROM versions v
        WHERE mod_id = ANY($1)
        ",
        &project_ids,
    )
    .fetch(pool)
    .try_fold(
        HashMap::new(),
        |mut acc: HashMap<DBProjectId, Vec<DBVersionId>>, m| {
            acc.entry(DBProjectId(m.mod_id))
                .or_default()
                .push(DBVersionId(m.id));
            async move { Ok(acc) }
        },
    )
    .await?;

    // Get project types, loaders
    #[derive(Default)]
    struct VersionLoaderData {
        loaders: Vec<String>,
        project_types: Vec<String>,
    }

    let all_version_ids = versions
        .iter()
        .flat_map(|(_, version_ids)| version_ids.iter())
        .map(|x| x.0)
        .collect::<Vec<i64>>();

    let loaders_ptypes: DashMap<DBVersionId, VersionLoaderData> = sqlx::query!(
        "
        SELECT DISTINCT version_id,
            ARRAY_AGG(DISTINCT l.loader) filter (where l.loader is not null) loaders,
            ARRAY_AGG(DISTINCT pt.name) filter (where pt.name is not null) project_types
        FROM versions v
        INNER JOIN loaders_versions lv ON v.id = lv.version_id
        INNER JOIN loaders l ON lv.loader_id = l.id
        INNER JOIN loaders_project_types lpt ON lpt.joining_loader_id = l.id
        INNER JOIN project_types pt ON pt.id = lpt.joining_project_type_id
        WHERE v.id = ANY($1)
        GROUP BY version_id
        ",
        &all_version_ids
    )
    .fetch(pool)
    .map_ok(|m| {
        let version_id = DBVersionId(m.version_id);

        let version_loader_data = VersionLoaderData {
            loaders: m.loaders.unwrap_or_default(),
            project_types: m.project_types.unwrap_or_default(),
        };
        (version_id, version_loader_data)
    })
    .try_collect()
    .await?;

    // Get version fields
    let version_fields: DashMap<DBVersionId, Vec<QueryVersionField>> =
        sqlx::query!(
            "
        SELECT version_id, field_id, int_value, enum_value, string_value
        FROM version_fields
        WHERE version_id = ANY($1)
        ",
            &all_version_ids,
        )
        .fetch(pool)
        .try_fold(
            DashMap::new(),
            |acc: DashMap<DBVersionId, Vec<QueryVersionField>>, m| {
                let qvf = QueryVersionField {
                    version_id: DBVersionId(m.version_id),
                    field_id: LoaderFieldId(m.field_id),
                    int_value: m.int_value,
                    enum_value: if m.enum_value == -1 {
                        None
                    } else {
                        Some(LoaderFieldEnumValueId(m.enum_value))
                    },
                    string_value: m.string_value,
                };

                acc.entry(DBVersionId(m.version_id)).or_default().push(qvf);
                async move { Ok(acc) }
            },
        )
        .await?;

    // Convert to partial versions
    let mut res_versions: HashMap<DBProjectId, Vec<PartialVersion>> =
        HashMap::new();
    for (project_id, version_ids) in versions.iter() {
        for version_id in version_ids {
            // Extract version-specific data fetched
            // We use 'remove' as every version is only in the map once
            let version_loader_data = loaders_ptypes
                .remove(version_id)
                .map(|(_, version_loader_data)| version_loader_data)
                .unwrap_or_default();

            let version_fields = version_fields
                .remove(version_id)
                .map(|(_, version_fields)| version_fields)
                .unwrap_or_default();

            res_versions
                .entry(*project_id)
                .or_default()
                .push(PartialVersion {
                    id: *version_id,
                    loaders: version_loader_data.loaders,
                    project_types: version_loader_data.project_types,
                    version_fields,
                });
        }
    }

    Ok(res_versions)
}
