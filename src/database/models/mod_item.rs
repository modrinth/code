use super::ids::*;

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
    pub categories: Vec<CategoryId>,
    pub initial_versions: Vec<super::version_item::VersionBuilder>,
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
            downloads: 0,
            icon_url: self.icon_url,
            issues_url: self.issues_url,
            source_url: self.source_url,
            wiki_url: self.wiki_url,
        };
        mod_struct.insert(&mut *transaction).await?;

        for mut version in self.initial_versions {
            version.mod_id = self.mod_id;
            version.insert(&mut *transaction).await?;
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
    pub downloads: i32,
    pub icon_url: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
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
                source_url, wiki_url
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11
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
                   issues_url, source_url, wiki_url,
                   team_id
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
                issues_url: row.issues_url,
                source_url: row.source_url,
                wiki_url: row.wiki_url,
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
                   issues_url, source_url, wiki_url,
                   team_id
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
                issues_url: m.issues_url,
                source_url: m.source_url,
                wiki_url: m.wiki_url,
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

            Ok(Some(QueryMod {
                inner,
                categories,
                versions,
            }))
        } else {
            Ok(None)
        }
    }
}

pub struct QueryMod {
    pub inner: Mod,

    pub categories: Vec<String>,
    pub versions: Vec<VersionId>,
}
