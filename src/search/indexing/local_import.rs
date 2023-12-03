use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashSet;
use futures::TryStreamExt;
use log::info;

use super::IndexingError;
use crate::database::models::{project_item, version_item, ProjectId, VersionId};
use crate::database::redis::RedisPool;
use crate::models;
use crate::search::UploadSearchProject;
use sqlx::postgres::PgPool;

pub async fn index_local(
    pool: PgPool,
    redis: &RedisPool,
) -> Result<(Vec<UploadSearchProject>, Vec<String>), IndexingError> {
    info!("Indexing local projects!");
    let loader_field_keys: Arc<DashSet<String>> = Arc::new(DashSet::new());

    let all_visible_ids: HashMap<VersionId, (ProjectId, String)> = sqlx::query!(
        "
        SELECT v.id id, m.id mod_id, u.username owner_username
        
        FROM versions v
        INNER JOIN mods m ON v.mod_id = m.id AND m.status = ANY($2)
        INNER JOIN team_members tm ON tm.team_id = m.team_id AND tm.is_owner = TRUE AND tm.accepted = TRUE
        INNER JOIN users u ON tm.user_id = u.id
        WHERE v.status != ANY($1)
        GROUP BY v.id, m.id, u.id
        ORDER BY m.id DESC;
        ",
        &*crate::models::projects::VersionStatus::iterator()
            .filter(|x| x.is_hidden())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        &*crate::models::projects::ProjectStatus::iterator()
            .filter(|x| x.is_searchable())
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    )
    .fetch_many(&pool)
    .try_filter_map(|e| async move {
        Ok(e.right().map(|m| {
            let project_id: ProjectId = ProjectId(m.mod_id);
            let version_id: VersionId = VersionId(m.id);
            (version_id, (project_id, m.owner_username))
        }))
    })
    .try_collect::<HashMap<_, _>>()
    .await?;

    let project_ids = all_visible_ids
        .values()
        .map(|(project_id, _)| project_id)
        .cloned()
        .collect::<Vec<_>>();
    let projects: HashMap<_, _> = project_item::Project::get_many_ids(&project_ids, &pool, redis)
        .await?
        .into_iter()
        .map(|p| (p.inner.id, p))
        .collect();

    let version_ids = all_visible_ids.keys().cloned().collect::<Vec<_>>();
    let versions: HashMap<_, _> = version_item::Version::get_many(&version_ids, &pool, redis)
        .await?
        .into_iter()
        .map(|v| (v.inner.id, v))
        .collect();

    let mut uploads = Vec::new();
    // TODO: could possibly clone less here?
    for (version_id, (project_id, owner_username)) in all_visible_ids {
        let m = projects.get(&project_id);
        let v = versions.get(&version_id);

        let m = match m {
            Some(m) => m,
            None => continue,
        };

        let v = match v {
            Some(v) => v,
            None => continue,
        };

        let version_id: crate::models::projects::VersionId = v.inner.id.into();
        let project_id: crate::models::projects::ProjectId = m.inner.id.into();
        let team_id: crate::models::teams::TeamId = m.inner.team_id.into();
        let organization_id: Option<crate::models::organizations::OrganizationId> =
            m.inner.organization_id.map(|x| x.into());
        let thread_id: crate::models::threads::ThreadId = m.thread_id.into();

        let all_version_ids = m
            .versions
            .iter()
            .map(|v| (*v).into())
            .collect::<Vec<crate::models::projects::VersionId>>();

        let mut additional_categories = m.additional_categories.clone();
        let mut categories = m.categories.clone();

        // Uses version loaders, not project loaders.
        categories.append(&mut v.loaders.clone());

        let display_categories = categories.clone();
        categories.append(&mut additional_categories);

        let version_fields = v.version_fields.clone();
        let loader_fields = models::projects::from_duplicate_version_fields(version_fields);
        for v in loader_fields.keys().cloned() {
            loader_field_keys.insert(v);
        }

        let license = match m.inner.license.split(' ').next() {
            Some(license) => license.to_string(),
            None => m.inner.license.clone(),
        };

        let open_source = match spdx::license_id(&license) {
            Some(id) => id.is_osi_approved(),
            _ => false,
        };

        // For loaders, get ALL loaders across ALL versions
        let mut loaders = all_version_ids
            .iter()
            .fold(vec![], |mut loaders, version_id| {
                let version = versions.get(&(*version_id).into());
                if let Some(version) = version {
                    loaders.extend(version.loaders.clone());
                }
                loaders
            });
        loaders.sort();
        loaders.dedup();

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

        let gallery = m
            .gallery_items
            .iter()
            .filter(|gi| !gi.featured)
            .map(|gi| gi.image_url.clone())
            .collect::<Vec<_>>();
        let featured_gallery = m
            .gallery_items
            .iter()
            .filter(|gi| gi.featured)
            .map(|gi| gi.image_url.clone())
            .collect::<Vec<_>>();
        let featured_gallery = featured_gallery.first().cloned();

        let usp = UploadSearchProject {
            version_id: version_id.to_string(),
            project_id: project_id.to_string(),
            name: m.inner.name.clone(),
            summary: m.inner.summary.clone(),
            categories,
            follows: m.inner.follows,
            downloads: m.inner.downloads,
            icon_url: m.inner.icon_url.clone(),
            author: owner_username,
            date_created: m.inner.approved.unwrap_or(m.inner.published),
            created_timestamp: m.inner.approved.unwrap_or(m.inner.published).timestamp(),
            date_modified: m.inner.updated,
            modified_timestamp: m.inner.updated.timestamp(),
            license,
            slug: m.inner.slug.clone(),
            project_types: m.project_types.clone(),
            gallery,
            featured_gallery,
            display_categories,
            open_source,
            color: m.inner.color,
            loader_fields,
            license_url: m.inner.license_url.clone(),
            monetization_status: Some(m.inner.monetization_status),
            team_id: team_id.to_string(),
            organization_id: organization_id.map(|x| x.to_string()),
            thread_id: thread_id.to_string(),
            versions: all_version_ids.iter().map(|x| x.to_string()).collect(),
            date_published: m.inner.published,
            date_queued: m.inner.queued,
            status: m.inner.status,
            requested_status: m.inner.requested_status,
            games: m.games.clone(),
            links: m.urls.clone(),
            gallery_items: m.gallery_items.clone(),
            loaders,
        };

        uploads.push(usp);
    }

    Ok((
        uploads,
        Arc::try_unwrap(loader_field_keys)
            .unwrap_or_default()
            .into_iter()
            .collect(),
    ))
}
