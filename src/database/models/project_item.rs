use super::ids::*;

#[derive(Clone, Debug)]
pub struct DonationUrl {
    pub project_id: ProjectId,
    pub platform_id: DonationPlatformId,
    pub platform_short: String,
    pub platform_name: String,
    pub url: String,
}

impl DonationUrl {
    pub async fn insert(
        &self,
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
            self.project_id as ProjectId,
            self.platform_id as DonationPlatformId,
            self.url,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct GalleryItem {
    pub project_id: ProjectId,
    pub image_url: String,
}

impl GalleryItem {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO mods_gallery (
                mod_id, image_url
            )
            VALUES (
                $1, $2
            )
            ",
            self.project_id as ProjectId,
            self.image_url,
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
    pub initial_versions: Vec<super::version_item::VersionBuilder>,
    pub status: StatusId,
    pub client_side: SideTypeId,
    pub server_side: SideTypeId,
    pub license: LicenseId,
    pub slug: Option<String>,
    pub donation_urls: Vec<DonationUrl>,
    pub gallery_items: Vec<GalleryItem>,
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
            published: chrono::Utc::now(),
            updated: chrono::Utc::now(),
            status: self.status,
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
            rejection_reason: None,
            rejection_body: None,
        };
        project_struct.insert(&mut *transaction).await?;

        for mut version in self.initial_versions {
            version.project_id = self.project_id;
            version.insert(&mut *transaction).await?;
        }

        for mut donation in self.donation_urls {
            donation.project_id = self.project_id;
            donation.insert(&mut *transaction).await?;
        }

        for mut gallery in self.gallery_items {
            gallery.project_id = self.project_id;
            gallery.insert(&mut *transaction).await?;
        }

        for category in self.categories {
            sqlx::query!(
                "
                INSERT INTO mods_categories (joining_mod_id, joining_category_id)
                VALUES ($1, $2)
                ",
                self.project_id as ProjectId,
                category as CategoryId,
            )
            .execute(&mut *transaction)
            .await?;
        }

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
    pub published: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
    pub status: StatusId,
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
    pub license: LicenseId,
    pub slug: Option<String>,
    pub rejection_reason: Option<String>,
    pub rejection_body: Option<String>,
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
                source_url, wiki_url, status, discord_url,
                client_side, server_side, license_url, license,
                slug, project_type
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12, $13,
                $14, $15, $16, $17,
                LOWER($18), $19
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
            self.status.0,
            self.discord_url.as_ref(),
            self.client_side as SideTypeId,
            self.server_side as SideTypeId,
            self.license_url.as_ref(),
            self.license as LicenseId,
            self.slug.as_ref(),
            self.project_type as ProjectTypeId
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
                   icon_url, body, body_url, published,
                   updated, status,
                   issues_url, source_url, wiki_url, discord_url, license_url,
                   team_id, client_side, server_side, license, slug,
                   rejection_reason, rejection_body
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
                body_url: row.body_url,
                icon_url: row.icon_url,
                published: row.published,
                updated: row.updated,
                issues_url: row.issues_url,
                source_url: row.source_url,
                wiki_url: row.wiki_url,
                license_url: row.license_url,
                discord_url: row.discord_url,
                client_side: SideTypeId(row.client_side),
                status: StatusId(row.status),
                server_side: SideTypeId(row.server_side),
                license: LicenseId(row.license),
                slug: row.slug,
                body: row.body,
                follows: row.follows,
                rejection_reason: row.rejection_reason,
                rejection_body: row.rejection_body,
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

        let project_ids_parsed: Vec<i64> = project_ids.into_iter().map(|x| x.0).collect();
        let projects = sqlx::query!(
            "
            SELECT id, project_type, title, description, downloads, follows,
                   icon_url, body, body_url, published,
                   updated, status,
                   issues_url, source_url, wiki_url, discord_url, license_url,
                   team_id, client_side, server_side, license, slug,
                   rejection_reason, rejection_body
            FROM mods
            WHERE id IN (SELECT * FROM UNNEST($1::bigint[]))
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
                body_url: m.body_url,
                icon_url: m.icon_url,
                published: m.published,
                updated: m.updated,
                issues_url: m.issues_url,
                source_url: m.source_url,
                wiki_url: m.wiki_url,
                license_url: m.license_url,
                discord_url: m.discord_url,
                client_side: SideTypeId(m.client_side),
                status: StatusId(m.status),
                server_side: SideTypeId(m.server_side),
                license: LicenseId(m.license),
                slug: m.slug,
                body: m.body,
                follows: m.follows,
                rejection_reason: m.rejection_reason,
                rejection_body: m.rejection_body,
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
                WHERE LOWER(slug) = LOWER($1)
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
                WHERE LOWER(slug) = LOWER($1)
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
        slug_or_project_id: String,
        executor: E,
    ) -> Result<Option<Project>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let id_option =
            crate::models::ids::base62_impl::parse_base62(&*slug_or_project_id.clone()).ok();

        if let Some(id) = id_option {
            let mut project = Project::get(ProjectId(id as i64), executor).await?;

            if project.is_none() {
                project = Project::get_from_slug(&slug_or_project_id, executor).await?;
            }

            Ok(project)
        } else {
            let project = Project::get_from_slug(&slug_or_project_id, executor).await?;

            Ok(project)
        }
    }

    pub async fn get_full_from_slug_or_project_id<'a, 'b, E>(
        slug_or_project_id: String,
        executor: E,
    ) -> Result<Option<QueryProject>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let id_option =
            crate::models::ids::base62_impl::parse_base62(&*slug_or_project_id.clone()).ok();

        if let Some(id) = id_option {
            let mut project = Project::get_full(ProjectId(id as i64), executor).await?;

            if project.is_none() {
                project = Project::get_full_from_slug(&slug_or_project_id, executor).await?;
            }

            Ok(project)
        } else {
            let project = Project::get_full_from_slug(&slug_or_project_id, executor).await?;
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
            m.icon_url icon_url, m.body body, m.body_url body_url, m.published published,
            m.updated updated, m.status status,
            m.issues_url issues_url, m.source_url source_url, m.wiki_url wiki_url, m.discord_url discord_url, m.license_url license_url,
            m.team_id team_id, m.client_side client_side, m.server_side server_side, m.license license, m.slug slug, m.rejection_reason rejection_reason, m.rejection_body rejection_body,
            s.status status_name, cs.name client_side_type, ss.name server_side_type, l.short short, l.name license_name, pt.name project_type_name,
            STRING_AGG(DISTINCT c.category, ',') categories, STRING_AGG(DISTINCT v.id::text, ',') versions, STRING_AGG(DISTINCT mg.image_url, ',') gallery,
            STRING_AGG(DISTINCT md.joining_platform_id || ', ' || md.url || ', ' || dp.short || ', ' || dp.name, ' ,') donations
            FROM mods m
            LEFT OUTER JOIN mods_categories mc ON joining_mod_id = m.id
            LEFT OUTER JOIN categories c ON mc.joining_category_id = c.id
            LEFT OUTER JOIN versions v ON v.mod_id = m.id
            LEFT OUTER JOIN mods_gallery mg ON mg.mod_id = m.id
            LEFT OUTER JOIN mods_donations md ON md.joining_mod_id = m.id
            LEFT OUTER JOIN donation_platforms dp ON md.joining_platform_id = dp.id
            INNER JOIN project_types pt ON pt.id = m.project_type
            INNER JOIN statuses s ON s.id = m.status
            INNER JOIN side_types cs ON m.client_side = cs.id
            INNER JOIN side_types ss ON m.server_side = ss.id
            INNER JOIN licenses l ON m.license = l.id
            WHERE m.id = $1
            GROUP BY m.id, s.id, cs.id, ss.id, l.id, pt.id;
            ",
            id as ProjectId,
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
                    body_url: m.body_url.clone(),
                    icon_url: m.icon_url.clone(),
                    published: m.published,
                    updated: m.updated,
                    issues_url: m.issues_url.clone(),
                    source_url: m.source_url.clone(),
                    wiki_url: m.wiki_url.clone(),
                    license_url: m.license_url.clone(),
                    discord_url: m.discord_url.clone(),
                    client_side: SideTypeId(m.client_side),
                    status: StatusId(m.status),
                    server_side: SideTypeId(m.server_side),
                    license: LicenseId(m.license),
                    slug: m.slug.clone(),
                    body: m.body.clone(),
                    follows: m.follows,
                    rejection_reason: m.rejection_reason,
                    rejection_body: m.rejection_body,
                },
                project_type: m.project_type_name,
                categories: m
                    .categories
                    .unwrap_or_default()
                    .split(',')
                    .map(|x| x.to_string())
                    .collect(),
                versions: m
                    .versions
                    .unwrap_or_default()
                    .split(',')
                    .map(|x| VersionId(x.parse().unwrap_or_default()))
                    .collect(),
                donation_urls: m
                    .donations
                    .unwrap_or_default()
                    .split(" ,")
                    .map(|d| {
                        let strings: Vec<&str> = d.split(", ").collect();

                        if strings.len() >= 3 {
                            Some(DonationUrl {
                                project_id: id,
                                platform_id: DonationPlatformId(strings[0].parse().unwrap_or(0)),
                                platform_short: strings[2].to_string(),
                                platform_name: strings[3].to_string(),
                                url: strings[1].to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect(),
                gallery_items: m
                    .gallery
                    .into_iter()
                    .map(|x| GalleryItem {
                        project_id: id,
                        image_url: x,
                    })
                    .collect(),
                status: crate::models::projects::ProjectStatus::from_str(&m.status_name),
                license_id: m.short,
                license_name: m.license_name,
                client_side: crate::models::projects::SideType::from_str(&m.client_side_type),
                server_side: crate::models::projects::SideType::from_str(&m.server_side_type),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many_full<'a, E>(
        project_ids: Vec<ProjectId>,
        exec: E,
    ) -> Result<Vec<QueryProject>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::TryStreamExt;

        let project_ids_parsed: Vec<i64> = project_ids.into_iter().map(|x| x.0).collect();
        sqlx::query!(
            "
            SELECT m.id id, m.project_type project_type, m.title title, m.description description, m.downloads downloads, m.follows follows,
            m.icon_url icon_url, m.body body, m.body_url body_url, m.published published,
            m.updated updated, m.status status,
            m.issues_url issues_url, m.source_url source_url, m.wiki_url wiki_url, m.discord_url discord_url, m.license_url license_url,
            m.team_id team_id, m.client_side client_side, m.server_side server_side, m.license license, m.slug slug, m.rejection_reason rejection_reason, m.rejection_body rejection_body,
            s.status status_name, cs.name client_side_type, ss.name server_side_type, l.short short, l.name license_name, pt.name project_type_name,
            STRING_AGG(DISTINCT c.category, ',') categories, STRING_AGG(DISTINCT v.id::text, ',') versions, STRING_AGG(DISTINCT mg.image_url, ',') gallery,
            STRING_AGG(DISTINCT md.joining_platform_id || ', ' || md.url || ', ' || dp.short || ', ' || dp.name, ' ,') donations
            FROM mods m
            LEFT OUTER JOIN mods_categories mc ON joining_mod_id = m.id
            LEFT OUTER JOIN categories c ON mc.joining_category_id = c.id
            LEFT OUTER JOIN versions v ON v.mod_id = m.id
            LEFT OUTER JOIN mods_gallery mg ON mg.mod_id = m.id
            LEFT OUTER JOIN mods_donations md ON md.joining_mod_id = m.id
            LEFT OUTER JOIN donation_platforms dp ON md.joining_platform_id = dp.id
            INNER JOIN project_types pt ON pt.id = m.project_type
            INNER JOIN statuses s ON s.id = m.status
            INNER JOIN side_types cs ON m.client_side = cs.id
            INNER JOIN side_types ss ON m.server_side = ss.id
            INNER JOIN licenses l ON m.license = l.id
            WHERE m.id IN (SELECT * FROM UNNEST($1::bigint[]))
            GROUP BY m.id, s.id, cs.id, ss.id, l.id, pt.id;
            ",
            &project_ids_parsed
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
                        body_url: m.body_url.clone(),
                        icon_url: m.icon_url.clone(),
                        published: m.published,
                        updated: m.updated,
                        issues_url: m.issues_url.clone(),
                        source_url: m.source_url.clone(),
                        wiki_url: m.wiki_url.clone(),
                        license_url: m.license_url.clone(),
                        discord_url: m.discord_url.clone(),
                        client_side: SideTypeId(m.client_side),
                        status: StatusId(m.status),
                        server_side: SideTypeId(m.server_side),
                        license: LicenseId(m.license),
                        slug: m.slug.clone(),
                        body: m.body.clone(),
                        follows: m.follows,
                        rejection_reason: m.rejection_reason,
                        rejection_body: m.rejection_body,
                    },
                    project_type: m.project_type_name,
                    categories: m.categories.unwrap_or_default().split(',').map(|x| x.to_string()).collect(),
                    versions: m.versions.unwrap_or_default().split(',').map(|x| VersionId(x.parse().unwrap_or_default())).collect(),
                        donation_urls: m
                            .donations
                            .unwrap_or_default()
                            .split(" ,")
                            .map(|d| {
                                let strings: Vec<&str> = d.split(", ").collect();

                                if strings.len() >= 3 {
                                    Some(DonationUrl {
                                        project_id: ProjectId(id),
                                        platform_id: DonationPlatformId(strings[0].parse().unwrap_or(0)),
                                        platform_short: strings[2].to_string(),
                                        platform_name: strings[3].to_string(),
                                        url: strings[1].to_string(),
                                    })
                                } else {
                                    None
                                }
                            })
                            .flatten()
                            .collect(),
                    gallery_items: m.gallery.iter().map(|x| GalleryItem {
                        project_id:  ProjectId(id),
                        image_url: x.to_string()
                    }).collect(),
                    status: crate::models::projects::ProjectStatus::from_str(&m.status_name),
                    license_id: m.short,
                    license_name: m.license_name,
                    client_side: crate::models::projects::SideType::from_str(&m.client_side_type),
                    server_side: crate::models::projects::SideType::from_str(&m.server_side_type),
                }}))
            })
            .try_collect::<Vec<QueryProject>>()
            .await
    }
}
#[derive(Clone, Debug)]
pub struct QueryProject {
    pub inner: Project,
    pub project_type: String,
    pub categories: Vec<String>,
    pub versions: Vec<VersionId>,
    pub donation_urls: Vec<DonationUrl>,
    pub gallery_items: Vec<GalleryItem>,
    pub status: crate::models::projects::ProjectStatus,
    pub license_id: String,
    pub license_name: String,
    pub client_side: crate::models::projects::SideType,
    pub server_side: crate::models::projects::SideType,
}
