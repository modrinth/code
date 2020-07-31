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
}
