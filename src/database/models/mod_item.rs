use super::ids::*;

pub struct DonationUrl {
    pub mod_id: ModId,
    pub platform_id: DonationPlatformId,
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
    pub body_url: String,
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
            body_url: self.body_url,
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
    pub body_url: String,
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
                id, team_id, title, description, body_url,
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
            &self.body_url,
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
                   icon_url, body_url, published,
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
                   icon_url, body_url, published,
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
        let result = Self::get(id, executor).await?;
        if let Some(inner) = result {
            use futures::TryStreamExt;
            let categories: Vec<String> = sqlx::query!(
                "
                SELECT category FROM mods_categories
                INNER JOIN categories ON joining_category_id = id
                WHERE joining_mod_id = $1
                ",
                id as ModId,
            )
            .fetch_many(executor)
            .try_filter_map(|e| async { Ok(e.right().map(|c| c.category)) })
            .try_collect::<Vec<String>>()
            .await?;

            let versions: Vec<VersionId> = sqlx::query!(
                "
                SELECT id FROM versions
                WHERE mod_id = $1
                ",
                id as ModId,
            )
            .fetch_many(executor)
            .try_filter_map(|e| async { Ok(e.right().map(|c| VersionId(c.id))) })
            .try_collect::<Vec<VersionId>>()
            .await?;

            let donations: Vec<DonationUrl> = sqlx::query!(
                "
                SELECT joining_platform_id, url FROM mods_donations
                WHERE joining_mod_id = $1
                ",
                id as ModId,
            )
            .fetch_many(executor)
            .try_filter_map(|e| async {
                Ok(e.right().map(|c| DonationUrl {
                    mod_id: id,
                    platform_id: DonationPlatformId(c.joining_platform_id),
                    url: c.url,
                }))
            })
            .try_collect::<Vec<DonationUrl>>()
            .await?;

            let status = sqlx::query!(
                "
                SELECT status FROM statuses
                WHERE id = $1
                ",
                inner.status.0,
            )
            .fetch_one(executor)
            .await?
            .status;

            let client_side = sqlx::query!(
                "
                SELECT name FROM side_types
                WHERE id = $1
                ",
                inner.client_side.0,
            )
                .fetch_one(executor)
                .await?
                .name;

            let server_side = sqlx::query!(
                "
                SELECT name FROM side_types
                WHERE id = $1
                ",
                inner.server_side.0,
            )
                .fetch_one(executor)
                .await?
                .name;

            let license = sqlx::query!(
                "
                SELECT short, name FROM licenses
                WHERE id = $1
                ",
                inner.license.0,
            )
                .fetch_one(executor)
                .await?;

            Ok(Some(QueryMod {
                inner,
                categories,
                versions,
                donation_urls: donations,
                status: crate::models::mods::ModStatus::from_str(&status),
                license_id: license.short,
                license_name: license.name,
                client_side: crate::models::mods::SideType::from_str(&client_side),
                server_side: crate::models::mods::SideType::from_str(&server_side)
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many_full<'a, E>(
        mod_ids: Vec<ModId>,
        exec: E,
    ) -> Result<Vec<Option<QueryMod>>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        // TODO: this could be optimized
        futures::future::try_join_all(mod_ids.into_iter().map(|id| Self::get_full(id, exec))).await
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
