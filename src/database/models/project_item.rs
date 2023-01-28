use super::ids::*;
use crate::models::projects::ProjectStatus;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct DonationUrl {
    pub platform_id: DonationPlatformId,
    pub platform_short: String,
    pub platform_name: String,
    pub url: String,
}

impl DonationUrl {
    pub async fn insert(
        &self,
        project_id: ProjectId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO mods_donations (
                joining_mod_id, joining_platform_id, url
            )
            VALUES (
                $1, $2, $3
            )
            ",
            project_id as ProjectId,
            self.platform_id as DonationPlatformId,
            self.url,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GalleryItem {
    pub image_url: String,
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub ordering: i64,
}

impl GalleryItem {
    pub async fn insert(
        &self,
        project_id: ProjectId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO mods_gallery (
                mod_id, image_url, featured, title, description, ordering
            )
            VALUES (
                $1, $2, $3, $4, $5, $6
            )
            ",
            project_id as ProjectId,
            self.image_url,
            self.featured,
            self.title,
            self.description,
            self.ordering
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }
}

pub struct ProjectBuilder {
    pub project_id: ProjectId,
    pub project_type_id: ProjectTypeId,
    pub team_id: TeamId,
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
    pub client_side: SideTypeId,
    pub server_side: SideTypeId,
    pub license: String,
    pub slug: Option<String>,
    pub donation_urls: Vec<DonationUrl>,
    pub gallery_items: Vec<GalleryItem>,
    pub color: Option<u32>,
}

impl ProjectBuilder {
    pub async fn insert(
        self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<ProjectId, super::DatabaseError> {
        let project_struct = Project {
            id: self.project_id,
            project_type: self.project_type_id,
            team_id: self.team_id,
            title: self.title,
            description: self.description,
            body: self.body,
            body_url: None,
            published: Utc::now(),
            updated: Utc::now(),
            approved: None,
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
            client_side: self.client_side,
            server_side: self.server_side,
            license: self.license,
            slug: self.slug,
            moderation_message: None,
            moderation_message_body: None,
            flame_anvil_project: None,
            flame_anvil_user: None,
            webhook_sent: false,
            color: self.color,
            loaders: vec![],
            game_versions: vec![],
        };
        project_struct.insert(&mut *transaction).await?;

        for mut version in self.initial_versions {
            version.project_id = self.project_id;
            version.insert(&mut *transaction).await?;
        }

        for donation in self.donation_urls {
            donation.insert(self.project_id, &mut *transaction).await?;
        }

        for gallery in self.gallery_items {
            gallery.insert(self.project_id, &mut *transaction).await?;
        }

        for category in self.categories {
            sqlx::query!(
                "
                INSERT INTO mods_categories (joining_mod_id, joining_category_id, is_additional)
                VALUES ($1, $2, FALSE)
                ",
                self.project_id as ProjectId,
                category as CategoryId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        for category in self.additional_categories {
            sqlx::query!(
                "
                INSERT INTO mods_categories (joining_mod_id, joining_category_id, is_additional)
                VALUES ($1, $2, TRUE)
                ",
                self.project_id as ProjectId,
                category as CategoryId,
            )
                .execute(&mut *transaction)
                .await?;
        }

        Project::update_game_versions(self.project_id, &mut *transaction)
            .await?;
        Project::update_loaders(self.project_id, &mut *transaction).await?;

        Ok(self.project_id)
    }
}
#[derive(Clone, Debug)]
pub struct Project {
    pub id: ProjectId,
    pub project_type: ProjectTypeId,
    pub team_id: TeamId,
    pub title: String,
    pub description: String,
    pub body: String,
    pub body_url: Option<String>,
    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub approved: Option<DateTime<Utc>>,
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
    pub client_side: SideTypeId,
    pub server_side: SideTypeId,
    pub license: String,
    pub slug: Option<String>,
    pub moderation_message: Option<String>,
    pub moderation_message_body: Option<String>,
    pub flame_anvil_project: Option<i32>,
    pub flame_anvil_user: Option<UserId>,
    pub webhook_sent: bool,
    pub color: Option<u32>,
    pub loaders: Vec<String>,
    pub game_versions: Vec<String>,
}

impl Project {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO mods (
                id, team_id, title, description, body,
                published, downloads, icon_url, issues_url,
                source_url, wiki_url, status, requested_status, discord_url,
                client_side, server_side, license_url, license,
                slug, project_type, color
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12, $13, $14,
                $15, $16, $17, $18,
                LOWER($19), $20, $21
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
            self.client_side as SideTypeId,
            self.server_side as SideTypeId,
            self.license_url.as_ref(),
            &self.license,
            self.slug.as_ref(),
            self.project_type as ProjectTypeId,
            self.color.map(|x| x as i32)
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn get<'a, 'b, E>(
        id: ProjectId,
        executor: E,
    ) -> Result<Option<Self>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT project_type, title, description, downloads, follows,
                   icon_url, body, published,
                   updated, approved, status, requested_status,
                   issues_url, source_url, wiki_url, discord_url, license_url,
                   team_id, client_side, server_side, license, slug,
                   moderation_message, moderation_message_body, flame_anvil_project,
                   flame_anvil_user, webhook_sent, color, loaders, game_versions
            FROM mods
            WHERE id = $1
            ",
            id as ProjectId,
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(Project {
                id,
                project_type: ProjectTypeId(row.project_type),
                team_id: TeamId(row.team_id),
                title: row.title,
                description: row.description,
                downloads: row.downloads,
                body_url: None,
                icon_url: row.icon_url,
                published: row.published,
                updated: row.updated,
                issues_url: row.issues_url,
                source_url: row.source_url,
                wiki_url: row.wiki_url,
                license_url: row.license_url,
                discord_url: row.discord_url,
                client_side: SideTypeId(row.client_side),
                status: ProjectStatus::from_str(&row.status),
                requested_status: row
                    .requested_status
                    .map(|x| ProjectStatus::from_str(&x)),
                server_side: SideTypeId(row.server_side),
                license: row.license,
                slug: row.slug,
                body: row.body,
                follows: row.follows,
                moderation_message: row.moderation_message,
                moderation_message_body: row.moderation_message_body,
                approved: row.approved,
                flame_anvil_project: row.flame_anvil_project,
                flame_anvil_user: row.flame_anvil_user.map(UserId),
                webhook_sent: row.webhook_sent,
                color: row.color.map(|x| x as u32),
                loaders: row.loaders,
                game_versions: row.game_versions,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many<'a, E>(
        project_ids: Vec<ProjectId>,
        exec: E,
    ) -> Result<Vec<Project>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let project_ids_parsed: Vec<i64> =
            project_ids.into_iter().map(|x| x.0).collect();
        let projects = sqlx::query!(
            "
            SELECT id, project_type, title, description, downloads, follows,
                   icon_url, body, published,
                   updated, approved, status, requested_status,
                   issues_url, source_url, wiki_url, discord_url, license_url,
                   team_id, client_side, server_side, license, slug,
                   moderation_message, moderation_message_body, flame_anvil_project,
                   flame_anvil_user, webhook_sent, color, loaders, game_versions
            FROM mods
            WHERE id = ANY($1)
            ",
            &project_ids_parsed
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|m| Project {
                id: ProjectId(m.id),
                project_type: ProjectTypeId(m.project_type),
                team_id: TeamId(m.team_id),
                title: m.title,
                description: m.description,
                downloads: m.downloads,
                body_url: None,
                icon_url: m.icon_url,
                published: m.published,
                updated: m.updated,
                issues_url: m.issues_url,
                source_url: m.source_url,
                wiki_url: m.wiki_url,
                license_url: m.license_url,
                discord_url: m.discord_url,
                client_side: SideTypeId(m.client_side),
                status: ProjectStatus::from_str(
                    &m.status,
                ),
                requested_status: m.requested_status.map(|x| ProjectStatus::from_str(
                    &x,
                )),
                server_side: SideTypeId(m.server_side),
                license: m.license,
                slug: m.slug,
                body: m.body,
                follows: m.follows,
                moderation_message: m.moderation_message,
                moderation_message_body: m.moderation_message_body,
                approved: m.approved,
                flame_anvil_project: m.flame_anvil_project,
                flame_anvil_user: m.flame_anvil_user.map(UserId),
                webhook_sent: m.webhook_sent,
                color: m.color.map(|x| x as u32),
                loaders: m.loaders,
                game_versions: m.game_versions,
            }))
        })
        .try_collect::<Vec<Project>>()
        .await?;

        Ok(projects)
    }

    pub async fn remove_full(
        id: ProjectId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, sqlx::error::Error> {
        let result = sqlx::query!(
            "
            SELECT team_id FROM mods WHERE id = $1
            ",
            id as ProjectId,
        )
        .fetch_optional(&mut *transaction)
        .await?;

        let team_id: TeamId = if let Some(id) = result {
            TeamId(id.team_id)
        } else {
            return Ok(None);
        };

        sqlx::query!(
            "
            DELETE FROM mod_follows
            WHERE mod_id = $1
            ",
            id as ProjectId
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM mods_gallery
            WHERE mod_id = $1
            ",
            id as ProjectId
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM mod_follows
            WHERE mod_id = $1
            ",
            id as ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM reports
            WHERE mod_id = $1
            ",
            id as ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM mods_categories
            WHERE joining_mod_id = $1
            ",
            id as ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM mods_donations
            WHERE joining_mod_id = $1
            ",
            id as ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        use futures::TryStreamExt;
        let versions: Vec<VersionId> = sqlx::query!(
            "
            SELECT id FROM versions
            WHERE mod_id = $1
            ",
            id as ProjectId,
        )
        .fetch_many(&mut *transaction)
        .try_filter_map(|e| async { Ok(e.right().map(|c| VersionId(c.id))) })
        .try_collect::<Vec<VersionId>>()
        .await?;

        for version in versions {
            super::Version::remove_full(version, transaction).await?;
        }

        sqlx::query!(
            "
            DELETE FROM dependencies WHERE mod_dependency_id = $1
            ",
            id as ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            UPDATE payouts_values
            SET mod_id = NULL
            WHERE (mod_id = $1)
            ",
            id as ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM mods
            WHERE id = $1
            ",
            id as ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM team_members
            WHERE team_id = $1
            ",
            team_id as TeamId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM teams
            WHERE id = $1
            ",
            team_id as TeamId,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(Some(()))
    }

    pub async fn get_full_from_slug<'a, 'b, E>(
        slug: &str,
        executor: E,
    ) -> Result<Option<QueryProject>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let id = sqlx::query!(
            "
            SELECT id FROM mods
            WHERE slug = LOWER($1)
            ",
            slug
        )
        .fetch_optional(executor)
        .await?;

        if let Some(project_id) = id {
            Project::get_full(ProjectId(project_id.id), executor).await
        } else {
            Ok(None)
        }
    }

    pub async fn get_from_slug<'a, 'b, E>(
        slug: &str,
        executor: E,
    ) -> Result<Option<Project>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let id = sqlx::query!(
            "
            SELECT id FROM mods
            WHERE slug = LOWER($1)
            ",
            slug
        )
        .fetch_optional(executor)
        .await?;

        if let Some(project_id) = id {
            Project::get(ProjectId(project_id.id), executor).await
        } else {
            Ok(None)
        }
    }

    pub async fn get_from_slug_or_project_id<'a, 'b, E>(
        slug_or_project_id: &str,
        executor: E,
    ) -> Result<Option<Project>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let id_option =
            crate::models::ids::base62_impl::parse_base62(slug_or_project_id)
                .ok();

        if let Some(id) = id_option {
            let mut project =
                Project::get(ProjectId(id as i64), executor).await?;

            if project.is_none() {
                project = Project::get_from_slug(slug_or_project_id, executor)
                    .await?;
            }

            Ok(project)
        } else {
            let project =
                Project::get_from_slug(slug_or_project_id, executor).await?;

            Ok(project)
        }
    }

    pub async fn get_full_from_slug_or_project_id<'a, 'b, E>(
        slug_or_project_id: &str,
        executor: E,
    ) -> Result<Option<QueryProject>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let id_option =
            crate::models::ids::base62_impl::parse_base62(slug_or_project_id)
                .ok();

        if let Some(id) = id_option {
            let mut project =
                Project::get_full(ProjectId(id as i64), executor).await?;

            if project.is_none() {
                project =
                    Project::get_full_from_slug(slug_or_project_id, executor)
                        .await?;
            }

            Ok(project)
        } else {
            let project =
                Project::get_full_from_slug(slug_or_project_id, executor)
                    .await?;
            Ok(project)
        }
    }

    pub async fn get_full<'a, 'b, E>(
        id: ProjectId,
        executor: E,
    ) -> Result<Option<QueryProject>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT m.id id, m.project_type project_type, m.title title, m.description description, m.downloads downloads, m.follows follows,
            m.icon_url icon_url, m.body body, m.published published,
            m.updated updated, m.approved approved, m.status status, m.requested_status requested_status,
            m.issues_url issues_url, m.source_url source_url, m.wiki_url wiki_url, m.discord_url discord_url, m.license_url license_url,
            m.team_id team_id, m.client_side client_side, m.server_side server_side, m.license license, m.slug slug, m.moderation_message moderation_message, m.moderation_message_body moderation_message_body,
            cs.name client_side_type, ss.name server_side_type, pt.name project_type_name, m.flame_anvil_project flame_anvil_project, m.flame_anvil_user flame_anvil_user, m.webhook_sent webhook_sent, m.color,
            m.loaders loaders, m.game_versions game_versions,
            ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null and mc.is_additional is false) categories,
            ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null and mc.is_additional is true) additional_categories,
            JSONB_AGG(DISTINCT jsonb_build_object('id', v.id, 'date_published', v.date_published)) filter (where v.id is not null) versions,
            JSONB_AGG(DISTINCT jsonb_build_object('image_url', mg.image_url, 'featured', mg.featured, 'title', mg.title, 'description', mg.description, 'created', mg.created, 'ordering', mg.ordering)) filter (where mg.image_url is not null) gallery,
            JSONB_AGG(DISTINCT jsonb_build_object('platform_id', md.joining_platform_id, 'platform_short', dp.short, 'platform_name', dp.name,'url', md.url)) filter (where md.joining_platform_id is not null) donations
            FROM mods m
            INNER JOIN project_types pt ON pt.id = m.project_type
            INNER JOIN side_types cs ON m.client_side = cs.id
            INNER JOIN side_types ss ON m.server_side = ss.id
            LEFT JOIN mods_donations md ON md.joining_mod_id = m.id
            LEFT JOIN donation_platforms dp ON md.joining_platform_id = dp.id
            LEFT JOIN mods_categories mc ON mc.joining_mod_id = m.id
            LEFT JOIN categories c ON mc.joining_category_id = c.id
            LEFT JOIN versions v ON v.mod_id = m.id AND v.status = ANY($2)
            LEFT JOIN mods_gallery mg ON mg.mod_id = m.id
            WHERE m.id = $1
            GROUP BY pt.id, cs.id, ss.id, m.id;
            ",
            id as ProjectId,
            &*crate::models::projects::VersionStatus::iterator().filter(|x| x.is_listed()).map(|x| x.to_string()).collect::<Vec<String>>()
        )
            .fetch_optional(executor)
            .await?;

        if let Some(m) = result {
            Ok(Some(QueryProject {
                inner: Project {
                    id: ProjectId(m.id),
                    project_type: ProjectTypeId(m.project_type),
                    team_id: TeamId(m.team_id),
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
                    client_side: SideTypeId(m.client_side),
                    status: ProjectStatus::from_str(&m.status),
                    requested_status: m
                        .requested_status
                        .map(|x| ProjectStatus::from_str(&x)),
                    server_side: SideTypeId(m.server_side),
                    license: m.license.clone(),
                    slug: m.slug.clone(),
                    body: m.body.clone(),
                    follows: m.follows,
                    moderation_message: m.moderation_message,
                    moderation_message_body: m.moderation_message_body,
                    approved: m.approved,
                    flame_anvil_project: m.flame_anvil_project,
                    flame_anvil_user: m.flame_anvil_user.map(UserId),
                    webhook_sent: m.webhook_sent,
                    color: m.color.map(|x| x as u32),
                    loaders: m.loaders,
                    game_versions: m.game_versions,
                },
                project_type: m.project_type_name,
                categories: m.categories.unwrap_or_default(),
                additional_categories: m
                    .additional_categories
                    .unwrap_or_default(),
                versions: {
                    #[derive(Deserialize)]
                    struct Version {
                        pub id: VersionId,
                        pub date_published: DateTime<Utc>,
                    }

                    let mut versions: Vec<Version> =
                        serde_json::from_value(m.versions.unwrap_or_default())
                            .ok()
                            .unwrap_or_default();

                    versions.sort_by(|a, b| {
                        a.date_published.cmp(&b.date_published)
                    });

                    versions.into_iter().map(|x| x.id).collect()
                },
                gallery_items: {
                    let mut gallery: Vec<GalleryItem> =
                        serde_json::from_value(m.gallery.unwrap_or_default())
                            .ok()
                            .unwrap_or_default();

                    gallery.sort_by(|a, b| a.ordering.cmp(&b.ordering));

                    gallery
                },
                donation_urls: serde_json::from_value(
                    m.donations.unwrap_or_default(),
                )
                .ok()
                .unwrap_or_default(),
                client_side: crate::models::projects::SideType::from_str(
                    &m.client_side_type,
                ),
                server_side: crate::models::projects::SideType::from_str(
                    &m.server_side_type,
                ),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many_full<'a, E>(
        project_ids: &[ProjectId],
        exec: E,
    ) -> Result<Vec<QueryProject>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::TryStreamExt;

        let project_ids_parsed: Vec<i64> =
            project_ids.iter().map(|x| x.0).collect();
        sqlx::query!(
            "
            SELECT m.id id, m.project_type project_type, m.title title, m.description description, m.downloads downloads, m.follows follows,
            m.icon_url icon_url, m.body body, m.published published,
            m.updated updated, m.approved approved, m.status status, m.requested_status requested_status,
            m.issues_url issues_url, m.source_url source_url, m.wiki_url wiki_url, m.discord_url discord_url, m.license_url license_url,
            m.team_id team_id, m.client_side client_side, m.server_side server_side, m.license license, m.slug slug, m.moderation_message moderation_message, m.moderation_message_body moderation_message_body,
            cs.name client_side_type, ss.name server_side_type, pt.name project_type_name, m.flame_anvil_project flame_anvil_project, m.flame_anvil_user flame_anvil_user, m.webhook_sent, m.color,
            m.loaders loaders, m.game_versions game_versions,
            ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null and mc.is_additional is false) categories,
            ARRAY_AGG(DISTINCT c.category) filter (where c.category is not null and mc.is_additional is true) additional_categories,
            JSONB_AGG(DISTINCT jsonb_build_object('id', v.id, 'date_published', v.date_published)) filter (where v.id is not null) versions,
            JSONB_AGG(DISTINCT jsonb_build_object('image_url', mg.image_url, 'featured', mg.featured, 'title', mg.title, 'description', mg.description, 'created', mg.created, 'ordering', mg.ordering)) filter (where mg.image_url is not null) gallery,
            JSONB_AGG(DISTINCT jsonb_build_object('platform_id', md.joining_platform_id, 'platform_short', dp.short, 'platform_name', dp.name,'url', md.url)) filter (where md.joining_platform_id is not null) donations
            FROM mods m
            INNER JOIN project_types pt ON pt.id = m.project_type
            INNER JOIN side_types cs ON m.client_side = cs.id
            INNER JOIN side_types ss ON m.server_side = ss.id
            LEFT JOIN mods_donations md ON md.joining_mod_id = m.id
            LEFT JOIN donation_platforms dp ON md.joining_platform_id = dp.id
            LEFT JOIN mods_categories mc ON mc.joining_mod_id = m.id
            LEFT JOIN categories c ON mc.joining_category_id = c.id
            LEFT JOIN versions v ON v.mod_id = m.id AND v.status = ANY($2)
            LEFT JOIN mods_gallery mg ON mg.mod_id = m.id
            WHERE m.id = ANY($1)
            GROUP BY pt.id, cs.id, ss.id, m.id;
            ",
            &project_ids_parsed,
            &*crate::models::projects::VersionStatus::iterator().filter(|x| x.is_listed()).map(|x| x.to_string()).collect::<Vec<String>>()
        )
            .fetch_many(exec)
            .try_filter_map(|e| async {
                Ok(e.right().map(|m| {
                    let id = m.id;

                    QueryProject {
                        inner: Project {
                            id: ProjectId(id),
                            project_type: ProjectTypeId(m.project_type),
                            team_id: TeamId(m.team_id),
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
                            client_side: SideTypeId(m.client_side),
                            status: ProjectStatus::from_str(
                                &m.status,
                            ),
                            requested_status: m.requested_status.map(|x| ProjectStatus::from_str(
                                &x,
                            )),
                            server_side: SideTypeId(m.server_side),
                            license: m.license.clone(),
                            slug: m.slug.clone(),
                            body: m.body.clone(),
                            follows: m.follows,
                            moderation_message: m.moderation_message,
                            moderation_message_body: m.moderation_message_body,
                            approved: m.approved,
                            flame_anvil_project: m.flame_anvil_project,
                            flame_anvil_user: m.flame_anvil_user.map(UserId),
                            webhook_sent: m.webhook_sent,
                            color: m.color.map(|x| x as u32),
                            loaders: m.loaders,
                            game_versions: m.game_versions,
                        },
                        project_type: m.project_type_name,
                        categories: m.categories.unwrap_or_default(),
                        additional_categories: m.additional_categories.unwrap_or_default(),
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
                        client_side: crate::models::projects::SideType::from_str(&m.client_side_type),
                        server_side: crate::models::projects::SideType::from_str(&m.server_side_type),
                    }}))
            })
            .try_collect::<Vec<QueryProject>>()
            .await
    }

    pub async fn update_game_versions(
        id: ProjectId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            UPDATE mods
            SET game_versions = (
                SELECT COALESCE(ARRAY_AGG(DISTINCT gv.version) filter (where gv.version is not null), array[]::varchar[])
                FROM versions v
                     INNER JOIN game_versions_versions gvv ON v.id = gvv.joining_version_id
                     INNER JOIN game_versions gv on gvv.game_version_id = gv.id
                WHERE v.mod_id = mods.id AND v.status != ANY($2)
            )
            WHERE id = $1
            ",
            id as ProjectId,
            &*crate::models::projects::VersionStatus::iterator().filter(|x| x.is_hidden()).map(|x| x.to_string()).collect::<Vec<String>>()
        )
            .execute(&mut *transaction)
            .await?;

        Ok(())
    }

    pub async fn update_loaders(
        id: ProjectId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            UPDATE mods
            SET loaders = (
                SELECT COALESCE(ARRAY_AGG(DISTINCT l.loader) filter (where l.loader is not null), array[]::varchar[])
                FROM versions v
                     INNER JOIN loaders_versions lv ON lv.version_id = v.id
                     INNER JOIN loaders l on lv.loader_id = l.id
                WHERE v.mod_id = mods.id AND v.status != ANY($2)
            )
            WHERE id = $1
            ",
            id as ProjectId,
            &*crate::models::projects::VersionStatus::iterator().filter(|x| x.is_hidden()).map(|x| x.to_string()).collect::<Vec<String>>()
        )
            .execute(&mut *transaction)
            .await?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct QueryProject {
    pub inner: Project,
    pub project_type: String,
    pub categories: Vec<String>,
    pub additional_categories: Vec<String>,
    pub versions: Vec<VersionId>,
    pub donation_urls: Vec<DonationUrl>,
    pub gallery_items: Vec<GalleryItem>,
    pub client_side: crate::models::projects::SideType,
    pub server_side: crate::models::projects::SideType,
}
