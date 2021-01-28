use super::ids::*;

pub struct DonationUrl {
    pub mod_id: ModId,
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
            self.mod_id as ModId,
            self.platform_id as DonationPlatformId,
            self.url,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }
}

pub struct ModBuilder {
    pub mod_id: ModId,
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
}

impl ModBuilder {
    pub async fn insert(
        self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<ModId, super::DatabaseError> {
        let mod_struct = Mod {
            id: self.mod_id,
            team_id: self.team_id,
            title: self.title,
            description: self.description,
            body: self.body,
            body_url: None,
            published: chrono::Utc::now(),
            updated: chrono::Utc::now(),
            status: self.status,
            downloads: 0,
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
        };
        mod_struct.insert(&mut *transaction).await?;

        for mut version in self.initial_versions {
            version.mod_id = self.mod_id;
            version.insert(&mut *transaction).await?;
        }

        for mut donation in self.donation_urls {
            donation.mod_id = self.mod_id;
            donation.insert(&mut *transaction).await?;
        }

        for category in self.categories {
            sqlx::query!(
                "
                INSERT INTO mods_categories (joining_mod_id, joining_category_id)
                VALUES ($1, $2)
                ",
                self.mod_id as ModId,
                category as CategoryId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        Ok(self.mod_id)
    }
}

pub struct Mod {
    pub id: ModId,
    pub team_id: TeamId,
    pub title: String,
    pub description: String,
    pub body: String,
    pub body_url: Option<String>,
    pub published: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
    pub status: StatusId,
    pub downloads: i32,
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
}

impl Mod {
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
                slug
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12, $13,
                $14, $15, $16, $17,
                $18
            )
            ",
            self.id as ModId,
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
            self.slug.as_ref()
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn get<'a, 'b, E>(id: ModId, executor: E) -> Result<Option<Self>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT title, description, downloads,
                   icon_url, body, body_url, published,
                   updated, status,
                   issues_url, source_url, wiki_url, discord_url, license_url,
                   team_id, client_side, server_side, license, slug
            FROM mods
            WHERE id = $1
            ",
            id as ModId,
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(Mod {
                id,
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
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many<'a, E>(mod_ids: Vec<ModId>, exec: E) -> Result<Vec<Mod>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let mod_ids_parsed: Vec<i64> = mod_ids.into_iter().map(|x| x.0).collect();
        let mods = sqlx::query!(
            "
            SELECT id, title, description, downloads,
                   icon_url, body, body_url, published,
                   updated, status,
                   issues_url, source_url, wiki_url, discord_url, license_url,
                   team_id, client_side, server_side, license, slug
            FROM mods
            WHERE id IN (SELECT * FROM UNNEST($1::bigint[]))
            ",
            &mod_ids_parsed
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|m| Mod {
                id: ModId(m.id),
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
            }))
        })
        .try_collect::<Vec<Mod>>()
        .await?;

        Ok(mods)
    }

    pub async fn remove_full<'a, 'b, E>(
        id: ModId,
        exec: E,
    ) -> Result<Option<()>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let result = sqlx::query!(
            "
            SELECT team_id FROM mods WHERE id = $1
            ",
            id as ModId,
        )
        .fetch_optional(exec)
        .await?;

        let team_id: TeamId = if let Some(id) = result {
            TeamId(id.team_id)
        } else {
            return Ok(None);
        };

        sqlx::query!(
            "
            DELETE FROM mods_categories
            WHERE joining_mod_id = $1
            ",
            id as ModId,
        )
        .execute(exec)
        .await?;

        sqlx::query!(
            "
            DELETE FROM mods_donations
            WHERE joining_mod_id = $1
            ",
            id as ModId,
        )
        .execute(exec)
        .await?;

        use futures::TryStreamExt;
        let versions: Vec<VersionId> = sqlx::query!(
            "
            SELECT id FROM versions
            WHERE mod_id = $1
            ",
            id as ModId,
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|c| VersionId(c.id))) })
        .try_collect::<Vec<VersionId>>()
        .await?;

        for version in versions {
            super::Version::remove_full(version, exec).await?;
        }

        sqlx::query!(
            "
            DELETE FROM mods
            WHERE id = $1
            ",
            id as ModId,
        )
        .execute(exec)
        .await?;

        sqlx::query!(
            "
            DELETE FROM team_members
            WHERE team_id = $1
            ",
            team_id as TeamId,
        )
        .execute(exec)
        .await?;

        sqlx::query!(
            "
            DELETE FROM teams
            WHERE id = $1
            ",
            team_id as TeamId,
        )
        .execute(exec)
        .await?;

        Ok(Some(()))
    }

    pub async fn get_full_from_slug<'a, 'b, E>(
        slug: String,
        executor: E,
    ) -> Result<Option<QueryMod>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let id = sqlx::query!(
            "
                SELECT id FROM mods
                WHERE slug = $1
                ",
            slug
        )
        .fetch_optional(executor)
        .await?;

        if let Some(mod_id) = id {
            Mod::get_full(ModId(mod_id.id), executor).await
        } else {
            Ok(None)
        }
    }

    pub async fn get_full<'a, 'b, E>(
        id: ModId,
        executor: E,
    ) -> Result<Option<QueryMod>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let result = sqlx::query!(
            "
            SELECT m.id id, m.title title, m.description description, m.downloads downloads,
            m.icon_url icon_url, m.body body, m.body_url body_url, m.published published,
            m.updated updated, m.status status,
            m.issues_url issues_url, m.source_url source_url, m.wiki_url wiki_url, m.discord_url discord_url, m.license_url license_url,
            m.team_id team_id, m.client_side client_side, m.server_side server_side, m.license license, m.slug slug,
            s.status status_name, cs.name client_side_type, ss.name server_side_type, l.short short, l.name license_name,
            STRING_AGG(DISTINCT c.category, ',') categories, STRING_AGG(DISTINCT v.id::text, ',') versions
            FROM mods m
            LEFT OUTER JOIN mods_categories mc ON joining_mod_id = m.id
            LEFT OUTER JOIN categories c ON mc.joining_category_id = c.id
            LEFT OUTER JOIN versions v ON v.mod_id = m.id
            INNER JOIN statuses s ON s.id = m.status
            INNER JOIN side_types cs ON m.client_side = cs.id
            INNER JOIN side_types ss ON m.server_side = ss.id
            INNER JOIN licenses l ON m.license = l.id
            WHERE m.id = $1
            GROUP BY m.id, s.id, cs.id, ss.id, l.id;
            ",
            id as ModId,
        )
            .fetch_optional(executor)
            .await?;

        if let Some(m) = result {
            Ok(Some(QueryMod {
                inner: Mod {
                    id: ModId(m.id),
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
                },
                categories: m
                    .categories
                    .unwrap_or_default()
                    .split(",")
                    .map(|x| x.to_string())
                    .collect(),
                versions: m
                    .versions
                    .unwrap_or_default()
                    .split(",")
                    .map(|x| VersionId(x.parse().unwrap_or_default()))
                    .collect(),
                donation_urls: vec![],
                status: crate::models::mods::ModStatus::from_str(&m.status_name),
                license_id: m.short,
                license_name: m.license_name,
                client_side: crate::models::mods::SideType::from_str(&m.client_side_type),
                server_side: crate::models::mods::SideType::from_str(&m.server_side_type),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many_full<'a, E>(
        mod_ids: Vec<ModId>,
        exec: E,
    ) -> Result<Vec<QueryMod>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::TryStreamExt;

        let mod_ids_parsed: Vec<i64> = mod_ids.into_iter().map(|x| x.0).collect();
        sqlx::query!(
            "
            SELECT m.id id, m.title title, m.description description, m.downloads downloads,
            m.icon_url icon_url, m.body body, m.body_url body_url, m.published published,
            m.updated updated, m.status status,
            m.issues_url issues_url, m.source_url source_url, m.wiki_url wiki_url, m.discord_url discord_url, m.license_url license_url,
            m.team_id team_id, m.client_side client_side, m.server_side server_side, m.license license, m.slug slug,
            s.status status_name, cs.name client_side_type, ss.name server_side_type, l.short short, l.name license_name,
            STRING_AGG(DISTINCT c.category, ',') categories, STRING_AGG(DISTINCT v.id::text, ',') versions
            FROM mods m
            LEFT OUTER JOIN mods_categories mc ON joining_mod_id = m.id
            LEFT OUTER JOIN categories c ON mc.joining_category_id = c.id
            LEFT OUTER JOIN versions v ON v.mod_id = m.id
            INNER JOIN statuses s ON s.id = m.status
            INNER JOIN side_types cs ON m.client_side = cs.id
            INNER JOIN side_types ss ON m.server_side = ss.id
            INNER JOIN licenses l ON m.license = l.id
            WHERE m.id IN (SELECT * FROM UNNEST($1::bigint[]))
            GROUP BY m.id, s.id, cs.id, ss.id, l.id;
            ",
            &mod_ids_parsed
        )
            .fetch_many(exec)
            .try_filter_map(|e| async {
                Ok(e.right().map(|m| QueryMod {
                    inner: Mod {
                        id: ModId(m.id),
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
                    },
                    categories: m.categories.unwrap_or_default().split(",").map(|x| x.to_string()).collect(),
                    versions: m.versions.unwrap_or_default().split(",").map(|x| VersionId(x.parse().unwrap_or_default())).collect(),
                    donation_urls: vec![],
                    status: crate::models::mods::ModStatus::from_str(&m.status_name),
                    license_id: m.short,
                    license_name: m.license_name,
                    client_side: crate::models::mods::SideType::from_str(&m.client_side_type),
                    server_side: crate::models::mods::SideType::from_str(&m.server_side_type),
                }))
            })
            .try_collect::<Vec<QueryMod>>()
            .await
    }
}

pub struct QueryMod {
    pub inner: Mod,

    pub categories: Vec<String>,
    pub versions: Vec<VersionId>,
    pub donation_urls: Vec<DonationUrl>,
    pub status: crate::models::mods::ModStatus,
    pub license_id: String,
    pub license_name: String,
    pub client_side: crate::models::mods::SideType,
    pub server_side: crate::models::mods::SideType,
}
