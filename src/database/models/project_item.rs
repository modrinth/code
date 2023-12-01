use super::loader_fields::VersionField;
use super::{ids::*, User};
use crate::database::models;
use crate::database::models::DatabaseError;
use crate::database::redis::RedisPool;
use crate::models::ids::base62_impl::{parse_base62, to_base62};
use crate::models::projects::{MonetizationStatus, ProjectStatus};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

pub const PROJECTS_NAMESPACE: &str = "projects";
pub const PROJECTS_SLUGS_NAMESPACE: &str = "projects_slugs";
const PROJECTS_DEPENDENCIES_NAMESPACE: &str = "projects_dependencies";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinkUrl {
    pub platform_id: LinkPlatformId,
    pub platform_name: String,
    pub url: String,
    pub donation: bool, // Is this a donation link
}

impl LinkUrl {
    pub async fn insert_many_projects(
        links: Vec<Self>,
        project_id: ProjectId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        let (project_ids, platform_ids, urls): (Vec<_>, Vec<_>, Vec<_>) = links
            .into_iter()
            .map(|url| (project_id.0, url.platform_id.0, url.url))
            .multiunzip();
        sqlx::query!(
            "
            INSERT INTO mods_links (
                joining_mod_id, joining_platform_id, url
            )
            SELECT * FROM UNNEST($1::bigint[], $2::int[], $3::varchar[])
            ",
            &project_ids[..],
            &platform_ids[..],
            &urls[..],
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GalleryItem {
    pub image_url: String,
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub ordering: i64,
}

impl GalleryItem {
    pub async fn insert_many(
        items: Vec<Self>,
        project_id: ProjectId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        let (project_ids, image_urls, featureds, titles, descriptions, orderings): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = items
            .into_iter()
            .map(|gi| {
                (
                    project_id.0,
                    gi.image_url,
                    gi.featured,
                    gi.title,
                    gi.description,
                    gi.ordering,
                )
            })
            .multiunzip();
        sqlx::query!(
            "
            INSERT INTO mods_gallery (
                mod_id, image_url, featured, title, description, ordering
            )
            SELECT * FROM UNNEST ($1::bigint[], $2::varchar[], $3::bool[], $4::varchar[], $5::varchar[], $6::bigint[])
            ",
            &project_ids[..],
            &image_urls[..],
            &featureds[..],
            &titles[..] as &[Option<String>],
            &descriptions[..] as &[Option<String>],
            &orderings[..]
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

#[derive(derive_new::new)]
pub struct ModCategory {
    project_id: ProjectId,
    category_id: CategoryId,
    is_additional: bool,
}

impl ModCategory {
    pub async fn insert_many(
        items: Vec<Self>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let (project_ids, category_ids, is_additionals): (Vec<_>, Vec<_>, Vec<_>) = items
            .into_iter()
            .map(|mc| (mc.project_id.0, mc.category_id.0, mc.is_additional))
            .multiunzip();
        sqlx::query!(
            "
            INSERT INTO mods_categories (joining_mod_id, joining_category_id, is_additional)
            SELECT * FROM UNNEST ($1::bigint[], $2::int[], $3::bool[])
            ",
            &project_ids[..],
            &category_ids[..],
            &is_additionals[..]
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct ProjectBuilder {
    pub project_id: ProjectId,
    pub team_id: TeamId,
    pub organization_id: Option<OrganizationId>,
    pub title: String,
    pub description: String,
    pub body: String,
    pub icon_url: Option<String>,
    pub license_url: Option<String>,
    pub categories: Vec<CategoryId>,
    pub additional_categories: Vec<CategoryId>,
    pub initial_versions: Vec<super::version_item::VersionBuilder>,
    pub status: ProjectStatus,
    pub requested_status: Option<ProjectStatus>,
    pub license: String,
    pub slug: Option<String>,
    pub link_urls: Vec<LinkUrl>,
    pub gallery_items: Vec<GalleryItem>,
    pub color: Option<u32>,
    pub monetization_status: MonetizationStatus,
}

impl ProjectBuilder {
    pub async fn insert(
        self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<ProjectId, DatabaseError> {
        let project_struct = Project {
            id: self.project_id,
            team_id: self.team_id,
            organization_id: self.organization_id,
            title: self.title,
            description: self.description,
            body: self.body,
            body_url: None,
            published: Utc::now(),
            updated: Utc::now(),
            approved: None,
            queued: if self.status == ProjectStatus::Processing {
                Some(Utc::now())
            } else {
                None
            },
            status: self.status,
            requested_status: self.requested_status,
            downloads: 0,
            follows: 0,
            icon_url: self.icon_url,
            license_url: self.license_url,
            license: self.license,
            slug: self.slug,
            moderation_message: None,
            moderation_message_body: None,
            webhook_sent: false,
            color: self.color,
            monetization_status: self.monetization_status,
            loaders: vec![],
        };
        project_struct.insert(&mut *transaction).await?;

        let ProjectBuilder {
            link_urls,
            gallery_items,
            categories,
            additional_categories,
            ..
        } = self;

        for mut version in self.initial_versions {
            version.project_id = self.project_id;
            version.insert(&mut *transaction).await?;
        }

        LinkUrl::insert_many_projects(link_urls, self.project_id, &mut *transaction).await?;

        GalleryItem::insert_many(gallery_items, self.project_id, &mut *transaction).await?;

        let project_id = self.project_id;
        let mod_categories = categories
            .into_iter()
            .map(|c| ModCategory::new(project_id, c, false))
            .chain(
                additional_categories
                    .into_iter()
                    .map(|c| ModCategory::new(project_id, c, true)),
            )
            .collect_vec();
        ModCategory::insert_many(mod_categories, &mut *transaction).await?;

        Ok(self.project_id)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub team_id: TeamId,
    pub organization_id: Option<OrganizationId>,
    pub title: String,
    pub description: String,
    pub body: String,
    pub body_url: Option<String>,
    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub approved: Option<DateTime<Utc>>,
    pub queued: Option<DateTime<Utc>>,
    pub status: ProjectStatus,
    pub requested_status: Option<ProjectStatus>,
    pub downloads: i32,
    pub follows: i32,
    pub icon_url: Option<String>,
    pub license_url: Option<String>,
    pub license: String,
    pub slug: Option<String>,
    pub moderation_message: Option<String>,
    pub moderation_message_body: Option<String>,
    pub webhook_sent: bool,
    pub color: Option<u32>,
    pub monetization_status: MonetizationStatus,
    pub loaders: Vec<String>,
}

impl Project {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO mods (
                id, team_id, title, description, body,
                published, downloads, icon_url, status, requested_status,
                license_url, license,
                slug, color, monetization_status
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, 
                $7, $8, $9, $10, 
                $11, $12, 
                LOWER($13), $14, $15
            )
            ",
            self.id as ProjectId,
            self.team_id as TeamId,
            &self.title,
            &self.description,
            &self.body,
            self.published,
            self.downloads,
            self.icon_url.as_ref(),
            self.status.as_str(),
            self.requested_status.map(|x| x.as_str()),
            self.license_url.as_ref(),
            &self.license,
            self.slug.as_ref(),
            self.color.map(|x| x as i32),
            self.monetization_status.as_str(),
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn remove(
        id: ProjectId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        let project = Self::get_id(id, &mut **transaction, redis).await?;

        if let Some(project) = project {
            Project::clear_cache(id, project.inner.slug, Some(true), redis).await?;

            sqlx::query!(
                "
                DELETE FROM mod_follows
                WHERE mod_id = $1
                ",
                id as ProjectId
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM mods_gallery
                WHERE mod_id = $1
                ",
                id as ProjectId
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM mod_follows
                WHERE mod_id = $1
                ",
                id as ProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM reports
                WHERE mod_id = $1
                ",
                id as ProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM mods_categories
                WHERE joining_mod_id = $1
                ",
                id as ProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM mods_links
                WHERE joining_mod_id = $1
                ",
                id as ProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            for version in project.versions {
                super::Version::remove_full(version, redis, transaction).await?;
            }

            sqlx::query!(
                "
                DELETE FROM dependencies WHERE mod_dependency_id = $1
                ",
                id as ProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                UPDATE payouts_values
                SET mod_id = NULL
                WHERE (mod_id = $1)
                ",
                id as ProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            models::Thread::remove_full(project.thread_id, transaction).await?;

            sqlx::query!(
                "
                DELETE FROM mods
                WHERE id = $1
                ",
                id as ProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            models::TeamMember::clear_cache(project.inner.team_id, redis).await?;

            let affected_user_ids = sqlx::query!(
                "
                DELETE FROM team_members
                WHERE team_id = $1
                RETURNING user_id
                ",
                project.inner.team_id as TeamId,
            )
            .fetch_many(&mut **transaction)
            .try_filter_map(|e| async { Ok(e.right().map(|x| UserId(x.user_id))) })
            .try_collect::<Vec<_>>()
            .await?;

            User::clear_project_cache(&affected_user_ids, redis).await?;

            sqlx::query!(
                "
                DELETE FROM teams
                WHERE id = $1
                ",
                project.inner.team_id as TeamId,
            )
            .execute(&mut **transaction)
            .await?;

            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    pub async fn get<'a, 'b, E>(
        string: &str,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<QueryProject>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Project::get_many(&[string], executor, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_id<'a, 'b, E>(
        id: ProjectId,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<QueryProject>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Project::get_many(&[crate::models::ids::ProjectId::from(id)], executor, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many_ids<'a, E>(
        project_ids: &[ProjectId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<QueryProject>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let ids = project_ids
            .iter()
            .map(|x| crate::models::ids::ProjectId::from(*x))
            .collect::<Vec<_>>();
        Project::get_many(&ids, exec, redis).await
    }

    pub async fn get_many<'a, E, T: ToString>(
        project_strings: &[T],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<QueryProject>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        if project_strings.is_empty() {
            return Ok(Vec::new());
        }

        let mut redis = redis.connect().await?;

        let mut found_projects = Vec::new();
        let mut remaining_strings = project_strings
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();

        let mut project_ids = project_strings
            .iter()
            .flat_map(|x| parse_base62(&x.to_string()).map(|x| x as i64))
            .collect::<Vec<_>>();

        project_ids.append(
            &mut redis
                .multi_get::<i64>(
                    PROJECTS_SLUGS_NAMESPACE,
                    project_strings.iter().map(|x| x.to_string().to_lowercase()),
                )
                .await?
                .into_iter()
                .flatten()
                .collect(),
        );
        if !project_ids.is_empty() {
            let projects = redis
                .multi_get::<String>(
                    PROJECTS_NAMESPACE,
                    project_ids.iter().map(|x| x.to_string()),
                )
                .await?;
            for project in projects {
                if let Some(project) =
                    project.and_then(|x| serde_json::from_str::<QueryProject>(&x).ok())
                {
                    remaining_strings.retain(|x| {
                        &to_base62(project.inner.id.0 as u64) != x
                            && project.inner.slug.as_ref().map(|x| x.to_lowercase())
                                != Some(x.to_lowercase())
                    });
                    found_projects.push(project);
                    continue;
                }
            }
        }
        if !remaining_strings.is_empty() {
            let project_ids_parsed: Vec<i64> = remaining_strings
                .iter()
                .flat_map(|x| parse_base62(&x.to_string()).ok())
                .map(|x| x as i64)
                .collect();

            // TODO: Possible improvements to look into:
            // - use multiple queries instead of CTES (for cleanliness?)
            // - repeated joins to mods in separate CTEs- perhaps 1 CTE for mods and use later (in mods_gallery_json, mods_donations_json, etc.)
            let db_projects: Vec<QueryProject> = sqlx::query!(
                "
                WITH version_fields_cte AS (
                    SELECT mod_id, version_id, field_id, int_value, enum_value, string_value
                    FROM mods m
                    INNER JOIN versions v ON m.id = v.mod_id
                    INNER JOIN version_fields vf ON v.id = vf.version_id
                    WHERE m.id = ANY($1) OR m.slug = ANY($2)
                ),
				version_fields_json AS (
					SELECT DISTINCT mod_id,
                    JSONB_AGG( 
                        DISTINCT jsonb_build_object('version_id', version_id, 'field_id', field_id, 'int_value', int_value, 'enum_value', enum_value, 'string_value', string_value)
                    ) version_fields_json
                    FROM version_fields_cte
                    GROUP BY mod_id
				),
				loader_fields_cte AS (
					SELECT DISTINCT vf.mod_id, vf.version_id, lf.*, l.loader
					FROM loader_fields lf
                    INNER JOIN version_fields_cte vf ON lf.id = vf.field_id
					LEFT JOIN loaders_versions lv ON vf.version_id = lv.version_id
					LEFT JOIN loaders l ON lv.loader_id = l.id
                    GROUP BY vf.mod_id, vf.version_id, lf.enum_type, lf.id, l.loader
				),
                loader_fields_json AS (
                    SELECT DISTINCT mod_id,
                        JSONB_AGG(
                        DISTINCT jsonb_build_object(
                            'version_id', lf.version_id,
                            'lf_id', id, 'loader_name', loader, 'field', field, 'field_type', field_type, 'enum_type', enum_type, 'min_val', min_val, 'max_val', max_val, 'optional', optional
                        )
                    ) filter (where lf.id is not null) loader_fields_json
                    FROM loader_fields_cte lf
                    GROUP BY mod_id
                ),
                loader_field_enum_values_json AS (
                    SELECT DISTINCT mod_id,
                        JSONB_AGG(
                        DISTINCT jsonb_build_object(
                            'id', lfev.id, 'enum_id', lfev.enum_id, 'value', lfev.value, 'ordering', lfev.ordering, 'created', lfev.created, 'metadata', lfev.metadata
                        ) 
                    ) filter (where lfev.id is not null) loader_field_enum_values_json
                    FROM loader_field_enum_values lfev
                    INNER JOIN loader_fields_cte lf on lf.enum_type = lfev.enum_id
                    GROUP BY mod_id
                ),
                versions_cte AS (
                    SELECT DISTINCT mod_id, v.id as id, date_published
                    FROM mods m
                    INNER JOIN versions v ON m.id = v.mod_id AND v.status = ANY($3)
                    WHERE m.id = ANY($1) OR m.slug = ANY($2)
                ),
                versions_json AS (
                    SELECT DISTINCT mod_id,
                        JSONB_AGG(
                        DISTINCT jsonb_build_object(
                            'id', id, 'date_published', date_published
                        )
                    ) filter (where id is not null) versions_json
                    FROM versions_cte
                    GROUP BY mod_id
                ),
                loaders_cte AS (
                    SELECT DISTINCT mod_id, l.id as id, l.loader
                    FROM versions_cte
                    INNER JOIN loaders_versions lv ON versions_cte.id = lv.version_id
                    INNER JOIN loaders l ON lv.loader_id = l.id 
                ),
                mods_gallery_json AS (
                    SELECT DISTINCT mod_id,
                        JSONB_AGG(
                        DISTINCT jsonb_build_object(
                            'image_url', mg.image_url, 'featured', mg.featured, 'title', mg.title, 'description', mg.description, 'created', mg.created, 'ordering', mg.ordering
                        )
                    ) filter (where image_url is not null) mods_gallery_json
                    FROM mods_gallery mg
                    INNER JOIN mods m ON mg.mod_id = m.id
                    WHERE m.id = ANY($1) OR m.slug = ANY($2)
                    GROUP BY mod_id
                ),
                links_json AS (
                    SELECT DISTINCT joining_mod_id as mod_id,
                        JSONB_AGG(
                        DISTINCT jsonb_build_object(
                            'platform_id', ml.joining_platform_id, 'platform_name', lp.name,'url', ml.url, 'donation', lp.donation
                        )
                    ) filter (where ml.joining_platform_id is not null) links_json
                    FROM mods_links ml
                    INNER JOIN mods m ON ml.joining_mod_id = m.id AND m.id = ANY($1) OR m.slug = ANY($2)
                    INNER JOIN link_platforms lp ON ml.joining_platform_id = lp.id
                    GROUP BY mod_id
                )
                
                SELECT m.id id, m.title title, m.description description, m.downloads downloads, m.follows follows,
                m.icon_url icon_url, m.body body, m.published published,
                m.updated updated, m.approved approved, m.queued, m.status status, m.requested_status requested_status,
                m.license_url license_url,
                m.team_id team_id, m.organization_id organization_id, m.license license, m.slug slug, m.moderation_message moderation_message, m.moderation_message_body moderation_message_body,
                m.webhook_sent, m.color,
                t.id thread_id, m.monetization_status monetization_status,
                ARRAY_AGG(DISTINCT l.loader) filter (where l.loader is not null) loaders,
                ARRAY_AGG(DISTINCT pt.name) filter (where pt.name is not null) project_types,
                ARRAY_AGG(DISTINCT g.slug) filter (where g.slug is not null) games,
                ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null and mc.is_additional is false) categories,
                ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null and mc.is_additional is true) additional_categories,
                v.versions_json versions,
                mg.mods_gallery_json gallery,
                ml.links_json links,
                vf.version_fields_json version_fields,
                lf.loader_fields_json loader_fields,
                lfev.loader_field_enum_values_json loader_field_enum_values
                FROM mods m                
                INNER JOIN threads t ON t.mod_id = m.id
                LEFT JOIN mods_gallery_json mg ON mg.mod_id = m.id
                LEFT JOIN links_json ml ON ml.mod_id = m.id
                LEFT JOIN mods_categories mc ON mc.joining_mod_id = m.id
                LEFT JOIN categories c ON mc.joining_category_id = c.id
                LEFT JOIN versions_json v ON v.mod_id = m.id
                LEFT JOIN loaders_cte l on l.mod_id = m.id
                LEFT JOIN loaders_project_types lpt ON lpt.joining_loader_id = l.id
                LEFT JOIN project_types pt ON pt.id = lpt.joining_project_type_id
                LEFT JOIN loaders_project_types_games lptg ON lptg.loader_id = l.id AND lptg.project_type_id = pt.id
                LEFT JOIN games g ON lptg.game_id = g.id
                LEFT OUTER JOIN version_fields_json vf ON m.id = vf.mod_id
                LEFT OUTER JOIN loader_fields_json lf ON m.id = lf.mod_id
                LEFT OUTER JOIN loader_field_enum_values_json lfev ON m.id = lfev.mod_id
                WHERE m.id = ANY($1) OR m.slug = ANY($2)
                GROUP BY t.id, m.id, version_fields_json, loader_fields_json, loader_field_enum_values_json, versions_json, mods_gallery_json, links_json;
                ",
                &project_ids_parsed,
                &remaining_strings.into_iter().map(|x| x.to_string().to_lowercase()).collect::<Vec<_>>(),
                &*crate::models::projects::VersionStatus::iterator().filter(|x| x.is_listed()).map(|x| x.to_string()).collect::<Vec<String>>()
            )
                .fetch_many(exec)
                .try_filter_map(|e| async {
                    Ok(e.right().map(|m| {
                        let id = m.id;
                    QueryProject {
                        inner: Project {
                            id: ProjectId(id),
                            team_id: TeamId(m.team_id),
                            organization_id: m.organization_id.map(OrganizationId),
                            title: m.title.clone(),
                            description: m.description.clone(),
                            downloads: m.downloads,
                            body_url: None,
                            icon_url: m.icon_url.clone(),
                            published: m.published,
                            updated: m.updated,
                            license_url: m.license_url.clone(),
                            status: ProjectStatus::from_string(
                                &m.status,
                            ),
                            requested_status: m.requested_status.map(|x| ProjectStatus::from_string(
                                &x,
                            )),
                            license: m.license.clone(),
                            slug: m.slug.clone(),
                            body: m.body.clone(),
                            follows: m.follows,
                            moderation_message: m.moderation_message,
                            moderation_message_body: m.moderation_message_body,
                            approved: m.approved,
                            webhook_sent: m.webhook_sent,
                            color: m.color.map(|x| x as u32),
                            queued: m.queued,
                            monetization_status: MonetizationStatus::from_string(
                                &m.monetization_status,
                            ),
                            loaders: m.loaders.unwrap_or_default(),
                        },
                        categories: m.categories.unwrap_or_default(),
                        additional_categories: m.additional_categories.unwrap_or_default(),
                        project_types: m.project_types.unwrap_or_default(),
                        games: m.games.unwrap_or_default(),
                        versions: {
                                #[derive(Deserialize)]
                                struct Version {
                                    pub id: VersionId,
                                    pub date_published: DateTime<Utc>,
                                }

                                let mut versions: Vec<Version> = serde_json::from_value(
                                    m.versions.unwrap_or_default(),
                                )
                                    .ok()
                                    .unwrap_or_default();

                                versions.sort_by(|a, b| a.date_published.cmp(&b.date_published));
                                versions.into_iter().map(|x| x.id).collect()
                            },
                            gallery_items: {
                                let mut gallery: Vec<GalleryItem> = serde_json::from_value(
                                    m.gallery.unwrap_or_default(),
                                ).ok().unwrap_or_default();

                                gallery.sort_by(|a, b| a.ordering.cmp(&b.ordering));

                                gallery
                            },
                            urls: serde_json::from_value(
                                m.links.unwrap_or_default(),
                            ).unwrap_or_default(),
                        aggregate_version_fields: VersionField::from_query_json(m.loader_fields, m.version_fields, m.loader_field_enum_values, true),
                        thread_id: ThreadId(m.thread_id),
                    }}))
                })
                .try_collect::<Vec<QueryProject>>()
                .await?;

            for project in db_projects {
                redis
                    .set_serialized_to_json(PROJECTS_NAMESPACE, project.inner.id.0, &project, None)
                    .await?;
                if let Some(slug) = &project.inner.slug {
                    redis
                        .set(
                            PROJECTS_SLUGS_NAMESPACE,
                            &slug.to_lowercase(),
                            &project.inner.id.0.to_string(),
                            None,
                        )
                        .await?;
                }
                found_projects.push(project);
            }
        }

        Ok(found_projects)
    }

    pub async fn get_dependencies<'a, E>(
        id: ProjectId,
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<(Option<VersionId>, Option<ProjectId>, Option<ProjectId>)>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        type Dependencies = Vec<(Option<VersionId>, Option<ProjectId>, Option<ProjectId>)>;

        let mut redis = redis.connect().await?;

        let dependencies = redis
            .get_deserialized_from_json::<Dependencies>(
                PROJECTS_DEPENDENCIES_NAMESPACE,
                &id.0.to_string(),
            )
            .await?;
        if let Some(dependencies) = dependencies {
            return Ok(dependencies);
        }

        let dependencies: Dependencies = sqlx::query!(
            "
            SELECT d.dependency_id, COALESCE(vd.mod_id, 0) mod_id, d.mod_dependency_id
            FROM versions v
            INNER JOIN dependencies d ON d.dependent_id = v.id
            LEFT JOIN versions vd ON d.dependency_id = vd.id
            WHERE v.mod_id = $1
            ",
            id as ProjectId
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|x| {
                (
                    x.dependency_id.map(VersionId),
                    if x.mod_id == Some(0) {
                        None
                    } else {
                        x.mod_id.map(ProjectId)
                    },
                    x.mod_dependency_id.map(ProjectId),
                )
            }))
        })
        .try_collect::<Dependencies>()
        .await?;

        redis
            .set_serialized_to_json(PROJECTS_DEPENDENCIES_NAMESPACE, id.0, &dependencies, None)
            .await?;
        Ok(dependencies)
    }

    pub async fn clear_cache(
        id: ProjectId,
        slug: Option<String>,
        clear_dependencies: Option<bool>,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;

        redis
            .delete_many([
                (PROJECTS_NAMESPACE, Some(id.0.to_string())),
                (PROJECTS_SLUGS_NAMESPACE, slug.map(|x| x.to_lowercase())),
                (
                    PROJECTS_DEPENDENCIES_NAMESPACE,
                    if clear_dependencies.unwrap_or(false) {
                        Some(id.0.to_string())
                    } else {
                        None
                    },
                ),
            ])
            .await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryProject {
    pub inner: Project,
    pub categories: Vec<String>,
    pub additional_categories: Vec<String>,
    pub versions: Vec<VersionId>,
    pub project_types: Vec<String>,
    pub games: Vec<String>,
    pub urls: Vec<LinkUrl>,
    pub gallery_items: Vec<GalleryItem>,
    pub thread_id: ThreadId,
    pub aggregate_version_fields: Vec<VersionField>,
}
