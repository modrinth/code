use super::ApiError;
use crate::auth::check_is_moderator_from_headers;
use crate::database;
use crate::models;
use crate::models::mods::{ModId, ModStatus, VersionType};
use crate::models::teams::TeamId;
use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct ResultCount {
    #[serde(default = "default_count")]
    count: i16,
}

fn default_count() -> i16 {
    100
}

/// A mod returned from the API moderation routes
#[derive(Serialize)]
pub struct ModerationMod {
    /// The ID of the mod, encoded as a base62 string.
    pub id: ModId,
    /// The slug of a mod, used for vanity URLs
    pub slug: Option<String>,
    /// The team of people that has ownership of this mod.
    pub team: TeamId,
    /// The title or name of the mod.
    pub title: String,
    /// A short description of the mod.
    pub description: String,
    /// The long description of the mod.
    pub body: String,
    /// The date at which the mod was first published.
    pub published: DateTime<Utc>,
    /// The date at which the mod was first published.
    pub updated: DateTime<Utc>,
    /// The status of the mod
    pub status: ModStatus,

    /// The total number of downloads the mod has had.
    pub downloads: u32,
    /// The URL of the icon of the mod
    pub icon_url: Option<String>,
    /// An optional link to where to submit bugs or issues with the mod.
    pub issues_url: Option<String>,
    /// An optional link to the source code for the mod.
    pub source_url: Option<String>,
    /// An optional link to the mod's wiki page or other relevant information.
    pub wiki_url: Option<String>,
    /// An optional link to the mod's discord
    pub discord_url: Option<String>,
}

#[get("mods")]
pub async fn mods(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    count: web::Query<ResultCount>,
) -> Result<HttpResponse, ApiError> {
    check_is_moderator_from_headers(req.headers(), &**pool).await?;

    use futures::stream::TryStreamExt;

    let mods = sqlx::query!(
        "
        SELECT * FROM mods
        WHERE status = (
            SELECT id FROM statuses WHERE status = $1
        )
        ORDER BY updated ASC
        LIMIT $2;
        ",
        ModStatus::Processing.as_str(),
        count.count as i64
    )
    .fetch_many(&**pool)
    .try_filter_map(|e| async {
        Ok(e.right().map(|m| ModerationMod {
            id: database::models::ids::ModId(m.id).into(),
            slug: m.slug,
            team: database::models::ids::TeamId(m.team_id).into(),
            title: m.title,
            description: m.description,
            body: m.body,
            published: m.published,
            icon_url: m.icon_url,
            issues_url: m.issues_url,
            source_url: m.source_url,
            status: ModStatus::Processing,
            updated: m.updated,
            downloads: m.downloads as u32,
            wiki_url: m.wiki_url,
            discord_url: m.discord_url,
        }))
    })
    .try_collect::<Vec<ModerationMod>>()
    .await
    .map_err(|e| ApiError::DatabaseError(e.into()))?;

    Ok(HttpResponse::Ok().json(mods))
}

/// Returns a list of versions that need to be approved
#[get("versions")]
pub async fn versions(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    count: web::Query<ResultCount>,
) -> Result<HttpResponse, ApiError> {
    check_is_moderator_from_headers(req.headers(), &**pool).await?;

    use futures::stream::TryStreamExt;

    let versions = sqlx::query!(
        "
        SELECT * FROM versions
        WHERE accepted = FALSE
        ORDER BY date_published ASC
        LIMIT $1;
        ",
        count.count as i64
    )
    .fetch_many(&**pool)
    .try_filter_map(|e| async {
        Ok(e.right().map(|m| models::mods::Version {
            id: database::models::ids::VersionId(m.id).into(),
            mod_id: database::models::ids::ModId(m.mod_id).into(),
            author_id: database::models::ids::UserId(m.author_id).into(),
            featured: m.featured,
            name: m.name,
            version_number: m.version_number,
            changelog: m.changelog,
            changelog_url: m.changelog_url,
            date_published: m.date_published,
            downloads: m.downloads as u32,
            version_type: VersionType::Release,
            files: vec![],
            dependencies: vec![],
            game_versions: vec![],
            loaders: vec![],
        }))
    })
    .try_collect::<Vec<models::mods::Version>>()
    .await
    .map_err(|e| ApiError::DatabaseError(e.into()))?;

    Ok(HttpResponse::Ok().json(versions))
}
