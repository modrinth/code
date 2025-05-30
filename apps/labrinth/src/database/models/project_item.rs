use super::loader_fields::{
    QueryLoaderField, QueryLoaderFieldEnumValue, QueryVersionField,
    VersionField,
};
use super::{DBUser, ids::*};
use crate::database::models;
use crate::database::models::DatabaseError;
use crate::database::redis::RedisPool;
use crate::models::projects::{MonetizationStatus, ProjectStatus};
use ariadne::ids::base62_impl::parse_base62;
use chrono::{DateTime, Utc};
use dashmap::{DashMap, DashSet};
use futures::TryStreamExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::hash::Hash;

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
        project_id: DBProjectId,
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
pub struct DBGalleryItem {
    pub image_url: String,
    pub raw_image_url: String,
    pub featured: bool,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub ordering: i64,
}

impl DBGalleryItem {
    pub async fn insert_many(
        items: Vec<Self>,
        project_id: DBProjectId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        let (
            project_ids,
            image_urls,
            raw_image_urls,
            featureds,
            names,
            descriptions,
            orderings,
        ): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) = items
            .into_iter()
            .map(|gi| {
                (
                    project_id.0,
                    gi.image_url,
                    gi.raw_image_url,
                    gi.featured,
                    gi.name,
                    gi.description,
                    gi.ordering,
                )
            })
            .multiunzip();
        sqlx::query!(
            "
            INSERT INTO mods_gallery (
                mod_id, image_url, raw_image_url, featured, name, description, ordering
            )
            SELECT * FROM UNNEST ($1::bigint[], $2::varchar[], $3::varchar[], $4::bool[], $5::varchar[], $6::varchar[], $7::bigint[])
            ",
            &project_ids[..],
            &image_urls[..],
            &raw_image_urls[..],
            &featureds[..],
            &names[..] as &[Option<String>],
            &descriptions[..] as &[Option<String>],
            &orderings[..]
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

pub struct DBModCategory {
    pub project_id: DBProjectId,
    pub category_id: CategoryId,
    pub is_additional: bool,
}

impl DBModCategory {
    pub async fn insert_many(
        items: Vec<Self>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let (project_ids, category_ids, is_additionals): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = items
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
    pub project_id: DBProjectId,
    pub team_id: DBTeamId,
    pub organization_id: Option<DBOrganizationId>,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub raw_icon_url: Option<String>,
    pub license_url: Option<String>,
    pub categories: Vec<CategoryId>,
    pub additional_categories: Vec<CategoryId>,
    pub initial_versions: Vec<super::version_item::VersionBuilder>,
    pub status: ProjectStatus,
    pub requested_status: Option<ProjectStatus>,
    pub license: String,
    pub slug: Option<String>,
    pub link_urls: Vec<LinkUrl>,
    pub gallery_items: Vec<DBGalleryItem>,
    pub color: Option<u32>,
    pub monetization_status: MonetizationStatus,
}

impl ProjectBuilder {
    pub async fn insert(
        self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<DBProjectId, DatabaseError> {
        let project_struct = DBProject {
            id: self.project_id,
            team_id: self.team_id,
            organization_id: self.organization_id,
            name: self.name,
            summary: self.summary,
            description: self.description,
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
            raw_icon_url: self.raw_icon_url,
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

        LinkUrl::insert_many_projects(
            link_urls,
            self.project_id,
            &mut *transaction,
        )
        .await?;

        DBGalleryItem::insert_many(
            gallery_items,
            self.project_id,
            &mut *transaction,
        )
        .await?;

        let project_id = self.project_id;
        let mod_categories = categories
            .into_iter()
            .map(|category_id| DBModCategory {
                project_id,
                category_id,
                is_additional: false,
            })
            .chain(additional_categories.into_iter().map(|category_id| {
                DBModCategory {
                    project_id,
                    category_id,
                    is_additional: true,
                }
            }))
            .collect_vec();
        DBModCategory::insert_many(mod_categories, &mut *transaction).await?;

        Ok(self.project_id)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DBProject {
    pub id: DBProjectId,
    pub team_id: DBTeamId,
    pub organization_id: Option<DBOrganizationId>,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub approved: Option<DateTime<Utc>>,
    pub queued: Option<DateTime<Utc>>,
    pub status: ProjectStatus,
    pub requested_status: Option<ProjectStatus>,
    pub downloads: i32,
    pub follows: i32,
    pub icon_url: Option<String>,
    pub raw_icon_url: Option<String>,
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

impl DBProject {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "
            INSERT INTO mods (
                id, team_id, name, summary, description,
                published, downloads, icon_url, raw_icon_url, status, requested_status,
                license_url, license,
                slug, color, monetization_status, organization_id
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11,
                $12, $13,
                LOWER($14), $15, $16, $17
            )
            ",
            self.id as DBProjectId,
            self.team_id as DBTeamId,
            &self.name,
            &self.summary,
            &self.description,
            self.published,
            self.downloads,
            self.icon_url.as_ref(),
            self.raw_icon_url.as_ref(),
            self.status.as_str(),
            self.requested_status.map(|x| x.as_str()),
            self.license_url.as_ref(),
            &self.license,
            self.slug.as_ref(),
            self.color.map(|x| x as i32),
            self.monetization_status.as_str(),
            self.organization_id.map(|x| x.0 as i64),
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn remove(
        id: DBProjectId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        let project = Self::get_id(id, &mut **transaction, redis).await?;

        if let Some(project) = project {
            DBProject::clear_cache(id, project.inner.slug, Some(true), redis)
                .await?;

            sqlx::query!(
                "
                DELETE FROM mod_follows
                WHERE mod_id = $1
                ",
                id as DBProjectId
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM mods_gallery
                WHERE mod_id = $1
                ",
                id as DBProjectId
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM mod_follows
                WHERE mod_id = $1
                ",
                id as DBProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            models::DBThread::remove_full(project.thread_id, transaction)
                .await?;

            sqlx::query!(
                "
                UPDATE reports
                SET mod_id = NULL
                WHERE mod_id = $1
                ",
                id as DBProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM mods_categories
                WHERE joining_mod_id = $1
                ",
                id as DBProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM mods_links
                WHERE joining_mod_id = $1
                ",
                id as DBProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            for version in project.versions {
                super::DBVersion::remove_full(version, redis, transaction)
                    .await?;
            }

            sqlx::query!(
                "
                DELETE FROM dependencies WHERE mod_dependency_id = $1
                ",
                id as DBProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                UPDATE payouts_values
                SET mod_id = NULL
                WHERE (mod_id = $1)
                ",
                id as DBProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            sqlx::query!(
                "
                DELETE FROM mods
                WHERE id = $1
                ",
                id as DBProjectId,
            )
            .execute(&mut **transaction)
            .await?;

            models::DBTeamMember::clear_cache(project.inner.team_id, redis)
                .await?;

            let affected_user_ids = sqlx::query!(
                "
                DELETE FROM team_members
                WHERE team_id = $1
                RETURNING user_id
                ",
                project.inner.team_id as DBTeamId,
            )
            .fetch(&mut **transaction)
            .map_ok(|x| DBUserId(x.user_id))
            .try_collect::<Vec<_>>()
            .await?;

            DBUser::clear_project_cache(&affected_user_ids, redis).await?;

            sqlx::query!(
                "
                DELETE FROM teams
                WHERE id = $1
                ",
                project.inner.team_id as DBTeamId,
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
    ) -> Result<Option<ProjectQueryResult>, DatabaseError>
    where
        E: sqlx::Acquire<'a, Database = sqlx::Postgres>,
    {
        DBProject::get_many(&[string], executor, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_id<'a, 'b, E>(
        id: DBProjectId,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<ProjectQueryResult>, DatabaseError>
    where
        E: sqlx::Acquire<'a, Database = sqlx::Postgres>,
    {
        DBProject::get_many(
            &[crate::models::ids::ProjectId::from(id)],
            executor,
            redis,
        )
        .await
        .map(|x| x.into_iter().next())
    }

    pub async fn get_many_ids<'a, E>(
        project_ids: &[DBProjectId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<ProjectQueryResult>, DatabaseError>
    where
        E: sqlx::Acquire<'a, Database = sqlx::Postgres>,
    {
        let ids = project_ids
            .iter()
            .map(|x| crate::models::ids::ProjectId::from(*x))
            .collect::<Vec<_>>();
        DBProject::get_many(&ids, exec, redis).await
    }

    pub async fn get_many<
        'a,
        E,
        T: Display + Hash + Eq + PartialEq + Clone + Debug,
    >(
        project_strings: &[T],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<ProjectQueryResult>, DatabaseError>
    where
        E: sqlx::Acquire<'a, Database = sqlx::Postgres>,
    {
        let val = redis.get_cached_keys_with_slug(
            PROJECTS_NAMESPACE,
            PROJECTS_SLUGS_NAMESPACE,
            false,
            project_strings,
            |ids| async move {
                let mut exec = exec.acquire().await?;
                let project_ids_parsed: Vec<i64> = ids
                    .iter()
                    .flat_map(|x| parse_base62(&x.to_string()).ok())
                    .map(|x| x as i64)
                    .collect();
                let slugs = ids
                    .into_iter()
                    .map(|x| x.to_string().to_lowercase())
                    .collect::<Vec<_>>();

                let all_version_ids = DashSet::new();
                let versions: DashMap<DBProjectId, Vec<(DBVersionId, DateTime<Utc>)>> = sqlx::query!(
                    "
                    SELECT DISTINCT mod_id, v.id as id, date_published
                    FROM mods m
                    INNER JOIN versions v ON m.id = v.mod_id AND v.status = ANY($3)
                    WHERE m.id = ANY($1) OR m.slug = ANY($2)
                    ",
                    &project_ids_parsed,
                    &slugs,
                    &*crate::models::projects::VersionStatus::iterator()
                        .filter(|x| x.is_listed())
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                )
                    .fetch(&mut *exec)
                    .try_fold(
                        DashMap::new(),
                        |acc: DashMap<DBProjectId, Vec<(DBVersionId, DateTime<Utc>)>>, m| {
                            let version_id = DBVersionId(m.id);
                            let date_published = m.date_published;
                            all_version_ids.insert(version_id);
                            acc.entry(DBProjectId(m.mod_id))
                                .or_default()
                                .push((version_id, date_published));
                            async move { Ok(acc) }
                        },
                    )
                    .await?;

                let loader_field_enum_value_ids = DashSet::new();
                let version_fields: DashMap<DBProjectId, Vec<QueryVersionField>> = sqlx::query!(
                    "
                    SELECT DISTINCT mod_id, version_id, field_id, int_value, enum_value, string_value
                    FROM versions v
                    INNER JOIN version_fields vf ON v.id = vf.version_id
                    WHERE v.id = ANY($1)
                    ",
                    &all_version_ids.iter().map(|x| x.0).collect::<Vec<_>>()
                )
                    .fetch(&mut *exec)
                    .try_fold(
                        DashMap::new(),
                        |acc: DashMap<DBProjectId, Vec<QueryVersionField>>, m| {
                            let qvf = QueryVersionField {
                                version_id: DBVersionId(m.version_id),
                                field_id: LoaderFieldId(m.field_id),
                                int_value: m.int_value,
                                enum_value: if m.enum_value == -1  { None } else { Some(LoaderFieldEnumValueId(m.enum_value)) },
                                string_value: m.string_value,
                            };

                            if m.enum_value != -1 {
                                loader_field_enum_value_ids.insert(LoaderFieldEnumValueId(m.enum_value));
                            }

                            acc.entry(DBProjectId(m.mod_id)).or_default().push(qvf);
                            async move { Ok(acc) }
                        },
                    )
                    .await?;

                let loader_field_enum_values: Vec<QueryLoaderFieldEnumValue> = sqlx::query!(
                    "
                    SELECT DISTINCT id, enum_id, value, ordering, created, metadata
                    FROM loader_field_enum_values lfev
                    WHERE id = ANY($1)
                    ORDER BY enum_id, ordering, created DESC
                    ",
                    &loader_field_enum_value_ids
                        .iter()
                        .map(|x| x.0)
                        .collect::<Vec<_>>()
                )
                    .fetch(&mut *exec)
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

                let mods_gallery: DashMap<DBProjectId, Vec<DBGalleryItem>> = sqlx::query!(
                    "
                    SELECT DISTINCT mod_id, mg.image_url, mg.raw_image_url, mg.featured, mg.name, mg.description, mg.created, mg.ordering
                    FROM mods_gallery mg
                    INNER JOIN mods m ON mg.mod_id = m.id
                    WHERE m.id = ANY($1) OR m.slug = ANY($2)
                    ",
                    &project_ids_parsed,
                    &slugs
                ).fetch(&mut *exec)
                    .try_fold(DashMap::new(), |acc : DashMap<DBProjectId, Vec<DBGalleryItem>>, m| {
                        acc.entry(DBProjectId(m.mod_id))
                            .or_default()
                            .push(DBGalleryItem {
                                image_url: m.image_url,
                                raw_image_url: m.raw_image_url,
                                featured: m.featured.unwrap_or(false),
                                name: m.name,
                                description: m.description,
                                created: m.created,
                                ordering: m.ordering,
                            });
                        async move { Ok(acc) }
                    }
                    ).await?;

                let links: DashMap<DBProjectId, Vec<LinkUrl>> = sqlx::query!(
                    "
                    SELECT DISTINCT joining_mod_id as mod_id, joining_platform_id as platform_id, lp.name as platform_name, url, lp.donation as donation
                    FROM mods_links ml
                    INNER JOIN mods m ON ml.joining_mod_id = m.id
                    INNER JOIN link_platforms lp ON ml.joining_platform_id = lp.id
                    WHERE m.id = ANY($1) OR m.slug = ANY($2)
                    ",
                    &project_ids_parsed,
                    &slugs
                ).fetch(&mut *exec)
                    .try_fold(DashMap::new(), |acc : DashMap<DBProjectId, Vec<LinkUrl>>, m| {
                        acc.entry(DBProjectId(m.mod_id))
                            .or_default()
                            .push(LinkUrl {
                                platform_id: LinkPlatformId(m.platform_id),
                                platform_name: m.platform_name,
                                url: m.url,
                                donation: m.donation,
                            });
                        async move { Ok(acc) }
                    }
                    ).await?;

                #[derive(Default)]
                struct VersionLoaderData {
                    loaders: Vec<String>,
                    project_types: Vec<String>,
                    games: Vec<String>,
                    loader_loader_field_ids: Vec<LoaderFieldId>,
                }

                let loader_field_ids = DashSet::new();
                let loaders_ptypes_games: DashMap<DBProjectId, VersionLoaderData> = sqlx::query!(
                    "
                    SELECT DISTINCT mod_id,
                        ARRAY_AGG(DISTINCT l.loader) filter (where l.loader is not null) loaders,
                        ARRAY_AGG(DISTINCT pt.name) filter (where pt.name is not null) project_types,
                        ARRAY_AGG(DISTINCT g.slug) filter (where g.slug is not null) games,
                        ARRAY_AGG(DISTINCT lfl.loader_field_id) filter (where lfl.loader_field_id is not null) loader_fields
                    FROM versions v
                    INNER JOIN loaders_versions lv ON v.id = lv.version_id
                    INNER JOIN loaders l ON lv.loader_id = l.id
                    INNER JOIN loaders_project_types lpt ON lpt.joining_loader_id = l.id
                    INNER JOIN project_types pt ON pt.id = lpt.joining_project_type_id
                    INNER JOIN loaders_project_types_games lptg ON lptg.loader_id = l.id AND lptg.project_type_id = pt.id
                    INNER JOIN games g ON lptg.game_id = g.id
                    LEFT JOIN loader_fields_loaders lfl ON lfl.loader_id = l.id
                    WHERE v.id = ANY($1)
                    GROUP BY mod_id
                    ",
                    &all_version_ids.iter().map(|x| x.0).collect::<Vec<_>>()
                ).fetch(&mut *exec)
                    .map_ok(|m| {
                        let project_id = DBProjectId(m.mod_id);

                        // Add loader fields to the set we need to fetch
                        let loader_loader_field_ids = m.loader_fields.unwrap_or_default().into_iter().map(LoaderFieldId).collect::<Vec<_>>();
                        for loader_field_id in loader_loader_field_ids.iter() {
                            loader_field_ids.insert(*loader_field_id);
                        }

                        // Add loader + loader associated data to the map
                        let version_loader_data = VersionLoaderData {
                            loaders: m.loaders.unwrap_or_default(),
                            project_types: m.project_types.unwrap_or_default(),
                            games: m.games.unwrap_or_default(),
                            loader_loader_field_ids,
                        };

                        (project_id, version_loader_data)

                    }
                    ).try_collect().await?;

                let loader_fields: Vec<QueryLoaderField> = sqlx::query!(
                    "
                    SELECT DISTINCT id, field, field_type, enum_type, min_val, max_val, optional
                    FROM loader_fields lf
                    WHERE id = ANY($1)
                    ",
                    &loader_field_ids.iter().map(|x| x.0).collect::<Vec<_>>()
                )
                    .fetch(&mut *exec)
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

                let projects = sqlx::query!(
                    "
                    SELECT m.id id, m.name name, m.summary summary, m.downloads downloads, m.follows follows,
                    m.icon_url icon_url, m.raw_icon_url raw_icon_url, m.description description, m.published published,
                    m.approved approved, m.queued, m.status status, m.requested_status requested_status,
                    m.license_url license_url,
                    m.team_id team_id, m.organization_id organization_id, m.license license, m.slug slug, m.moderation_message moderation_message, m.moderation_message_body moderation_message_body,
                    m.webhook_sent, m.color,
                    t.id thread_id, m.monetization_status monetization_status,
                    ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null and mc.is_additional is false) categories,
                    ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null and mc.is_additional is true) additional_categories
                    FROM mods m
                    INNER JOIN threads t ON t.mod_id = m.id
                    LEFT JOIN mods_categories mc ON mc.joining_mod_id = m.id
                    LEFT JOIN categories c ON mc.joining_category_id = c.id
                    WHERE m.id = ANY($1) OR m.slug = ANY($2)
                    GROUP BY t.id, m.id;
                    ",
                    &project_ids_parsed,
                    &slugs,
                )
                    .fetch(&mut *exec)
                    .try_fold(DashMap::new(), |acc, m| {
                        let id = m.id;
                        let project_id = DBProjectId(id);
                        let VersionLoaderData {
                            loaders,
                            project_types,
                            games,
                            loader_loader_field_ids,
                        } = loaders_ptypes_games.remove(&project_id).map(|x|x.1).unwrap_or_default();
                        // Each version is a tuple of (DBVersionId, DateTime<Utc>)
                        let mut versions = versions.remove(&project_id).map(|x| x.1).unwrap_or_default();
                        versions.sort_by(|a, b| a.1.cmp(&b.1));
                        let mut gallery = mods_gallery.remove(&project_id).map(|x| x.1).unwrap_or_default();
                        let urls = links.remove(&project_id).map(|x| x.1).unwrap_or_default();
                        let version_fields = version_fields.remove(&project_id).map(|x| x.1).unwrap_or_default();

                        let loader_fields = loader_fields.iter()
                            .filter(|x| loader_loader_field_ids.contains(&x.id))
                            .collect::<Vec<_>>();

                        let project = ProjectQueryResult {
                            inner: DBProject {
                                id: DBProjectId(id),
                                team_id: DBTeamId(m.team_id),
                                organization_id: m.organization_id.map(DBOrganizationId),
                                name: m.name.clone(),
                                summary: m.summary.clone(),
                                downloads: m.downloads,
                                icon_url: m.icon_url.clone(),
                                raw_icon_url: m.raw_icon_url.clone(),
                                published: m.published,
                                updated: versions.iter().map(|x| x.1).next_back().unwrap_or(m.published),
                                license_url: m.license_url.clone(),
                                status: ProjectStatus::from_string(
                                    &m.status,
                                ),
                                requested_status: m.requested_status.map(|x| ProjectStatus::from_string(
                                    &x,
                                )),
                                license: m.license.clone(),
                                slug: m.slug.clone(),
                                description: m.description.clone(),
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
                                loaders,
                            },
                            categories: m.categories.unwrap_or_default(),
                            additional_categories: m.additional_categories.unwrap_or_default(),
                            project_types,
                            games,
                            versions: versions.into_iter().map(|x| x.0).collect(),
                            gallery_items: {
                                gallery.sort_by(|a, b| a.ordering.cmp(&b.ordering));
                                gallery
                            },
                            urls,
                            aggregate_version_fields: VersionField::from_query_json(version_fields, &loader_fields, &loader_field_enum_values, true),
                            thread_id: DBThreadId(m.thread_id),
                        };

                        acc.insert(m.id, (m.slug, project));
                        async move { Ok(acc) }
                    })
                    .await?;

                Ok(projects)
            },
        ).await?;

        Ok(val)
    }

    pub async fn get_dependencies<'a, E>(
        id: DBProjectId,
        exec: E,
        redis: &RedisPool,
    ) -> Result<
        Vec<(
            Option<DBVersionId>,
            Option<DBProjectId>,
            Option<DBProjectId>,
        )>,
        DatabaseError,
    >
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        type Dependencies = Vec<(
            Option<DBVersionId>,
            Option<DBProjectId>,
            Option<DBProjectId>,
        )>;

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
            id as DBProjectId
        )
        .fetch(exec)
        .map_ok(|x| {
            (
                x.dependency_id.map(DBVersionId),
                if x.mod_id == Some(0) {
                    None
                } else {
                    x.mod_id.map(DBProjectId)
                },
                x.mod_dependency_id.map(DBProjectId),
            )
        })
        .try_collect::<Dependencies>()
        .await?;

        redis
            .set_serialized_to_json(
                PROJECTS_DEPENDENCIES_NAMESPACE,
                id.0,
                &dependencies,
                None,
            )
            .await?;
        Ok(dependencies)
    }

    pub async fn clear_cache(
        id: DBProjectId,
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
pub struct ProjectQueryResult {
    pub inner: DBProject,
    pub categories: Vec<String>,
    pub additional_categories: Vec<String>,
    pub versions: Vec<DBVersionId>,
    pub project_types: Vec<String>,
    pub games: Vec<String>,
    pub urls: Vec<LinkUrl>,
    pub gallery_items: Vec<DBGalleryItem>,
    pub thread_id: DBThreadId,
    pub aggregate_version_fields: Vec<VersionField>,
}
