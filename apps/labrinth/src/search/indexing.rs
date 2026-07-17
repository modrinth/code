use chrono::{DateTime, Utc};
use dashmap::DashMap;
use eyre::Result;
use futures::TryStreamExt;
use heck::ToKebabCase;
use itertools::Itertools;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use tracing::{info, warn};

use crate::database::PgPool;
use crate::database::models::loader_fields::{
    QueryLoaderField, QueryLoaderFieldEnumValue, QueryVersionField,
    VersionField,
};
use crate::database::models::{
    DBOrganizationId, DBProjectId, DBUserId, DBVersionId, LoaderFieldEnumId,
    LoaderFieldEnumValueId, LoaderFieldId,
};
use crate::database::redis::RedisPool;
use crate::models::exp;
use crate::models::ids::{ProjectId, VersionId};
use crate::models::projects::{DependencyType, from_duplicate_version_fields};
use crate::models::v2::projects::LegacyProject;
use crate::routes::v2_reroute;
use crate::search::{
    SearchDocumentBatch, SearchProjectDependency, UploadSearchProject,
    UploadSearchVersion,
};
use crate::util::error::Context;

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
    components: exp::ProjectSerial,
}

fn normalize_for_search(s: &str) -> String {
    static SPECIAL_CHARS_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[^a-zA-Z0-9-.\s]").expect("valid regex"));

    SPECIAL_CHARS_RE.replace_all(s, "").to_kebab_case()
}

fn searchable_statuses() -> Vec<String> {
    crate::models::projects::ProjectStatus::iterator()
        .filter(|x| x.is_searchable())
        .map(|x| x.to_string())
        .collect()
}

struct ProjectOwner {
    username: String,
    user_id: DBUserId,
    org_name: Option<String>,
    org_id: Option<DBOrganizationId>,
}

pub async fn index_local(
    pool: &PgPool,
    redis: &RedisPool,
    cursor: i64,
    limit: i64,
) -> eyre::Result<(SearchDocumentBatch, i64)> {
    info!("Indexing local projects!");

    let searchable_statuses = searchable_statuses();

    let db_projects = sqlx::query!(
		r#"
        SELECT m.id id, m.name name, m.summary summary, m.downloads downloads, m.follows follows,
        m.icon_url icon_url, m.updated updated, m.approved approved, m.published, m.license license, m.slug slug, m.color,
        m.components AS "components: sqlx::types::Json<exp::ProjectSerial>"
        FROM mods m
        WHERE m.status = ANY($1) AND m.id > $3
        GROUP BY m.id
        ORDER BY m.id ASC
        LIMIT $2;
        "#,
        &searchable_statuses,
        limit,
        cursor,
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
                components: m.components.0,
            }
        })
        .try_collect::<Vec<PartialProject>>()
        .await
        .wrap_err("failed to fetch projects")?;

    let project_ids = db_projects.iter().map(|x| x.id.0).collect::<Vec<i64>>();
    let Some(largest) = project_ids.iter().max() else {
        return Ok((SearchDocumentBatch::default(), i64::MAX));
    };

    let documents =
        build_search_documents(pool, redis, db_projects, None).await?;
    Ok((documents, *largest))
}

pub async fn build_project_documents(
    pool: &PgPool,
    redis: &RedisPool,
    project_ids: &[ProjectId],
) -> eyre::Result<Vec<UploadSearchProject>> {
    let version_ids = HashSet::new();
    Ok(
        build_search_document_batch(pool, redis, project_ids, &version_ids)
            .await?
            .projects,
    )
}

pub async fn build_version_change_documents(
    pool: &PgPool,
    redis: &RedisPool,
    project_ids: &[ProjectId],
    version_ids: &[VersionId],
) -> eyre::Result<SearchDocumentBatch> {
    let version_ids = version_ids
        .iter()
        .copied()
        .map(DBVersionId::from)
        .collect::<HashSet<_>>();
    build_search_document_batch(pool, redis, project_ids, &version_ids).await
}

async fn build_search_document_batch(
    pool: &PgPool,
    redis: &RedisPool,
    project_ids: &[ProjectId],
    version_ids: &HashSet<DBVersionId>,
) -> eyre::Result<SearchDocumentBatch> {
    let searchable_statuses = searchable_statuses();
    let project_ids = project_ids
        .iter()
        .map(|project_id| DBProjectId::from(*project_id).0)
        .collect::<Vec<_>>();

    let db_projects = sqlx::query!(
        r#"
		SELECT m.id id, m.name name, m.summary summary, m.downloads downloads, m.follows follows,
		m.icon_url icon_url, m.updated updated, m.approved approved, m.published, m.license license, m.slug slug, m.color,
		m.components AS "components: sqlx::types::Json<exp::ProjectSerial>"
		FROM mods m
		WHERE m.status = ANY($1) AND m.id = ANY($2)
		GROUP BY m.id
		ORDER BY m.id ASC;
		"#,
        &searchable_statuses,
        &project_ids,
    )
    .fetch(pool)
    .map_ok(|m| PartialProject {
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
        components: m.components.0,
    })
    .try_collect::<Vec<PartialProject>>()
    .await
    .wrap_err("failed to fetch project")?;

    build_search_documents(pool, redis, db_projects, Some(version_ids)).await
}

async fn build_search_documents(
    pool: &PgPool,
    redis: &RedisPool,
    db_projects: Vec<PartialProject>,
    version_ids: Option<&HashSet<DBVersionId>>,
) -> eyre::Result<SearchDocumentBatch> {
    let searchable_statuses = searchable_statuses();
    let project_ids = db_projects.iter().map(|x| x.id.0).collect::<Vec<i64>>();
    let project_components = db_projects
        .iter()
        .map(|project| (ProjectId::from(project.id), &project.components))
        .collect::<Vec<_>>();
    let project_query_context =
        exp::project::fetch_query_context(&project_components, pool, redis)
            .await
            .wrap_err("failed to fetch query context")?;

    info!("Indexing local dependencies!");

    let dependencies: DashMap<DBProjectId, Vec<SearchProjectDependency>> =
        sqlx::query!(
            "
            SELECT DISTINCT v.mod_id dependent_project_id,
                d.mod_dependency_id dependency_project_id,
                d.dependency_type dependency_type,
                m.name dependency_name,
                m.slug dependency_slug,
                m.icon_url dependency_icon_url
            FROM versions v
            INNER JOIN dependencies d ON d.dependent_id = v.id
            INNER JOIN mods m ON m.id = d.mod_dependency_id
            WHERE v.mod_id = ANY($1)
                AND d.mod_dependency_id IS NOT NULL
                AND m.status = ANY($2)
            ",
            &project_ids,
            &searchable_statuses,
        )
        .fetch(pool)
        .try_fold(
            DashMap::new(),
            |acc: DashMap<DBProjectId, Vec<SearchProjectDependency>>, m| {
                if let Some(dependency_project_id) = m.dependency_project_id {
                    acc.entry(DBProjectId(m.dependent_project_id))
                        .or_default()
                        .push(SearchProjectDependency {
                            project_id: ProjectId::from(DBProjectId(
                                dependency_project_id,
                            ))
                            .to_string(),
                            dependency_type: DependencyType::from_string(
                                &m.dependency_type,
                            ),
                            name: m.dependency_name,
                            slug: m.dependency_slug,
                            icon_url: m.dependency_icon_url,
                        });
                }

                async move { Ok(acc) }
            },
        )
        .await
        .wrap_err("failed to fetch project dependencies")?;

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
    let mut versions = load_project_versions(pool, project_ids.clone()).await?;

    info!("Indexing local org owners!");

    let mods_org_owners: DashMap<DBProjectId, ProjectOwner> = sqlx::query!(
        "
        SELECT m.id mod_id, u.username, u.id uid, o.name orgname, o.id oid
        FROM mods m
        INNER JOIN organizations o ON o.id = m.organization_id
        INNER JOIN team_members tm ON tm.is_owner = TRUE and tm.team_id = o.team_id
        INNER JOIN users u ON u.id = tm.user_id
        WHERE m.id = ANY($1)
        ",
        &*project_ids,
    )
    .fetch(pool)
    .try_fold(DashMap::new(), |acc: DashMap<DBProjectId, ProjectOwner>, m| {
        acc.insert(DBProjectId(m.mod_id), ProjectOwner {
			username: m.username,
			user_id: DBUserId(m.uid),
			org_name: Some(m.orgname),
			org_id: Some(DBOrganizationId(m.oid)),
		});
        async move { Ok(acc) }
    })
    .await?;

    info!("Indexing local team owners!");

    let mods_team_owners: DashMap<DBProjectId, ProjectOwner> = sqlx::query!(
        "
        SELECT m.id mod_id, u.username, u.id uid
        FROM mods m
        INNER JOIN team_members tm ON tm.is_owner = TRUE and tm.team_id = m.team_id
        INNER JOIN users u ON u.id = tm.user_id
        WHERE m.id = ANY($1)
        ",
        &project_ids,
    )
    .fetch(pool)
    .try_fold(DashMap::new(), |acc: DashMap<DBProjectId, ProjectOwner>, m| {
        acc.insert(DBProjectId(m.mod_id), ProjectOwner {
			username: m.username,
			user_id: DBUserId(m.uid),
			org_name: None,
			org_id: None,
		});
        async move { Ok(acc) }
    })
    .await?;

    info!("Getting all loader fields!");
    let loader_field_definitions: Vec<QueryLoaderField> = sqlx::query!(
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
    let loader_field_definitions: Vec<&QueryLoaderField> =
        loader_field_definitions.iter().collect();

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
    let mut project_uploads = Vec::new();
    let mut version_uploads = Vec::new();

    let total_len = db_projects.len();
    let mut count = 0;
    for project in db_projects {
        count += 1;

        if count % 1000 == 0 {
            info!("projects index prog: {count}/{total_len}");
        }
        let Some((
            _,
            ProjectOwner {
                username,
                user_id,
                org_name,
                org_id,
            },
        )) = mods_org_owners
            .remove(&project.id)
            .or_else(|| mods_team_owners.remove(&project.id))
        else {
            warn!(
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
        let dependencies = dependencies
            .get(&project.id)
            .map(|x| x.clone())
            .unwrap_or_default();
        let dependency_project_ids = dependencies
            .iter()
            .map(|dependency| dependency.project_id.clone())
            .collect::<Vec<_>>();
        let compatible_dependency_project_ids = dependencies
            .iter()
            .filter(|dependency| {
                matches!(
                    dependency.dependency_type,
                    DependencyType::Required
                        | DependencyType::Optional
                        | DependencyType::Embedded
                )
            })
            .map(|dependency| dependency.project_id.clone())
            .collect::<Vec<_>>();

        if let Some(versions) = versions.remove(&project.id) {
            let Some(latest_version) = versions.iter().max_by(|a, b| {
                a.date_published
                    .cmp(&b.date_published)
                    .then_with(|| a.id.0.cmp(&b.id.0))
            }) else {
                continue;
            };

            let project_version_fields = versions
                .iter()
                .flat_map(|x| x.version_fields.clone())
                .collect::<Vec<_>>();
            let aggregated_version_fields = VersionField::from_query_json(
                project_version_fields,
                &loader_field_definitions,
                &loader_field_enum_values,
                true,
            );
            let unvectorized_loader_fields = aggregated_version_fields
                .iter()
                .map(|field| {
                    (field.field_name.clone(), field.value.serialize_internal())
                })
                .collect();
            let mut loader_fields =
                from_duplicate_version_fields(aggregated_version_fields);
            let project_loader_fields = loader_fields.clone();

            let mut project_loaders = versions
                .iter()
                .flat_map(|x| x.loaders.clone())
                .collect::<Vec<_>>();
            project_loaders.sort();
            project_loaders.dedup();

            let mut project_types = versions
                .iter()
                .flat_map(|x| x.project_types.clone())
                .collect::<Vec<_>>();
            project_types.sort();
            project_types.dedup();
            exp::compat::correct_project_types(
                &project.components,
                &mut project_types,
            );

            let project_id = ProjectId::from(project.id).to_string();
            version_uploads.extend(versions.iter().filter_map(|version| {
                if version_ids.is_some_and(|version_ids| {
                    !version_ids.contains(&version.id)
                }) {
                    return None;
                }

                let version_fields = VersionField::from_query_json(
                    version.version_fields.clone(),
                    &loader_field_definitions,
                    &loader_field_enum_values,
                    false,
                );
                let unvectorized_loader_fields = version_fields
                    .iter()
                    .map(|field| {
                        (
                            field.field_name.clone(),
                            field.value.serialize_internal(),
                        )
                    })
                    .collect();
                let mut fields = from_duplicate_version_fields(version_fields);
                let mut version_project_types = version.project_types.clone();
                exp::compat::correct_project_types(
                    &project.components,
                    &mut version_project_types,
                );

                let mut version_categories = version.loaders.clone();
                let mrpack_loaders = fields
                    .get("mrpack_loaders")
                    .into_iter()
                    .flatten()
                    .filter_map(|value| value.as_str().map(String::from))
                    .collect::<Vec<_>>();
                version_categories.extend(mrpack_loaders);
                if fields.contains_key("mrpack_loaders") {
                    version_categories.retain(|category| category != "mrpack");
                }
                version_categories.sort();
                version_categories.dedup();

                let (_, v2_og_project_type) =
                    LegacyProject::get_project_type(&version_project_types);
                let (client_side, server_side) =
                    v2_reroute::convert_v3_side_types_to_v2_side_types(
                        &unvectorized_loader_fields,
                        Some(&v2_og_project_type),
                    );
                if let Ok(client_side) = serde_json::to_value(client_side) {
                    fields.insert("client_side".to_string(), vec![client_side]);
                }
                if let Ok(server_side) = serde_json::to_value(server_side) {
                    fields.insert("server_side".to_string(), vec![server_side]);
                }
                fields.retain(|field, _| {
                    matches!(
                        field.as_str(),
                        "environment"
                            | "game_versions"
                            | "client_side"
                            | "server_side"
                    )
                });

                Some(UploadSearchVersion {
                    version_id: VersionId::from(version.id).to_string(),
                    project_id: project_id.clone(),
                    categories: version_categories,
                    project_types: version_project_types,
                    version_published_timestamp: version
                        .date_published
                        .timestamp(),
                    loader_fields: fields,
                })
            }));

            let mut project_categories = categories;
            project_categories.sort();
            project_categories.dedup();
            let mut categories = project_categories.clone();
            categories.extend(project_loaders.iter().cloned());

            let mrpack_loaders = loader_fields
                .get("mrpack_loaders")
                .into_iter()
                .flatten()
                .filter_map(|value| value.as_str().map(String::from))
                .collect::<Vec<_>>();
            categories.extend(mrpack_loaders);
            if loader_fields.contains_key("mrpack_loaders") {
                categories.retain(|category| category != "mrpack");
            }
            categories.sort();
            categories.dedup();

            let (_, v2_og_project_type) =
                LegacyProject::get_project_type(&project_types);
            let (client_side, server_side) =
                v2_reroute::convert_v3_side_types_to_v2_side_types(
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

            let components = project
                .components
                .clone()
                .into_query(ProjectId::from(project.id), &project_query_context)
                .wrap_err("failed to populate query components")?;
            let indexed_name = normalize_for_search(&project.name);

            project_uploads.push(UploadSearchProject {
                version_id: crate::models::ids::VersionId::from(
                    latest_version.id,
                )
                .to_string(),
                project_id,
                name: project.name,
                indexed_name,
                summary: project.summary,
                categories,
                project_categories,
                display_categories,
                follows: project.follows,
                downloads: project.downloads,
                log_downloads: (project.downloads.max(1) as f64).ln(),
                icon_url: project.icon_url,
                author: username.clone(),
                author_id: ariadne::ids::UserId::from(user_id).to_string(),
                organization: org_name,
                organization_id: org_id.map(|id| {
                    crate::models::ids::OrganizationId::from(id).to_string()
                }),
                indexed_author: normalize_for_search(&username),
                date_created: project.approved,
                created_timestamp: project.approved.timestamp(),
                date_modified: project.updated,
                modified_timestamp: project.updated.timestamp(),
                version_published_timestamp: latest_version
                    .date_published
                    .timestamp(),
                license,
                slug: project.slug,
                project_types: project_types.clone(),
                all_project_types: project_types,
                gallery,
                featured_gallery,
                open_source,
                color: project.color.map(|x| x as u32),
                dependency_project_ids,
                compatible_dependency_project_ids,
                dependencies,
                project_loader_fields,
                loader_fields,
                loaders: project_loaders,
                components,
            });
        }
    }

    Ok(SearchDocumentBatch {
        projects: project_uploads,
        versions: version_uploads,
    })
}

struct PartialVersion {
    id: DBVersionId,
    loaders: Vec<String>,
    project_types: Vec<String>,
    version_fields: Vec<QueryVersionField>,
    date_published: DateTime<Utc>,
}

async fn load_project_versions(
    pool: &PgPool,
    project_ids: Vec<i64>,
) -> Result<HashMap<DBProjectId, Vec<PartialVersion>>> {
    let versions: HashMap<DBProjectId, Vec<(DBVersionId, DateTime<Utc>)>> =
        sqlx::query!(
            "
        SELECT v.id, v.mod_id, v.date_published
        FROM versions v
        WHERE mod_id = ANY($1)
        ",
            &project_ids,
        )
        .fetch(pool)
        .try_fold(
            HashMap::new(),
            |mut acc: HashMap<
                DBProjectId,
                Vec<(DBVersionId, DateTime<Utc>)>,
            >,
             m| {
                acc.entry(DBProjectId(m.mod_id))
                    .or_default()
                    .push((DBVersionId(m.id), m.date_published));
                async move { Ok(acc) }
            },
        )
        .await
        .wrap_err("failed to fetch versions")?;

    // Get project types, loaders
    #[derive(Default)]
    struct VersionLoaderData {
        loaders: Vec<String>,
        project_types: Vec<String>,
    }

    let all_version_ids = versions
        .values()
        .flat_map(|version_ids| version_ids.iter())
        .map(|(version_id, _)| version_id.0)
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
    .await
    .wrap_err("failed to fetch loaders and project types")?;

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
        .await
        .wrap_err("failed to fetch version fields")?;

    // Get version fields

    // Convert to partial versions
    let mut res_versions: HashMap<DBProjectId, Vec<PartialVersion>> =
        HashMap::new();
    for (project_id, version_ids) in &versions {
        for (version_id, date_published) in version_ids {
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
                    date_published: *date_published,
                });
        }
    }

    Ok(res_versions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_for_search_removes_special_chars() {
        assert_eq!(normalize_for_search("Xaero's Minimap"), "xaeros-minimap");
        assert_eq!(normalize_for_search("JourneyMap"), "journey-map");
        assert_eq!(normalize_for_search("journey-map"), "journey-map");
        assert_eq!(normalize_for_search("SomeUserName"), "some-user-name");
    }

    #[test]
    fn test_normalize_for_search_handles_whitespace() {
        assert_eq!(
            normalize_for_search("Some  Project  Name"),
            "some-project-name"
        );
        assert_eq!(normalize_for_search("  padded  "), "padded");
    }

    #[test]
    fn test_normalize_for_search_handles_numbers() {
        assert_eq!(normalize_for_search("Project 123"), "project-123");
        assert_eq!(normalize_for_search("Test 1.0"), "test-1-0");
    }
}
