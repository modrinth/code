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
pub struct DonationUrl {
    pub platform_id: DonationPlatformId,
    pub platform_short: String,
    pub platform_name: String,
    pub url: String,
}

impl DonationUrl {
    pub async fn insert_many_projects(
        donation_urls: Vec<Self>,
        project_id: ProjectId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        let (project_ids, platform_ids, urls): (Vec<_>, Vec<_>, Vec<_>) = donation_urls
            .into_iter()
            .map(|url| (project_id.0, url.platform_id.0, url.url))
            .multiunzip();
        sqlx::query!(
            "
            INSERT INTO mods_donations (
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
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub license_url: Option<String>,
    pub discord_url: Option<String>,
    pub categories: Vec<CategoryId>,
    pub additional_categories: Vec<CategoryId>,
    pub initial_versions: Vec<super::version_item::VersionBuilder>,
    pub status: ProjectStatus,
    pub requested_status: Option<ProjectStatus>,
    pub license: String,
    pub slug: Option<String>,
    pub donation_urls: Vec<DonationUrl>,
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
            issues_url: self.issues_url,
            source_url: self.source_url,
            wiki_url: self.wiki_url,
            license_url: self.license_url,
            discord_url: self.discord_url,
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
            donation_urls,
            gallery_items,
            categories,
            additional_categories,
            ..
        } = self;

        for mut version in self.initial_versions {
            version.project_id = self.project_id;
            version.insert(&mut *transaction).await?;
        }

        DonationUrl::insert_many_projects(donation_urls, self.project_id, &mut *transaction)
            .await?;

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
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub license_url: Option<String>,
    pub discord_url: Option<String>,
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
                published, downloads, icon_url, issues_url,
                source_url, wiki_url, status, requested_status, discord_url,
                license_url, license,
                slug, color, monetization_status
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12, $13, $14,
                $15, $16, 
                LOWER($17), $18, $19
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
            self.issues_url.as_ref(),
            self.source_url.as_ref(),
            self.wiki_url.as_ref(),
            self.status.as_str(),
            self.requested_status.map(|x| x.as_str()),
            self.discord_url.as_ref(),
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
                DELETE FROM mods_donations
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
                .multi_get::<i64, _>(
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
                .multi_get::<String, _>(PROJECTS_NAMESPACE, project_ids)
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

            let db_projects: Vec<QueryProject> = sqlx::query!(
                "
                SELECT m.id id, m.title title, m.description description, m.downloads downloads, m.follows follows,
                m.icon_url icon_url, m.body body, m.published published,
                m.updated updated, m.approved approved, m.queued, m.status status, m.requested_status requested_status,
                m.issues_url issues_url, m.source_url source_url, m.wiki_url wiki_url, m.discord_url discord_url, m.license_url license_url,
                m.team_id team_id, m.organization_id organization_id, m.license license, m.slug slug, m.moderation_message moderation_message, m.moderation_message_body moderation_message_body,
                m.webhook_sent, m.color,
                t.id thread_id, m.monetization_status monetization_status,
                ARRAY_AGG(DISTINCT l.loader) filter (where l.loader is not null) loaders,
                ARRAY_AGG(DISTINCT pt.name) filter (where pt.name is not null) project_types,
                ARRAY_AGG(DISTINCT g.name) filter (where g.name is not null) games,
                ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null and mc.is_additional is false) categories,
                ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null and mc.is_additional is true) additional_categories,
                JSONB_AGG(DISTINCT jsonb_build_object('id', v.id, 'date_published', v.date_published)) filter (where v.id is not null) versions,
                JSONB_AGG(DISTINCT jsonb_build_object('image_url', mg.image_url, 'featured', mg.featured, 'title', mg.title, 'description', mg.description, 'created', mg.created, 'ordering', mg.ordering)) filter (where mg.image_url is not null) gallery,
                JSONB_AGG(DISTINCT jsonb_build_object('platform_id', md.joining_platform_id, 'platform_short', dp.short, 'platform_name', dp.name,'url', md.url)) filter (where md.joining_platform_id is not null) donations
                FROM mods m                
                INNER JOIN threads t ON t.mod_id = m.id
                LEFT JOIN mods_gallery mg ON mg.mod_id = m.id
                LEFT JOIN mods_donations md ON md.joining_mod_id = m.id
                LEFT JOIN donation_platforms dp ON md.joining_platform_id = dp.id
                LEFT JOIN mods_categories mc ON mc.joining_mod_id = m.id
                LEFT JOIN categories c ON mc.joining_category_id = c.id
                LEFT JOIN versions v ON v.mod_id = m.id AND v.status = ANY($3)
                LEFT JOIN loaders_versions lv ON lv.version_id = v.id
                LEFT JOIN loaders l on lv.loader_id = l.id
                LEFT JOIN loaders_project_types lpt ON lpt.joining_loader_id = l.id
                LEFT JOIN project_types pt ON pt.id = lpt.joining_project_type_id
                LEFT JOIN loaders_project_types_games lptg ON lptg.loader_id = l.id AND lptg.project_type_id = pt.id
                LEFT JOIN games g ON lptg.game_id = g.id
                WHERE m.id = ANY($1) OR m.slug = ANY($2)
                GROUP BY t.id, m.id;
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
                            issues_url: m.issues_url.clone(),
                            source_url: m.source_url.clone(),
                            wiki_url: m.wiki_url.clone(),
                            license_url: m.license_url.clone(),
                            discord_url: m.discord_url.clone(),
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
                            donation_urls: serde_json::from_value(
                                m.donations.unwrap_or_default(),
                            ).ok().unwrap_or_default(),
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
                            slug.to_lowercase(),
                            project.inner.id.0,
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

        let dependencies = redis
            .get_deserialized_from_json::<Dependencies, _>(PROJECTS_DEPENDENCIES_NAMESPACE, id.0)
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
    pub donation_urls: Vec<DonationUrl>,
    pub gallery_items: Vec<GalleryItem>,
    pub thread_id: ThreadId,
}
